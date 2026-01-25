use aws_config;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::CompletedPart;
use aws_sdk_s3::Client as S3Client;
use aws_types::region::Region;
use chrono::{DateTime, Utc};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

/// File storage errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Delete failed: {0}")]
    DeleteFailed(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid chunk: {0}")]
    InvalidChunk(String),

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("Upload session expired")]
    SessionExpired,

    #[error("File too large: {0} bytes (max: {1} bytes)")]
    FileTooLarge(u64, u64),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
}

/// Chunked upload session
#[derive(Debug, Clone)]
pub struct ChunkedUploadSession {
    pub upload_id: Uuid,
    pub space_id: Uuid,
    pub document_id: Option<Uuid>,
    pub file_name: String,
    pub content_type: String,
    pub total_size: u64,
    pub chunk_size: u64,
    pub total_chunks: u32,
    pub uploaded_chunks: Vec<u32>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// S3 storage configuration
#[derive(Debug, Clone)]
pub struct S3StorageConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub use_ssl: bool,
}

/// S3 storage client (compatible with MinIO)
#[derive(Clone)]
pub struct S3Storage {
    client: S3Client,
    bucket: String,
}

impl S3Storage {
    /// Create new S3 storage client
    pub async fn new(config: S3StorageConfig) -> Result<Self, StorageError> {
        let region = Region::new(config.region.clone());

        // Load AWS config from environment
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        // Create credentials
        let credentials = Credentials::new(&config.access_key, &config.secret_key, None, None, "miniwiki");

        // Build S3 config with custom endpoint
        let sdk_config = S3ConfigBuilder::from(&aws_config)
            .region(region)
            .endpoint_url(if config.use_ssl {
                format!("https://{}", config.endpoint)
            } else {
                format!("http://{}", config.endpoint)
            })
            .force_path_style(true)
            .credentials_provider(credentials)
            .build();

        let client = S3Client::from_conf(sdk_config);

        // Verify bucket exists
        let _ = client
            .head_bucket()
            .bucket(&config.bucket)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
        })
    }

    /// Get presigned upload URL
    pub async fn presigned_upload_url(
        &self,
        object_name: &str,
        content_type: &str,
        expires_seconds: i32,
    ) -> Result<String, StorageError> {
        let expires_secs: u64 = expires_seconds
            .try_into()
            .map_err(|_| StorageError::UploadFailed("expires_seconds must be non-negative".into()))?;

        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::builder()
            .expires_in(Duration::from_secs(expires_secs))
            .build()
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(object_name)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;

        Ok(presigned.uri().to_string())
    }

    /// Get presigned download URL
    pub async fn presigned_download_url(
        &self,
        object_name: &str,
        expires_seconds: i32,
    ) -> Result<String, StorageError> {
        let expires_secs: u64 = expires_seconds
            .try_into()
            .map_err(|_| StorageError::DownloadFailed("expires_seconds must be non-negative".into()))?;

        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::builder()
            .expires_in(Duration::from_secs(expires_secs))
            .build()
            .map_err(|e| StorageError::DownloadFailed(e.to_string()))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object_name)
            .presigned(presigning_config)
            .await
            .map_err(|e| StorageError::DownloadFailed(e.to_string()))?;

        Ok(presigned.uri().to_string())
    }

    /// Upload file
    pub async fn upload_file(&self, object_name: &str, content: &[u8], content_type: &str) -> Result<(), StorageError> {
        let byte_stream = ByteStream::from(content.to_vec());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(object_name)
            .body(byte_stream)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;

        Ok(())
    }

    /// Upload chunk (for multipart upload)
    pub async fn upload_chunk(
        &self,
        object_name: &str,
        chunk_number: u32,
        upload_id: &str,
        content: &[u8],
    ) -> Result<CompletedPart, StorageError> {
        let byte_stream = ByteStream::from(content.to_vec());

        let result = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(object_name)
            .upload_id(upload_id)
            .part_number(chunk_number as i32)
            .body(byte_stream)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;

        let e_tag = result.e_tag.ok_or_else(|| {
            StorageError::UploadFailed(format!("Missing ETag for chunk {} of {}", chunk_number, object_name))
        })?;

        Ok(CompletedPart::builder().part_number(chunk_number as i32).e_tag(e_tag).build())
    }

    /// Download file
    pub async fn download_file(&self, object_name: &str) -> Result<Vec<u8>, StorageError> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object_name)
            .send()
            .await
            .map_err(|e| StorageError::DownloadFailed(e.to_string()))?;

        let data = result
            .body
            .collect()
            .await
            .map_err(|e| StorageError::DownloadFailed(e.to_string()))?;

        Ok(data.to_vec())
    }

    /// Delete file
    pub async fn delete_file(&self, object_name: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(object_name)
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;

        Ok(())
    }

    /// Check if file exists
    pub async fn file_exists(&self, object_name: &str) -> Result<bool, StorageError> {
        match self.client.head_object().bucket(&self.bucket).key(object_name).send().await {
            Ok(_) => Ok(true),
            Err(e) => match e.into_service_error() {
                aws_sdk_s3::operation::head_object::HeadObjectError::NotFound(_) => Ok(false),
                other => Err(StorageError::DownloadFailed(other.to_string())),
            },
        }
    }

    /// Validate file type
    pub fn validate_file_type(content_type: &str) -> Result<(), StorageError> {
        const ALLOWED_TYPES: &[&str] = &["image/", "application/pdf", "text/", "video/", "audio/"];

        for allowed in ALLOWED_TYPES {
            if content_type.starts_with(allowed) {
                return Ok(());
            }
        }

        Err(StorageError::UnsupportedFileType(content_type.to_string()))
    }

    /// Validate file size
    pub fn validate_file_size(size: u64, max_size: u64) -> Result<(), StorageError> {
        if size > max_size {
            Err(StorageError::FileTooLarge(size, max_size))
        } else {
            Ok(())
        }
    }

    /// Get bucket name
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

/// Get S3 storage configuration from environment
/// Returns Result to ensure required values are explicitly provided
pub fn config_from_env() -> Result<S3StorageConfig, std::env::VarError> {
    Ok(S3StorageConfig {
        endpoint: std::env::var("S3_ENDPOINT")?,
        access_key: std::env::var("S3_ACCESS_KEY")?,
        secret_key: std::env::var("S3_SECRET_KEY")?,
        bucket: std::env::var("S3_BUCKET")?,
        region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        use_ssl: std::env::var("S3_USE_SSL")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(true),
    })
}

/// Get S3 storage configuration with unsafe defaults for development only
/// WARNING: This function uses insecure defaults and should only be used in development
pub fn config_from_env_dev() -> S3StorageConfig {
    S3StorageConfig {
        endpoint: std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "localhost:9000".to_string()),
        access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
        secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
        bucket: std::env::var("S3_BUCKET").unwrap_or_else(|_| "miniwiki-files".to_string()),
        region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        use_ssl: std::env::var("S3_USE_SSL")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false),
    }
}

/// Get S3 storage configuration with custom environment accessor
/// This allows for testing without mutating global environment variables
pub fn config_from_env_with<F>(get_var: F) -> S3StorageConfig
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    S3StorageConfig {
        endpoint: get_var("S3_ENDPOINT").unwrap_or_else(|_| "localhost:9000".to_string()),
        access_key: get_var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
        secret_key: get_var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
        bucket: get_var("S3_BUCKET").unwrap_or_else(|_| "miniwiki-files".to_string()),
        region: get_var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        use_ssl: get_var("S3_USE_SSL")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_display() {
        let error = StorageError::ConnectionFailed("connection refused".to_string());
        assert_eq!(error.to_string(), "Connection failed: connection refused");

        let error = StorageError::FileNotFound("test.txt".to_string());
        assert_eq!(error.to_string(), "File not found: test.txt");

        let error = StorageError::ChecksumMismatch;
        assert_eq!(error.to_string(), "Checksum mismatch");
    }

    #[test]
    fn test_chunked_upload_session_creation() {
        let session = ChunkedUploadSession {
            upload_id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            document_id: Some(Uuid::new_v4()),
            file_name: "test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            total_size: 1_000_000,
            chunk_size: 100_000,
            total_chunks: 10,
            uploaded_chunks: Vec::new(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        assert_eq!(session.file_name, "test.pdf");
        assert_eq!(session.total_chunks, 10);
        assert!(session.uploaded_chunks.is_empty());
    }

    #[test]
    fn test_s3_storage_config_creation() {
        let config = S3StorageConfig {
            endpoint: "localhost:9000".to_string(),
            access_key: "test_key".to_string(),
            secret_key: "test_secret".to_string(),
            bucket: "test-bucket".to_string(),
            region: "us-east-1".to_string(),
            use_ssl: true,
        };

        assert_eq!(config.endpoint, "localhost:9000");
        assert!(config.use_ssl);
    }

    #[test]
    fn test_validate_file_type_valid_images() {
        assert!(S3Storage::validate_file_type("image/png").is_ok());
        assert!(S3Storage::validate_file_type("image/jpeg").is_ok());
        assert!(S3Storage::validate_file_type("image/gif").is_ok());
        assert!(S3Storage::validate_file_type("image/webp").is_ok());
    }

    #[test]
    fn test_validate_file_type_valid_pdf() {
        assert!(S3Storage::validate_file_type("application/pdf").is_ok());
    }

    #[test]
    fn test_validate_file_type_valid_text() {
        assert!(S3Storage::validate_file_type("text/plain").is_ok());
        assert!(S3Storage::validate_file_type("text/html").is_ok());
        assert!(S3Storage::validate_file_type("text/css").is_ok());
        assert!(S3Storage::validate_file_type("text/csv").is_ok());
    }

    #[test]
    fn test_validate_file_type_valid_video() {
        assert!(S3Storage::validate_file_type("video/mp4").is_ok());
        assert!(S3Storage::validate_file_type("video/webm").is_ok());
    }

    #[test]
    fn test_validate_file_type_valid_audio() {
        assert!(S3Storage::validate_file_type("audio/mpeg").is_ok());
        assert!(S3Storage::validate_file_type("audio/ogg").is_ok());
    }

    #[test]
    fn test_validate_file_type_invalid() {
        assert!(S3Storage::validate_file_type("application/octet-stream").is_err());
        assert!(S3Storage::validate_file_type("application/x-executable").is_err());
        // text/x-python is actually allowed (starts with "text/")
        // Use a vendor-specific type instead
        assert!(S3Storage::validate_file_type("application/vnd.ms-excel").is_err());
    }

    #[test]
    fn test_validate_file_size_valid() {
        assert!(S3Storage::validate_file_size(1_000_000, 10_000_000).is_ok());
        assert!(S3Storage::validate_file_size(0, 10_000_000).is_ok());
        assert!(S3Storage::validate_file_size(10_000_000, 10_000_000).is_ok());
    }

    #[test]
    fn test_validate_file_size_invalid() {
        let result = S3Storage::validate_file_size(10_000_001, 10_000_000);
        assert!(result.is_err());
        match result {
            Err(StorageError::FileTooLarge(size, max)) => {
                assert_eq!(size, 10_000_001);
                assert_eq!(max, 10_000_000);
            },
            _ => panic!("Expected StorageError::FileTooLarge"),
        }
    }

    #[test]
    fn test_config_from_env_defaults() {
        // Use a closure that always returns Err, simulating missing env vars
        let mock_env = |_key: &str| -> Result<String, std::env::VarError> {
            Err(std::env::VarError::NotPresent)
        };

        let config = config_from_env_with(&mock_env);
        assert_eq!(config.endpoint, "localhost:9000");
        assert_eq!(config.access_key, "minioadmin");
        assert_eq!(config.secret_key, "minioadmin");
        assert_eq!(config.bucket, "miniwiki-files");
        assert_eq!(config.region, "us-east-1");
        assert!(!config.use_ssl);
    }
}
