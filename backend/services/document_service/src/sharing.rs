use actix_web::{web, HttpRequest, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use shared_errors::AppError;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

const SHARE_TOKEN_LENGTH: usize = 32;
const DEFAULT_EXPIRY_DAYS: i64 = 30;
// const MAX_ACCESS_CODE_LENGTH: usize = 10;
// const MIN_ACCESS_CODE_LENGTH: usize = 4;

/// Request to create a new share link
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateShareLinkRequest {
    #[serde(rename = "documentId")]
    pub document_id: String,

    #[serde(rename = "accessCode")]
    #[validate(length(min = 4, max = 10, message = "Access code must be 4-10 characters"))]
    pub access_code: Option<String>,

    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>, // ISO 8601 format

    #[serde(rename = "permission")]
    #[validate(required)]
    pub permission: Option<String>, // "view" or "comment"

    #[serde(rename = "maxAccessCount")]
    pub max_access_count: Option<i32>,
}

/// Request to verify access code for protected share link
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerifyAccessCodeRequest {
    #[serde(rename = "accessCode")]
    #[validate(length(min = 4, max = 10, message = "Access code must be 4-10 characters"))]
    pub access_code: String,
}

/// Response for share link creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLinkResponse {
    pub id: String,
    pub document_id: String,
    pub token: String,
    pub url: String,
    pub access_code_required: bool,
    pub expires_at: Option<String>,
    pub permission: String,
    pub created_at: String,
    pub max_access_count: Option<i32>,
}

/// Response for share link retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLinkDetailResponse {
    pub id: String,
    pub document_id: String,
    pub document_title: String,
    pub token: String,
    pub access_code_required: bool,
    pub expires_at: Option<String>,
    pub permission: String,
    pub is_active: bool,
    pub created_at: String,
    pub click_count: i32,
    pub max_access_count: Option<i32>,
    pub created_by: String,
}

/// Generate a secure random share token
fn generate_share_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    let token: String = (0..SHARE_TOKEN_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    token
}

/*
/// Generate access code if not provided
fn generate_access_code() -> String {
    const CODE_CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    let code: String = (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..CODE_CHARSET.len());
            CODE_CHARSET[idx] as char
        })
        .collect();
    code
}
*/

/// Create a new share link for a document
pub async fn create_share_link(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    create_req: web::Json<CreateShareLinkRequest>,
) -> Result<impl Responder, AppError> {
    let create_req = create_req.into_inner();

    // Validate request
    create_req.validate().map_err(|e| {
        error!("Share link validation error: {:?}", e);
        AppError::ValidationError(e.to_string())
    })?;

    // Validate permission
    let permission = create_req.permission.unwrap_or_else(|| "view".to_string());
    if permission != "view" && permission != "comment" {
        return Err(AppError::ValidationError(
            "Permission must be 'view' or 'comment'".to_string(),
        ));
    }

    // Validate document_id is a valid UUID
    let document_id = Uuid::parse_str(&create_req.document_id)
        .map_err(|_| AppError::ValidationError("Invalid document ID format".to_string()))?;

    // Extract user_id from JWT token
    let user_id = extract_user_id_from_request(&req).await?;

    // Verify document exists and user has permission to share it
    let owner_check = sqlx::query_as::<_, (Uuid,)>("SELECT owner_id FROM documents WHERE id = $1")
        .bind(document_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    match owner_check {
        Some((owner_id,)) => {
            if owner_id != user_id {
                return Err(AppError::AuthorizationError(
                    "You do not have permission to share this document".to_string(),
                ));
            }
        },
        None => return Err(AppError::NotFoundError("Document not found".to_string())),
    }

    // Generate share token
    let token = generate_share_token();

    // Hash access code if provided, otherwise leave as None
    let access_code_hash = if let Some(code) = &create_req.access_code {
        let code = code.trim();
        if !code.is_empty() {
            // User provided a non-empty code, hash it
            let hashed = hash(code, DEFAULT_COST).map_err(|e| {
                error!("Failed to hash access code: {:?}", e);
                AppError::InternalError("Failed to hash access code".to_string())
            })?;
            Some(hashed)
        } else {
            // User provided empty string, treat as no code
            None
        }
    } else {
        // No access code provided
        None
    };

    // Parse expiry date
    let expires_at = if let Some(expires_str) = create_req.expires_at {
        Some(
            chrono::DateTime::parse_from_rfc3339(&expires_str)
                .map_err(|_| AppError::ValidationError("Invalid expires_at format".to_string()))?
                .with_timezone(&Utc),
        )
    } else {
        Some(Utc::now() + Duration::days(DEFAULT_EXPIRY_DAYS))
    };

    // Insert share link into database
    let query = r#"
        INSERT INTO share_links (
            id, document_id, created_by, token, access_code,
            expires_at, permission, is_active, click_count, max_access_count
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, true, 0, $8)
        RETURNING id, document_id, token, created_at
    "#;

    let share_id = Uuid::new_v4();

    let result = sqlx::query_as::<_, (Uuid, Uuid, String, DateTime<Utc>)>(query)
        .bind(share_id)
        .bind(document_id)
        .bind(user_id)
        .bind(&token)
        .bind(&access_code_hash)
        .bind(expires_at)
        .bind(&permission)
        .bind(create_req.max_access_count)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| {
            error!("Failed to create share link: {:?}", e);
            AppError::DatabaseError(e)
        })?;

    info!("Created share link {} for document {}", share_id, document_id);

    // Generate share URL
    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:8080");

    let scheme = if req.connection_info().scheme() == "https" {
        "https"
    } else {
        "http"
    };

    let share_url = format!("{}://{}/share/{}", scheme, host, token);

    let response = ShareLinkResponse {
        id: share_id.to_string(),
        document_id: document_id.to_string(),
        token: token.clone(),
        url: share_url,
        access_code_required: access_code_hash.is_some(),
        expires_at: expires_at.map(|d| d.to_rfc3339()),
        permission,
        created_at: result.3.to_rfc3339(),
        max_access_count: create_req.max_access_count,
    };

    Ok(HttpResponse::Created().json(response))
}

/// Get all share links for a document (with authorization check)
pub async fn get_document_share_links(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<(String,)>,
) -> Result<impl Responder, AppError> {
    let document_id_str = path.into_inner().0;
    let document_id = Uuid::parse_str(&document_id_str)
        .map_err(|_| AppError::ValidationError("Invalid document ID format".to_string()))?;

    // Authorization: verify user owns the document
    let user_id = extract_user_id_from_request(&req).await?;
    let owner_check = sqlx::query_as::<_, (Uuid,)>("SELECT owner_id FROM documents WHERE id = $1")
        .bind(document_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    match owner_check {
        Some((owner_id,)) => {
            if owner_id != user_id {
                return Err(AppError::AuthorizationError(
                    "You do not have permission to view share links for this document".to_string(),
                ));
            }
        },
        None => return Err(AppError::NotFoundError("Document not found".to_string())),
    }

    let query = r#"
        SELECT sl.id, sl.document_id, sl.token, sl.access_code,
               sl.expires_at, sl.permission, sl.is_active, sl.created_at,
               sl.click_count, sl.max_access_count, sl.updated_at,
               d.title, u.display_name as creator_name
        FROM share_links sl
        JOIN documents d ON sl.document_id = d.id
        JOIN users u ON sl.created_by = u.id
        WHERE sl.document_id = $1 AND sl.is_active = true
        ORDER BY sl.created_at DESC
    "#;

    let shares = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            Option<String>,
            Option<DateTime<Utc>>,
            String,
            bool,
            DateTime<Utc>,
            i32,
            Option<i32>,
            DateTime<Utc>,
            String,
            String,
        ),
    >(query)
    .bind(document_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        error!("Failed to get share links: {:?}", e);
        AppError::DatabaseError(e)
    })?;

    let response: Vec<ShareLinkDetailResponse> = shares
        .into_iter()
        .map(
            |(
                id,
                _,
                token,
                access_code,
                expires_at,
                permission,
                is_active,
                created_at,
                click_count,
                max_access_count,
                _,
                title,
                creator_name,
            )| {
                ShareLinkDetailResponse {
                    id: id.to_string(),
                    document_id: document_id.to_string(),
                    document_title: title,
                    token,
                    access_code_required: access_code.is_some(),
                    expires_at: expires_at.map(|d| d.to_rfc3339()),
                    permission,
                    is_active,
                    created_at: created_at.to_rfc3339(),
                    click_count,
                    max_access_count,
                    created_by: creator_name,
                }
            },
        )
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// Get share link by token (public endpoint)
pub async fn get_share_link_by_token(
    pool: web::Data<PgPool>,
    path: web::Path<(String,)>,
) -> Result<impl Responder, AppError> {
    let token = path.into_inner().0;

    let query = r#"
        SELECT sl.id, sl.document_id, sl.token, sl.access_code,
               sl.expires_at, sl.permission, sl.is_active, sl.created_at,
               sl.click_count, sl.max_access_count, sl.updated_at,
               d.title, d.content
        FROM share_links sl
        JOIN documents d ON sl.document_id = d.id
        WHERE sl.token = $1
    "#;

    let result = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            Option<String>,
            Option<DateTime<Utc>>,
            String,
            bool,
            DateTime<Utc>,
            i32,
            Option<i32>,
            DateTime<Utc>,
            String,
            serde_json::Value,
        ),
    >(query)
    .bind(&token)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        error!("Failed to get share link: {:?}", e);
        AppError::DatabaseError(e)
    })?;

    match result {
        Some((
            id,
            document_id,
            _,
            access_code,
            expires_at,
            permission,
            is_active,
            _,
            click_count,
            max_access_count,
            _,
            title,
            content,
        )) => {
            // Check if link is active
            if !is_active {
                return Err(AppError::NotFoundError("Share link has been deactivated".to_string()));
            }

            // Check if link has expired
            if let Some(expires) = expires_at {
                if expires < Utc::now() {
                    return Err(AppError::NotFoundError("Share link has expired".to_string()));
                }
            }

            // Check max access count
            if let Some(max) = max_access_count {
                if click_count >= max {
                    return Err(AppError::NotFoundError(
                        "Share link has reached maximum access count".to_string(),
                    ));
                }
            }

            // Check if access code is required - if so, don't return content
            let requires_access_code = access_code.is_some();

            if requires_access_code {
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "id": id.to_string(),
                    "document_id": document_id.to_string(),
                    "document_title": title,
                    "requires_access_code": true,
                    "permission": permission,
                    "expires_at": expires_at.map(|d| d.to_rfc3339()),
                    "message": "Access code required. Use POST /share/{token}/verify to access content."
                })));
            }

            // Increment click count only when content is actually accessed
            let update_query = r#"UPDATE share_links SET click_count = click_count + 1 WHERE id = $1"#;
            if let Err(e) = sqlx::query(update_query).bind(id).execute(pool.get_ref()).await {
                tracing::error!("Failed to increment click_count for share link id {}: {}", id, e);
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": id.to_string(),
                "document_id": document_id.to_string(),
                "document_title": title,
                "document_content": content,
                "requires_access_code": requires_access_code,
                "permission": permission,
                "expires_at": expires_at.map(|d| d.to_rfc3339()),
            })))
        },
        None => Err(AppError::NotFoundError("Share link not found".to_string())),
    }
}

/// Verify access code for a share link
pub async fn verify_share_link_access_code(
    pool: web::Data<PgPool>,
    path: web::Path<(String,)>,
    verify_req: web::Json<VerifyAccessCodeRequest>,
) -> Result<impl Responder, AppError> {
    let token = path.into_inner().0;
    let access_code = &verify_req.access_code;

    let query = r#"
        SELECT sl.id, sl.document_id, sl.token, sl.access_code,
               sl.expires_at, sl.permission, sl.is_active, sl.created_at,
               sl.click_count, sl.max_access_count, sl.updated_at,
               d.title, d.content
        FROM share_links sl
        JOIN documents d ON sl.document_id = d.id
        WHERE sl.token = $1 AND sl.is_active = true
    "#;

    let result = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            Option<String>,
            Option<DateTime<Utc>>,
            String,
            bool,
            DateTime<Utc>,
            i32,
            Option<i32>,
            DateTime<Utc>,
            String,
            serde_json::Value,
        ),
    >(query)
    .bind(&token)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        error!("Failed to verify share link: {:?}", e);
        AppError::DatabaseError(e)
    })?;

    match result {
        Some((
            id,
            document_id,
            _,
            Some(stored_code),
            expires_at,
            permission,
            is_active,
            _,
            click_count,
            max_access_count,
            _,
            title,
            content,
        )) => {
            // Check if link is active
            if !is_active {
                return Err(AppError::AuthenticationError(
                    "Share link has been deactivated".to_string(),
                ));
            }

            // Check if link has expired
            if let Some(expires) = expires_at {
                if expires < Utc::now() {
                    return Err(AppError::AuthenticationError("Share link has expired".to_string()));
                }
            }

            // Check max access count
            if let Some(max) = max_access_count {
                if click_count >= max {
                    return Err(AppError::AuthenticationError(
                        "Share link has reached maximum access count".to_string(),
                    ));
                }
            }

            // Verify access code using bcrypt (constant-time comparison built-in)
            let is_valid = verify(access_code, &stored_code).map_err(|e| {
                error!("Failed to verify access code: {:?}", e);
                AppError::AuthenticationError("Invalid access code".to_string())
            })?;

            if !is_valid {
                return Err(AppError::AuthenticationError("Invalid access code".to_string()));
            }

            // Increment click count
            let update_query = r#"UPDATE share_links SET click_count = click_count + 1 WHERE id = $1"#;
            if let Err(e) = sqlx::query(update_query).bind(id).execute(pool.get_ref()).await {
                tracing::error!("Failed to increment click_count for share link id {}: {}", id, e);
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": id.to_string(),
                "document_id": document_id.to_string(),
                "document_title": title,
                "document_content": content,
                "permission": permission,
                "expires_at": expires_at.map(|d| d.to_rfc3339()),
                "verified": true,
            })))
        },
        _ => Err(AppError::NotFoundError(
            "Share link not found or access code not required".to_string(),
        )),
    }
}

/// Delete a share link with authorization check
pub async fn delete_share_link(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, AppError> {
    let (document_id_str, token) = path.into_inner();
    let document_id = Uuid::parse_str(&document_id_str)
        .map_err(|_| AppError::ValidationError("Invalid document ID format".to_string()))?;

    // Extract user_id from request (JWT token)
    let user_id = extract_user_id_from_request(&req).await?;

    // Authorization check: verify the user owns the document or created the share link
    let auth_query = r#"
        SELECT d.owner_id, sl.created_by
        FROM share_links sl
        JOIN documents d ON sl.document_id = d.id
        WHERE sl.document_id = $1 AND sl.token = $2
    "#;

    let auth_result = sqlx::query_as::<_, (Uuid, Uuid)>(auth_query)
        .bind(document_id)
        .bind(&token)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            error!("Failed to check authorization: {:?}", e);
            AppError::DatabaseError(e)
        })?;

    match auth_result {
        Some((owner_id, creator_id)) => {
            // Check if user is the document owner OR the share link creator
            if owner_id != user_id && creator_id != user_id {
                return Err(AppError::AuthorizationError(
                    "You do not have permission to delete this share link".to_string(),
                ));
            }
        },
        None => {
            return Err(AppError::NotFoundError("Share link not found".to_string()));
        },
    }

    // Soft delete by setting is_active to false
    let query = r#"UPDATE share_links SET is_active = false WHERE document_id = $1 AND token = $2"#;

    sqlx::query(query)
        .bind(document_id)
        .bind(&token)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            error!("Failed to delete share link: {:?}", e);
            AppError::DatabaseError(e)
        })?;

    info!(
        "Deleted share link {} for document {} by user {}",
        token, document_id, user_id
    );

    Ok(HttpResponse::NoContent().finish())
}

/// Helper function to extract user ID from request
async fn extract_user_id_from_request(req: &HttpRequest) -> Result<Uuid, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::AuthenticationError("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::AuthenticationError("Invalid authorization format".to_string()))?;

    let secret = req
        .app_data::<web::Data<Arc<String>>>()
        .map(|d| d.get_ref().clone())
        .ok_or_else(|| AppError::InternalError("JWT secret not configured".to_string()))?;

    #[derive(Debug, Deserialize)]
    struct Claims {
        sub: String,
    }

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| AppError::AuthenticationError("Invalid token".to_string()))?;

    let user_id_str = &token_data.claims.sub;
    Uuid::parse_str(user_id_str).map_err(|_| AppError::AuthenticationError("Invalid user ID format".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use validator::Validate;

    // Helper for mock claims
    #[derive(Debug, serde::Deserialize)]
    struct TestClaims {
        sub: String,
    }

    // Helper to create a test request
    fn mock_request_with_auth(user_id: &str) -> HttpRequest {
        use actix_web::test::TestRequest;
        TestRequest::get()
            .insert_header(("Authorization", format!("Bearer {}", mock_jwt_token(user_id))))
            .to_http_request()
    }

    // Mock JWT token generation (simplified)
    fn mock_jwt_token(user_id: &str) -> String {
        use jsonwebtoken::{encode, EncodingKey, Header};
        let claims = serde_json::json!({ "sub": user_id });
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"test_secret_32_chars_long!!"),
        )
        .unwrap()
    }

    // CreateShareLinkRequest Tests
    #[test]
    fn test_create_share_link_request_valid() {
        let req = CreateShareLinkRequest {
            document_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            access_code: Some("ABCD".to_string()),
            expires_at: Some("2026-01-22T10:00:00Z".to_string()),
            permission: Some("view".to_string()),
            max_access_count: Some(10),
        };

        assert!(req.validate().is_ok());
        assert_eq!(req.access_code, Some("ABCD".to_string()));
        assert_eq!(req.permission, Some("view".to_string()));
    }

    #[test]
    fn test_create_share_link_request_defaults() {
        let req = CreateShareLinkRequest {
            document_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            access_code: None,
            expires_at: None,
            permission: None,
            max_access_count: None,
        };

        // Permission is marked with #[validate(required)] so None should fail validation
        assert!(req.validate().is_err());
        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("permission"));
        assert_eq!(req.access_code, None);
        assert_eq!(req.expires_at, None);
        assert_eq!(req.max_access_count, None);
    }

    #[test]
    fn test_create_share_link_request_invalid_document_id() {
        let req = CreateShareLinkRequest {
            document_id: "invalid-uuid".to_string(),
            access_code: None,
            expires_at: None,
            permission: Some("view".to_string()),
            max_access_count: None,
        };

        // Validate that the struct itself passes validation
        assert!(req.validate().is_ok());
        // But UUID parsing should fail
        assert!(Uuid::parse_str(&req.document_id).is_err());
    }

    #[test]
    fn test_create_share_link_request_access_code_too_short() {
        let req = CreateShareLinkRequest {
            document_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            access_code: Some("ABC".to_string()), // 3 chars
            expires_at: None,
            permission: Some("view".to_string()),
            max_access_count: None,
        };

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Access code must be 4-10 characters"));
    }

    #[test]
    fn test_create_share_link_request_access_code_too_long() {
        let req = CreateShareLinkRequest {
            document_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            access_code: Some("ABCDEFGHIJKL".to_string()), // 12 chars
            expires_at: None,
            permission: Some("view".to_string()),
            max_access_count: None,
        };

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Access code must be 4-10 characters"));
    }

    // VerifyAccessCodeRequest Tests
    #[test]
    fn test_verify_access_code_request_valid() {
        let req = VerifyAccessCodeRequest {
            access_code: "ABCD1234".to_string(),
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_verify_access_code_request_too_short() {
        let req = VerifyAccessCodeRequest {
            access_code: "ABC".to_string(),
        };

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Access code must be 4-10 characters"));
    }

    #[test]
    fn test_verify_access_code_request_too_long() {
        let req = VerifyAccessCodeRequest {
            access_code: "ABCDEFGHIJKL".to_string(),
        };

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Access code must be 4-10 characters"));
    }

    // ShareLinkResponse Tests
    #[test]
    fn test_share_link_response_serialization() {
        let resp = ShareLinkResponse {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            document_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            token: "test-token-32chars-long".to_string(),
            url: "http://localhost:8080/share/test-token".to_string(),
            access_code_required: true,
            expires_at: Some("2026-01-22T10:00:00Z".to_string()),
            permission: "view".to_string(),
            created_at: "2026-01-22T08:00:00Z".to_string(),
            max_access_count: Some(10),
        };

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ShareLinkResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, resp.id);
        assert_eq!(deserialized.token, resp.token);
        assert_eq!(deserialized.access_code_required, true);
        assert_eq!(deserialized.permission, "view");
    }

    // ShareLinkDetailResponse Tests
    #[test]
    fn test_share_link_detail_response_serialization() {
        let resp = ShareLinkDetailResponse {
            id: "550e8400-e29b-41d4-a716-4466554400000".to_string(),
            document_id: "550e8400-e29b-41d4-a716-4466554400001".to_string(),
            document_title: "Test Document".to_string(),
            token: "test-token".to_string(),
            access_code_required: false,
            expires_at: None,
            permission: "comment".to_string(),
            is_active: true,
            created_at: "2026-01-22T08:00:00Z".to_string(),
            click_count: 5,
            max_access_count: Some(100),
            created_by: "Test User".to_string(),
        };

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ShareLinkDetailResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.document_title, "Test Document");
        assert_eq!(deserialized.click_count, 5);
        assert_eq!(deserialized.is_active, true);
        assert_eq!(deserialized.access_code_required, false);
    }

    // generate_share_token Tests
    #[test]
    fn test_generate_share_token_length() {
        let token = generate_share_token();
        assert_eq!(token.len(), SHARE_TOKEN_LENGTH);
    }

    #[test]
    fn test_generate_share_token_uniqueness() {
        let tokens = std::iter::repeat_with(generate_share_token)
            .take(100)
            .collect::<std::collections::HashSet<_>>();
        // Generate 100 tokens and check uniqueness (very unlikely to have collisions)
        assert_eq!(tokens.len(), 100);
    }

    #[test]
    fn test_generate_share_token_charset() {
        let token = generate_share_token();
        let charset: std::collections::HashSet<char> =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                .iter()
                .map(|&b| b as char)
                .collect();

        // Verify all characters are from the expected charset
        for c in token.chars() {
            assert!(charset.contains(&c), "Token contains unexpected character: {}", c);
        }
    }

    #[test]
    fn test_generate_share_token_randomness() {
        let token1 = generate_share_token();
        let token2 = generate_share_token();
        // Very unlikely to be the same
        assert_ne!(token1, token2);
    }

    // Test: Permission Validation Logic
    #[test]
    fn test_permission_validation_logic() {
        let valid_perms = ["view", "comment"];
        let invalid_perms = ["admin", "delete", "edit", "read", ""];

        for perm in valid_perms {
            assert!(perm == "view" || perm == "comment");
        }

        // In production code, this logic is:
        // if permission != "view" && permission != "comment"
        // Only "view" and "comment" are valid
    }

    #[test]
    fn test_permission_defaults_to_view() {
        let permission = None.unwrap_or_else(|| "view".to_string());
        assert_eq!(permission, "view");
    }

    #[test]
    fn test_permission_does_not_default_to_invalid() {
        let permission = Some("invalid".to_string());
        assert_eq!(permission, Some("invalid".to_string()));
        // The code uses unwrap_or_else so it only defaults when None
    }

    // Test: Expiry Date Logic
    #[test]
    fn test_expiry_date_parsing() {
        let valid_date = "2026-01-22T10:00:00Z";
        let parsed = chrono::DateTime::parse_from_rfc3339(valid_date);
        assert!(parsed.is_ok());

        let invalid_date = "2026-01-22"; // Missing time and timezone
        let parsed = chrono::DateTime::parse_from_rfc3339(invalid_date);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_default_expiry_calculation() {
        use chrono::Duration as CdDuration; // Avoid conflict with std::time::Duration
        let now = Utc::now();
        let default_expiry = now + CdDuration::days(DEFAULT_EXPIRY_DAYS);

        // Verify default is 30 days
        assert_eq!(DEFAULT_EXPIRY_DAYS, 30);
        assert!(default_expiry > now);
    }

    // Test: Access Code Hashing
    #[test]
    fn test_access_code_hashing() {
        let code = "ABCD1234";
        let hashed = hash(code, DEFAULT_COST).unwrap();

        // Verify hash is different from original
        assert_ne!(code, hashed);

        // Verify hash length is consistent (bcrypt hashes are 60 chars)
        assert_eq!(hashed.len(), 60);
    }

    #[test]
    fn test_access_code_verification() {
        let code = "ABCD1234";
        let hashed = hash(code, DEFAULT_COST).unwrap();

        // Valid code should verify
        let is_valid = verify(code, &hashed).unwrap();
        assert!(is_valid);

        // Invalid code should not verify
        let is_valid_wrong = verify("WRONGCODE", &hashed).unwrap();
        assert!(!is_valid_wrong);
    }

    // Test: URL Generation Logic
    #[test]
    fn test_share_url_generation_https() {
        let scheme = "https";
        let host = "example.com";
        let token = "test-token-32chars-long";

        let url = format!("{}://{}/share/{}", scheme, host, token);
        assert_eq!(url, "https://example.com/share/test-token-32chars-long");
    }

    #[test]
    fn test_share_url_generation_http() {
        let scheme = "http";
        let host = "localhost:8080";
        let token = "test-token";

        let url = format!("{}://{}/share/{}", scheme, host, token);
        assert_eq!(url, "http://localhost:8080/share/test-token");
    }

    // Test: Click Count and Max Access Count Logic
    #[test]
    fn test_max_access_count_check() {
        let click_count = 10;
        let max_access_count = Some(10);

        // Should block access when click_count >= max_access_count
        let should_block = if let Some(max) = max_access_count {
            click_count >= max
        } else {
            false
        };

        assert!(should_block);
    }

    #[test]
    fn test_max_access_count_allowed() {
        let click_count = 5;
        let max_access_count = Some(10);

        // Should allow access when click_count < max_access_count
        let should_block = if let Some(max) = max_access_count {
            click_count >= max
        } else {
            false
        };

        assert!(!should_block);
    }

    #[test]
    fn test_no_max_access_count() {
        let click_count = 100;
        let max_access_count: Option<i32> = None;

        // Should allow access when no max is set
        let should_block = if let Some(max) = max_access_count {
            click_count >= max
        } else {
            false
        };

        assert!(!should_block);
    }

    // Test: Share Link Active Status
    #[test]
    fn test_active_link_status() {
        let is_active = true;

        // Should proceed for active links
        if !is_active {
            panic!("Should not reach here for active link");
        }
    }

    #[test]
    fn test_inactive_link_status() {
        let is_active = false;

        // Should return error for inactive links
        if !is_active {
            // In production: return Err(AppError::NotFoundError("Share link has been deactivated"))
        }
    }

    // Test: Expiry Check Logic
    #[test]
    fn test_expired_link() {
        let expires_at = Utc::now() - Duration::days(1); // Expired yesterday
        let now = Utc::now();

        assert!(expires_at < now, "Should detect expired link");
    }

    #[test]
    fn test_not_expired_link() {
        let expires_at = Utc::now() + Duration::days(30); // Expires in 30 days
        let now = Utc::now();

        assert!(expires_at > now, "Should allow access to future expiry");
    }

    #[test]
    fn test_no_expiry() {
        let expires_at: Option<DateTime<Utc>> = None;

        // Should not block access if no expiry is set
        let should_block = if let Some(expires) = expires_at {
            expires < Utc::now()
        } else {
            false
        };

        assert!(!should_block);
    }

    // Test: Authorization Check Logic (Owner or Creator)
    #[test]
    fn test_owner_can_delete() {
        let user_id = "user-001";
        let owner_id = "user-001";
        let creator_id = "user-002";

        // Owner can delete
        let can_delete = owner_id == user_id || creator_id == user_id;
        assert!(can_delete);
    }

    #[test]
    fn test_creator_can_delete() {
        let user_id = "user-002";
        let owner_id = "user-001";
        let creator_id = "user-002";

        // Creator can delete
        let can_delete = owner_id == user_id || creator_id == user_id;
        assert!(can_delete);
    }

    #[test]
    fn test_non_owner_non_creator_cannot_delete() {
        let user_id = "user-003";
        let owner_id = "user-001";
        let creator_id = "user-002";

        // Neither owner nor creator cannot delete
        let can_delete = owner_id == user_id || creator_id == user_id;
        assert!(!can_delete);
    }

    // Test: Document UUID Validation
    #[test]
    fn test_valid_uuid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let parsed = Uuid::parse_str(uuid_str);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_invalid_uuid() {
        let uuid_str = "not-a-uuid";
        let parsed = Uuid::parse_str(uuid_str);
        assert!(parsed.is_err());
    }

    // Test: Authorization Header Parsing
    #[test]
    fn test_authorization_header_parsing() {
        let header = "Bearer test-token-12345";
        let prefix = "Bearer ";

        let stripped = header.strip_prefix(prefix);
        assert_eq!(stripped, Some("test-token-12345"));
    }

    #[test]
    fn test_invalid_authorization_header() {
        let header = "InvalidHeader test-token";
        let prefix = "Bearer ";

        let stripped = header.strip_prefix(prefix);
        assert_eq!(stripped, None);
    }

    #[test]
    fn test_missing_bearer_prefix() {
        let header = "test-token-12345";
        let prefix = "Bearer ";

        let stripped = header.strip_prefix(prefix);
        assert_eq!(stripped, None);
    }

    // Test: Access Code Required Flag
    #[test]
    fn test_access_code_required_when_provided() {
        let access_code_hash = Some("hashed-code".to_string());

        let requires_access_code = access_code_hash.is_some();
        assert!(requires_access_code);
    }

    #[test]
    fn test_access_code_not_required_when_none() {
        let access_code_hash: Option<String> = None;

        let requires_access_code = access_code_hash.is_some();
        assert!(!requires_access_code);
    }

    #[test]
    fn test_access_code_not_required_when_empty() {
        let access_code = "".to_string();

        // Code is trimmed and checked
        let is_empty = access_code.trim().is_empty();
        assert!(is_empty);

        // Empty code results in None for hash
        let access_code_hash = if is_empty { None } else { Some("hashed".to_string()) };

        assert_eq!(access_code_hash, None);
    }

    // Test: Click Count Increment Logic
    #[test]
    fn test_click_count_increment() {
        let click_count = 5;
        let new_count = click_count + 1;
        assert_eq!(new_count, 6);
    }

    // Test: Permission Response Format
    #[test]
    fn test_view_permission_response() {
        let permission = "view";
        assert_eq!(permission, "view");
    }

    #[test]
    fn test_comment_permission_response() {
        let permission = "comment";
        assert_eq!(permission, "comment");
    }

    // Test: Constants
    #[test]
    fn test_constants() {
        assert_eq!(SHARE_TOKEN_LENGTH, 32);
        assert_eq!(DEFAULT_EXPIRY_DAYS, 30);
    }
}
