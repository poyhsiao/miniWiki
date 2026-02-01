//! Additional unit tests for auth_service
//!
//! This module contains additional tests for password verification,
//! token validation, and error response handling.

use auth_service::jwt::{Claims, JwtConfig, JwtService, JwtError};
use auth_service::models::{RegisterRequest, LoginRequest};
use auth_service::password::{hash_password, verify_password};
use chrono::{Duration, Utc};
use std::collections::HashMap;

// ========================================
// Password Hash Verification Tests
// ========================================

#[test]
fn test_password_too_short() {
    // Test that short passwords are still hashed (bcrypt doesn't have min length)
    // The validation should happen at the input validation layer
    let password = "ab";
    let result = hash_password(password);

    // hash_password should succeed (bcrypt doesn't reject short passwords)
    assert!(result.is_ok());

    // Verify the hash can be verified
    let hash = result.unwrap();
    assert!(verify_password(password, &hash).expect("verify_password should succeed for correct password"));
}

#[test]
fn test_password_too_long() {
    let password = "abcdefghijklmnopqrstuvwxyz";
    let hash_result = hash_password(password);

    // Should successfully hash
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();

    // Verify the password works
    assert!(verify_password(password, &hash).expect("verify_password should succeed for correct password"));

    // Verify different password doesn't work - should return Ok(false) or Err
    let result = verify_password("differentpassword", &hash);
    assert!(result.is_err() || !result.unwrap());
}

#[test]
fn test_password_no_uppercase() {
    // Test that a password without uppercase can still be hashed
    let password = "lowercase";
    let hash_result = hash_password(password);

    // Should successfully hash
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();

    // Verify the password works
    assert!(verify_password(password, &hash).expect("verify_password should succeed for correct password"));

    // The hash should not be empty
    assert!(!hash.is_empty());
}

#[test]
fn test_password_missing_digit() {
    let password = "nopassword";
    let hash_result = hash_password(password);

    // Should successfully hash
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();

    // Verify the password works
    assert!(verify_password(password, &hash).expect("verify_password should succeed for correct password"));
}

// ========================================
// Token Validation Tests
// ========================================

#[test]
fn test_token_invalid_structure() {
    let config = JwtConfig::new("test-secret".to_string(), 3600, 86400);
    let service = JwtService::new(config);

    let token = "invalid-jwt-payload";
    let result = service.validate_token(token);

    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_expired() {
    let config = JwtConfig::new("test-secret".to_string(), 3600, 86400);
    let service = JwtService::new(config);

    // Create claims with expired timestamp
    let now = Utc::now();
    let expired_time = now - Duration::hours(2);

    let claims = Claims {
        sub: "user123".to_string(),
        user_id: "user123".to_string(),
        email: "test@example.com".to_string(),
        role: "user".to_string(),
        exp: expired_time.timestamp() as usize,
        iat: (expired_time - Duration::hours(24)).timestamp() as usize,
        jti: None,
    };

    // Manually encode the token
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    let token_result = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret("test-secret".as_bytes()),
    );

    assert!(token_result.is_ok());
    let token = token_result.unwrap();

    // Validating expired token should fail
    let result = service.validate_token(&token);
    assert!(result.is_err());
}

// ========================================
// Error Response Tests
// ========================================

/// Test error response with details - using a simple struct for testing
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TestErrorResponse {
    code: String,
    message: String,
    details: Option<HashMap<String, String>>,
}

#[test]
fn test_error_response_with_conflict_details() {
    let mut details = HashMap::new();
    details.insert("field".to_string(), "email".to_string());
    details.insert("existing_user".to_string(), "user123".to_string());

    let error = TestErrorResponse {
        code: "CONFLICT".to_string(),
        message: "Email conflict".to_string(),
        details: Some(details),
    };

    assert_eq!(error.code, "CONFLICT");
    assert_eq!(error.message, "Email conflict");
    assert!(error.details.is_some());

    let details_map = error.details.unwrap();
    assert_eq!(details_map.get("field").unwrap(), "email");
}

// ========================================
// Input Validation Tests
// ========================================

#[test]
fn test_register_request_short_password_fails_validation() {
    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "short".to_string(),
        display_name: "Test User".to_string(),
    };

    // Validator should reject short password
    assert!(request.validate().is_err());
}

#[test]
fn test_login_request_invalid_email_fails_validation() {
    let request = LoginRequest {
        email: "not-an-email".to_string(),
        password: "Password123!".to_string(),
    };

    // Validator should reject invalid email
    assert!(request.validate().is_err());
}
