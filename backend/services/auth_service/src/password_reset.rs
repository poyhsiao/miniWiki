use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::repository::AuthRepository;
use shared_errors::AppError;
use shared_models::entities::User;

// ============================================================================
// Request/Response Models
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequestRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResendVerificationEmailRequest {
    pub email: String,
}

// ============================================================================
// Public Functions (called by email_verification.rs)
// ============================================================================

/// Reset user password using valid reset token
pub async fn reset_password(
    req: web::Json<PasswordResetRequest>,
    repo: web::Data<AuthRepository>,
    _jwt_service: web::Data<crate::jwt::JwtService>,
) -> impl actix_web::Responder {
    if let Err(e) = validate_token_format(&req.token) {
        return HttpResponse::BadRequest()
            .json(json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    if let Err(e) = validate_password_strength(&req.new_password) {
        return HttpResponse::BadRequest()
            .json(json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Find reset token and verify it's valid
    let reset_info = match find_valid_reset_token(&req.token, repo.clone()).await {
        Ok(info) => info,
        Err(e) => {
            return match e {
                AppError::NotFound(msg) => HttpResponse::NotFound()
                    .json(json!({ "error": "INVALID_TOKEN", "message": msg })),
                AppError::ValidationError(msg) => HttpResponse::BadRequest()
                    .json(json!({ "error": "INVALID_TOKEN", "message": msg })),
                _ => HttpResponse::InternalServerError()
                    .json(json!({ "error": "INTERNAL_ERROR", "message": e.to_string() })),
            };
        }
    };

    // Hash new password
    use crate::password::hash_password;
    let new_password_hash = match hash_password(&req.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "INTERNAL_ERROR", "message": e.to_string() }));
        }
    };

    // Update user password
    match update_user_password(reset_info.user_id, &new_password_hash, repo.clone()).await {
        Ok(_) => {},
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
        }
    }

    // Mark token as used
    match mark_reset_token_used(&req.token, repo).await {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("Failed to mark reset token as used: {}", e);
        }
    }

    HttpResponse::Ok()
        .json(json!({ "message": "Password reset successfully".to_string() }))
}

/// Request password reset for email
pub async fn request_password_reset(
    req: web::Json<serde_json::Value>,
    repo: web::Data<AuthRepository>,
) -> impl actix_web::Responder {
    let email = match req.get("email") {
        Some(e) => match e.as_str() {
            Some(email_str) => email_str,
            None => {
                return HttpResponse::BadRequest()
                    .json(json!({ "error": "VALIDATION_ERROR", "message": "Email must be a string" }));
            }
        },
        None => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "VALIDATION_ERROR", "message": "Email is required" }));
        }
    };

    if let Err(e) = validate_email_format(email) {
        return HttpResponse::BadRequest()
            .json(json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Check if user exists (but don't reveal if not found for security)
    let _user = match find_user_by_email(email, repo.clone()).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // User doesn't exist, but return success for security
            return HttpResponse::Ok()
                .json(json!({ "message": "If the email is registered, a reset link has been sent".to_string() }));
        }
        Err(e) => {
            tracing::error!("Failed to lookup user: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process request".to_string() }));
        }
    };

    // Generate and store password reset token
    let token = generate_reset_token();
    if let Err(e) = store_reset_token(_user.id, &token, repo.clone()).await {
        tracing::error!("Failed to store reset token: {}", e);
        return HttpResponse::InternalServerError()
            .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process request".to_string() }));
    }

    // TODO: Send email with reset link
    // For now, just log the token (in production, this should send an email)
    tracing::info!("Password reset token generated for {}: {}", email, token);

    HttpResponse::Ok()
        .json(json!({ "message": "If the email is registered, a reset link has been sent".to_string() }))
}

/// Resend verification email
pub async fn resend_verification_email(
    req: web::Json<serde_json::Value>,
    repo: web::Data<AuthRepository>,
) -> impl actix_web::Responder {
    let email = match req.get("email") {
        Some(e) => match e.as_str() {
            Some(email_str) => email_str,
            None => {
                return HttpResponse::BadRequest()
                    .json(json!({ "error": "VALIDATION_ERROR", "message": "Email must be a string" }));
            }
        },
        None => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "VALIDATION_ERROR", "message": "Email is required" }));
        }
    };

    if let Err(e) = validate_email_format(email) {
        return HttpResponse::BadRequest()
            .json(json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Check if user exists
    let user = match find_user_by_email(email, repo.clone()).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(json!({ "error": "USER_NOT_FOUND", "message": "No user found with this email".to_string() }));
        }
        Err(e) => {
            tracing::error!("Failed to lookup user: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process request".to_string() }));
        }
    };

    if user.is_email_verified {
        return HttpResponse::BadRequest()
            .json(json!({ "error": "ALREADY_VERIFIED", "message": "Email is already verified".to_string() }));
    }

    // Generate and store new verification token
    let token = generate_verification_token();
    if let Err(e) = store_verification_token(user.id, &token, repo).await {
        tracing::error!("Failed to store verification token: {}", e);
        return HttpResponse::InternalServerError()
            .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process request".to_string() }));
    }

    // TODO: Send email with verification link
    tracing::info!("Verification token resent for {}: {}", email, token);

    HttpResponse::Ok()
        .json(json!({ "message": "Verification email sent successfully".to_string() }))
}

// ============================================================================
// Helper Functions
// ============================================================================

struct ResetTokenInfo {
    user_id: Uuid,
}

fn validate_token_format(token: &str) -> Result<(), String> {
    if token.len() != 64 {
        return Err("Token must be 64 characters long".to_string());
    }
    if !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Token must contain only hexadecimal characters".to_string());
    }
    Ok(())
}

fn validate_email_format(email: &str) -> Result<(), String> {
    // Basic email validation (should use proper email validator in production)
    if !email.contains('@') || !email.contains('.') || email.len() < 5 || email.len() > 255 {
        return Err("Invalid email format".to_string());
    }
    Ok(())
}

fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one number".to_string());
    }
    Ok(())
}

fn generate_reset_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut hash = format!("{:?}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
    hash.push_str(&Uuid::new_v4().to_string());
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(hash.as_bytes());
    format!("{:x}", hash)
}

fn generate_verification_token() -> String {
    generate_reset_token()
}

// ============================================================================
// Database Operations (TODO: These should be in AuthRepository)
// ============================================================================

async fn find_valid_reset_token(
    token: &str,
    repo: web::Data<AuthRepository>,
) -> Result<ResetTokenInfo, AppError> {
    // TODO: Implement actual database lookup
    // This should check password_resets table for valid token
    Err(AppError::NotFound("Reset token not found or expired".to_string()))
}

async fn mark_reset_token_used(
    token: &str,
    repo: web::Data<AuthRepository>,
) -> Result<(), AppError> {
    // TODO: Implement actual database update
    // This should update password_resets table setting used_at
    Ok(())
}

async fn find_user_by_email(
    email: &str,
    repo: web::Data<AuthRepository>,
) -> Result<Option<User>, AppError> {
    // TODO: This should use AuthRepository.find_by_email
    repo.find_by_email(email).await
}

async fn update_user_password(
    user_id: Uuid,
    password_hash: &str,
    repo: web::Data<AuthRepository>,
) -> Result<(), AppError> {
    // TODO: Implement actual password update
    Ok(())
}

async fn store_reset_token(
    user_id: Uuid,
    token: &str,
    repo: web::Data<AuthRepository>,
) -> Result<(), AppError> {
    // TODO: Implement actual token storage in password_resets table
    Ok(())
}

async fn store_verification_token(
    user_id: Uuid,
    token: &str,
    repo: web::Data<AuthRepository>,
) -> Result<(), AppError> {
    // TODO: Implement actual token storage in email_verifications table
    Ok(())
}
