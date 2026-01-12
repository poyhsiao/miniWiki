// T044: Password reset endpoint tests
// Following TDD principles, these tests should FAIL initially
// Run with: cargo test -p miniwiki-backend-tests --test auth::password_reset_test

use actix_web::{test, App, http::StatusCode};
use miniwiki_backend::auth_service::handlers::logout;
use sqlx::PgPool;

#[actix_web::test]
async fn test_request_password_reset_success() {
    // GIVEN: Valid email address
    // WHEN: User requests password reset
    // THEN: Should return 200 OK (or always return 200 to prevent email enumeration)

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let request_data = serde_json::json!({
        "email": "user@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/auth/password/reset-request")
        .set_json(&request_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::OK, "Should return 200 OK");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    // Note: For security, we always return 200 even if email doesn't exist
    assert!(body.get("message").is_some(), "Should return success message");
}

#[actix_web::test]
async fn test_reset_password_with_valid_token() {
    // GIVEN: Valid password reset token
    // WHEN: User resets password with valid token
    // THEN: Should return 200 OK with success message

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let reset_data = serde_json::json!({
        "token": "valid_reset_token_1234567890123456789",
        "newPassword": "NewSecurePass123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/password/reset")
        .set_json(&reset_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::OK, "Should return 200 OK");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert!(body.get("message").is_some(), "Should return success message");
}

#[actix_web::test]
async fn test_reset_password_with_invalid_token() {
    // GIVEN: Invalid or expired reset token
    // WHEN: User tries to reset password with invalid token
    // THEN: Should return 400 Bad Request with error message

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let reset_data = serde_json::json!({
        "token": "invalid_token_123",
        "newPassword": "NewSecurePass123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/password/reset")
        .set_json(&reset_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "AUTH_INVALID_TOKEN", "Should return correct error code");
}

#[actix_web::test]
async fn test_reset_password_with_weak_password() {
    // GIVEN: Valid token but weak new password
    // WHEN: User tries to reset with weak password
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let reset_data = serde_json::json!({
        "token": "valid_reset_token_1234567890123456789",
        "newPassword": "weak" // Doesn't meet requirements
    });

    let req = test::TestRequest::post()
        .uri("/auth/password/reset")
        .set_json(&reset_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
    assert!(body.get("message").and_then(|v| v.as_str()).unwrap_or_default().contains("password"), "Should mention password");
}

#[actix_web::test]
async fn test_reset_password_with_short_password() {
    // GIVEN: Valid token but password too short (< 8 chars)
    // WHEN: User tries to reset with short password
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let reset_data = serde_json::json!({
        "token": "valid_reset_token_1234567890123456789",
        "newPassword": "Short1"
    });

    let req = test::TestRequest::post()
        .uri("/auth/password/reset")
        .set_json(&reset_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
    assert!(body.get("message").and_then(|v| v.as_str()).unwrap_or_default().contains("password"), "Should mention password length");
}

#[actix_web::test]
async fn test_reset_password_with_invalid_token_format() {
    // GIVEN: Token with incorrect format (not 64 chars)
    // WHEN: User tries to reset with malformed token
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let reset_data = serde_json::json!({
        "token": "short", // Not 64 characters
        "newPassword": "NewSecurePass123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/password/reset")
        .set_json(&reset_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
}

#[actix_web::test]
async fn test_reset_password_missing_token() {
    // GIVEN: Missing token field
    // WHEN: User tries to reset without token
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let reset_data = serde_json::json!({
        // token is missing
        "newPassword": "NewSecurePass123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/password/reset")
        .set_json(&reset_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
}

#[actix_web::test]
async fn test_reset_password_missing_new_password() {
    // GIVEN: Missing newPassword field
    // WHEN: User tries to reset without new password
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let reset_data = serde_json::json!({
        "token": "valid_reset_token_1234567890123456789"
        // newPassword is missing
    });

    let req = test::TestRequest::post()
        .uri("/auth/password/reset")
        .set_json(&reset_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
}

#[actix_web::test]
async fn test_resend_verification_email_success() {
    // GIVEN: Valid email address
    // WHEN: User requests to resend verification email
    // THEN: Should return 200 OK

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let request_data = serde_json::json!({
        "email": "user@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/auth/verify-email/resend")
        .set_json(&request_data);

    let resp = logout(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::OK, "Should return 200 OK");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert!(body.get("message").is_some(), "Should return success message");
}

// Helper function to extract response body
fn response_body(resp: &actix_web::dev::ServiceResponse) -> Vec<u8> {
    match resp.response() {
        actix_web::body::Body::Bytes(bytes) => bytes.to_vec(),
        _ => vec![],
    }
}
