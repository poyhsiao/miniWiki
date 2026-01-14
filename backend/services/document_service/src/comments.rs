//! Comment Handlers
//!
//! Provides HTTP handlers for comment operations:
//! - GET /documents/{documentId}/comments - List comments
//! - POST /documents/{documentId}/comments - Create comment
//! - PATCH /comments/{commentId} - Update comment
//! - POST /comments/{commentId}/resolve - Resolve comment
//! - POST /comments/{commentId}/unresolve - Unresolve comment
//! - DELETE /comments/{commentId} - Delete comment

use actix_web::{web, Responder, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use validator::Validate;
use tracing::error;
use shared_errors::AppError;

use crate::models::*;
use crate::repository::DocumentRepository;
use crate::repository::CommentRow;

/// Extract user ID from request header
fn extract_user_id(req: &HttpRequest) -> Result<String, AppError> {
    req.headers()
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::AuthenticationError("Missing X-User-Id header".to_string()))
}

/// Extract user name from request header (optional)
fn extract_user_name(req: &HttpRequest) -> String {
    req.headers()
        .get("X-User-Name")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown User".to_string())
}

/// Convert database row to CommentResponse
fn comment_row_to_response(
    row: &CommentRow,
    author_name: &str,
    author_avatar: Option<&str>,
) -> CommentResponse {
    CommentResponse {
        id: row.id.to_string(),
        document_id: row.document_id.to_string(),
        parent_id: row.parent_id.map(|u| u.to_string()),
        author_id: row.author_id.to_string(),
        author_name: author_name.to_string(),
        author_avatar: author_avatar.map(|s| s.to_string()),
        content: row.content.clone(),
        is_resolved: row.is_resolved,
        resolved_by: row.resolved_by.map(|u| u.to_string()),
        resolved_at: row.resolved_at.map(|t| t.and_utc().to_rfc3339()),
        created_at: row.created_at.and_utc().to_rfc3339(),
        updated_at: row.updated_at.and_utc().to_rfc3339(),
    }
}

/// List comments for a document
pub async fn list_comments(
    document_id: web::Path<String>,
    query: web::Query<ListCommentsQuery>,
    repo: web::Data<DocumentRepository>,
    http_req: HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();
    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            &e.error_code,
            &e.message,
        )),
    };

    // Check document access
    match repo.check_document_access(&document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have permission to view comments on this document",
            ));
        }
        Err(e) => {
            error!("Database error checking document access: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to verify document access",
            ));
        }
    }

    // Get comments
    match repo.list_comments(&document_id, query.parent_id.as_deref(), query.limit, query.offset).await {
        Ok((comments, total)) => {
            let comment_responses: Vec<CommentResponse> = comments
                .iter()
                .map(|row| {
                    let author_name = row.author_name.clone().unwrap_or_else(|| "Unknown User".to_string());
                    let author_avatar = row.author_avatar.as_deref();
                    comment_row_to_response(row, &author_name, author_avatar)
                })
                .collect();

            HttpResponse::Ok().json(ApiResponse::success(CommentListResponse {
                comments: comment_responses,
                total,
                limit: query.limit.unwrap_or(50),
                offset: query.offset.unwrap_or(0),
            }))
        }
        Err(e) => {
            error!("Database error listing comments: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to list comments",
            ))
        }
    }
}

/// Create a new comment
pub async fn create_comment(
    document_id: web::Path<String>,
    req: web::Json<CreateCommentRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: HttpRequest,
) -> impl Responder {
    let document_id = document_id.into_inner();

    // Validate request
    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            &e.error_code,
            &e.message,
        )),
    };

    let user_name = extract_user_name(&http_req);

    // Check document access (need at least commenter role)
    match repo.check_document_access(&document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have permission to add comments to this document",
            ));
        }
        Err(e) => {
            error!("Database error checking document access: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to verify document access",
            ));
        }
    }

    // If parent_id is provided, verify parent comment exists and belongs to the same document
    if let Some(ref parent_id) = req.parent_id {
        match repo.get_comment(parent_id).await {
            Ok(Some(parent_comment)) => {
                if parent_comment.document_id.to_string() != document_id {
                    return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                        "INVALID_PARENT",
                        "Parent comment must belong to the same document",
                    ));
                }
            }
            Ok(None) => {
                return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    "PARENT_NOT_FOUND",
                    "Parent comment not found",
                ));
            }
            Err(e) => {
                error!("Database error checking parent comment: {:?}", e);
                return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "DATABASE_ERROR",
                    "Failed to verify parent comment",
                ));
            }
        }
    }

    // Create comment
    match repo.create_comment(&document_id, &user_id, &user_name, &req.content, req.parent_id.as_deref()).await {
        Ok(comment) => {
            let response = comment_row_to_response(
                &comment,
                &user_name,
                None,
            );
            HttpResponse::Created().json(ApiResponse::success(response))
        }
        Err(e) => {
            error!("Database error creating comment: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to create comment",
            ))
        }
    }
}

/// Update a comment
pub async fn update_comment(
    comment_id: web::Path<String>,
    req: web::Json<UpdateCommentRequest>,
    repo: web::Data<DocumentRepository>,
    http_req: HttpRequest,
) -> impl Responder {
    let comment_id = comment_id.into_inner();

    // Validate request
    if let Err(validation_errors) = (&*req).validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "VALIDATION_ERROR",
            &format!("Validation failed: {:?}", validation_errors),
        ));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            &e.error_code,
            &e.message,
        )),
    };

    // Get existing comment
    match repo.get_comment(&comment_id).await {
        Ok(Some(comment)) => {
            // Check if user is the author
            if comment.author_id.to_string() != user_id {
                return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                    "ACCESS_DENIED",
                    "You can only edit your own comments",
                ));
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "NOT_FOUND",
                "Comment not found",
            ));
        }
        Err(e) => {
            error!("Database error getting comment: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to get comment",
            ));
        }
    }

    // Update comment
    match repo.update_comment(&comment_id, &req.content).await {
        Ok(comment) => {
            let author_name = comment.author_name.clone().unwrap_or_else(|| "Unknown User".to_string());
            let author_avatar = comment.author_avatar.as_deref();
            let response = comment_row_to_response(&comment, &author_name, author_avatar);
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => {
            error!("Database error updating comment: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to update comment",
            ))
        }
    }
}

/// Resolve a comment
pub async fn resolve_comment(
    comment_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: HttpRequest,
) -> impl Responder {
    let comment_id = comment_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            &e.error_code,
            &e.message,
        )),
    };

    // Get existing comment
    match repo.get_comment(&comment_id).await {
        Ok(Some(comment)) => {
            // Check if user can resolve (author or editor+)
            if comment.author_id.to_string() == user_id {
                // Author can resolve their own comment
            } else {
                // Check document access for editing
                match repo.check_document_access(&comment.document_id.to_string(), &user_id).await {
                    Ok(true) => {},
                    Ok(false) => {
                        return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                            "ACCESS_DENIED",
                            "You don't have permission to resolve comments",
                        ));
                    }
                    Err(e) => {
                        error!("Database error checking document access: {:?}", e);
                        return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "DATABASE_ERROR",
                            "Failed to verify access",
                        ));
                    }
                }
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "NOT_FOUND",
                "Comment not found",
            ));
        }
        Err(e) => {
            error!("Database error getting comment: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to get comment",
            ));
        }
    }

    // Resolve comment
    match repo.resolve_comment(&comment_id, &user_id).await {
        Ok(comment) => {
            let author_name = comment.author_name.clone().unwrap_or_else(|| "Unknown User".to_string());
            let author_avatar = comment.author_avatar.as_deref();
            let response = comment_row_to_response(&comment, &author_name, author_avatar);
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => {
            error!("Database error resolving comment: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to resolve comment",
            ))
        }
    }
}

/// Unresolve a comment
pub async fn unresolve_comment(
    comment_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: HttpRequest,
) -> impl Responder {
    let comment_id = comment_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            &e.error_code,
            &e.message,
        )),
    };

    // Get existing comment
    match repo.get_comment(&comment_id).await {
        Ok(Some(comment)) => {
            // Check if user can unresolve (editor+ only)
            match repo.check_document_access(&comment.document_id.to_string(), &user_id).await {
                Ok(true) => {},
                Ok(false) => {
                    return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                        "ACCESS_DENIED",
                        "You don't have permission to unresolve comments",
                    ));
                }
                Err(e) => {
                    error!("Database error checking document access: {:?}", e);
                    return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                        "DATABASE_ERROR",
                        "Failed to verify access",
                    ));
                }
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "NOT_FOUND",
                "Comment not found",
            ));
        }
        Err(e) => {
            error!("Database error getting comment: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to get comment",
            ));
        }
    }

    // Unresolve comment
    match repo.unresolve_comment(&comment_id).await {
        Ok(comment) => {
            let author_name = comment.author_name.clone().unwrap_or_else(|| "Unknown User".to_string());
            let author_avatar = comment.author_avatar.as_deref();
            let response = comment_row_to_response(&comment, &author_name, author_avatar);
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => {
            error!("Database error unresolving comment: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to unresolve comment",
            ))
        }
    }
}

/// Delete a comment
pub async fn delete_comment(
    comment_id: web::Path<String>,
    repo: web::Data<DocumentRepository>,
    http_req: HttpRequest,
) -> impl Responder {
    let comment_id = comment_id.into_inner();

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            &e.error_code,
            &e.message,
        )),
    };

    // Get existing comment
    match repo.get_comment(&comment_id).await {
        Ok(Some(comment)) => {
            // Check if user can delete (author or editor+)
            if comment.author_id.to_string() == user_id {
                // Author can delete their own comment
            } else {
                // Check document access for editing
                match repo.check_document_access(&comment.document_id.to_string(), &user_id).await {
                    Ok(true) => {},
                    Ok(false) => {
                        return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                            "ACCESS_DENIED",
                            "You don't have permission to delete this comment",
                        ));
                    }
                    Err(e) => {
                        error!("Database error checking document access: {:?}", e);
                        return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            "DATABASE_ERROR",
                            "Failed to verify access",
                        ));
                    }
                }
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "NOT_FOUND",
                "Comment not found",
            ));
        }
        Err(e) => {
            error!("Database error getting comment: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to get comment",
            ));
        }
    }

    // Delete comment
    match repo.delete_comment(&comment_id).await {
        Ok(_) => {
            HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                "message": "Comment deleted successfully"
            })))
        }
        Err(e) => {
            error!("Database error deleting comment: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to delete comment",
            ))
        }
    }
}
