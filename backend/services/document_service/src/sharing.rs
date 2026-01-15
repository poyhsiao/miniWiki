use actix_web::{web, Responder, HttpResponse, HttpRequest};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use rand::Rng;
use tracing::{info, error};
use serde::{Serialize, Deserialize};
use validator::Validate;
use shared_errors::AppError;
use bcrypt::{hash, verify, DEFAULT_COST};

const SHARE_TOKEN_LENGTH: usize = 32;
const DEFAULT_EXPIRY_DAYS: i64 = 30;
const MAX_ACCESS_CODE_LENGTH: usize = 10;
const MIN_ACCESS_CODE_LENGTH: usize = 4;

/// Request to create a new share link
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateShareLinkRequest {
    #[serde(rename = "documentId")]
    pub document_id: String,

    #[serde(rename = "accessCode")]
    #[validate(length(min = "4", max = "10", message = "Access code must be 4-10 characters"))]
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
    #[validate(length(min = "4", max = "10", message = "Access code must be 4-10 characters"))]
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
            "Permission must be 'view' or 'comment'".to_string()
        ));
    }

    // Validate document_id is a valid UUID
    let document_id = Uuid::parse_str(&create_req.document_id).map_err(|_| {
        AppError::ValidationError("Invalid document ID format".to_string())
    })?;

    // Extract user_id from JWT token (optional for some flows)
    let user_id = extract_user_id_from_request(&req).await?;

    // Generate share token
    let token = generate_share_token();

    // Hash access code if provided, otherwise leave as None
    let access_code_hash = if let Some(code) = &create_req.access_code {
        let code = code.trim();
        if !code.is_empty() {
            // User provided a non-empty code, hash it
            let hashed = hash(code, DEFAULT_COST)
                .map_err(|e| {
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
        Some(chrono::DateTime::parse_from_rfc3339(&expires_str)
            .map_err(|_| AppError::ValidationError("Invalid expires_at format".to_string()))?
            .with_timezone(&Utc))
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

/// Get all share links for a document
pub async fn get_document_share_links(
    pool: web::Data<PgPool>,
    path: web::Path<(String,)>,
) -> Result<impl Responder, AppError> {
    let document_id_str = path.into_inner().0;
    let document_id = Uuid::parse_str(&document_id_str).map_err(|_| {
        AppError::ValidationError("Invalid document ID format".to_string())
    })?;

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

    let shares = sqlx::query_as::<_, (
        Uuid, Uuid, String, Option<String>, Option<DateTime<Utc>>,
        String, bool, DateTime<Utc>, i32, Option<i32>, DateTime<Utc>,
        String, String
    )>(query)
        .bind(document_id)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            error!("Failed to get share links: {:?}", e);
            AppError::DatabaseError(e)
        })?;

    let response: Vec<ShareLinkDetailResponse> = shares
        .into_iter()
        .map(|(id, _, token, access_code, expires_at, permission, is_active, created_at, click_count, max_access_count, _, title, creator_name)| {
            ShareLinkDetailResponse {
                id: id.to_string(),
                document_id: document_id.to_string(),
                document_title: title,
                token,
                access_code_required: access_code.is_some(),
                expires_at: expires_at.map(|d| d.to_rfc3339()),
                permission: permission,
                is_active,
                created_at: created_at.to_rfc3339(),
                click_count,
                max_access_count,
                created_by: creator_name,
            }
        })
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

    let result = sqlx::query_as::<_, (
        Uuid, Uuid, String, Option<String>, Option<DateTime<Utc>>,
        String, bool, DateTime<Utc>, i32, Option<i32>, DateTime<Utc>,
        String, serde_json::Value
    )>(query)
        .bind(&token)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            error!("Failed to get share link: {:?}", e);
            AppError::DatabaseError(e)
        })?;

    match result {
        Some((id, document_id, _, access_code, expires_at, permission, is_active, _, click_count, max_access_count, _, title, content)) => {
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
                    return Err(AppError::NotFoundError("Share link has reached maximum access count".to_string()));
                }
            }

            // Increment click count
            let update_query = r#"UPDATE share_links SET click_count = click_count + 1 WHERE id = $1"#;
            sqlx::query(update_query)
                .bind(id)
                .execute(pool.get_ref())
                .await
                .ok(); // Ignore errors on click count update

            // Check if access code is required
            let requires_access_code = access_code.is_some();

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": id.to_string(),
                "document_id": document_id.to_string(),
                "document_title": title,
                "document_content": content,
                "requires_access_code": requires_access_code,
                "permission": permission,
                "expires_at": expires_at.map(|d| d.to_rfc3339()),
            })))
        }
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

    let result = sqlx::query_as::<_, (
        Uuid, Uuid, String, Option<String>, Option<DateTime<Utc>>,
        String, bool, DateTime<Utc>, i32, Option<i32>, DateTime<Utc>,
        String, serde_json::Value
    )>(query)
        .bind(&token)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            error!("Failed to verify share link: {:?}", e);
            AppError::DatabaseError(e)
        })?;

    match result {
        Some((id, document_id, _, Some(stored_code), expires_at, permission, is_active, _, click_count, max_access_count, _, title, content)) => {
            // Check if link is active
            if !is_active {
                return Err(AppError::AuthenticationError("Share link has been deactivated".to_string()));
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
                    return Err(AppError::AuthenticationError("Share link has reached maximum access count".to_string()));
                }
            }

            // Verify access code using bcrypt (constant-time comparison built-in)
            let is_valid = verify(access_code, &stored_code)
                .map_err(|e| {
                    error!("Failed to verify access code: {:?}", e);
                    AppError::AuthenticationError("Invalid access code".to_string())
                })?;

            if !is_valid {
                return Err(AppError::AuthenticationError("Invalid access code".to_string()));
            }

            // Increment click count
            let update_query = r#"UPDATE share_links SET click_count = click_count + 1 WHERE id = $1"#;
            sqlx::query(update_query)
                .bind(id)
                .execute(pool.get_ref())
                .await
                .ok();

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": id.to_string(),
                "document_id": document_id.to_string(),
                "document_title": title,
                "document_content": content,
                "permission": permission,
                "expires_at": expires_at.map(|d| d.to_rfc3339()),
                "verified": true,
            })))
        }
        _ => Err(AppError::NotFoundError("Share link not found or access code not required".to_string())),
    }
}

/// Delete a share link with authorization check
pub async fn delete_share_link(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, AppError> {
    let (document_id_str, token) = path.into_inner();
    let document_id = Uuid::parse_str(&document_id_str).map_err(|_| {
        AppError::ValidationError("Invalid document ID format".to_string())
    })?;

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
                return Err(AppError::AuthorizationError("You do not have permission to delete this share link".to_string()));
            }
        }
        None => {
            return Err(AppError::NotFoundError("Share link not found".to_string()));
        }
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

    info!("Deleted share link {} for document {} by user {}", token, document_id, user_id);

    Ok(HttpResponse::NoContent().finish())
}

/// Helper function to extract user ID from request
async fn extract_user_id_from_request(_req: &HttpRequest) -> Result<Uuid, AppError> {
    // In a real implementation, this would extract user ID from JWT token
    // For now, we'll return a default UUID if not found
    // This should be replaced with proper JWT extraction
    Ok(Uuid::nil())
}
