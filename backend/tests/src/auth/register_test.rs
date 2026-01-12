// T042: Registration endpoint tests
// Following TDD principles, these tests should FAIL initially
// Run with: cargo test -p miniwiki-backend-tests --test auth::register_test

use actix_web::{test, App, http::StatusCode};
use miniwiki_backend::auth_service::handlers::register;
use miniwiki_backend::auth_service::repository::AuthRepository;
use sqlx::PgPool;

#[actix_web::test]
async fn test_register_success() {
    // GIVEN: Valid registration data
    // WHEN: User registers with valid email, password, and display name
    // THEN: Should return 201 with user data and verification message

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let register_data = serde_json::json!({
        "email": "newuser@example.com",
        "password": "SecurePass123",
        "display_name": "New User"
    });

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&register_data);

    let resp = register(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::CREATED, "Should return 201 Created");

    let body: serde_json::Value = serde_json::from_slice(&response_body(&resp)).unwrap();
    assert!(body.get("userId").is_some(), "Should return userId");
    assert!(body.get("message").is_some(), "Should return verification message");
}

#[actix_web::test]
async fn test_register_email_already_exists() {
    // GIVEN: Email already registered
    // WHEN: Try to register with existing email
    // THEN: Should return 409 Conflict with error message

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let register_data = serde_json::json!({
        "email": "existing@example.com", // Assume this email exists
        "password": "SecurePass123",
        "display_name": "Existing User"
    });

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&register_data);

    let resp = register(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::CONFLICT, "Should return 409 Conflict");

    let body: serde_json::Value = serde_json::from_slice(&response_body(&resp)).unwrap();
    assert_eq!(body["error"], "AUTH_EMAIL_EXISTS", "Should return correct error code");
}

#[actix_web::test]
async fn test_register_weak_password() {
    // GIVEN: Weak password (doesn't meet requirements)
    // WHEN: Try to register with weak password
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let register_data = serde_json::json!({
        "email": "weak@example.com",
        "password": "weak", // Too short, no uppercase, no number
        "display_name": "Weak Password User"
    });

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&register_data);

    let resp = register(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
    assert!(body["message"].as_str().unwrap().contains("password"), "Should mention password requirements");
}

#[actix_web::test]
async fn test_register_invalid_email_format() {
    // GIVEN: Invalid email format
    // WHEN: Try to register with invalid email
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let register_data = serde_json::json!({
        "email": "invalid-email", // Not a valid email format
        "password": "SecurePass123",
        "display_name": "Invalid Email User"
    });

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&register_data);

    let resp = register(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
    assert!(body["message"].as_str().unwrap().contains("email"), "Should mention email format");
}

#[actix_web::test]
async fn test_register_missing_display_name() {
    // GIVEN: Missing display_name field
    // WHEN: Try to register without display name
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let register_data = serde_json::json!({
        "email": "missingname@example.com",
        "password": "SecurePass123"
        // display_name is missing
    });

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&register_data);

    let resp = register(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
}

#[actix_web::test]
async fn test_register_password_too_short() {
    // GIVEN: Password less than minimum length (8 chars)
    // WHEN: Try to register with short password
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let register_data = serde_json::json!({
        "email": "shortpwd@example.com",
        "password": "Short1", // Only 6 chars
        "display_name": "Short Password User"
    });

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&register_data);

    let resp = register(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
    assert!(body["message"].as_str().unwrap().contains("password"), "Should mention password length");
}

// Helper function to extract response body
fn response_body(resp: &actix_web::dev::ServiceResponse) -> Vec<u8> {
    match resp.response() {
        actix_web::body::Body::Bytes(bytes) => bytes.to_vec(),
        _ => vec![],
    }
}
