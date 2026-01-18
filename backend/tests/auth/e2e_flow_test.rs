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
        .post(&format!("http://localhost:{}/v1/auth/login", app.port))
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
        .get(&format!("http://localhost:{}/v1/space-docs/{}/documents", app.port, space.id))
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

    // Get auth token
    let (token, _user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Perform logout
    let logout_response = app
        .client
        .post(&format!("http://localhost:{}/v1/auth/logout", app.port))
        .header("Authorization", format!("Bearer {}", token))
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
        .get(&format!("http://localhost:{}/v1/space-docs/{}/documents", app.port, space.id))
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
    // This tests signature validation rather than token expiry
    let invalid_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE1MTYyMzkwMjJ9.invalid_signature";

    // Try to access protected endpoint with invalid token
    let response = app
        .client
        .get(&format!("http://localhost:{}/v1/auth/me", app.port))
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
            .get(&format!("http://localhost:{}/v1/auth/me", app.port))
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
        .get(&format!("http://localhost:{}/v1/auth/me", app.port))
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
        .get(&format!("http://localhost:{}/v1/auth/me", app.port))
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

/// Test that logout invalidates the token
/// Verifies that after logout, the same token is no longer valid
#[tokio::test]
async fn test_e2e_logout_invalidates_token() {
    let app = TestApp::create().await;

    // Create test user
    let test_user = app.create_test_user().await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Verify token is valid before logout
    let pre_logout_response = app
        .client
        .get(&format!("http://localhost:{}/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Pre-logout auth request failed");

    assert!(
        pre_logout_response.status() == 200,
        "Token should be valid before logout, got: {}",
        pre_logout_response.status()
    );

    // Perform logout
    let logout_response = app
        .client
        .post(&format!("http://localhost:{}/v1/auth/logout", app.port))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Logout request failed");

    assert!(
        logout_response.status() == 200,
        "Logout should succeed, got: {}",
        logout_response.status()
    );

    // Try to use the same token after logout
    let post_logout_response = app
        .client
        .get(&format!("http://localhost:{}/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Post-logout auth request failed");

    // Token should be invalid after logout
    assert!(
        post_logout_response.status() == 401,
        "Token should be invalid after logout, got: {}",
        post_logout_response.status()
    );
}

/// Test new login after logout works correctly
/// Verifies that a fresh session can be created after logging out
#[tokio::test]
async fn test_e2e_new_login_after_logout() {
    let app = TestApp::create().await;

    // Create test user
    let test_user = app.create_test_user().await;

    // First login - use actual server login
    let token1 = app.login_user(&test_user.email, "TestPass123!")
        .await
        .expect("First login should succeed");

    // Verify first token works
    let response1 = app
        .client
        .get(&format!("http://localhost:{}/v1/auth/me", app.port))
        .header("Authorization", format!("Bearer {}", token1))
        .header("X-User-Id", test_user.id.to_string())
        .send()
        .await
        .expect("First auth request failed");

    assert!(response1.status() == 200, "First token should work");

    // Logout
    let logout_response = app
        .client
        .post(&format!("http://localhost:{}/v1/auth/logout", app.port))
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
    assert_ne!(
        token1, token2,
        "New token should be different from old token after logout and re-login"
    );

    // Verify new token works
    let response2 = app
        .client
        .get(&format!("http://localhost:{}/v1/auth/me", app.port))
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
