use actix_web::{web, Responder, HttpResponse};
use crate::models::*;
use crate::repository::DocumentRepository;
use shared_errors::AppError;
use validator::Validate;

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

// Placeholder for user extraction - in real implementation, this would come from JWT
fn extract_user_id(req: &actix_web::HttpRequest) -> Result<String, AppError> {
    req.headers()
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::AuthenticationError("Missing X-User-Id header".to_string()))
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
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("VALIDATION_ERROR", &format!("Validation failed: {:?}", validation_errors)));
    }

    // Get user ID from header (in production, this comes from JWT)
    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check space access
    if !repo.check_space_access(&space_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this space"));
    }

    // Create document
    match repo.create(
        &space_id,
        req.parent_id.as_deref(),
        &req.title,
        req.icon.as_deref(),
        req.content.clone(),
        &user_id,
    ).await {
        Ok(document) => {
            HttpResponse::Created()
                .json(ApiResponse::<CreateDocumentResponse>::success(CreateDocumentResponse {
                    id: document.id.to_string(),
                    message: "Document created successfully".to_string(),
                    document: document_row_to_response(&document),
                }))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.get_by_id(&document_id).await {
        Ok(Some(document)) => {
            HttpResponse::Ok()
                .json(ApiResponse::<DocumentResponse>::success(document_row_to_response(&document)))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("DOC_NOT_FOUND", "Document not found"))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("VALIDATION_ERROR", &format!("Validation failed: {:?}", validation_errors)));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.update(
        &document_id,
        req.title.as_deref(),
        req.icon.as_deref(),
        req.content.clone(),
        &user_id,
    ).await {
        Ok(Some(document)) => {
            HttpResponse::Ok()
                .json(ApiResponse::<DocumentResponse>::success(document_row_to_response(&document)))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("DOC_NOT_FOUND", "Document not found or archived"))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.delete(&document_id).await {
        Ok(true) => {
            HttpResponse::Ok()
                .json(ApiResponse::<()>::success(()))
        }
        Ok(false) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("DOC_NOT_FOUND", "Document not found or already archived"))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
    if !repo.check_space_access(&space_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this space"));
    }

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    match repo.list_in_space(&space_id, query.parent_id.as_deref(), limit, offset).await {
        Ok((documents, total)) => {
            HttpResponse::Ok()
                .json(ApiResponse::<DocumentListResponse>::success(DocumentListResponse {
                    documents: documents.iter().map(|d| document_row_to_response(d)).collect(),
                    total,
                    limit,
                    offset,
                }))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.get_children(&document_id).await {
        Ok((children, total)) => {
            HttpResponse::Ok()
                .json(ApiResponse::<ChildrenResponse>::success(ChildrenResponse {
                    documents: children.iter().map(|d| document_row_to_response(d)).collect(),
                    total,
                }))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.get_document_path(&document_id).await {
        Ok(path) => {
            HttpResponse::Ok()
                .json(ApiResponse::<DocumentPathResponse>::success(DocumentPathResponse {
                    path: path.into_iter()
                        .map(|(id, title, level)| DocumentPathItem {
                            id: id.to_string(),
                            title,
                            level,
                        })
                        .collect(),
                }))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("VALIDATION_ERROR", &format!("Validation failed: {:?}", validation_errors)));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.create_version(
        &document_id,
        req.content.clone(),
        &req.title,
        &user_id,
        req.change_summary.as_deref(),
    ).await {
        Ok(version) => {
            HttpResponse::Created()
                .json(ApiResponse::<CreateVersionResponse>::success(CreateVersionResponse {
                    id: version.id.to_string(),
                    version_number: version.version_number,
                    message: "Version created successfully".to_string(),
                    version: version_row_to_response(&version),
                }))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    match repo.list_versions(&document_id, limit, offset).await {
        Ok((versions, total)) => {
            HttpResponse::Ok()
                .json(ApiResponse::<VersionListResponse>::success(VersionListResponse {
                    versions: versions.iter().map(|v| version_row_to_response(v)).collect(),
                    total,
                    limit,
                    offset,
                }))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.get_version(&document_id, version_number).await {
        Ok(Some(version)) => {
            HttpResponse::Ok()
                .json(ApiResponse::<VersionResponse>::success(version_row_to_response(&version)))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("VERSION_NOT_FOUND", "Version not found"))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.restore_version(&document_id, version_number, &user_id).await {
        Ok(Some(document)) => {
            HttpResponse::Ok()
                .json(ApiResponse::<RestoreVersionResponse>::success(RestoreVersionResponse {
                    document: document_row_to_response(&document),
                    message: format!("Successfully restored to version {}", version_number),
                    restored_from_version: version_number,
                }))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("VERSION_NOT_FOUND", "Version not found"))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
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

    let from_version = query.get("from")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or("Missing 'from' parameter");
    let to_version = query.get("to")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or("Missing 'to' parameter");

    // Handle parameter errors
    let (from_version, to_version) = match (from_version, to_version) {
        (Ok(from), Ok(to)) => (from, to),
        (Err(msg), _) => return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("INVALID_PARAM", msg)),
        (_, Err(msg)) => return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("INVALID_PARAM", msg)),
    };

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    // Check document access
    if !repo.check_document_access(&document_id, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("ACCESS_DENIED", "You don't have access to this document"));
    }

    match repo.get_version_diff(&document_id, from_version, to_version).await {
        Ok(Some((from_content, to_content))) => {
            HttpResponse::Ok()
                .json(ApiResponse::<VersionDiffResponse>::success(VersionDiffResponse {
                    from_version,
                    to_version,
                    from_content,
                    to_content,
                }))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("VERSION_NOT_FOUND", "One or both versions not found"))
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", &e.to_string()))
        }
    }
}
