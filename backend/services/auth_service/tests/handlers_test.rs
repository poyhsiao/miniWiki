//! Unit tests for auth_service models and JWT service
//!
//! This module contains tests for:
//! - LoginRequest, RegisterRequest, etc.
//! - JwtService token generation and validation
//! - Password hashing and verification
//! - Error response formats

use auth_service::handlers::mask_email;
use auth_service::jwt::{Claims, JwtConfig, JwtService};
use auth_service::models::*;
use auth_service::password::{hash_password, verify_password};

// Test mask_email function
#[test]
fn test_mask_email_standard() {
    let email = "user@example.com";
    let masked = mask_email(email);
    assert!(masked.contains("@example.com"));
    assert!(masked.starts_with("us"));
    assert!(masked.contains("***"));
    assert_eq!(masked, "us***@example.com");
}

#[test]
fn test_mask_email_short_name() {
    let email = "ab@example.com";
    let masked = mask_email(email);
    assert_eq!(masked, "ab***@example.com");
}

#[test]
fn test_mask_email_no_at_symbol() {
    let email = "invalidemail";
    let masked = mask_email(email);
    assert_eq!(masked, "***@***.***");
}

// Test LoginRequest validation
#[test]
fn test_login_request_valid() {
    let req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "Password123!".to_string(),
    };
    assert_eq!(req.email, "test@example.com");
    assert_eq!(req.password, "Password123!");
}

#[test]
fn test_login_request_empty() {
    let req = LoginRequest {
        email: "".to_string(),
        password: "".to_string(),
    };
    assert!(req.email.is_empty());
    assert!(req.password.is_empty());
}

// Test LoginResponse
#[test]
fn test_login_response_structure() {
    let response = LoginResponse {
        user: UserResponse {
            id: "user-123".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: None,
            is_email_verified: true,
        },
        access_token: "access-token".to_string(),
        refresh_token: "refresh-token".to_string(),
        expires_in: 3600,
    };

    assert_eq!(response.user.id, "user-123");
    assert_eq!(response.access_token, "access-token");
    assert_eq!(response.expires_in, 3600);
}

// Test RegisterRequest
#[test]
fn test_register_request_valid() {
    let req = RegisterRequest {
        email: "newuser@example.com".to_string(),
        password: "Password123!".to_string(),
        display_name: "New User".to_string(),
    };
    assert_eq!(req.email, "newuser@example.com");
    assert_eq!(req.display_name, "New User");
}

// Test LogoutRequest
#[test]
fn test_logout_request_with_token() {
    let req = LogoutRequest {
        refresh_token: Some("token123".to_string()),
    };
    assert!(req.refresh_token.is_some());
    assert_eq!(req.refresh_token.unwrap(), "token123");
}

#[test]
fn test_logout_request_without_token() {
    let req = LogoutRequest { refresh_token: None };
    assert!(req.refresh_token.is_none());
}

// Test RefreshRequest
#[test]
fn test_refresh_request_valid() {
    let req = RefreshRequest {
        refresh_token: "refresh-token-123".to_string(),
    };
    assert_eq!(req.refresh_token, "refresh-token-123");
}

// Test RefreshResponse
#[test]
fn test_refresh_response_structure() {
    let response = RefreshResponse {
        access_token: "new-access-token".to_string(),
        expires_in: 1800,
    };
    assert_eq!(response.access_token, "new-access-token");
    assert_eq!(response.expires_in, 1800);
}

// Test UserResponse
#[test]
fn test_user_response_full() {
    let response = UserResponse {
        id: "id-123".to_string(),
        email: "user@test.com".to_string(),
        display_name: "Test".to_string(),
        avatar_url: Some("https://avatar.url".to_string()),
        is_email_verified: false,
    };
    assert_eq!(response.id, "id-123");
    assert!(response.avatar_url.is_some());
}

#[test]
fn test_user_response_no_avatar() {
    let response = UserResponse {
        id: "id-123".to_string(),
        email: "user@test.com".to_string(),
        display_name: "Test".to_string(),
        avatar_url: None,
        is_email_verified: true,
    };
    assert!(response.avatar_url.is_none());
    assert!(response.is_email_verified);
}

// Test RegisterResponse
#[test]
fn test_register_response_structure() {
    let response = RegisterResponse {
        user: UserResponse {
            id: "id-123".to_string(),
            email: "user@test.com".to_string(),
            display_name: "Test".to_string(),
            avatar_url: None,
            is_email_verified: false,
        },
        message: "Registration successful".to_string(),
    };
    assert_eq!(response.message, "Registration successful");
    assert_eq!(response.user.id, "id-123");
}

// Test JwtService configuration
#[test]
fn test_jwt_config_creation() {
    let config = JwtConfig::new("secret".to_string(), 3600, 86400);
    assert_eq!(config.secret, "secret");
    assert_eq!(config.access_expiry, 3600);
    assert_eq!(config.refresh_expiry, 86400);
}

#[test]
fn test_jwt_service_creation() {
    let config = JwtConfig::new("secret".to_string(), 3600, 86400);
    let service = JwtService::new(config);
    assert_eq!(service.config.access_expiry, 3600);
}

// Test JwtService token generation and validation
#[tokio::test]
async fn test_jwt_generate_access_token() {
    let config = JwtConfig::new("test-secret".to_string(), 3600, 86400);
    let service = JwtService::new(config);

    let token = service.generate_access_token("user123", "test@example.com", "user");
    assert!(token.is_ok());
    let token_str = token.unwrap();
    assert!(!token_str.is_empty());
}

#[tokio::test]
async fn test_jwt_generate_refresh_token() {
    let config = JwtConfig::new("test-secret".to_string(), 3600, 86400);
    let service = JwtService::new(config);

    let token = service.generate_refresh_token("user123");
    assert!(token.is_ok());
    let token_str = token.unwrap();
    assert!(!token_str.is_empty());
}

#[tokio::test]
async fn test_jwt_validate_valid_token() {
    let config = JwtConfig::new("test-secret".to_string(), 3600, 86400);
    let service = JwtService::new(config);

    let token = service.generate_access_token("user123", "test@example.com", "user");
    assert!(token.is_ok());

    let claims = service.validate_token(&token.unwrap());
    assert!(claims.is_ok());
    let claims_data = claims.unwrap();
    assert_eq!(claims_data.user_id, "user123");
    assert_eq!(claims_data.email, "test@example.com");
}

#[tokio::test]
async fn test_jwt_validate_invalid_token() {
    let config = JwtConfig::new("test-secret".to_string(), 3600, 86400);
    let service = JwtService::new(config);

    let result = service.validate_token("invalid-token");
    assert!(result.is_err());
}

// Test token extraction from header
#[test]
fn test_extract_token_valid_bearer() {
    let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test";
    let token = JwtService::extract_token_from_header(header);
    assert!(token.is_some());
    assert!(token.unwrap().starts_with("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
}

#[test]
fn test_extract_token_no_bearer() {
    let header = "Basic dXNlcjpwYXNz";
    let token = JwtService::extract_token_from_header(header);
    assert!(token.is_none());
}

#[test]
fn test_extract_token_only_bearer() {
    let header = "Bearer ";
    let token = JwtService::extract_token_from_header(header);
    assert!(token.is_some());
    assert!(token.unwrap().is_empty());
}

// Test Claims structure
#[test]
fn test_claims_structure() {
    let claims = Claims {
        sub: "user123".to_string(),
        user_id: "user123".to_string(),
        email: "test@example.com".to_string(),
        role: "user".to_string(),
        exp: 1234567890,
        iat: 1234567890,
        jti: None,
    };

    assert_eq!(claims.sub, "user123");
    assert_eq!(claims.email, "test@example.com");
    assert!(claims.jti.is_none());
}

#[test]
fn test_claims_with_jti() {
    let claims = Claims {
        sub: "user123".to_string(),
        user_id: "user123".to_string(),
        email: "test@example.com".to_string(),
        role: "user".to_string(),
        exp: 1234567890,
        iat: 1234567890,
        jti: Some("unique-id".to_string()),
    };

    assert!(claims.jti.is_some());
    assert_eq!(claims.jti.unwrap(), "unique-id");
}

// Test password functions
#[test]
fn test_password_hash_different_for_same_password() {
    let password = "TestPassword123!";
    let hash1 = hash_password(password).expect("Should hash successfully");
    let hash2 = hash_password(password).expect("Should hash successfully");

    // Different hashes due to random salt
    assert_ne!(hash1, hash2);

    // But both should verify
    assert!(verify_password(password, &hash1).unwrap());
    assert!(verify_password(password, &hash2).unwrap());
}

#[test]
fn test_password_hash_minimum_length() {
    let result = hash_password("Ab1!");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_password_verify_empty() {
    let result = verify_password("Test123!", "");
    assert!(result.is_err());
}

// Test error response format
#[test]
fn test_error_response_format() {
    let error_response = serde_json::json!({
        "error": "VALIDATION_ERROR",
        "message": "Invalid email format"
    });

    assert_eq!(error_response["error"], "VALIDATION_ERROR");
    assert_eq!(error_response["message"], "Invalid email format");
}

#[test]
fn test_auth_error_response() {
    let error_response = serde_json::json!({
        "error": "AUTHENTICATION_ERROR",
        "message": "Invalid credentials"
    });

    assert_eq!(error_response["error"], "AUTHENTICATION_ERROR");
    assert!(error_response["message"].is_string());
}

#[test]
fn test_database_error_response() {
    let error_response = serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Connection failed"
    });

    assert_eq!(error_response["error"], "DATABASE_ERROR");
    assert_eq!(error_response["message"], "Connection failed");
}

// Test conflict error response
#[test]
fn test_conflict_error_response() {
    let error_response = serde_json::json!({
        "error": "CONFLICT",
        "message": "Email already registered"
    });

    assert_eq!(error_response["error"], "CONFLICT");
    assert!(error_response["message"].as_str().unwrap().contains("already"));
}

// Test unauthorized error
#[test]
fn test_unauthorized_error_response() {
    let error_response = serde_json::json!({
        "error": "UNAUTHORIZED",
        "message": "Missing or invalid authorization"
    });

    assert_eq!(error_response["error"], "UNAUTHORIZED");
}

// Test generate_jwt_token utility function
#[tokio::test]
async fn test_generate_jwt_token_utility() {
    use auth_service::jwt::generate_jwt_token;
    use uuid::Uuid;

    let user_id = Uuid::new_v4();
    let token = generate_jwt_token(user_id, "test@example.com");

    assert!(token.is_ok());
    let token_str = token.unwrap();
    assert!(!token_str.is_empty());
}
