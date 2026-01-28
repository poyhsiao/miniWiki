use crate::export::{ExportFormat, ExportService};
use crate::models::*;
use crate::repository::DocumentRepository;
use actix_web::{web, HttpResponse, Responder};
use jsonwebtoken;
use shared_errors::AppError;
use std::sync::Once;
use tracing::{error, warn};
use validator::Validate;

// Helper for access check with proper error handling
// Returns Ok(true) if access granted, Ok(false) if denied, Err for DB errors
async fn check_document_access(repo: &DocumentRepository, document_id: &str, user_id: &str) -> Result<bool, AppError> {
    match repo.check_document_access(document_id, user_id).await {
        Ok(true) => Ok(true),
        Ok(false) => Ok(false),
        Err(e) => {
            error!("Database error checking document access: {:?}", e);
            Err(AppError::DatabaseError(e))
        },
    }
}

// Helper for space access check with proper error handling
// Returns Ok(true) if access granted, Ok(false) if denied, Err for DB errors
async fn check_space_access(repo: &DocumentRepository, space_id: &str, user_id: &str) -> Result<bool, AppError> {
    match repo.check_space_access(space_id, user_id).await {
        Ok(true) => Ok(true),
        Ok(false) => Ok(false),
        Err(e) => {
            error!("Database error checking space access: {:?}", e);
            Err(AppError::DatabaseError(e))
        },
    }
}

// Helper to convert DocumentRow to DocumentResponse
fn document_row_to_response(row: &crate::repository::DocumentRow) -> DocumentResponse {
    DocumentResponse {
        id: row.id.to_string(),
        space_id: row.space_id.to_string(),
        parent_id: row.parent_id.map(|u| u.to_string()),
        title: row.title.clone(),
        icon: row.icon.clone(),
        content: row.content.0.clone(),
        content_size: row.content_size,
        is_archived: row.is_archived,
        created_by: row.created_by.to_string(),
        last_edited_by: row.last_edited_by.to_string(),
        created_at: row.created_at.and_utc().to_rfc3339(),
        updated_at: row.updated_at.and_utc().to_rfc3339(),
    }
}

// Helper to convert DocumentVersionRow to VersionResponse
fn version_row_to_response(row: &crate::repository::DocumentVersionRow) -> VersionResponse {
    VersionResponse {
        id: row.id.to_string(),
        document_id: row.document_id.to_string(),
        version_number: row.version_number,
        title: row.title.clone(),
        content: row.content.0.clone(),
        created_by: row.created_by.to_string(),
        created_at: row.created_at.and_utc().to_rfc3339(),
        change_summary: row.change_summary.clone(),
    }
}

// User extraction - supports both JWT Authorization header and X-User-Id header for backward compatibility
fn extract_user_id(req: &actix_web::HttpRequest) -> Result<String, AppError> {
    static JWT_WARNING_ONCE: Once = Once::new();

    // Get JWT secret from environment variable, with fallback to default for test/debug mode only
    let jwt_secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            // Allow fallback to test secret in debug/test mode for consistency with routes/mod.rs
            #[cfg(any(debug_assertions, test))]
            {
                JWT_WARNING_ONCE.call_once(|| {
                    warn!("Using default JWT secret. Set JWT_SECRET environment variable in production!");
                });
                "test-secret-key-for-testing-only-do-not-use-in-production".to_string()
            }
            #[cfg(not(any(debug_assertions, test)))]
            {
                return Err(AppError::AuthenticationError("JWT_SECRET not configured".to_string()));
            }
        },
    };

    // First try JWT Authorization header (preferred method)
    if let Some(auth_header) = req.headers().get("authorization") {
        if let Ok(token_str) = auth_header.to_str() {
            if token_str.starts_with("Bearer ") {
                let token = &token_str[7..];
                let decoding_key = jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes());
                // Explicitly enforce HS256 algorithm for security
                let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

                match jsonwebtoken::decode::<serde_json::Value>(token, &decoding_key, &validation) {
                    Ok(token_data) => {
                        // Try to extract "sub" claim with validation
                        if let Some(sub) = token_data.claims.get("sub") {
                            if let Some(user_id_str) =
                                sub.as_str()
                                    .and_then(|s| if !s.is_empty() { Some(s.to_string()) } else { None })
                            {
                                return Ok(user_id_str);
                            }
                        }

                        // Try to extract "user_id" claim with validation
                        if let Some(user_id) = token_data.claims.get("user_id") {
                            if let Some(user_id_str) =
                                user_id
                                    .as_str()
                                    .and_then(|s| if !s.is_empty() { Some(s.to_string()) } else { None })
                            {
                                return Ok(user_id_str);
                            }
                        }

                        // JWT decoded but no valid user ID found
                        return Err(AppError::AuthenticationError(
                            "JWT token missing or contains empty user ID claim".to_string(),
                        ));
                    },
                    Err(e) => {
                        // JWT decode failed, return error instead of falling back
                        return Err(AppError::AuthenticationError(format!("Invalid JWT token: {}", e)));
                    },
                }
            }
        }
    }

    // Fall back to X-User-Id header for backward compatibility
    req.headers()
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::AuthenticationError("Missing or invalid authentication".to_string()))
}

// Create document
pub async fn create_document(
    space_id: web::Path<String>,
    req: web::Json<CreateDocumentRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let space_id = space_id.into_inner();

    // Validate request
    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    // Get user ID from header (in production, this comes from JWT)
    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check space access
    match check_space_access(&repo, &space_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this space",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    // Check if user has permission to create documents (owner or editor)
    match repo.get_user_space_role(&space_id, &user_id).await {
        Ok(Some(role)) if role == "owner" || role == "editor" => {},
        Ok(Some(_)) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "PERMISSION_DENIED",
                "You don't have permission to create documents in this space",
            ));
        },
        Ok(None) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You are not a member of this space",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    // Create document
    match repo
        .create(
            &space_id,
            req.parent_id.as_deref(),
            &req.title,
            req.icon.as_deref(),
            req.content.clone(),
            &user_id,
        )
        .await
    {
        Ok(document) => {
            HttpResponse::Created().json(ApiResponse::<CreateDocumentResponse>::success(CreateDocumentResponse {
                id: document.id.to_string(),
                message: "Document created successfully".to_string(),
                document: document_row_to_response(&document),
            }))
        },
        Err(e) => {
            error!("Database error creating document: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Get document by ID
pub async fn get_document(
    document_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.get_by_id(&document_id).await {
        Ok(Some(document)) => HttpResponse::Ok().json(ApiResponse::<DocumentResponse>::success(
            document_row_to_response(&document),
        )),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("DOC_NOT_FOUND", "Document not found")),
        Err(e) => {
            error!("Database error getting document: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Update document
pub async fn update_document(
    document_id: web::Path<String>,
    req: web::Json<UpdateDocumentRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo
        .update(
            &document_id,
            req.title.as_deref(),
            req.icon.as_deref(),
            req.content.clone(),
            &user_id,
        )
        .await
    {
        Ok(Some(document)) => HttpResponse::Ok().json(ApiResponse::<DocumentResponse>::success(
            document_row_to_response(&document),
        )),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "DOC_NOT_FOUND",
            "Document not found or archived",
        )),
        Err(e) => {
            error!("Database error updating document: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Delete document (soft delete)
pub async fn delete_document(
    document_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.delete(&document_id).await {
        Ok(true) => HttpResponse::Ok().json(ApiResponse::<()>::success(())),
        Ok(false) => HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "DOC_NOT_FOUND",
            "Document not found or already archived",
        )),
        Err(e) => {
            error!("Database error deleting document: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// List documents in a space
pub async fn list_documents(
    space_id: web::Path<String>,
    query: web::Query<ListDocumentsQuery>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let space_id = space_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check space access
    match check_space_access(&repo, &space_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this space",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    match repo.list_in_space(&space_id, query.parent_id.as_deref(), limit, offset).await {
        Ok((documents, total)) => {
            HttpResponse::Ok().json(ApiResponse::<DocumentListResponse>::success(DocumentListResponse {
                documents: documents.iter().map(|d| document_row_to_response(d)).collect(),
                total,
                limit,
                offset,
            }))
        },
        Err(e) => {
            error!("Database error listing documents: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Get document children
pub async fn get_document_children(
    document_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.get_children(&document_id).await {
        Ok((children, total)) => HttpResponse::Ok().json(ApiResponse::<ChildrenResponse>::success(ChildrenResponse {
            documents: children.iter().map(|d| document_row_to_response(d)).collect(),
            total,
        })),
        Err(e) => {
            error!("Database error getting document children: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Get document path (hierarchy)
pub async fn get_document_path(
    document_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.get_document_path(&document_id).await {
        Ok(path) => HttpResponse::Ok().json(ApiResponse::<DocumentPathResponse>::success(DocumentPathResponse {
            path: path
                .into_iter()
                .map(|(id, title, level)| DocumentPathItem {
                    id: id.to_string(),
                    title,
                    level,
                })
                .collect(),
        })),
        Err(e) => {
            error!("Database error getting document path: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Create document version
pub async fn create_version(
    document_id: web::Path<String>,
    req: web::Json<CreateVersionRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo
        .create_version(
            &document_id,
            req.content.clone(),
            &req.title,
            &user_id,
            req.change_summary.as_deref(),
        )
        .await
    {
        Ok(version) => {
            HttpResponse::Created().json(ApiResponse::<CreateVersionResponse>::success(CreateVersionResponse {
                id: version.id.to_string(),
                version_number: version.version_number,
                message: "Version created successfully".to_string(),
                version: version_row_to_response(&version),
            }))
        },
        Err(e) => {
            error!("Database error creating version: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// List document versions
pub async fn list_versions(
    document_id: web::Path<String>,
    query: web::Query<ListVersionsQuery>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    match repo.list_versions(&document_id, limit, offset).await {
        Ok((versions, total)) => {
            HttpResponse::Ok().json(ApiResponse::<VersionListResponse>::success(VersionListResponse {
                versions: versions.iter().map(|v| version_row_to_response(v)).collect(),
                total,
                limit,
                offset,
            }))
        },
        Err(e) => {
            error!("Database error listing versions: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Get specific version
pub async fn get_version(
    path: actix_web::web::Path<(String, i32)>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let (document_id, version_number) = path.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.get_version(&document_id, version_number).await {
        Ok(Some(version)) => HttpResponse::Ok().json(ApiResponse::<VersionResponse>::success(version_row_to_response(
            &version,
        ))),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("VERSION_NOT_FOUND", "Version not found")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            "DATABASE_ERROR",
            "A database error occurred. Please try again later.",
        )),
    }
}

// Restore version
pub async fn restore_version(
    path: actix_web::web::Path<(String, i32)>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let (document_id, version_number) = path.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.restore_version(&document_id, version_number, &user_id).await {
        Ok(Some(document)) => {
            HttpResponse::Ok().json(ApiResponse::<RestoreVersionResponse>::success(RestoreVersionResponse {
                document: document_row_to_response(&document),
                message: format!("Successfully restored to version {}", version_number),
                restored_from_version: version_number,
            }))
        },
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("VERSION_NOT_FOUND", "Version not found")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            "DATABASE_ERROR",
            "A database error occurred. Please try again later.",
        )),
    }
}

// Get version diff
pub async fn get_version_diff(
    document_id: web::Path<String>,
    query: web::Query<serde_json::Value>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    let from_version = query
        .get("from")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or("Missing 'from' parameter");
    let to_version = query
        .get("to")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or("Missing 'to' parameter");

    // Handle parameter errors
    let (from_version, to_version) = match (from_version, to_version) {
        (Ok(from), Ok(to)) => (from, to),
        (Err(msg), _) => return HttpResponse::BadRequest().json(ApiResponse::<()>::error("INVALID_PARAM", msg)),
        (_, Err(msg)) => return HttpResponse::BadRequest().json(ApiResponse::<()>::error("INVALID_PARAM", msg)),
    };

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.get_version_diff(&document_id, from_version, to_version).await {
        Ok(Some((from_content, to_content))) => {
            HttpResponse::Ok().json(ApiResponse::<VersionDiffResponse>::success(VersionDiffResponse {
                from_version,
                to_version,
                from_content,
                to_content,
            }))
        },
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "VERSION_NOT_FOUND",
            "One or both versions not found",
        )),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            "DATABASE_ERROR",
            "A database error occurred. Please try again later.",
        )),
    }
}

// Export document handler
pub async fn export_document(
    document_id: web::Path<String>,
    query: web::Query<ExportQuery>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    // Parse format from query parameter
    let format = match query.format.as_deref() {
        Some("markdown") | Some("md") => ExportFormat::Markdown,
        Some("html") | Some("htm") => ExportFormat::Html,
        Some("pdf") => ExportFormat::Pdf,
        Some("json") => ExportFormat::Json,
        Some(fmt) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "INVALID_FORMAT",
                &format!(
                    "Unknown export format: {}. Supported formats: markdown, html, pdf, json",
                    fmt
                ),
            ));
        },
        None => ExportFormat::Markdown, // Default to markdown
    };

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    match check_document_access(&repo, &document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this document",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    // Get document
    match repo.get_by_id(&document_id).await {
        Ok(Some(document)) => {
            // Create export service with temp directory
            let temp_dir = std::env::temp_dir().join("miniwiki_exports");
            let export_service = ExportService::new(temp_dir);

            // Create metadata
            let metadata = Some(crate::export::DocumentMetadata {
                id: document.id.to_string(),
                title: document.title.clone(),
                created_at: Some(document.created_at),
                updated_at: Some(document.updated_at),
                created_by: Some(document.created_by.to_string()),
                icon: document.icon.clone(),
            });

            // Export the document
            match export_service
                .export_document(&document_id, &document.title, &document.content.0, metadata, format)
                .await
            {
                Ok(export_response) => {
                    // Read the file and return as response
                    let file_path = export_service.output_dir().join(&export_response.file_name);
                    match std::fs::read(&file_path) {
                        Ok(file_content) => {
                            // Simple filename escaping for Content-Disposition
                            let safe_filename = export_response.file_name.replace('"', "\\\"");
                            let content_disposition = format!("attachment; filename=\"{}\"", safe_filename);

                            HttpResponse::Ok()
                                .content_type(export_response.content_type)
                                .insert_header(("Content-Disposition", content_disposition))
                                .body(file_content)
                        },
                        Err(e) => {
                            error!("Error reading exported file: {:?}", e);
                            HttpResponse::InternalServerError()
                                .json(ApiResponse::<()>::error("EXPORT_ERROR", "Failed to read exported file"))
                        },
                    }
                },
                Err(e) => {
                    error!("Export error: {:?}", e);
                    HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                        "EXPORT_ERROR",
                        &format!("Export failed: {}", e),
                    ))
                },
            }
        },
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("DOC_NOT_FOUND", "Document not found")),
        Err(e) => {
            error!("Database error getting document for export: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Space handlers
pub async fn list_spaces(repo: web::Data<DocumentRepository>, http_req: actix_web::HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    match repo.list_spaces(&user_id).await {
        Ok(spaces) => {
            let total = spaces.len() as i32;
            HttpResponse::Ok().json(ApiResponse::<SpaceListResponse>::success(SpaceListResponse {
                spaces: spaces.into_iter().map(|s| space_row_to_response(&s)).collect(),
                total,
            }))
        },
        Err(e) => {
            error!("Database error listing spaces: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

pub async fn create_space(
    req: web::Json<CreateSpaceRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    match repo
        .create_space(
            &user_id,
            &req.name,
            req.icon.as_deref(),
            req.description.as_deref(),
            req.is_public,
        )
        .await
    {
        Ok(space) => HttpResponse::Created().json(ApiResponse::<SpaceResponse>::success(space_row_to_response(&space))),
        Err(e) => {
            error!("Database error creating space: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

pub async fn get_space(
    space_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let space_id = space_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check access (public spaces accessible by anyone, private spaces require membership)
    match repo.check_space_access(&space_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this space",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.get_space(&space_id).await {
        Ok(Some(space)) => {
            HttpResponse::Ok().json(ApiResponse::<SpaceResponse>::success(space_row_to_response(&space)))
        },
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("SPACE_NOT_FOUND", "Space not found")),
        Err(e) => {
            error!("Database error getting space: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

pub async fn update_space(
    space_id: web::Path<String>,
    req: web::Json<UpdateSpaceRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let space_id = space_id.into_inner();

    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check if user is owner
    match repo.is_space_owner(&space_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "Only space owner can update space",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo
        .update_space(
            &space_id,
            req.name.as_deref(),
            req.icon.as_deref(),
            req.description.as_deref(),
            req.is_public,
        )
        .await
    {
        Ok(Some(space)) => {
            HttpResponse::Ok().json(ApiResponse::<SpaceResponse>::success(space_row_to_response(&space)))
        },
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("SPACE_NOT_FOUND", "Space not found")),
        Err(e) => {
            error!("Database error updating space: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

pub async fn delete_space(
    space_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let space_id = space_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check if user is owner
    match repo.is_space_owner(&space_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "Only space owner can delete space",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.delete_space(&space_id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(ApiResponse::<()>::error("SPACE_NOT_FOUND", "Space not found")),
        Err(e) => {
            error!("Database error deleting space: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Space membership handlers
pub async fn list_space_members(
    space_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let space_id = space_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check access
    match repo.check_space_access(&space_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this space",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.list_space_members(&space_id).await {
        Ok(members) => {
            let total = members.len() as i32;
            HttpResponse::Ok().json(ApiResponse::<MemberListResponse>::success(MemberListResponse {
                members: members.into_iter().map(|m| membership_row_to_response(&m)).collect(),
                total,
            }))
        },
        Err(e) => {
            error!("Database error listing members: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

pub async fn add_space_member(
    space_id: web::Path<String>,
    req: web::Json<AddMemberRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let space_id = space_id.into_inner();

    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check if user is owner or editor
    match repo.get_user_space_role(&space_id, &user_id).await {
        Ok(Some(role)) if role == "owner" || role == "editor" => {},
        Ok(Some(_)) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "Insufficient permissions to add members",
            ));
        },
        Ok(None) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have access to this space",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    match repo.add_space_member(&space_id, &req.user_id, &req.role, &user_id).await {
        Ok(membership) => HttpResponse::Created().json(ApiResponse::<MemberResponse>::success(
            membership_row_to_response(&membership),
        )),
        Err(e) => {
            error!("Database error adding member: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

pub async fn update_space_member(
    path: actix_web::web::Path<(String, String)>,
    req: web::Json<UpdateMemberRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let (space_id, member_user_id) = path.into_inner();

    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check if current user is owner
    match repo.is_space_owner(&space_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "Only space owner can update member roles",
            ));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ));
        },
    }

    // Cannot change owner role
    if let Ok(true) = repo.is_space_owner(&space_id, &member_user_id).await {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "INVALID_OPERATION",
            "Cannot change owner role",
        ));
    }

    match repo.update_space_member(&space_id, &member_user_id, &req.role).await {
        Ok(Some(membership)) => HttpResponse::Ok().json(ApiResponse::<MemberResponse>::success(
            membership_row_to_response(&membership),
        )),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("MEMBER_NOT_FOUND", "Member not found")),
        Err(e) => {
            error!("Database error updating member: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

pub async fn remove_space_member(
    path: actix_web::web::Path<(String, String)>,
    repo: web::Data<DocumentRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let (space_id, member_user_id) = path.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check permissions: owner can remove anyone, member can remove themselves
    let is_owner = repo.is_space_owner(&space_id, &user_id).await.unwrap_or(false);
    let is_self = member_user_id == user_id;

    if !is_owner && !is_self {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "ACCESS_DENIED",
            "Insufficient permissions to remove this member",
        ));
    }

    // Cannot remove owner
    if is_owner && repo.is_space_owner(&space_id, &member_user_id).await.unwrap_or(false) {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "INVALID_OPERATION",
            "Cannot remove space owner",
        ));
    }

    match repo.remove_space_member(&space_id, &member_user_id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(ApiResponse::<()>::error("MEMBER_NOT_FOUND", "Member not found")),
        Err(e) => {
            error!("Database error removing member: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ))
        },
    }
}

// Helper functions for space conversions
fn space_row_to_response(row: &crate::repository::SpaceRow) -> SpaceResponse {
    SpaceResponse {
        id: row.id.to_string(),
        owner_id: row.owner_id.to_string(),
        name: row.name.clone(),
        icon: row.icon.clone(),
        description: row.description.clone(),
        is_public: row.is_public,
        created_at: row.created_at.and_utc().to_rfc3339(),
        updated_at: row.updated_at.and_utc().to_rfc3339(),
        user_role: row.user_role.clone(),
    }
}

fn membership_row_to_response(row: &crate::repository::SpaceMembershipRow) -> MemberResponse {
    MemberResponse {
        id: row.id.to_string(),
        space_id: row.space_id.to_string(),
        user_id: row.user_id.to_string(),
        role: row.role.clone(),
        joined_at: row.joined_at.and_utc().to_rfc3339(),
        invited_by: row.invited_by.to_string(),
    }
}
