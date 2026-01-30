use actix_web::{web, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::repository::AuthRepository;
use shared_errors::AppError;
use shared_models::entities::User;

// ============================================================================
// Request/Response Models
// ============================================================================

#[derive(Debug, serde::Deserialize)]
pub struct PasswordResetRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct PasswordResetRequestRequest {
    pub email: String,
}

#[derive(Debug, serde::Deserialize)]
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
    // Validate token format
    if let Err(e) = validate_token_format(&req.token) {
        return HttpResponse::BadRequest().json(json!({ "error": "VALIDATION_ERROR", "message": e }));
    }

    // Validate password strength using shared security module
    if let Err(e) = shared_security::validate_password_strength(&req.new_password) {
        return HttpResponse::BadRequest().json(json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Find reset token and verify it's valid
    let reset_info = match find_valid_reset_token(&req.token, repo.clone()).await {
        Ok(info) => info,
        Err(e) => {
            return match e {
                AppError::NotFoundError(msg) => {
                    HttpResponse::NotFound().json(json!({ "error": "INVALID_TOKEN", "message": msg }))
                },
                AppError::ValidationError(msg) => {
                    HttpResponse::BadRequest().json(json!({ "error": "INVALID_TOKEN", "message": msg }))
                },
                _ => HttpResponse::InternalServerError()
                    .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to validate reset token" })),
            };
        },
    };

    // Hash new password using shared security module
    let new_password_hash = match shared_security::hash_password(&req.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process password" }));
        },
    };

    // Update user password
    match update_user_password(reset_info.user_id, &new_password_hash, repo.clone()).await {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Failed to update password: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "DATABASE_ERROR", "message": "Failed to update password" }));
        },
    }

    // Mark token as used
    match mark_reset_token_used(&req.token, repo).await {
        Ok(_) => {},
        Err(e) => {
            tracing::warn!("Failed to mark reset token as used: {}", e);
        },
    }

    HttpResponse::Ok().json(json!({ "message": "Password reset successfully".to_string() }))
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
            },
        },
        None => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "VALIDATION_ERROR", "message": "Email is required" }));
        },
    };

    // Validate email using shared security module
    if let Err(e) = shared_security::validate_email(email) {
        return HttpResponse::BadRequest().json(json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Check if user exists (but don't reveal if not found for security)
    let user = match find_user_by_email(email, repo.clone()).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // User doesn't exist, but return success for security
            // This prevents email enumeration attacks
            return HttpResponse::Ok()
                .json(json!({ "message": "If the email is registered, a reset link has been sent".to_string() }));
        },
        Err(e) => {
            tracing::error!("Failed to lookup user: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process request" }));
        },
    };

    // Generate and store password reset token using shared security module
    let token = shared_security::generate_reset_token(64);
    if let Err(e) = store_reset_token(user.id, &token, repo.clone()).await {
        tracing::error!("Failed to store reset token: {}", e);
        return HttpResponse::InternalServerError()
            .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process request" }));
    }

    // TODO: Send email with reset link
    // For now, just log the token (in production, this should send an email)
    tracing::info!("Password reset token generated for {}", email);
    tracing::debug!("Password reset token: {}", token);

    HttpResponse::Ok().json(json!({ "message": "If the email is registered, a reset link has been sent".to_string() }))
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
            },
        },
        None => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "VALIDATION_ERROR", "message": "Email is required" }));
        },
    };

    // Validate email using shared security module
    if let Err(e) = shared_security::validate_email(email) {
        return HttpResponse::BadRequest().json(json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Check if user exists (but don't reveal if not found for security)
    // This prevents email enumeration attacks, matching behavior of request_password_reset
    let user = match find_user_by_email(email, repo.clone()).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // User doesn't exist, but return success for security
            // This prevents email enumeration attacks
            return HttpResponse::Ok().json(
                json!({ "message": "If the email is registered, a verification email has been sent".to_string() }),
            );
        },
        Err(e) => {
            tracing::error!("Failed to lookup user: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process request" }));
        },
    };

    if user.is_email_verified {
        // Don't reveal verification status to prevent enumeration attacks
        return HttpResponse::Ok()
            .json(json!({ "message": "If email is registered, a verification email has been sent".to_string() }));
    }

    // Generate and store new verification token using shared security module
    let token = shared_security::generate_reset_token(64);
    if let Err(e) = store_verification_token(user.id, &token, repo).await {
        tracing::error!("Failed to store verification token: {}", e);
        return HttpResponse::InternalServerError()
            .json(json!({ "error": "INTERNAL_ERROR", "message": "Failed to process request" }));
    }

    // TODO: Send email with verification link
    tracing::info!("Verification token resent for {}", email);
    tracing::debug!("Verification token: {}", token);

    HttpResponse::Ok().json(json!({ "message": "Verification email sent successfully".to_string() }))
}

// ============================================================================
// Helper Functions
// ============================================================================

struct ResetTokenInfo {
    user_id: Uuid,
    #[allow(dead_code)]
    expires_at: chrono::DateTime<chrono::Utc>,
}

fn validate_token_format(token: &str) -> Result<(), String> {
    if token.len() != 64 {
        return Err("Token must be 64 characters long".to_string());
    }
    if !token.chars().all(|c| c.is_alphanumeric()) {
        return Err("Token must contain only alphanumeric characters".to_string());
    }
    Ok(())
}

#[allow(dead_code)]
fn generate_verification_token() -> String {
    shared_security::generate_reset_token(64)
}

// ============================================================================
// Database Operations (TODO: These should be in AuthRepository)
// ============================================================================

async fn find_valid_reset_token(_token: &str, _repo: web::Data<AuthRepository>) -> Result<ResetTokenInfo, AppError> {
    // TODO: Implement actual database query to verify reset token exists, is unused, and hasn't expired
    Err(AppError::NotFoundError(
        "Password reset functionality not yet implemented".to_string(),
    ))
}

async fn mark_reset_token_used(_token: &str, _repo: web::Data<AuthRepository>) -> Result<(), AppError> {
    // TODO: Implement actual database update to mark reset token as used
    Err(AppError::NotFoundError(
        "Password reset functionality not yet implemented".to_string(),
    ))
}

async fn find_user_by_email(email: &str, repo: web::Data<AuthRepository>) -> Result<Option<User>, AppError> {
    // TODO: This should use AuthRepository.find_by_email
    repo.find_by_email(email).await.map_err(AppError::DatabaseError)
}

async fn update_user_password(
    _user_id: Uuid,
    _password_hash: &str,
    _repo: web::Data<AuthRepository>,
) -> Result<(), AppError> {
    // TODO: Implement actual password update in users table
    Err(AppError::NotFoundError(
        "Password reset functionality not yet implemented".to_string(),
    ))
}

async fn store_reset_token(_user_id: Uuid, _token: &str, _repo: web::Data<AuthRepository>) -> Result<(), AppError> {
    // TODO: Implement actual token storage in password_resets table
    Err(AppError::NotFoundError(
        "Password reset functionality not yet implemented".to_string(),
    ))
}

async fn store_verification_token(
    _user_id: Uuid,
    _token: &str,
    _repo: web::Data<AuthRepository>,
) -> Result<(), AppError> {
    // TODO: Implement actual token storage in email_verifications table
    Err(AppError::NotFoundError(
        "Password reset functionality not yet implemented".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Token Validation Tests
    // ========================================

    #[test]
    fn test_validate_token_format_valid() {
        let valid_token = shared_security::generate_reset_token(64);
        assert!(validate_token_format(&valid_token).is_ok());
    }

    #[test]
    fn test_validate_token_format_too_short() {
        let short_token = "test_short";
        assert!(validate_token_format(&short_token).is_err());
    }

    #[test]
    fn test_validate_token_format_too_long() {
        let long_token = format!("test_{}", "a".repeat(100));
        assert!(validate_token_format(&long_token).is_err());
    }

    #[test]
    fn test_validate_token_format_invalid_chars() {
        // Exactly 64 chars with underscores (invalid character)
        let invalid_token = "abcd_fgh_jklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ01";
        assert!(validate_token_format(&invalid_token).is_err());
    }

    #[test]
    fn test_validate_token_format_special_chars() {
        // Exactly 64 chars with special characters (invalid)
        let invalid_token = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOP!@#$%^&*()XY";
        assert!(validate_token_format(&invalid_token).is_err());
    }

    // ========================================
    // Token Generation Tests
    // ========================================

    #[test]
    fn test_generate_reset_token_length() {
        let token = shared_security::generate_reset_token(64);
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_generate_reset_token_unique() {
        let token1 = shared_security::generate_reset_token(64);
        let token2 = shared_security::generate_reset_token(64);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_reset_token_alphanumeric_only() {
        let token = shared_security::generate_reset_token(64);
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_verification_token_format() {
        let verification_token = generate_verification_token();
        assert_eq!(verification_token.len(), 64);
        assert!(verification_token.chars().all(|c| c.is_alphanumeric()));
    }

    // ========================================
    // Email Validation Integration Tests
    // ========================================

    #[test]
    fn test_email_validation_valid_emails() {
        let valid_emails = vec![
            "user@example.com",
            "user.name@example.com",
            "user+tag@example.com",
            "user_name@example.com",
            "user-name@example.co.uk",
        ];
        for email in valid_emails {
            assert!(
                shared_security::validate_email(email).is_ok(),
                "Email should be valid: {}",
                email
            );
        }
    }

    #[test]
    fn test_email_validation_invalid_emails() {
        let invalid_emails = vec![
            "",
            "invalid",
            "@example.com",
            "user@",
            "user@.com",
            "user..name@example.com",
            ".user@example.com",
            "user.@example.com",
        ];
        for email in invalid_emails {
            assert!(
                shared_security::validate_email(email).is_err(),
                "Email should be invalid: {}",
                email
            );
        }
    }

    // ========================================
    // Password Validation Integration Tests
    // ========================================

    #[test]
    fn test_password_validation_valid_passwords() {
        let valid_passwords = vec!["TestPass123", "MyPassword1", "SecurePass2024", "AnotherValid123"];
        for password in valid_passwords {
            assert!(
                shared_security::validate_password_strength(password).is_ok(),
                "Password should be valid: {}",
                password
            );
        }
    }

    #[test]
    fn test_password_validation_invalid_passwords() {
        let invalid_passwords = vec!["short", "alllowercase", "ALLUPPERCASE", "NoDigits", "NoDigitsButLong"];
        for password in invalid_passwords {
            assert!(
                shared_security::validate_password_strength(password).is_err(),
                "Password should be invalid: {}",
                password
            );
        }
    }
}
