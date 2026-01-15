use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc, DateTime};
use sqlx::FromRow;

/// File entity from database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub space_id: Uuid,
    pub document_id: Option<Uuid>,
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub storage_path: String,
    pub storage_bucket: String,
    pub checksum: String,
    pub is_deleted: bool,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

/// File with uploader info (for detail responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDetail {
    pub file: File,
    pub uploaded_by: UploaderInfo,
}

/// Uploader user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploaderInfo {
    pub id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

/// Pagination query for file list
#[derive(Debug, Deserialize)]
pub struct FileListQuery {
    pub document_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// File list response
#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub files: Vec<FileResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Upload response
#[derive(Debug, Serialize, Deserialize)]
pub struct FileResponse {
    pub id: Uuid,
    pub space_id: Uuid,
    pub document_id: Option<Uuid>,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub download_url: String,
    pub created_at: NaiveDateTime,
}

/// Detailed file response
#[derive(Debug, Serialize, Deserialize)]
pub struct FileDetailResponse {
    pub file: FileResponse,
    pub uploaded_by: UploaderInfo,
    pub checksum: String,
    pub storage_path: String,
    pub deleted_at: Option<NaiveDateTime>,
}

/// Chunked upload session response
#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkedUploadInitResponse {
    pub upload_id: Uuid,
    pub upload_url: Option<String>,
    pub chunk_size: u64,
    pub total_chunks: u32,
    pub expires_at: NaiveDateTime,
}

/// Chunk upload response
#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkUploadResponse {
    pub chunk_number: u32,
    pub uploaded_bytes: u64,
    pub chunks_uploaded: u32,
    pub total_chunks: u32,
    pub expires_at: NaiveDateTime,
}

/// Presigned URL response
#[derive(Debug, Serialize, Deserialize)]
pub struct PresignedUrlResponse {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub expires_in: i32,
    pub expires_at: DateTime<Utc>,
}

/// Message response
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message: String,
}

/// Bulk delete response
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkDeleteResponse {
    pub deleted: Vec<Uuid>,
    pub failed: Vec<FailedDelete>,
}

/// Failed delete item
#[derive(Debug, Serialize, Deserialize)]
pub struct FailedDelete {
    pub file_id: Uuid,
    pub reason: String,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Request to initialize chunked upload
#[derive(Debug, Deserialize)]
pub struct InitChunkedUploadRequest {
    pub space_id: Uuid,
    pub document_id: Option<Uuid>,
    pub file_name: String,
    pub content_type: String,
    pub total_size: u64,
    pub chunk_size: Option<u64>,
}

/// Request to upload a chunk
#[derive(Debug, Deserialize)]
pub struct UploadChunkRequest {
    pub chunk_number: u32,
    pub content: Vec<u8>,
}

/// Request to complete chunked upload
#[derive(Debug, Deserialize)]
pub struct CompleteChunkedUploadRequest {
    pub total_size: u64,
    pub checksum: String,
}

/// Request for presigned upload URL
#[derive(Debug, Deserialize)]
pub struct PresignedUploadRequest {
    pub space_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub expires_in: Option<i32>,
}

/// Request for presigned download URL
#[derive(Debug, Deserialize)]
pub struct PresignedDownloadRequest {
    pub expires_in: Option<i32>,
}

/// Request to bulk delete files
#[derive(Debug, Deserialize)]
pub struct BulkDeleteRequest {
    pub file_ids: Vec<Uuid>,
}
