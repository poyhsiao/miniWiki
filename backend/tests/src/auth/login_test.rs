// T043: Login endpoint tests
// Following TDD principles, these tests should FAIL initially
// Run with: cargo test -p miniwiki-backend-tests --test auth::login_test

use actix_web::{test, App, http::StatusCode};
use miniwiki_backend::auth_service::handlers::login;
use miniwiki_backend::auth_service::repository::AuthRepository;
use sqlx::PgPool;

#[actix_web::test]
async fn test_login_success() {
    // GIVEN: Valid credentials
    // WHEN: User logs in with correct email and password
    // THEN: Should return 200 OK with tokens and user data

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let login_data = serde_json::json!({
        "email": "user@example.com",
        "password": "SecurePass123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_data);

    let resp = login(req, &pool).await;

    // RED phase: This should fail until implementation is complete
    assert_eq!(resp.status(), StatusCode::OK, "Should return 200 OK");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert!(body.get("accessToken").is_some(), "Should return access token");
    assert!(body.get("refreshToken").is_some(), "Should return refresh token");
    assert!(body.get("expiresIn").is_some(), "Should return expiry time");

    let user_data = body.get("user").unwrap();
    assert!(user_data.get("id").is_some(), "Should return user ID");
    assert!(user_data.get("email").is_some(), "Should return user email");
    assert!(user_data.get("displayName").is_some(), "Should return user display name");
}

#[actix_web::test]
async fn test_login_invalid_credentials() {
    // GIVEN: Invalid email or password
    // WHEN: User logs in with incorrect credentials
    // THEN: Should return 401 Unauthorized with error message

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let login_data = serde_json::json!({
        "email": "user@example.com",
        "password": "WrongPassword123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_data);

    let resp = login(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "Should return 401 Unauthorized");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "AUTHENTICATION_ERROR", "Should return correct error code");
    assert!(body.get("message").is_some(), "Should return error message");
}

#[actix_web::test]
async fn test_login_nonexistent_user() {
    // GIVEN: Email doesn't exist in database
    // WHEN: Try to login with unregistered email
    // THEN: Should return 401 Unauthorized (same as invalid credentials for security)

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let login_data = serde_json::json!({
        "email": "nonexistent@example.com",
        "password": "AnyPassword123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_data);

    let resp = login(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "Should return 401 Unauthorized");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "AUTHENTICATION_ERROR", "Should return correct error code");
}

#[actix_web::test]
async fn test_login_email_not_verified() {
    // GIVEN: User exists but email not verified
    // WHEN: Try to login with unverified email
    // THEN: Should return 401 Unauthorized with verification error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let login_data = serde_json::json!({
        "email": "unverified@example.com",
        "password": "SecurePass123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_data);

    let resp = login(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "Should return 401 Unauthorized");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "AUTHENTICATION_ERROR", "Should return correct error code");
    let message = body.get("message").and_then(|v| v.as_str()).unwrap_or_default();
    assert!(message.contains("verify") || message.contains("email"), "Should mention email verification");
}

#[actix_web::test]
async fn test_login_missing_email() {
    // GIVEN: Missing email field
    // WHEN: Try to login without email
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let login_data = serde_json::json!({
        // email is missing
        "password": "SecurePass123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_data);

    let resp = login(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
}

#[actix_web::test]
async fn test_login_missing_password() {
    // GIVEN: Missing password field
    // WHEN: Try to login without password
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let login_data = serde_json::json!({
        "email": "user@example.com"
        // password is missing
    });

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_data);

    let resp = login(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
}

#[actix_web::test]
async fn test_login_invalid_email_format() {
    // GIVEN: Invalid email format
    // WHEN: Try to login with malformed email
    // THEN: Should return 400 Bad Request with validation error

    let app = App::new().await;
    let pool = app.app_data::<PgPool>().clone();

    let login_data = serde_json::json!({
        "email": "not-an-email", // Invalid format
        "password": "SecurePass123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_data);

    let resp = login(req, &pool).await;

    // RED phase: This should fail initially
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should return 400 Bad Request");

    let body: serde_json::Value = serde_json::from_slice(&test::response_body(&resp)).unwrap();
    assert_eq!(body["error"], "VALIDATION_ERROR", "Should return validation error");
}

// Helper function to extract response body
fn response_body(resp: &actix_web::dev::ServiceResponse) -> Vec<u8> {
    match resp.response() {
        actix_web::body::Body::Bytes(bytes) => bytes.to_vec(),
        _ => vec![],
    }
}
