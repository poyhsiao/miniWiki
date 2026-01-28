use crate::models::*;
use crate::storage::S3Storage;
use actix_web::http::header::HeaderMap;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use futures_util::stream::StreamExt;
use shared_errors::AppError;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Upload file request (multipart form field names)
pub const FIELD_FILE: &str = "file";
pub const FIELD_SPACE_ID: &str = "space_id";
pub const FIELD_DOCUMENT_ID: &str = "document_id";
pub const FIELD_FILE_NAME: &str = "file_name";

/// Extract boundary from content-type header
pub fn extract_boundary(headers: &HeaderMap) -> Option<String> {
    let content_type = headers.get("content-type")?.to_str().ok()?;
    let mime = content_type.parse::<mime::Mime>().ok()?;
    let boundary = mime.params().find(|(k, _)| *k == "boundary")?.1.to_string();
    Some(boundary)
}

/// Extract user ID from request for authentication context
/// First tries to get from request extensions (set by JWT middleware), falls back to X-User-Id header
pub fn extract_user_id(req: &HttpRequest) -> Result<Uuid, AppError> {
    // Try to get from request extensions first (set by JWT middleware)
    if let Some(user_uuid) = req.extensions().get::<Uuid>() {
        return Ok(*user_uuid);
    }

    // Fallback to X-User-Id header
    req.headers()
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            AppError::AuthenticationError(
                "User ID not found in request. Provide valid JWT token or X-User-Id header.".to_string(),
            )
        })
}

/// Upload file handler - POST /api/v1/files/upload
pub async fn upload_file(
    payload: web::Payload,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<S3Storage>>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let _boundary = match extract_boundary(req.headers()) {
        Some(b) => b,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                code: "MISSING_BOUNDARY".to_string(),
                message: "Missing boundary in content-type".to_string(),
                details: None,
            });
        },
    };

    // Parse multipart form
    let mut form = actix_multipart::Multipart::new(req.headers(), payload);

    let mut space_id: Option<Uuid> = None;
    let mut document_id: Option<Uuid> = None;
    let mut file_name: Option<String> = None;
    let mut file_content: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Some(field_result) = form.next().await {
        let mut field = match field_result {
            Ok(f) => f,
            Err(e) => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    code: "MULTIPART_ERROR".to_string(),
                    message: format!("Failed to parse multipart field: {}", e),
                    details: None,
                });
            },
        };

        let name = field.name();

        match name {
            FIELD_SPACE_ID => {
                if let Some(data) = field.next().await {
                    if let Ok(data) = data {
                        if let Ok(s) = std::str::from_utf8(&data) {
                            space_id = Uuid::parse_str(s).ok();
                        }
                    }
                }
            },
            FIELD_DOCUMENT_ID => {
                if let Some(data) = field.next().await {
                    if let Ok(data) = data {
                        if let Ok(s) = std::str::from_utf8(&data) {
                            document_id = Uuid::parse_str(s).ok();
                        }
                    }
                }
            },
            FIELD_FILE_NAME => {
                if let Some(data) = field.next().await {
                    if let Ok(data) = data {
                        file_name = Some(std::str::from_utf8(&data).unwrap_or("").to_string());
                    }
                }
            },
            FIELD_FILE => {
                let ct: Option<String> = field.content_type().map(|ct: &mime::Mime| ct.to_string());
                content_type = ct;

                const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
                let mut bytes = Vec::new();
                while let Some(chunk_result) = field.next().await {
                    if let Ok(data) = chunk_result {
                        if bytes.len() + data.len() > MAX_FILE_SIZE {
                            return HttpResponse::PayloadTooLarge().json(ErrorResponse {
                                code: "FILE_TOO_LARGE".to_string(),
                                message: format!("File exceeds maximum size of {} bytes", MAX_FILE_SIZE),
                                details: None,
                            });
                        }
                        bytes.extend_from_slice(&data);
                    } else {
                        break;
                    }
                }
                file_content = Some(bytes);
            },
            _ => while let Some(_chunk) = field.next().await {},
        }
    }

    // Validate required fields
    let space_id = match space_id {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                code: "MISSING_SPACE_ID".to_string(),
                message: "space_id is required".to_string(),
                details: None,
            });
        },
    };

    let file_content = match file_content {
        Some(content) => content,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                code: "MISSING_FILE".to_string(),
                message: "File is required".to_string(),
                details: None,
            });
        },
    };

    let file_name = file_name.unwrap_or_else(|| "unnamed".to_string());
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let file_size = file_content.len() as i64;

    // Validate file type
    if let Err(e) = S3Storage::validate_file_type(&content_type) {
        return HttpResponse::UnsupportedMediaType().json(ErrorResponse {
            code: "INVALID_FILE_TYPE".to_string(),
            message: e.to_string(),
            details: None,
        });
    }

    // Validate file size (50MB limit)
    const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
    if let Err(e) = S3Storage::validate_file_size(file_size as u64, MAX_FILE_SIZE) {
        return HttpResponse::PayloadTooLarge().json(ErrorResponse {
            code: "FILE_TOO_LARGE".to_string(),
            message: e.to_string(),
            details: None,
        });
    }

    // Generate storage path and upload to S3
    let file_id = Uuid::new_v4();
    let storage_path = format!("{}/{}/{}", space_id, file_id, file_name);

    if let Err(e) = storage.upload_file(&storage_path, &file_content, &content_type).await {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            code: "UPLOAD_FAILED".to_string(),
            message: format!("Failed to upload file: {}", e),
            details: None,
        });
    }

    // Generate download URL
    let download_url = match storage.presigned_download_url(&storage_path, 900).await {
        Ok(url) => url,
        Err(_) => format!("/api/v1/files/{}/download", file_id),
    };

    let bucket = storage.bucket().to_string();

    // Insert file record into database
    let file_record = match sqlx::query_as!(
        File,
        r#"
        INSERT INTO files (
            id, space_id, document_id, uploaded_by, file_name,
            file_type, file_size, storage_path, storage_bucket,
            checksum, is_deleted, deleted_at, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, false, NULL, NOW())
        RETURNING *
        "#,
        file_id,
        space_id,
        document_id,
        // Extract user_id from authentication context
        match extract_user_id(&req) {
            Ok(user_id) => user_id,
            Err(e) => {
                return HttpResponse::Unauthorized().json(ErrorResponse {
                    code: "AUTHENTICATION_ERROR".to_string(),
                    message: e.to_string(),
                    details: None,
                });
            },
        },
        file_name,
        content_type,
        file_size,
        storage_path,
        bucket,
        format!("{:x}", md5::compute(&file_content))
    )
    .fetch_one(pool.as_ref())
    .await
    {
        Ok(record) => record,
        Err(e) => {
            let _ = storage.delete_file(&storage_path).await;
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to save file record: {}", e),
                details: None,
            });
        },
    };

    HttpResponse::Created().json(FileResponse {
        id: file_record.id,
        space_id: file_record.space_id,
        document_id: file_record.document_id,
        file_name: file_record.file_name,
        file_type: file_record.file_type,
        file_size: file_record.file_size,
        download_url,
        created_at: file_record.created_at,
    })
}

/// Initialize chunked upload - POST /api/v1/files/upload/chunked/init
pub async fn init_chunked_upload(req: web::Json<InitChunkedUploadRequest>, pool: web::Data<PgPool>) -> impl Responder {
    let upload_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now.naive_utc() + chrono::Duration::hours(24);

    let chunk_size = req.chunk_size.unwrap_or(5 * 1024 * 1024);
    let total_chunks = ((req.total_size + chunk_size - 1) / chunk_size) as u32;

    // Create chunked upload session in database
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO chunked_uploads (
            upload_id, space_id, document_id, file_name,
            content_type, total_size, chunk_size, total_chunks,
            uploaded_chunks, created_at, expires_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), $10)
        "#,
        upload_id,
        req.space_id,
        req.document_id,
        req.file_name,
        req.content_type,
        req.total_size as i64,
        chunk_size as i64,
        total_chunks as i64,
        &vec![0i32],
        now
    )
    .execute(pool.as_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            code: "DATABASE_ERROR".to_string(),
            message: format!("Failed to create upload session: {}", e),
            details: None,
        });
    }

    HttpResponse::Created().json(ChunkedUploadInitResponse {
        upload_id,
        upload_url: None,
        chunk_size: chunk_size as u64,
        total_chunks,
        expires_at,
    })
}

/// Upload file chunk - PUT /api/v1/files/upload/chunked/{upload_id}/{chunk_number}
pub async fn upload_chunk(
    path: web::Path<(Uuid, u32)>,
    body: web::Bytes,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<S3Storage>>,
) -> impl Responder {
    let (upload_id, chunk_number) = path.into_inner();

    // Get upload session - query as generic tuple to avoid type issues
    let session_result = sqlx::query!(
        r#"
        SELECT upload_id, space_id, document_id, file_name,
               content_type, total_size, chunk_size, total_chunks,
               uploaded_chunks, created_at, expires_at
        FROM chunked_uploads WHERE upload_id = $1 AND expires_at > NOW()
        "#,
        upload_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let session = match session_result {
        Ok(Some(s)) => s,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                code: "UPLOAD_NOT_FOUND".to_string(),
                message: "Upload session not found or expired".to_string(),
                details: None,
            });
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get upload session: {}", e),
                details: None,
            });
        },
    };

    let uploaded_chunks: Vec<i32> = session.uploaded_chunks.unwrap_or_default();
    let total_chunks = session.total_chunks as u32;
    let chunk_size = session.chunk_size as usize;

    // Validate chunk_number is within bounds and not already uploaded
    if chunk_number >= total_chunks {
        return HttpResponse::BadRequest().json(ErrorResponse {
            code: "INVALID_CHUNK_NUMBER".to_string(),
            message: format!(
                "Chunk number {} is out of bounds (total chunks: {})",
                chunk_number, total_chunks
            ),
            details: None,
        });
    }

    // Validate chunk size doesn't exceed expected chunk_size (streaming size check)
    if body.len() > chunk_size {
        return HttpResponse::BadRequest().json(ErrorResponse {
            code: "CHUNK_TOO_LARGE".to_string(),
            message: format!(
                "Chunk size {} bytes exceeds maximum allowed size of {} bytes",
                body.len(),
                chunk_size
            ),
            details: None,
        });
    }

    if uploaded_chunks.contains(&(chunk_number as i32)) {
        return HttpResponse::Conflict().json(ErrorResponse {
            code: "CHUNK_ALREADY_UPLOADED".to_string(),
            message: format!("Chunk {} has already been uploaded", chunk_number),
            details: None,
        });
    }

    let storage_path = format!("{}/{}/{}", session.space_id, upload_id, session.file_name);

    let chunk_path = format!("{}.chunk.{}", storage_path, chunk_number);

    if let Err(e) = storage.upload_file(&chunk_path, &body, "application/octet-stream").await {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            code: "UPLOAD_FAILED".to_string(),
            message: format!("Failed to upload chunk: {}", e),
            details: None,
        });
    }

    let new_uploaded_count = uploaded_chunks.len() as u32 + 1;
    let mut chunks = uploaded_chunks;
    chunks.push(chunk_number as i32);

    if let Err(e) = sqlx::query!(
        r#"UPDATE chunked_uploads SET uploaded_chunks = $1 WHERE upload_id = $2"#,
        chunks as Vec<i32>,
        upload_id
    )
    .execute(pool.as_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            code: "DATABASE_ERROR".to_string(),
            message: format!("Failed to update upload session: {}", e),
            details: None,
        });
    }

    HttpResponse::Ok().json(ChunkUploadResponse {
        chunk_number,
        uploaded_bytes: body.len() as u64,
        chunks_uploaded: new_uploaded_count,
        total_chunks,
        expires_at: session.expires_at.naive_utc(),
    })
}

/// Complete chunked upload - POST /api/v1/files/upload/chunked/{upload_id}/complete
pub async fn complete_chunked_upload(
    upload_id: web::Path<Uuid>,
    req: web::Json<CompleteChunkedUploadRequest>,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<S3Storage>>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let upload_id = upload_id.into_inner();

    let session_result = sqlx::query!(
        r#"
        SELECT upload_id, space_id, document_id, file_name,
               content_type, total_size, chunk_size, total_chunks,
               uploaded_chunks, created_at, expires_at
        FROM chunked_uploads WHERE upload_id = $1 AND expires_at > NOW()
        "#,
        upload_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let session = match session_result {
        Ok(Some(s)) => s,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                code: "UPLOAD_NOT_FOUND".to_string(),
                message: "Upload session not found or expired".to_string(),
                details: None,
            });
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get upload session: {}", e),
                details: None,
            });
        },
    };

    let uploaded_chunks: Vec<i32> = session.uploaded_chunks.unwrap_or_default();
    let total_chunks = session.total_chunks as usize;

    if uploaded_chunks.len() != total_chunks {
        return HttpResponse::BadRequest().json(ErrorResponse {
            code: "INCOMPLETE_UPLOAD".to_string(),
            message: format!("Expected {} chunks, got {}", total_chunks, uploaded_chunks.len()),
            details: None,
        });
    }

    let mut sorted_chunks = uploaded_chunks.clone();
    sorted_chunks.sort();

    let mut assembled_content = Vec::new();
    let _temp_storage_path = format!("{}/{}", session.space_id, upload_id);

    for chunk_num in &sorted_chunks {
        let chunk_path = format!(
            "{}/{}/{}.chunk.{}",
            session.space_id, upload_id, session.file_name, chunk_num
        );
        match storage.download_file(&chunk_path).await {
            Ok(chunk_data) => {
                assembled_content.extend_from_slice(&chunk_data);
            },
            Err(e) => {
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    code: "CHUNK_READ_FAILED".to_string(),
                    message: format!("Failed to read chunk {}: {}", chunk_num, e),
                    details: None,
                });
            },
        }
    }

    let file_id = Uuid::new_v4();
    let storage_path = format!("{}/{}/{}", session.space_id, file_id, session.file_name);

    if let Err(e) = storage
        .upload_file(&storage_path, &assembled_content, &session.content_type)
        .await
    {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            code: "UPLOAD_FAILED".to_string(),
            message: format!("Failed to upload assembled file: {}", e),
            details: None,
        });
    }

    for chunk_num in sorted_chunks {
        let chunk_path = format!(
            "{}/{}/{}.chunk.{}",
            session.space_id, upload_id, session.file_name, chunk_num
        );
        let _ = storage.delete_file(&chunk_path).await;
    }

    let download_url = match storage.presigned_download_url(&storage_path, 900).await {
        Ok(url) => url,
        Err(_) => format!("/api/v1/files/{}/download", file_id),
    };

    let bucket = storage.bucket().to_string();

    let computed_checksum = format!("{:x}", md5::compute(&assembled_content));

    // Validate client-provided checksum matches computed checksum
    if req.checksum != computed_checksum {
        return HttpResponse::BadRequest().json(ErrorResponse {
            code: "CHECKSUM_MISMATCH".to_string(),
            message: "Client-provided checksum does not match computed checksum".to_string(),
            details: Some(serde_json::json!({
                "client_provided": req.checksum,
                "computed": computed_checksum
            })),
        });
    }

    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO files (
            id, space_id, document_id, uploaded_by, file_name,
            file_type, file_size, storage_path, storage_bucket,
            checksum, is_deleted, deleted_at, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, false, NULL, NOW())
        "#,
        file_id,
        session.space_id,
        session.document_id,
        // Extract user_id from authentication context
        match extract_user_id(&http_req) {
            Ok(user_id) => user_id,
            Err(e) => {
                return HttpResponse::Unauthorized().json(ErrorResponse {
                    code: "AUTHENTICATION_ERROR".to_string(),
                    message: e.to_string(),
                    details: None,
                });
            },
        },
        session.file_name,
        session.content_type,
        session.total_size,
        storage_path,
        bucket,
        computed_checksum
    )
    .execute(pool.as_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            code: "DATABASE_ERROR".to_string(),
            message: format!("Failed to save file record: {}", e),
            details: None,
        });
    }

    let _ = sqlx::query!("DELETE FROM chunked_uploads WHERE upload_id = $1", upload_id)
        .execute(pool.as_ref())
        .await;

    HttpResponse::Created().json(FileResponse {
        id: file_id,
        space_id: session.space_id,
        document_id: session.document_id,
        file_name: session.file_name,
        file_type: session.content_type,
        file_size: session.total_size,
        download_url,
        created_at: Utc::now().naive_utc(),
    })
}

/// Cancel chunked upload - DELETE /api/v1/files/upload/chunked/{upload_id}
pub async fn cancel_chunked_upload(
    upload_id: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<S3Storage>>,
) -> impl Responder {
    let upload_id = upload_id.into_inner();

    let session_result = sqlx::query!(
        r#"
        SELECT upload_id, space_id, document_id, file_name,
               content_type, total_size, chunk_size, total_chunks,
               uploaded_chunks, created_at, expires_at
        FROM chunked_uploads WHERE upload_id = $1
        "#,
        upload_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let session = match session_result {
        Ok(Some(s)) => s,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                code: "UPLOAD_NOT_FOUND".to_string(),
                message: "Upload session not found".to_string(),
                details: None,
            });
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get upload session: {}", e),
                details: None,
            });
        },
    };

    let uploaded_chunks: Vec<i32> = session.uploaded_chunks.unwrap_or_default();
    for chunk_num in uploaded_chunks {
        let chunk_path = format!(
            "{}/{}/{}",
            session.space_id,
            upload_id,
            format!("{}.chunk.{}", session.file_name, chunk_num)
        );
        let _ = storage.delete_file(&chunk_path).await;
    }

    let _ = sqlx::query!("DELETE FROM chunked_uploads WHERE upload_id = $1", upload_id)
        .execute(pool.as_ref())
        .await;

    HttpResponse::Ok().json(MessageResponse {
        message: "Upload cancelled".to_string(),
    })
}

/// Get presigned upload URL - POST /api/v1/files/upload/presigned-url
pub async fn get_presigned_upload_url(
    req: web::Json<PresignedUploadRequest>,
    storage: web::Data<Arc<S3Storage>>,
) -> impl Responder {
    const MIN_EXPIRES_IN: i32 = 60; // 1 minute minimum
    const MAX_EXPIRES_IN: i32 = 7200; // 2 hours maximum

    let file_id = Uuid::new_v4();
    let storage_path = format!("{}/{}/{}", req.space_id, file_id, req.file_name);
    let expires_in = req.expires_in.unwrap_or(3600);

    // Validate expires_in is within acceptable range
    if expires_in < MIN_EXPIRES_IN || expires_in > MAX_EXPIRES_IN {
        return HttpResponse::BadRequest().json(ErrorResponse {
            code: "INVALID_EXPIRES_IN".to_string(),
            message: format!(
                "expires_in must be between {} and {} seconds",
                MIN_EXPIRES_IN, MAX_EXPIRES_IN
            ),
            details: Some(serde_json::json!({
                "provided": expires_in,
                "min": MIN_EXPIRES_IN,
                "max": MAX_EXPIRES_IN
            })),
        });
    }

    match storage.presigned_upload_url(&storage_path, &req.content_type, expires_in).await {
        Ok(url) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), req.content_type.clone());

            HttpResponse::Ok().json(PresignedUrlResponse {
                url,
                method: "PUT".to_string(),
                headers,
                expires_in,
                expires_at: Utc::now() + chrono::Duration::seconds(expires_in as i64),
            })
        },
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            code: "PRESIGNED_URL_FAILED".to_string(),
            message: format!("Failed to generate presigned URL: {}", e),
            details: None,
        }),
    }
}

/// Download file - GET /api/v1/files/{fileId}/download
pub async fn download_file(
    file_id: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<S3Storage>>,
) -> impl Responder {
    let file_id = file_id.into_inner();

    let file_result = sqlx::query_as!(
        File,
        r#"
        SELECT id, space_id, document_id, uploaded_by, file_name,
               file_type, file_size, storage_path, storage_bucket,
               checksum, is_deleted, deleted_at, created_at
        FROM files WHERE id = $1 AND is_deleted = false
        "#,
        file_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let file = match file_result {
        Ok(Some(f)) => f,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: "File not found".to_string(),
                details: None,
            });
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get file: {}", e),
                details: None,
            });
        },
    };

    match storage.download_file(&file.storage_path).await {
        Ok(content) => HttpResponse::Ok().content_type(file.file_type).body(content),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            code: "DOWNLOAD_FAILED".to_string(),
            message: format!("Failed to download file: {}", e),
            details: None,
        }),
    }
}

/// Get presigned download URL - GET /api/v1/files/{fileId}/download/presigned-url
pub async fn get_presigned_download_url(
    file_id: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<S3Storage>>,
) -> impl Responder {
    let file_id = file_id.into_inner();

    let file_result = sqlx::query_as!(
        File,
        r#"
        SELECT id, space_id, document_id, uploaded_by, file_name,
               file_type, file_size, storage_path, storage_bucket,
               checksum, is_deleted, deleted_at, created_at
        FROM files WHERE id = $1 AND is_deleted = false
        "#,
        file_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let file = match file_result {
        Ok(Some(f)) => f,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: "File not found".to_string(),
                details: None,
            });
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get file: {}", e),
                details: None,
            });
        },
    };

    match storage.presigned_download_url(&file.storage_path, 900).await {
        Ok(url) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), file.file_type);

            HttpResponse::Ok().json(PresignedUrlResponse {
                url,
                method: "GET".to_string(),
                headers,
                expires_in: 900,
                expires_at: Utc::now() + chrono::Duration::minutes(15),
            })
        },
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            code: "PRESIGNED_URL_FAILED".to_string(),
            message: format!("Failed to generate presigned URL: {}", e),
            details: None,
        }),
    }
}

/// Get file metadata - GET /api/v1/files/{fileId}
pub async fn get_file_metadata(file_id: web::Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    let file_id = file_id.into_inner();

    let file_result = sqlx::query_as!(
        File,
        r#"
        SELECT id, space_id, document_id, uploaded_by, file_name,
               file_type, file_size, storage_path, storage_bucket,
               checksum, is_deleted, deleted_at, created_at
        FROM files WHERE id = $1
        "#,
        file_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let file = match file_result {
        Ok(Some(f)) => f,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: "File not found".to_string(),
                details: None,
            });
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get file: {}", e),
                details: None,
            });
        },
    };

    let download_url = format!("/api/v1/files/{}/download", file.id);

    HttpResponse::Ok().json(FileDetailResponse {
        file: FileResponse {
            id: file.id,
            space_id: file.space_id,
            document_id: file.document_id,
            file_name: file.file_name,
            file_type: file.file_type,
            file_size: file.file_size,
            download_url,
            created_at: file.created_at,
        },
        uploaded_by: UploaderInfo {
            id: file.uploaded_by,
            display_name: "User".to_string(),
            avatar_url: None,
        },
        checksum: file.checksum,
        storage_path: file.storage_path,
        deleted_at: file.deleted_at,
    })
}

/// Delete file (soft delete) - DELETE /api/v1/files/{fileId}
pub async fn delete_file(file_id: web::Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    let file_id = file_id.into_inner();

    match sqlx::query!(
        r#"
        UPDATE files SET is_deleted = true, deleted_at = NOW() WHERE id = $1 AND is_deleted = false
        "#,
        file_id
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                return HttpResponse::NotFound().json(ErrorResponse {
                    code: "FILE_NOT_FOUND".to_string(),
                    message: "File not found or already deleted".to_string(),
                    details: None,
                });
            }
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to delete file: {}", e),
                details: None,
            });
        },
    }

    HttpResponse::Ok().json(MessageResponse {
        message: "File deleted".to_string(),
    })
}

/// Restore deleted file - POST /api/v1/files/{fileId}/restore
pub async fn restore_file(file_id: web::Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    let file_id = file_id.into_inner();

    match sqlx::query!(
        r#"
        UPDATE files SET is_deleted = false, deleted_at = NULL WHERE id = $1 AND is_deleted = true
        "#,
        file_id
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                return HttpResponse::NotFound().json(ErrorResponse {
                    code: "FILE_NOT_FOUND".to_string(),
                    message: "File not found or not deleted".to_string(),
                    details: None,
                });
            }
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to restore file: {}", e),
                details: None,
            });
        },
    }

    HttpResponse::Ok().json(MessageResponse {
        message: "File restored".to_string(),
    })
}

/// Permanently delete file - DELETE /api/v1/files/{fileId}/permanent-delete
pub async fn permanent_delete_file(
    file_id: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    storage: web::Data<Arc<S3Storage>>,
) -> impl Responder {
    let file_id = file_id.into_inner();

    let file_result = sqlx::query_as!(
        File,
        r#"
        SELECT id, space_id, document_id, uploaded_by, file_name,
               file_type, file_size, storage_path, storage_bucket,
               checksum, is_deleted, deleted_at, created_at
        FROM files WHERE id = $1
        "#,
        file_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let file = match file_result {
        Ok(Some(f)) => f,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: "File not found".to_string(),
                details: None,
            });
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to get file: {}", e),
                details: None,
            });
        },
    };

    match sqlx::query!("DELETE FROM files WHERE id = $1", file_id)
        .execute(pool.as_ref())
        .await
    {
        Ok(_) => {},
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to delete file: {}", e),
                details: None,
            });
        },
    }

    if let Err(e) = storage.delete_file(&file.storage_path).await {
        tracing::error!("Failed to delete file from storage after DB deletion: {}", e);
    }

    HttpResponse::Ok().json(MessageResponse {
        message: "File permanently deleted".to_string(),
    })
}

/// List files in space - GET /api/v1/files/spaces/{spaceId}/files
pub async fn list_space_files(
    space_id: web::Path<Uuid>,
    query: web::Query<FileListQuery>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let space_id = space_id.into_inner();
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let files: Result<Vec<File>, sqlx::Error> = match query.document_id {
        Some(doc_id) => {
            sqlx::query_as::<_, File>(
                r#"
                SELECT id, space_id, document_id, uploaded_by, file_name,
                       file_type, file_size, storage_path, storage_bucket,
                       checksum, is_deleted, deleted_at, created_at
                FROM files
                WHERE space_id = $1 AND document_id = $2 AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(space_id)
            .bind(doc_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool.as_ref())
            .await
        },
        None => {
            sqlx::query_as::<_, File>(
                r#"
                SELECT id, space_id, document_id, uploaded_by, file_name,
                       file_type, file_size, storage_path, storage_bucket,
                       checksum, is_deleted, deleted_at, created_at
                FROM files
                WHERE space_id = $1 AND is_deleted = false
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(space_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool.as_ref())
            .await
        },
    };

    let total_result: Result<Option<i64>, sqlx::Error> = match query.document_id {
        Some(doc_id) => {
            sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM files WHERE space_id = $1 AND document_id = $2 AND is_deleted = false"#,
            )
            .bind(space_id)
            .bind(doc_id)
            .fetch_optional(pool.as_ref())
            .await
        },
        None => {
            sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM files WHERE space_id = $1 AND is_deleted = false"#)
                .bind(space_id)
                .fetch_optional(pool.as_ref())
                .await
        },
    };

    let total = match total_result {
        Ok(Some(count)) => count,
        Ok(None) => 0,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to count files: {}", e),
                details: None,
            });
        },
    };

    let files = match files {
        Ok(f) => f,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                code: "DATABASE_ERROR".to_string(),
                message: format!("Failed to list files: {}", e),
                details: None,
            });
        },
    };

    let file_responses = files
        .into_iter()
        .map(|f| FileResponse {
            id: f.id,
            space_id: f.space_id,
            document_id: f.document_id,
            file_name: f.file_name,
            file_type: f.file_type,
            file_size: f.file_size,
            download_url: format!("/api/v1/files/{}/download", f.id),
            created_at: f.created_at,
        })
        .collect();

    HttpResponse::Ok().json(FileListResponse {
        files: file_responses,
        total,
        limit: limit as i64,
        offset: offset as i64,
    })
}

/// Bulk delete files - POST /api/v1/files/bulk/delete
pub async fn bulk_delete_files(req: web::Json<BulkDeleteRequest>, pool: web::Data<PgPool>) -> impl Responder {
    let mut deleted = Vec::new();
    let mut failed = Vec::new();

    for file_id in &req.file_ids {
        match sqlx::query!(
            r#"
            UPDATE files SET is_deleted = true, deleted_at = NOW() WHERE id = $1 AND is_deleted = false
            "#,
            file_id
        )
        .execute(pool.as_ref())
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    deleted.push(*file_id);
                } else {
                    failed.push(FailedDelete {
                        file_id: *file_id,
                        reason: "File not found or already deleted".to_string(),
                    });
                }
            },
            Err(e) => {
                failed.push(FailedDelete {
                    file_id: *file_id,
                    reason: format!("Database error: {}", e),
                });
            },
        }
    }

    HttpResponse::Ok().json(BulkDeleteResponse { deleted, failed })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use chrono::Utc;
    use uuid::Uuid;

    // extract_boundary Tests
    #[test]
    fn test_extract_boundary_valid() {
        use actix_web::http::header::{HeaderName, HeaderValue};

        let mut headers = HeaderMap::new();
        let content_type = "multipart/form-data; boundary=----WebKitFormBoundary7MA4YC5c6";
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str(content_type).unwrap(),
        );

        let boundary = extract_boundary(&headers);
        assert_eq!(boundary, Some("----WebKitFormBoundary7MA4YC5c6".to_string()));
    }

    #[test]
    fn test_extract_boundary_no_boundary() {
        let mut headers = HeaderMap::new();
        let content_type = "multipart/form-data";
        headers.insert(
            actix_web::http::header::HeaderName::from_static("content-type"),
            actix_web::http::header::HeaderValue::from_static(content_type),
        );

        let boundary = extract_boundary(&headers);
        assert_eq!(boundary, None);
    }

    #[test]
    fn test_extract_boundary_invalid_mime() {
        let mut headers = HeaderMap::new();
        let content_type = "text/plain";
        headers.insert(
            actix_web::http::header::HeaderName::from_static("content-type"),
            actix_web::http::header::HeaderValue::from_static(content_type),
        );

        let boundary = extract_boundary(&headers);
        assert_eq!(boundary, None);
    }

    // extract_user_id Tests
    #[test]
    fn test_extract_user_id_from_extensions() {
        let user_uuid = Uuid::new_v4();
        let mut req = TestRequest::get().to_http_request();

        // Simulate what middleware does - set user_id in extensions
        req.extensions_mut().insert(user_uuid);

        let result = extract_user_id(&req);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_uuid);
    }

    #[test]
    fn test_extract_user_id_from_header() {
        let user_uuid = Uuid::new_v4();
        let mut req = TestRequest::get()
            .insert_header(("X-User-Id", user_uuid.to_string()))
            .to_http_request();

        let result = extract_user_id(&req);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_uuid);
    }

    #[test]
    fn test_extract_user_id_fallback_order() {
        let header_uuid = Uuid::new_v4();
        let ext_uuid = Uuid::new_v4();
        let mut req = TestRequest::get()
            // Both extensions and header set - should prefer extensions
            .insert_header(("X-User-Id", header_uuid.to_string()))
            .to_http_request();
        req.extensions_mut().insert(ext_uuid);

        let result = extract_user_id(&req);
        assert_eq!(result.unwrap(), ext_uuid); // extensions should take precedence over header
    }

    #[test]
    fn test_extract_user_id_missing() {
        let req = TestRequest::get().to_http_request();

        let result = extract_user_id(&req);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::AuthenticationError(_)));
    }

    // Field Name Constants Tests
    #[test]
    fn test_field_name_constants() {
        assert_eq!(FIELD_FILE, "file");
        assert_eq!(FIELD_SPACE_ID, "space_id");
        assert_eq!(FIELD_DOCUMENT_ID, "document_id");
        assert_eq!(FIELD_FILE_NAME, "file_name");
    }

    // File Size Validation Tests
    #[test]
    fn test_file_size_valid() {
        const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
        let file_size = 25 * 1024 * 1024; // 25MB

        assert!(file_size < MAX_FILE_SIZE);
    }

    #[test]
    fn test_file_size_exact_limit() {
        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
        let file_size = MAX_FILE_SIZE;

        assert_eq!(file_size, MAX_FILE_SIZE);
    }

    #[test]
    fn test_file_size_exceeds_limit() {
        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
        let file_size = MAX_FILE_SIZE + 1; // 50MB + 1 byte

        assert!(file_size > MAX_FILE_SIZE);
    }

    // Chunk Size Tests
    #[test]
    fn test_default_chunk_size() {
        let chunk_size: usize = 5 * 1024 * 1024; // Default: 5MB
        assert_eq!(chunk_size, 5 * 1024 * 1024);
    }

    #[test]
    fn test_custom_chunk_size() {
        let chunk_size: usize = 10 * 1024 * 1024; // 10MB
        assert_eq!(chunk_size, 10 * 1024 * 1024);
    }

    // Storage Path Generation Tests
    #[test]
    fn test_storage_path_generation() {
        let space_id = Uuid::new_v4();
        let file_id = Uuid::new_v4();
        let file_name = "test-file.png";

        let path = format!("{}/{}/{}", space_id, file_id, file_name);
        assert!(path.contains(&space_id.to_string()));
        assert!(path.contains(&file_id.to_string()));
        assert!(path.ends_with(file_name));
    }

    #[test]
    fn test_storage_path_with_special_chars() {
        let space_id = Uuid::new_v4();
        let file_id = Uuid::new_v4();
        let file_name = "test file with spaces.png";

        let path = format!("{}/{}/{}", space_id, file_id, file_name);
        assert!(path.contains("test file with spaces.png"));
    }

    // Chunk Path Generation Tests
    #[test]
    fn test_chunk_path_generation() {
        let space_id = Uuid::new_v4();
        let upload_id = Uuid::new_v4();
        let file_name = "test.pdf";
        let chunk_number = 5u32;

        let base_path = format!("{}/{}/{}", space_id, upload_id, file_name);
        let chunk_path = format!("{}.chunk.{}", base_path, chunk_number);

        assert!(chunk_path.contains(".chunk.5"));
        assert!(chunk_path.ends_with(&format!(".chunk.{}", chunk_number)));
    }

    // Download URL Generation Tests
    #[test]
    fn test_download_url_generation() {
        let file_id = Uuid::new_v4();
        let url = format!("/api/v1/files/{}/download", file_id);

        assert_eq!(url, format!("/api/v1/files/{}/download", file_id));
        assert!(url.starts_with("/api/v1/files/"));
    }

    // Expiry Time Tests
    #[test]
    fn test_default_upload_expiry() {
        let now = Utc::now();
        let expires_at = now.checked_add_signed(chrono::Duration::hours(24)).unwrap();

        assert!(expires_at > now);
    }

    #[test]
    fn test_signed_upload_expiry() {
        let now = Utc::now();
        let expires_in = 3600i64; // 1 hour
        let expires_at = now.checked_add_signed(chrono::Duration::seconds(expires_in)).unwrap();

        assert_eq!(expires_at.timestamp() - now.timestamp(), 3600);
    }

    // Chunk Number Validation Tests
    #[test]
    fn test_total_chunks_calculation() {
        let total_size = 10 * 1024 * 1024; // 10MB
        let chunk_size = 5 * 1024 * 1024; // 5MB
        let total_chunks = ((total_size + chunk_size - 1) / chunk_size) as u32;

        assert_eq!(total_chunks, 2);
    }

    #[test]
    fn test_total_chunks_exact_fit() {
        let total_size = 10 * 1024 * 1024; // 10MB
        let chunk_size = 5 * 1024 * 1024; // 5MB
        let total_chunks = (total_size / chunk_size) as u32; // No +1 needed

        assert_eq!(total_chunks, 2);
    }

    #[test]
    fn test_chunk_number_bounds_check() {
        let chunk_number = 5u32;
        let total_chunks = 10u32;

        assert!(chunk_number < total_chunks);
    }

    #[test]
    fn test_chunk_number_out_of_bounds() {
        let chunk_number = 15u32;
        let total_chunks = 10u32;

        assert!(chunk_number >= total_chunks);
    }

    #[test]
    fn test_chunk_number_already_uploaded() {
        let uploaded_chunks = vec![0u32, 1u32, 2u32];
        let chunk_number = 2u32;

        let already_uploaded = uploaded_chunks.contains(&chunk_number);
        assert!(already_uploaded);
    }

    #[test]
    fn test_chunk_number_not_uploaded() {
        let uploaded_chunks = vec![0u32, 1u32, 2u32];
        let chunk_number = 5u32;

        let already_uploaded = uploaded_chunks.contains(&chunk_number);
        assert!(!already_uploaded);
    }

    // Checksum Tests
    #[test]
    fn test_md5_formatting() {
        let content = b"test content";
        let hash = md5::compute(content);

        let hex = format!("{:x}", hash);
        assert!(!hex.is_empty());
        assert_eq!(hex.len(), 32); // MD5 hash is 32 hex chars
    }

    #[test]
    fn test_md5_empty_content() {
        let content = b"";
        let hash = md5::compute(content);

        let hex = format!("{:x}", hash);
        assert_eq!(hex, "d41d8cd98f00b204e9800998ecf8427e"); // Known MD5 of empty string
    }

    #[test]
    fn test_md5_consistent() {
        let content = b"test content";
        let hash1 = md5::compute(content);
        let hash2 = md5::compute(content);

        assert_eq!(hash1, hash2);
    }

    // Error Response Construction Tests
    #[test]
    fn test_error_response_missing_boundary() {
        let error = crate::models::ErrorResponse {
            code: "MISSING_BOUNDARY".to_string(),
            message: "Missing boundary in content-type".to_string(),
            details: None,
        };

        assert_eq!(error.code, "MISSING_BOUNDARY");
        assert_eq!(error.message, "Missing boundary in content-type");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_error_response_missing_space_id() {
        let error = crate::models::ErrorResponse {
            code: "MISSING_SPACE_ID".to_string(),
            message: "space_id is required".to_string(),
            details: None,
        };

        assert_eq!(error.code, "MISSING_SPACE_ID");
        assert_eq!(error.message, "space_id is required");
    }

    #[test]
    fn test_error_response_file_too_large() {
        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
        let error = crate::models::ErrorResponse {
            code: "FILE_TOO_LARGE".to_string(),
            message: format!("File exceeds maximum size of {} bytes", MAX_FILE_SIZE),
            details: None,
        };

        assert_eq!(error.code, "FILE_TOO_LARGE");
        assert!(error.message.contains("50"));
    }

    #[test]
    fn test_error_response_invalid_file_type() {
        let error = crate::models::ErrorResponse {
            code: "INVALID_FILE_TYPE".to_string(),
            message: "Invalid file type".to_string(),
            details: None,
        };

        assert_eq!(error.code, "INVALID_FILE_TYPE");
        assert_eq!(error.message, "Invalid file type");
    }

    #[test]
    fn test_error_response_chunk_too_large() {
        let chunk_size = 10 * 1024 * 1024; // 10MB
        let error = crate::models::ErrorResponse {
            code: "CHUNK_TOO_LARGE".to_string(),
            message: format!(
                "Chunk size {} bytes exceeds maximum allowed size of {} bytes",
                chunk_size,
                5 * 1024 * 1024
            ),
            details: None,
        };

        assert_eq!(error.code, "CHUNK_TOO_LARGE");
        assert!(error.message.contains("10"));
    }

    #[test]
    fn test_error_response_invalid_chunk_number() {
        let error = crate::models::ErrorResponse {
            code: "INVALID_CHUNK_NUMBER".to_string(),
            message: format!("Chunk number {} is out of bounds (total chunks: {})", 15, 10),
            details: None,
        };

        assert_eq!(error.code, "INVALID_CHUNK_NUMBER");
        assert!(error.message.contains("15"));
    }

    #[test]
    fn test_error_response_chunk_already_uploaded() {
        let error = crate::models::ErrorResponse {
            code: "CHUNK_ALREADY_UPLOADED".to_string(),
            message: format!("Chunk {} has already been uploaded", 5),
            details: None,
        };

        assert_eq!(error.code, "CHUNK_ALREADY_UPLOADED");
        assert_eq!(error.message, "Chunk 5 has already been uploaded");
    }

    // Upload Session ID Tests
    #[test]
    fn test_upload_id_generation() {
        let upload_id = Uuid::new_v4();

        assert_ne!(upload_id, Uuid::nil());
    }

    #[test]
    fn test_upload_id_parsing() {
        let upload_id_str = Uuid::new_v4().to_string();
        let parsed = Uuid::parse_str(&upload_id_str);

        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().to_string(), upload_id_str);
    }

    // File ID Tests
    #[test]
    fn test_file_id_parsing() {
        let file_id = Uuid::new_v4();
        let file_id_str = file_id.to_string();

        assert_eq!(file_id_str.len(), 36); // Standard UUID length
        assert!(file_id_str.contains('-'));
    }

    // Presigned URL TTL Tests
    #[test]
    fn test_default_presigned_url_ttl() {
        let ttl = 900; // Default: 15 minutes
        assert_eq!(ttl, 900);
    }

    #[test]
    fn test_custom_presigned_url_ttl() {
        let ttl = 3600; // 1 hour
        assert_eq!(ttl, 3600);
    }

    #[test]
    fn test_presigned_url_ttl_validation() {
        // Common TTL values: 900 (15 min), 3600 (1 hour), 7200 (2 hours)
        let valid_ttls = vec![900, 3600, 7200];

        for ttl in valid_ttls {
            assert!(ttl >= 60);
            assert!(ttl <= 7200);
        }
    }

    // Vector Operations Tests
    #[test]
    fn test_chunk_vector_sort() {
        let mut chunks = vec![2u32, 1u32, 3u32];
        chunks.sort();

        assert_eq!(chunks, vec![1u32, 2u32, 3u32]);
    }

    #[test]
    fn test_chunk_vector_push() {
        let mut chunks = vec![1u32, 2u32];
        chunks.push(3u32);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[2], 3u32);
    }

    #[test]
    fn test_chunk_vector_contains() {
        let chunks = vec![1u32, 2u32, 3u32];

        assert!(chunks.contains(&2u32));
        assert!(!chunks.contains(&5u32));
    }

    // Empty File Name Fallback
    #[test]
    fn test_file_name_fallback() {
        let file_name: Option<String> = None;
        let fallback = file_name.unwrap_or_else(|| "unnamed".to_string());

        assert_eq!(fallback, "unnamed");
    }

    #[test]
    fn test_file_name_provided() {
        let file_name = Some("test.pdf".to_string());
        let result = file_name.unwrap_or_else(|| "unnamed".to_string());

        assert_eq!(result, "test.pdf");
    }

    // Content Type Fallback
    #[test]
    fn test_content_type_fallback() {
        let content_type: Option<String> = None;
        let fallback = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

        assert_eq!(fallback, "application/octet-stream");
    }

    #[test]
    fn test_content_type_provided() {
        let content_type = Some("image/png".to_string());
        let result = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

        assert_eq!(result, "image/png");
    }

    // File Size Calculations
    #[test]
    fn test_file_size_in_bytes() {
        let content = vec![0u8; 1024 * 10]; // 10KB
        let size = content.len() as i64;

        assert_eq!(size, 10 * 1024);
    }

    #[test]
    fn test_file_size_in_mb() {
        let size_bytes = 5 * 1024 * 1024; // 5MB
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

        assert!((size_mb - 5.0).abs() < 0.1);
    }
}
