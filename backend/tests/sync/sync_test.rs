//! Sync endpoint tests
//!
//! Tests for CRDT sync operations including state retrieval and update processing.
//! These tests verify the sync service handlers work correctly for offline-first sync.
//!
//! Run with: cargo test -p miniwiki-backend-tests sync::sync_test

use futures_util::future::join_all;
use uuid::Uuid;
use crate::helpers::TestApp;

/// Test sync state retrieval for a document
#[tokio::test]
async fn test_get_sync_state_success() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    let response = app
        .get(&format!("/api/v1/sync/documents/{}", document.id))
        .send()
        .await
        .expect("request failed");

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await.expect("json parsing failed");
    assert_eq!(result["success"], true);
    assert!(result["data"]["state_vector"].is_object() || result["data"]["state_vector"].is_null());
    assert!(result["data"]["document_id"].is_string());
    assert!(result["error"].is_null());
}

/// Test sync state retrieval for non-existent document
#[tokio::test]
async fn test_get_sync_state_not_found() {
    let app = TestApp::create().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .get(&format!("/api/v1/sync/documents/{}", fake_id))
        .send()
        .await
        .expect("GET request failed");

    assert_eq!(response.status(), 404);
}

/// Test sync update submission
#[tokio::test]
async fn test_post_sync_update_success() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    let update_payload = serde_json::json!({
        "update": "test_base64_encoded_update",
        "state_vector": {
            "client_id": user.id.to_string(),
            "clock": 1
        }
    });

    let response = app
        .post(&format!("/api/v1/sync/documents/{}", document.id))
        .json(&update_payload)
        .send()
        .await
        .expect("request failed");

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await.expect("json parsing failed");
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["merged"], true);
    assert!(result["error"].is_null());
}

/// Test sync update with invalid base64
#[tokio::test]
async fn test_post_sync_update_invalid_format() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    let invalid_payload = serde_json::json!({
        "update": "not-valid-base64!!!",
        "state_vector": {}
    });

    let response = app
        .post(&format!("/api/v1/sync/documents/{}", document.id))
        .json(&invalid_payload)
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status(), 400);
}

/// Test sync status endpoint
#[tokio::test]
async fn test_get_sync_status_success() {
    let app = TestApp::create().await;
    let _user = app.create_test_user().await;

    let response = app
        .get("/api/v1/sync/offline/status")
        .send()
        .await
        .expect("request failed");

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await.expect("json parsing failed");
    assert_eq!(result["success"], true);
    assert!(result["data"]["pending_documents"].is_number());
    assert!(result["data"]["last_sync_time"].is_string() || result["data"]["last_sync_time"].is_null());
    assert!(result["error"].is_null());
}

/// Test full sync trigger endpoint
#[tokio::test]
async fn test_post_full_sync_trigger() {
    let app = TestApp::create().await;
    let _user = app.create_test_user().await;

    let response = app
        .post("/api/v1/sync/offline/sync")
        .send()
        .await
        .expect("request failed");

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await.expect("json parsing failed");
    assert_eq!(result["success"], true);
    assert!(result["data"]["synced_documents"].is_number());
    assert!(result["data"]["failed_documents"].is_number());
    assert!(result["error"].is_null());
}

/// Test sync without authentication fails
#[tokio::test]
async fn test_sync_requires_authentication() {
    let app = TestApp::create().await;
    let fake_id = Uuid::new_v4();

    // Create unauthenticated request
    let response = app
        .client
        .post(&format!("http://localhost:{}/api/v1/sync/documents/{}", app.port, fake_id))
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status(), 401);
}

/// Test sync update for document user doesn't have access to
#[tokio::test]
async fn test_sync_update_unauthorized_document() {
    let app = TestApp::create().await;
    let user1 = app.create_test_user().await;
    let user2 = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user1.id).await;
    let document = app.create_test_document(&space.id, None).await;

    // Try to sync as user2 who is not a member of the space
    let update_payload = serde_json::json!({
        "update": "test_base64_encoded_update",
        "state_vector": {}
    });

    let response = app
        .post_as_user(&format!("/api/v1/sync/documents/{}", document.id), &user2.id)
        .json(&update_payload)
        .send()
        .await
        .expect("request failed");

    assert_eq!(response.status(), 403);
}

/// Test concurrent sync updates
#[tokio::test]
async fn test_concurrent_sync_updates() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    // Simulate multiple concurrent updates
    let updates = vec![
        serde_json::json!({
            "update": "update1_base64",
            "state_vector": {"client_id": user.id.to_string(), "clock": 1}
        }),
        serde_json::json!({
            "update": "update2_base64",
            "state_vector": {"client_id": user.id.to_string(), "clock": 2}
        }),
    ];

    // Send updates concurrently
    let handles: Vec<_> = updates
        .into_iter()
        .map(|payload| {
            app.post(&format!("/api/v1/sync/documents/{}", document.id))
                .json(&payload)
                .send()
        })
        .collect();

    let results: Vec<_> = join_all(handles).await;

    // All updates should succeed
    for result in results {
        let response = result.expect("request failed");
        assert!(response.status().is_success());
    }
}

/// Test sync with empty update
#[tokio::test]
async fn test_sync_with_empty_update() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    let empty_payload = serde_json::json!({
        "update": "",
        "state_vector": {}
    });

    let response = app
        .post(&format!("/api/v1/sync/documents/{}", document.id))
        .json(&empty_payload)
        .send()
        .await
        .expect("request failed");

    // Empty update should be handled gracefully
    assert!(response.status().is_success() || response.status() == 400);
}

/// Test sync state includes document metadata
#[tokio::test]
async fn test_sync_state_includes_metadata() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    let response = app
        .get(&format!("/api/v1/sync/documents/{}", document.id))
        .send()
        .await
        .expect("request failed");

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await.expect("json parsing failed");
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["document_id"], document.id.to_string());
    assert_eq!(result["data"]["title"], document.title);
    assert!(result["data"]["version"].is_number());
    assert!(result["error"].is_null());
}

/// Helper extension for TestApp to support user-specific requests
trait TestAppExt {
    fn post_as_user(&self, path: &str, user_id: &Uuid) -> reqwest::RequestBuilder;
}

impl TestAppExt for TestApp {
    fn post_as_user(&self, path: &str, user_id: &Uuid) -> reqwest::RequestBuilder {
        self.client
            .post(&format!("http://localhost:{}{}", self.port, path))
            .header("X-User-Id", user_id.to_string())
    }
}
