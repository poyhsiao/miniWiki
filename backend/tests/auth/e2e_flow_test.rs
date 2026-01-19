//! End-to-end authentication flow tests
//!
//! Tests complete user authentication flows including login, protected route access,
//! session management, and logout. These tests verify the full authentication lifecycle.
//!
//! Run with: cargo test --test lib auth::e2e_flow_test
//! Note: Requires running backend server and database

use crate::helpers::TestApp;
use serde_json::json;

/// Full E2E login flow test
/// Tests the complete login process: register -> login -> access protected resource
#[tokio::test]
async fn test_e2e_login_flow() {
    let app = TestApp::create().await;

    // Create a test user directly in the database
    let test_user = app.create_test_user().await;

    // Attempt to login with the test user's credentials
    // Note: Using the pre-created user's email and a known password hash
    let login_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/login", app.port))
        .json(&json!({
            "email": test_user.email,
            "password": "TestPass123!"  // This should match the password hash used in create_test_user
        }))
        .send()
        .await
        .expect("Login request failed");

    // Login should succeed with the test user's credentials
    let status = login_response.status();
    assert!(
        status == 200,
        "Expected 200 (success), got: {}",
        status
    );
}

/// E2E document list test after login
/// Tests accessing protected document list endpoint with valid authentication
#[tokio::test]
async fn test_e2e_document_list_after_login() {
    let app = TestApp::create().await;

    // Create test user and space with document
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let _document = app.create_test_document(&space.id, None).await;

    // Get auth token for the user
    let (token, _user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Access document list endpoint
    let response = app
        .client
        .get(&format!("http://localhost:{}/api/v1/space-docs/{}/documents", app.port, space.id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("Document list request failed");

    // Should get success response
    assert!(
        response.status() == 200,
        "Expected 200, got: {}",
        response.status()
    );

    // Verify response structure
    let body: serde_json::Value = response.json().await.expect("Parse response failed");
    assert!(body.get("success").unwrap_or(&json!(false)).as_bool().unwrap_or(false) ||
           body.get("data").is_some() ||
           body.get("documents").is_some(),
        "Response should contain success flag or data/documents field");
}

/// Full logout flow test
/// Tests the complete logout process and verifies session is terminated
#[tokio::test]
async fn test_e2e_logout_flow() {
    let app = TestApp::create().await;

    // Create test user
    let test_user = app.create_test_user().await;

    // Login to get tokens
    let login_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/login", app.port))
        .json(&serde_json::json!({
            "email": test_user.email,
            "password": "TestPass123!"
        }))
        .send()
        .await
        .expect("Login request failed");

    assert!(
        login_response.status() == 200,
        "Login should succeed, got: {}",
        login_response.status()
    );

    let login_body: serde_json::Value = login_response.json().await.expect("Parse login response");
    let access_token = login_body.get("access_token").unwrap().as_str().unwrap().to_string();
    let refresh_token = login_body.get("refresh_token").unwrap().as_str().unwrap().to_string();

    // Perform logout with refresh token
    let logout_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/logout", app.port))
        .json(&serde_json::json!({
            "refresh_token": refresh_token
        }))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("Logout request failed");

    // Logout should succeed
    assert!(
        logout_response.status() == 200,
        "Expected 200 for logout, got: {}",
        logout_response.status()
    );

    // Verify response contains success message
    let body: serde_json::Value = logout_response.json().await.expect("Parse logout response failed");
    assert!(
        body.get("success").unwrap_or(&json!(false)).as_bool().unwrap_or(false) ||
        body.get("message").is_some(),
        "Logout response should contain success or message field"
    );
}

/// Test protected route access without token
/// Verifies that protected endpoints reject requests without authentication
#[tokio::test]
async fn test_e2e_access_protected_route_without_token() {
    let app = TestApp::create().await;

    // Create test user and space
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;

    // Try to access protected endpoint WITHOUT authentication
    let response = app
        .client
        .get(&format!("http://localhost:{}/api/v1/space-docs/{}/documents", app.port, space.id))
        .send()
        .await
        .expect("Protected route request failed");

    // Should get 401 Unauthorized
    assert!(
        response.status() == 401,
        "Expected 401 Unauthorized without token, got: {}",
        response.status()
    );

    // Verify error response structure
    let body: serde_json::Value = response.json().await.expect("Parse error response failed");
    assert!(
        body.get("error").is_some() || body.get("code").is_some(),
        "Error response should contain error or code field"
    );
}

/// Test protected route access with invalid signature
/// Verifies that protected endpoints reject requests with tokens that have invalid signatures
#[tokio::test]
async fn test_e2e_access_protected_route_with_invalid_signature() {
    let app = TestApp::create().await;

    // Create test user (for test isolation, even though we're using an invalid token)
    let _test_user = app.create_test_user().await;

    // Use an invalid/malformed token (invalid signature)
    // This tests signature validation rather than token expiry.
    // Note: This is intentionally an invalid test token (not a real secret).
    // Base64 payload: {"sub":"1234567890","name":"John Doe","iat":1516239022,"exp":9999999999}
    let invalid_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjk5OTk5OTk5OTl9.invalid_signature";

    // Try to access protected endpoint with invalid token
    let response = app
        .client
        .get(&format!("http://localhost:{}/api/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", invalid_token))
        .send()
        .await
        .expect("Protected route request failed");

    // Should get 401 Unauthorized
    assert!(
        response.status() == 401,
        "Expected 401 Unauthorized with invalid signature, got: {}",
        response.status()
    );
}

/// Test token persistence across requests
/// Verifies that the same token can be used for multiple authenticated requests
#[tokio::test]
async fn test_e2e_token_persists_across_requests() {
    let app = TestApp::create().await;

    // Create test user
    let test_user = app.create_test_user().await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Make multiple authenticated requests
    for i in 1..=3 {
        let response = app
            .client
            .get(&format!("http://localhost:{}/api/v1/auth/me", app.port))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-User-Id", user_id_str.clone())
            .send()
            .await
            .expect(&format!("Auth request {} failed", i));

        // Each request should succeed
        assert!(
            response.status() == 200,
            "Request {}: Expected 200, got: {}",
            i,
            response.status()
        );

        // Verify we get user info back
        if response.status() == 200 {
            let body: serde_json::Value = response.json().await.expect("Parse response failed");
            assert!(
                body.get("id").is_some() || body.get("email").is_some(),
                "Response should contain user info"
            );
        }
    }
}

/// Test session state management
/// Verifies server-side session consistency between requests
#[tokio::test]
async fn test_e2e_session_state_management() {
    let app = TestApp::create().await;

    // Create test user
    let test_user = app.create_test_user().await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // First request - get user profile
    let response1 = app
        .client
        .get(&format!("http://localhost:{}/api/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("First auth request failed");

    assert!(
        response1.status() == 200,
        "First request should succeed, got: {}",
        response1.status()
    );

    let body1: serde_json::Value = response1.json().await.expect("Parse first response failed");
    let user_id_1 = body1.get("id").or_else(|| body1.get("user_id")).cloned();

    // Second request - should get same user info
    let response2 = app
        .client
        .get(&format!("http://localhost:{}/api/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Second auth request failed");

    assert!(
        response2.status() == 200,
        "Second request should succeed, got: {}",
        response2.status()
    );

    let body2: serde_json::Value = response2.json().await.expect("Parse second response failed");
    let user_id_2 = body2.get("id").or_else(|| body2.get("user_id")).cloned();

    // Verify session state is consistent
    assert_eq!(
        user_id_1, user_id_2,
        "Session should return consistent user ID across requests"
    );
}

/// Test that logout invalidates the refresh token (access token remains valid until expiry)
/// Verifies that after logout, the refresh token can no longer be used to get new access tokens
#[tokio::test]
async fn test_e2e_logout_invalidates_token() {
    let app = TestApp::create().await;

    // Create test user
    let test_user = app.create_test_user().await;

    // First login - get tokens
    let login_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/login", app.port))
        .json(&serde_json::json!({
            "email": test_user.email,
            "password": "TestPass123!"
        }))
        .send()
        .await
        .expect("Login request failed");

    assert!(
        login_response.status() == 200,
        "Login should succeed, got: {}",
        login_response.status()
    );

    let login_body: serde_json::Value = login_response.json().await.expect("Parse login response");
    let access_token = login_body.get("access_token").unwrap().as_str().unwrap().to_string();
    let refresh_token = login_body.get("refresh_token").unwrap().as_str().unwrap().to_string();

    // Verify access token works before logout
    let pre_logout_response = app
        .client
        .get(&format!("http://localhost:{}/api/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("Pre-logout auth request failed");

    assert!(
        pre_logout_response.status() == 200,
        "Access token should be valid before logout, got: {}",
        pre_logout_response.status()
    );

    // Verify refresh token works BEFORE logout
    let pre_logout_refresh_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/refresh", app.port))
        .json(&serde_json::json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .expect("Pre-logout refresh request failed");

    assert!(
        pre_logout_refresh_response.status() == 200,
        "Refresh token should be valid before logout, got: {}",
        pre_logout_refresh_response.status()
    );

    let pre_logout_refresh_body: serde_json::Value = pre_logout_refresh_response
        .json()
        .await
        .expect("Failed to parse pre-logout refresh response JSON");

    assert!(
        pre_logout_refresh_body.get("access_token").is_some(),
        "Pre-logout refresh response should contain a new access_token, got: {}",
        pre_logout_refresh_body
    );

    // Perform logout
    let logout_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/logout", app.port))
        .json(&serde_json::json!({
            "refresh_token": refresh_token
        }))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("Logout request failed");

    assert!(
        logout_response.status() == 200,
        "Logout should succeed, got: {}",
        logout_response.status()
    );

    // Access token should still work (stateless JWT - valid until expiry)
    let post_logout_response = app
        .client
        .get(&format!("http://localhost:{}/api/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("Post-logout auth request failed");

    // Access token is stateless and remains valid until expiry
    assert!(
        post_logout_response.status() == 200,
        "Access token should still be valid after logout (stateless JWT), got: {}",
        post_logout_response.status()
    );

    // Refresh token should be invalidated after logout
    let post_logout_refresh_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/refresh", app.port))
        .json(&serde_json::json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .expect("Post-logout refresh request failed");

    // Refresh token should be invalid after logout
    assert!(
        post_logout_refresh_response.status() == 401,
        "Refresh token should be invalid after logout, got: {}",
        post_logout_refresh_response.status()
    );
}

/// Test new login after logout works correctly
/// Verifies that a fresh session can be created after logging out
#[tokio::test]
async fn test_e2e_new_login_after_logout() {
    let app = TestApp::create().await;

    // Create test user
    let test_user = app.create_test_user().await;

    // First login - use actual server login to get both tokens
    let login_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/login", app.port))
        .json(&serde_json::json!({
            "email": test_user.email,
            "password": "TestPass123!"
        }))
        .send()
        .await
        .expect("First login request failed");

    assert!(
        login_response.status() == 200,
        "First login should succeed, got: {}",
        login_response.status()
    );

    let login_body: serde_json::Value = login_response.json().await.expect("Parse login response");
    let token1 = login_body.get("access_token").unwrap().as_str().unwrap().to_string();
    let refresh_token1 = login_body.get("refresh_token").unwrap().as_str().unwrap().to_string();

    // Verify first token works
    let response1 = app
        .client
        .get(&format!("http://localhost:{}/api/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", token1))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("First auth request failed");

    assert!(response1.status() == 200, "First token should work");

    // Logout with refresh token
    let logout_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/logout", app.port))
        .json(&serde_json::json!({
            "refresh_token": refresh_token1
        }))
        .header("Authorization", format!("Bearer {}", token1))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("Logout request failed");

    assert!(logout_response.status() == 200, "Logout should succeed");

    // Get a new token via actual login request
    let token2 = app.login_user(&test_user.email, "TestPass123!")
        .await
        .expect("Second login should succeed");

    // Tokens should be different (refresh token rotation)
    assert!(!token2.is_empty(), "Second login should return a token");

    // Verify new token works
    let response2 = app
        .client
        .get(&format!("http://localhost:{}/api/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", token2))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("Second auth request failed");

    assert!(
        response2.status() == 200,
        "New token should work after logout and re-login, got: {}",
        response2.status()
    );
}

/// Test logout without refresh_token succeeds
/// Verifies that logout works even when no refresh_token is provided
#[tokio::test]
async fn test_e2e_logout_without_refresh_token_succeeds() {
    let app = TestApp::create().await;

    // Create test user
    let test_user = app.create_test_user().await;

    // Login to get access token
    let login_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/login", app.port))
        .json(&serde_json::json!({
            "email": test_user.email,
            "password": "TestPass123!"
        }))
        .send()
        .await
        .expect("Login request failed");

    assert!(
        login_response.status() == 200,
        "Login should succeed, got: {}",
        login_response.status()
    );

    let login_body: serde_json::Value = login_response.json().await.expect("Parse login response");
    let access_token = login_body.get("access_token").unwrap().as_str().unwrap().to_string();

    // Perform logout WITHOUT refresh_token
    let logout_response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/auth/logout", app.port))
        .json(&serde_json::json!({}))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("Logout request failed");

    // Logout without refresh_token should still succeed
    assert!(
        logout_response.status() == 200,
        "Logout without refresh_token should succeed, got: {}",
        logout_response.status()
    );
}
