use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;
use aws_sdk_s3 as s3;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedPart};
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Region};
use aws_config::BehaviorVersion;
use aws_credential_types::credentials::Credentials;
use std::time::Duration;

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
    config: S3StorageConfig,
}

impl S3Storage {
    /// Create new S3 storage client
    pub async fn new(config: S3StorageConfig) -> Result<Self, StorageError> {
        let region = Region::new(config.region.clone());
        
        // Load AWS config from environment
        let aws_config: SdkConfig = aws_config::load_from_env().await;
        
        // Create credentials
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "miniwiki",
        );

        // Build S3 config with custom endpoint
        let sdk_config = S3ConfigBuilder::from(&aws_config)
            .region(region)
            .endpoint_url(
                if config.use_ssl {
                    format!("https://{}", config.endpoint)
                } else {
                    format!("http://{}", config.endpoint)
                }
            )
            .force_path_style(true)
            .credentials_provider(credentials)
            .build();

        let client = S3Client::from_conf(sdk_config);

        // Verify bucket exists
        let _ = client.head_bucket()
            .bucket(&config.bucket)
            .send()
            .await
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
            config,
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

        let presigned = self.client.put_object()
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

        let presigned = self.client.get_object()
            .bucket(&self.bucket)
            .key(object_name)
            .presigned(presigning_config)
            .await
            .map_err(|e| StorageError::DownloadFailed(e.to_string()))?;

        Ok(presigned.uri().to_string())
    }

    /// Upload file
    pub async fn upload_file(
        &self,
        object_name: &str,
        content: &[u8],
        content_type: &str,
    ) -> Result<(), StorageError> {
        let byte_stream = ByteStream::from(content.to_vec());

        self.client.put_object()
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

        let result = self.client.upload_part()
            .bucket(&self.bucket)
            .key(object_name)
            .upload_id(upload_id)
            .part_number(chunk_number as i32)
            .body(byte_stream)
            .send()
            .await
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;

        Ok(CompletedPart::builder()
            .part_number(chunk_number as i32)
            .e_tag(result.e_tag.unwrap_or_default())
            .build())
    }

    /// Download file
    pub async fn download_file(
        &self,
        object_name: &str,
    ) -> Result<Vec<u8>, StorageError> {
        let result = self.client.get_object()
            .bucket(&self.bucket)
            .key(object_name)
            .send()
            .await
            .map_err(|e| StorageError::DownloadFailed(e.to_string()))?;

        let data = result.body.collect().await
            .map_err(|e| StorageError::DownloadFailed(e.to_string()))?;
        
        Ok(data.to_vec())
    }

    /// Delete file
    pub async fn delete_file(
        &self,
        object_name: &str,
    ) -> Result<(), StorageError> {
        self.client.delete_object()
            .bucket(&self.bucket)
            .key(object_name)
            .send()
            .await
            .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;

        Ok(())
    }

    /// Check if file exists
    pub async fn file_exists(
        &self,
        object_name: &str,
    ) -> Result<bool, StorageError> {
        match self.client.head_object()
            .bucket(&self.bucket)
            .key(object_name)
            .send()
            .await {
            Ok(_) => Ok(true),
            Err(e) => {
                match e.into_service_error() {
                    aws_sdk_s3::operation::head_object::HeadObjectError::NotFound(_) => Ok(false),
                    other => Err(StorageError::DownloadFailed(other.to_string()))
                }
            }
        }
    }

    /// Generate storage path for file
    pub fn generate_storage_path(
        &self,
        space_id: Uuid,
        file_id: Uuid,
        file_name: &str,
    ) -> String {
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let extension = std::path::Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        if extension.is_empty() {
            format!("{}/{}/{}", space_id, file_id, date)
        } else {
            format!("{}/{}/{}/{}", space_id, file_id, date, file_name)
        }
    }

    /// Validate file type
    pub fn validate_file_type(content_type: &str) -> Result<(), StorageError> {
        const ALLOWED_TYPES: &[&str] = &[
            "image/",
            "application/pdf",
            "text/",
            "video/",
            "audio/",
        ];

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

/// Create S3 storage from environment
impl S3Storage {
    pub fn from_env() -> Result<Self, StorageError> {
        let config = S3StorageConfig {
            endpoint: std::env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "localhost:9000".to_string()),
            access_key: std::env::var("S3_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("S3_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            bucket: std::env::var("S3_BUCKET")
                .unwrap_or_else(|_| "miniwiki-files".to_string()),
            region: std::env::var("S3_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            use_ssl: std::env::var("S3_USE_SSL")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        };

        Err(StorageError::ConnectionFailed(
            "S3Storage::from_env() requires async runtime. Use S3Storage::new() instead.".to_string()
        ))
    }
}
