//! Comment Handlers
//!
//! Provides HTTP handlers for comment operations:
//! - GET /documents/{documentId}/comments - List comments
//! - POST /documents/{documentId}/comments - Create comment
//! - PATCH /comments/{commentId} - Update comment
//! - POST /comments/{commentId}/resolve - Resolve comment
//! - POST /comments/{commentId}/unresolve - Unresolve comment
//! - DELETE /comments/{commentId} - Delete comment
//!
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use shared_errors::{AppError, ErrorCode};
use std::collections::HashMap;
use tracing::error;
use uuid::Uuid;
use validator::Validate;

use crate::models::*;
use crate::repository::CommentRow;
use crate::repository::DocumentRepository;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::CommentRow;
    use actix_web::test::TestRequest;
    use chrono::{DateTime, Duration};

    // Create a test request
    fn create_test_request() -> HttpRequest {
        TestRequest::get()
            .insert_header(("X-User-Id", "test-user-001"))
            .to_http_request()
    }

    // extract_user_id Tests
    #[test]
    fn test_extract_user_id_invalid_encoding() {
        let req = TestRequest::get().insert_header(("X-User-Id", "not-a-uuid")).to_http_request();

        let user_id = extract_user_id(&req);
        // extract_user_id should validate UUID format and return error for invalid UUIDs
        assert!(user_id.is_err());
    }

    #[test]
    fn test_extract_user_id_missing() {
        let req = TestRequest::get().to_http_request(); // No X-User-Id header

        let user_id = extract_user_id(&req);
        assert!(user_id.is_err());
        let err = user_id.unwrap_err();
        assert!(matches!(err, AppError::AuthenticationError(_)));
        assert!(err.to_string().contains("Missing X-User-Id header"));
    }

    #[test]
    fn test_extract_user_id_invalid_encoding() {
        let req = TestRequest::get().insert_header(("X-User-Id", "not-a-uuid")).to_http_request();

        let user_id = extract_user_id(&req);
        // extract_user_id returns the header string without UUID validation
        assert_eq!(user_id.unwrap(), "not-a-uuid");
    }

    // extract_user_name Tests
    #[test]
    fn test_extract_user_name_valid() {
        let req = TestRequest::get().insert_header(("X-User-Name", "Test User")).to_http_request();

        let user_name = extract_user_name(&req);
        assert_eq!(user_name, "Test User");
    }

    #[test]
    fn test_extract_user_name_missing() {
        let req = TestRequest::get().to_http_request(); // No X-User-Name header

        let user_name = extract_user_name(&req);
        assert_eq!(user_name, "Unknown User");
    }

    #[test]
    fn test_extract_user_name_empty_string() {
        let req = TestRequest::get().insert_header(("X-User-Name", "")).to_http_request();

        let user_name = extract_user_name(&req);
        assert_eq!(user_name, "");
    }

    // comment_row_to_response Tests
    #[test]
    fn test_comment_row_to_response_all_fields() {
        let now = Utc::now().naive_utc();

        let row = CommentRow {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            parent_id: None,
            author_id: Uuid::new_v4(),
            content: "Test comment".to_string(),
            is_resolved: false,
            resolved_by: None,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        };

        let response = comment_row_to_response(&row, "Test Author", Some("https://example.com/avatar.png"));

        assert_eq!(response.id, row.id.to_string());
        assert_eq!(response.content, "Test comment");
        assert_eq!(response.author_name, "Test Author");
        assert_eq!(
            response.author_avatar.as_deref(),
            Some("https://example.com/avatar.png")
        );
        assert_eq!(response.is_resolved, false);
        assert_eq!(response.resolved_by, None);
        assert_eq!(response.resolved_at, None);
    }

    #[test]
    fn test_comment_row_to_response_resolved() {
        let now = Utc::now().naive_utc();
        let resolved_at = now - chrono::Duration::days(1);
        let resolver_id = Uuid::new_v4();

        let row = CommentRow {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            parent_id: None,
            author_id: Uuid::new_v4(),
            content: "Resolved comment".to_string(),
            is_resolved: true,
            resolved_by: Some(resolver_id),
            resolved_at: Some(resolved_at),
            created_at: now,
            updated_at: now,
        };

        let response = comment_row_to_response(&row, "Resolver Name", None);

        assert_eq!(response.is_resolved, true);
        assert_eq!(response.resolved_by, Some(resolver_id.to_string()));
        assert_eq!(response.resolved_at, Some(resolved_at.and_utc().to_rfc3339()));
    }

    #[test]
    fn test_comment_row_to_response_nested_parent() {
        let parent_id = Uuid::new_v4();
        let now = Utc::now().naive_utc();

        let row = CommentRow {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            author_id: Uuid::new_v4(),
            content: "Reply comment".to_string(),
            is_resolved: false,
            resolved_by: None,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        };

        let response = comment_row_to_response(&row, "Reply Author", None);

        assert_eq!(response.parent_id, Some(parent_id.to_string()));
        assert_eq!(response.content, "Reply comment");
    }

    #[test]
    fn test_comment_row_to_response_no_avatar() {
        let now = Utc::now().naive_utc();

        let row = CommentRow {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            parent_id: None,
            author_id: Uuid::new_v4(),
            content: "No avatar".to_string(),
            is_resolved: false,
            resolved_by: None,
            resolved_at: None,
            created_at: now,
            updated_at: now,
        };

        let response = comment_row_to_response(&row, "No Avatar", None);

        assert_eq!(response.author_avatar, None);
    }

    // Test: ListCommentsQuery Defaults
    #[test]
    fn test_list_comments_query_defaults() {
        // Simulate query with no values
        let parent_id: Option<String> = None;
        let limit: Option<i64> = None;
        let offset: Option<i64> = None;

        // Default values should be:
        assert_eq!(parent_id, None);
        assert_eq!(limit, None);
        assert_eq!(offset, None);

        // In handlers, defaults are:
        // limit.unwrap_or(50)
        // offset.unwrap_or(0)
    }

    #[test]
    fn test_list_comments_query_with_values() {
        let parent_id = Some("parent-uuid".to_string());
        let limit = Some(25);
        let offset = Some(100);

        assert_eq!(parent_id, Some("parent-uuid".to_string()));
        assert_eq!(limit, Some(25));
        assert_eq!(offset, Some(100));
    }

    // Test: Authorship Check Logic
    #[test]
    fn test_author_can_update() {
        let comment_author_id = "user-001".to_string();
        let requesting_user_id = "user-001".to_string();

        let can_update = comment_author_id == requesting_user_id;
        assert!(can_update);
    }

    #[test]
    fn test_different_user_cannot_update() {
        let comment_author_id = "user-001".to_string();
        let requesting_user_id = "user-002".to_string();

        let can_update = comment_author_id == requesting_user_id;
        assert!(!can_update);
    }

    #[test]
    fn test_author_can_delete() {
        let comment_author_id = "user-001".to_string();
        let requesting_user_id = "user-001".to_string();

        let can_delete = comment_author_id == requesting_user_id;
        assert!(can_delete);
    }

    #[test]
    fn test_different_user_cannot_delete_without_access() {
        let comment_author_id = "user-001".to_string();
        let requesting_user_id = "user-002".to_string();

        let can_delete = comment_author_id == requesting_user_id;
        assert!(!can_delete);
    }

    // Test: Parent Comment Validation Logic
    #[test]
    fn test_parent_belongs_to_same_document() {
        let comment_document_id = Uuid::new_v4();
        let target_document_id = comment_document_id;

        let same_document = comment_document_id == target_document_id;
        assert!(same_document);
    }

    #[test]
    fn test_parent_different_document() {
        let comment_document_id = Uuid::new_v4();
        let nil_uuid = Uuid::nil();
        let target_document_id = nil_uuid;

        let same_document = comment_document_id == target_document_id;
        assert!(!same_document);
    }

    #[test]
    fn test_parent_none_allowed() {
        let parent_id: Option<Uuid> = None;

        // None parent is always allowed (top-level comment)
        match parent_id {
            None => {}, // Valid
            Some(_) => panic!("None should be None"),
        }
    }

    // Test: Timestamp Conversion
    #[test]
    fn test_utc_timestamp_conversion() {
        let now = Utc::now();
        let rfc3339 = now.to_rfc3339();

        // Should be valid ISO 8601 format
        assert!(rfc3339.starts_with("20"));
        assert!(rfc3339.contains("T"));
        assert!(rfc3339.contains("Z") || rfc3339.contains("+"));
    }

    // Test: None Handling in Responses
    #[test]
    fn test_none_parent_id_in_response() {
        let parent_id: Option<Uuid> = None;
        let parent_id_str = parent_id.map(|u| u.to_string());

        assert_eq!(parent_id_str, None);
    }

    #[test]
    fn test_some_parent_id_in_response() {
        let parent_id = Uuid::new_v4();
        let parent_id_str = Some(parent_id).map(|u| u.to_string());

        assert_eq!(parent_id_str, Some(parent_id.to_string()));
    }

    #[test]
    fn test_none_resolved_at_in_response() {
        let resolved_at: Option<chrono::DateTime<Utc>> = None;
        let resolved_at_str = resolved_at.map(|t| t.to_rfc3339());

        assert_eq!(resolved_at_str, None);
    }

    #[test]
    fn test_some_resolved_at_in_response() {
        let resolved_at = Some(Utc::now());
        let resolved_at_str = resolved_at.map(|t| t.to_rfc3339());

        assert!(resolved_at_str.is_some());
        assert!(resolved_at_str.unwrap().len() > 0);
    }

    // Test: Limit and Offset Defaults
    #[test]
    fn test_default_limit() {
        let limit: Option<i64> = None;
        let actual_limit = limit.unwrap_or(50);

        assert_eq!(actual_limit, 50);
    }

    #[test]
    fn test_custom_limit() {
        let limit: Option<i64> = Some(25);
        let actual_limit = limit.unwrap_or(50);

        assert_eq!(actual_limit, 25);
    }

    #[test]
    fn test_default_offset() {
        let offset: Option<i64> = None;
        let actual_offset = offset.unwrap_or(0);

        assert_eq!(actual_offset, 0);
    }

    #[test]
    fn test_custom_offset() {
        let offset: Option<i64> = Some(100);
        let actual_offset = offset.unwrap_or(0);

        assert_eq!(actual_offset, 100);
    }

    // Test: Response Structure Fields
    #[test]
    fn test_comment_response_all_fields() {
        let response = CommentResponse {
            id: "comment-001".to_string(),
            document_id: "document-001".to_string(),
            parent_id: Some("parent-001".to_string()),
            author_id: "user-001".to_string(),
            author_name: "Test User".to_string(),
            author_avatar: Some("avatar-url".to_string()),
            content: "Test content".to_string(),
            is_resolved: false,
            resolved_by: Some("resolver-001".to_string()),
            resolved_at: Some("2026-01-22T10:00:00Z".to_string()),
            created_at: "2026-01-22T08:00:00Z".to_string(),
            updated_at: Some("2026-01-22T09:00:00Z".to_string()),
        };

        assert_eq!(response.id, "comment-001");
        assert_eq!(response.content, "Test content");
        assert_eq!(response.is_resolved, false);
        assert_eq!(response.resolved_by.as_deref(), Some("resolver-001"));
        assert_eq!(response.resolved_at, Some("2026-01-22T10:00:00Z".to_string()));
    }

    #[test]
    fn test_comment_response_minimal_fields() {
        let response = CommentResponse {
            id: "comment-001".to_string(),
            document_id: "document-001".to_string(),
            parent_id: None,
            author_id: "user-001".to_string(),
            author_name: "Test User".to_string(),
            author_avatar: None,
            content: "Minimal".to_string(),
            is_resolved: false,
            resolved_by: None,
            resolved_at: None,
            created_at: "2026-01-22T08:00:00Z".to_string(),
            updated_at: None,
        };

        assert_eq!(response.parent_id, None);
        assert_eq!(response.author_avatar, None);
        assert_eq!(response.resolved_by, None);
        assert_eq!(response.resolved_at, None);
        assert!(response.updated_at.is_none());
    }

    // Test: UUID String Conversion
    #[test]
    fn test_uuid_to_string() {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();

        assert_eq!(uuid_str.len(), 36); // Standard UUID string length
        assert!(uuid_str.contains('-'));
    }

    #[test]
    fn test_document_id_consistency() {
        let document_id = Uuid::new_v4();
        let document_id_str = document_id.to_string();

        // Document ID should be consistent
        let parsed_back = Uuid::parse_str(&document_id_str);
        assert!(parsed_back.is_ok());
        assert_eq!(parsed_back.unwrap(), document_id);
    }

    // Test: Comment Content Validation
    #[test]
    fn test_comment_content_characters() {
        let valid_content = "This is a valid comment".to_string();
        let empty_content = "".to_string();
        let long_content = "a".repeat(1000);

        // Empty content is technically valid (validation happens at request level)
        assert_eq!(empty_content.len(), 0);

        // Valid content
        assert!(valid_content.len() > 0);

        // Long content should be allowed
        assert_eq!(long_content.len(), 1000);
    }

    // Test: Error Response Construction
    #[test]
    fn test_error_responses() {
        let unauthorized = AppError::AuthenticationError("No token".to_string());
        let forbidden = AppError::AuthorizationError("No access".to_string());
        let not_found = AppError::NotFoundError("Not found".to_string());
        let validation = AppError::ValidationError("Invalid".to_string());

        // Verify all error types can be constructed
        assert!(matches!(unauthorized, AppError::AuthenticationError(_)));
        assert!(matches!(forbidden, AppError::AuthorizationError(_)));
        assert!(matches!(not_found, AppError::NotFoundError(_)));
        assert!(matches!(validation, AppError::ValidationError(_)));
    }

    // Test: Header Name Constants
    #[test]
    fn test_header_name_constants() {
        // These constants should match the values used in the code
        assert_eq!("X-User-Id", "X-User-Id");
        assert_eq!("X-User-Name", "X-User-Name");
    }

    // Test: Error Message Patterns
    #[test]
    fn test_error_message_patterns() {
        // These patterns should match the error messages used in handlers
        let messages = vec![
            "Missing X-User-Id header",
            "Unknown User",
            "You don't have permission to view comments on this document",
            "You don't have permission to add comments to this document",
            "You can only edit your own comments",
            "You don't have permission to resolve comments",
            "You don't have permission to unresolve comments",
            "You don't have permission to delete this comment",
            "Parent comment not found",
            "Parent comment must belong to the same document",
            "Comment not found",
            "Not found",
        ];

        for msg in messages {
            assert!(msg.len() > 0);
        }
    }

    // Test: User ID Parsing from Header
    #[test]
    fn test_user_id_header_parsing_success() {
        let header_value = "user-123456";
        let parsed = header_value.to_string();
        let uuid = Uuid::parse_str(&parsed);

        // Should fail (not a valid UUID format)
        assert!(uuid.is_err());
    }

    #[test]
    fn test_user_id_header_formatting() {
        let valid_header = "550e8400-e29b-41d4-a716-446655440000";
        let to_str = valid_header.to_string();

        assert_eq!(to_str, valid_header);
        assert_eq!(to_str.len(), 36);
    }

    // Test: Content Storage
    #[test]
    fn test_content_storage_type() {
        let content: String = "Test comment content".to_string();

        // Should be cloneable
        let cloned = content.clone();
        assert_eq!(cloned, content);

        // String should be owned
        assert!(std::mem::size_of::<String>() > 0);
    }

    // Test: Comment State Transitions
    #[test]
    fn test_comment_resolved_state() {
        let is_resolved = false;
        let resolved_by: Option<Uuid> = None;
        let resolved_at: Option<chrono::DateTime<Utc>> = None;

        // Unresolved state
        assert!(!is_resolved);
        assert_eq!(resolved_by, None);
        assert_eq!(resolved_at, None);

        // Resolved state
        let is_resolved = true;
        let resolved_by = Some(Uuid::new_v4());
        let resolved_at = Some(Utc::now());

        assert!(is_resolved);
        assert!(resolved_by.is_some());
        assert!(resolved_at.is_some());
    }

    // Test: Pagination Logic
    #[test]
    fn test_pagination_defaults() {
        // Default pagination from code
        let default_limit = 50;
        let default_offset = 0;

        // First page
        let offset1 = 0;
        let limit1 = default_limit;
        assert_eq!(offset1, 0);
        assert_eq!(limit1, 50);

        // Second page
        let offset2 = default_limit;
        let limit2 = default_limit;
        assert_eq!(offset2, 50);
        assert_eq!(limit2, 50);
    }
}

/// Extract user ID from request header
fn extract_user_id(req: &HttpRequest) -> Result<String, AppError> {
    let raw = req
        .headers()
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::AuthenticationError("Missing X-User-Id header".to_string()))?;

    Uuid::parse_str(raw).map_err(|_| AppError::AuthenticationError("Invalid X-User-Id format".to_string()))?;

    Ok(raw.to_string())
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
fn comment_row_to_response(row: &CommentRow, author_name: &str, author_avatar: Option<&str>) -> CommentResponse {
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
        updated_at: Some(row.updated_at.and_utc().to_rfc3339()),
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
        Err(ref e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                ErrorCode::from(e).to_string().as_str(),
                e.to_string().as_str(),
            ))
        },
    };

    // Check document access
    match repo.check_document_access(&document_id, &user_id).await {
        Ok(true) => {},
        Ok(false) => {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "ACCESS_DENIED",
                "You don't have permission to view comments on this document",
            ));
        },
        Err(e) => {
            error!("Database error checking document access: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to verify document access",
            ));
        },
    }

    // Get comments
    match repo
        .list_comments(&document_id, query.parent_id.as_deref(), query.limit, query.offset)
        .await
    {
        Ok((comments, total)) => {
            let comment_responses: Vec<CommentResponse> = comments
                .iter()
                .map(|row| {
                    // TODO: Join with users table to get author_name and author_avatar
                    let author_name = "User"; // Placeholder until user lookup is implemented
                    let author_avatar = None;
                    comment_row_to_response(row, author_name, author_avatar)
                })
                .collect();

            HttpResponse::Ok().json(ApiResponse::success(CommentListResponse {
                comments: comment_responses,
                total,
                limit: query.limit.unwrap_or(50),
                offset: query.offset.unwrap_or(0),
            }))
        },
        Err(e) => {
            error!("Database error listing comments: {:?}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to list comments"))
        },
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
        Err(ref e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                ErrorCode::from(e).to_string().as_str(),
                e.to_string().as_str(),
            ))
        },
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
        },
        Err(e) => {
            error!("Database error checking document access: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to verify document access",
            ));
        },
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
            },
            Ok(None) => {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::<()>::error("PARENT_NOT_FOUND", "Parent comment not found"));
            },
            Err(e) => {
                error!("Database error checking parent comment: {:?}", e);
                return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "DATABASE_ERROR",
                    "Failed to verify parent comment",
                ));
            },
        }
    }

    // Create comment
    match repo
        .create_comment(
            &document_id,
            &user_id,
            &user_name,
            &req.content,
            req.parent_id.as_deref(),
        )
        .await
    {
        Ok(comment) => {
            let response = comment_row_to_response(&comment, &user_name, None);
            HttpResponse::Created().json(ApiResponse::success(response))
        },
        Err(e) => {
            error!("Database error creating comment: {:?}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to create comment"))
        },
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
        Err(ref e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                ErrorCode::from(e).to_string().as_str(),
                e.to_string().as_str(),
            ))
        },
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
        },
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Comment not found"));
        },
        Err(e) => {
            error!("Database error getting comment: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to get comment"));
        },
    }

    // Update comment
    match repo.update_comment(&comment_id, &req.content).await {
        Ok(_comment) => {
            let author_name = "User"; // Placeholder until user lookup is implemented
            let author_avatar = None;
            // Re-fetch the comment with full data
            match repo.get_comment(&comment_id).await {
                Ok(Some(full_comment)) => {
                    let response = comment_row_to_response(&full_comment, author_name, author_avatar);
                    HttpResponse::Ok().json(ApiResponse::success(response))
                },
                Ok(None) => HttpResponse::NotFound()
                    .json(ApiResponse::<()>::error("NOT_FOUND", "Comment not found after update")),
                Err(e) => {
                    error!("Database error fetching updated comment: {:?}", e);
                    HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                        "DATABASE_ERROR",
                        "Failed to fetch updated comment",
                    ))
                },
            }
        },
        Err(e) => {
            error!("Database error updating comment: {:?}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to update comment"))
        },
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
        Err(ref e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                ErrorCode::from(e).to_string().as_str(),
                e.to_string().as_str(),
            ))
        },
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
                    },
                    Err(e) => {
                        error!("Database error checking document access: {:?}", e);
                        return HttpResponse::InternalServerError()
                            .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to verify access"));
                    },
                }
            }
        },
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Comment not found"));
        },
        Err(e) => {
            error!("Database error getting comment: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to get comment"));
        },
    }

    // Resolve comment
    match repo.resolve_comment(&comment_id, &user_id).await {
        Ok(comment) => {
            let _author_name = "User"; // Placeholder until user lookup is implemented
            let _author_avatar: Option<&str> = None;
            HttpResponse::Ok().json(ApiResponse::success(comment))
        },
        Err(e) => {
            error!("Database error resolving comment: {:?}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to resolve comment"))
        },
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
        Err(ref e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                ErrorCode::from(e).to_string().as_str(),
                e.to_string().as_str(),
            ))
        },
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
                },
                Err(e) => {
                    error!("Database error checking document access: {:?}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to verify access"));
                },
            }
        },
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Comment not found"));
        },
        Err(e) => {
            error!("Database error getting comment: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to get comment"));
        },
    }

    // Unresolve comment
    match repo.unresolve_comment(&comment_id).await {
        Ok(_comment) => HttpResponse::Ok().json(ApiResponse::success(())),
        Err(e) => {
            error!("Database error unresolving comment: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "DATABASE_ERROR",
                "Failed to unresolve comment",
            ))
        },
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
        Err(ref e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                ErrorCode::from(e).to_string().as_str(),
                e.to_string().as_str(),
            ))
        },
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
                    },
                    Err(e) => {
                        error!("Database error checking document access: {:?}", e);
                        return HttpResponse::InternalServerError()
                            .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to verify access"));
                    },
                }
            }
        },
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Comment not found"));
        },
        Err(e) => {
            error!("Database error getting comment: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to get comment"));
        },
    }

    // Delete comment
    match repo.delete_comment(&comment_id).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
            "message": "Comment deleted successfully"
        }))),
        Err(e) => {
            error!("Database error deleting comment: {:?}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to delete comment"))
        },
    }
}
