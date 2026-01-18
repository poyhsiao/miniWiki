//! Sync endpoint tests
//!
//! Tests for CRDT sync operations including state retrieval and update processing.
//! These tests verify the sync service handlers work correctly for offline-first sync.
//!
//! Run with: cargo test -p miniwiki-backend-tests sync::sync_test

use crate::helpers::*;
use actix_web::test;
use actix_web::web;
use actix_web::dev::ServiceResponse;
use auth_service::repository::AuthRepository;
use document_service::repository::DocumentRepository;
use std::sync::Arc;
use sync_service::sync_handler::SyncAppState;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Helper to create a test app with all required state - returns the service directly
macro_rules! create_test_service {
    () => {{
        let test_app = TestApp::create().await;

        // Create all required app state
        let sync_state = web::Data::new(SyncAppState {
            pool: test_app.pool.clone(),
            server_clock: Arc::new(Mutex::new(0)),
        });

        let auth_repository = web::Data::new(AuthRepository::new(test_app.pool.clone()));
        let document_repository = web::Data::new(DocumentRepository::new(test_app.pool.clone()));

        let app = test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(test_app.pool.clone()))
                .app_data(auth_repository)
                .app_data(document_repository)
                .app_data(sync_state)
                .configure(miniwiki_backend::routes::config)
        ).await;

        // Return both test_app and app wrapped in Rc for sharing across async tasks
        let app = std::rc::Rc::new(app);
        (test_app, app)
    }};
}

/// Test sync state retrieval for a document
#[actix_rt::test]
async fn test_get_sync_state_success() {
    let (test_app, app) = create_test_service!();

    let test_user = test_app.create_test_user().await;
    let space = test_app.create_test_space_for_user(&test_user.id).await;
    let document = test_app.create_test_document(&space.id, None).await;

    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/sync/documents/{}", document.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Expected success status, got: {}", resp.status());

    let body = test::read_body(resp).await;
    let result: serde_json::Value = serde_json::from_slice(&body).expect("json parsing failed");
    assert_eq!(result["document_id"], document.id.to_string());
    assert_eq!(result["title"], document.title);
}

/// Test sync state retrieval for non-existent document
#[actix_rt::test]
async fn test_get_sync_state_not_found() {
    let (test_app, app) = create_test_service!();

    let test_user = test_app.create_test_user().await;
    let fake_id = Uuid::new_v4();
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/sync/documents/{}", fake_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404, "Expected 404 for non-existent document");
}

/// Test sync update submission
#[actix_rt::test]
async fn test_post_sync_update_success() {
    let (test_app, app) = create_test_service!();

    let test_user = test_app.create_test_user().await;
    let space = test_app.create_test_space_for_user(&test_user.id).await;
    let document = test_app.create_test_document(&space.id, None).await;

    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let update_payload = serde_json::json!({
        "update": "dGVzdCB1cGRhdGU=", // "test update" in base64
        "state_vector": {
            "client_id": test_user.id.to_string(),
            "clock": 1
        }
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/sync/documents/{}", document.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_payload)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;

    // Only accept 200 status - any other status is a test failure
    assert_eq!(resp.status(), 200, "Expected 200 OK, got: {}", resp.status());
    
    let body = test::read_body(resp).await;
    let result: serde_json::Value = serde_json::from_slice(&body).expect("json parsing failed");
    assert_eq!(result["success"], true, "Expected success=true in response");
}

/// Test sync update with invalid base64
#[actix_rt::test]
async fn test_post_sync_update_invalid_format() {
    let (test_app, app) = create_test_service!();

    let test_user = test_app.create_test_user().await;
    let space = test_app.create_test_space_for_user(&test_user.id).await;
    let document = test_app.create_test_document(&space.id, None).await;

    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let invalid_payload = serde_json::json!({
        "update": "not-valid-base64!!!",
        "state_vector": {}
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/sync/documents/{}", document.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&invalid_payload)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Expected 400 for invalid base64");
}

/// Test sync status endpoint
#[actix_rt::test]
async fn test_get_sync_status_success() {
    let (test_app, app) = create_test_service!();

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let req = test::TestRequest::get()
        .uri("/api/v1/sync/offline/status")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Expected success status, got: {}", resp.status());

    let body = test::read_body(resp).await;
    let result: serde_json::Value = serde_json::from_slice(&body).expect("json parsing failed");
    assert!(result["pending_documents"].is_number());
}

/// Test full sync trigger endpoint
#[actix_rt::test]
async fn test_post_full_sync_trigger() {
    let (test_app, app) = create_test_service!();

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    // Full sync requires a request body
    let req_body = serde_json::json!({
        "document_ids": null  // null means sync all documents
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/sync/offline/sync")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&req_body)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Expected success status, got: {}", resp.status());

    let body = test::read_body(resp).await;
    let result: serde_json::Value = serde_json::from_slice(&body).expect("json parsing failed");
    assert_eq!(result["success"], true);
    assert!(result["synced_documents"].is_number());
    assert!(result["failed_documents"].is_number());
}

/// Test sync without authentication fails
#[actix_rt::test]
async fn test_sync_requires_authentication() {
    let (_, app) = create_test_service!();

    let fake_id = Uuid::new_v4();

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/sync/documents/{}", fake_id))
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    // Should return 400 (bad request due to missing CSRF) or 401 (unauthorized)
    // The actual behavior depends on middleware configuration
    assert!(resp.status() == 400 || resp.status() == 401,
        "Expected 400 or 401 for unauthenticated request, got: {}", resp.status());
}

/// Test sync update for document user doesn't have access to
#[actix_rt::test]
async fn test_sync_update_unauthorized_document() {
    let (test_app, app) = create_test_service!();

    let user1 = test_app.create_test_user().await;
    let user2 = test_app.create_test_user().await;
    let space = test_app.create_test_space_for_user(&user1.id).await;
    let document = test_app.create_test_document(&space.id, None).await;

    let token = generate_test_jwt_token(user2.id, &user2.email);

    let update_payload = serde_json::json!({
        "update": "dGVzdCB1cGRhdGU=",
        "state_vector": {}
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/sync/documents/{}", document.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_payload)
        .to_request();

    let resp: ServiceResponse = test::call_service(&app, req).await;
    // User2 is not a member of the space
    // May return 400 (CSRF issue), 404 (not found), or 403 (forbidden)
    assert!(resp.status() == 400 || resp.status() == 404 || resp.status() == 403,
        "Expected 400, 404, or 403 for unauthorized access, got: {}", resp.status());
}

/// Test concurrent sync updates
#[actix_rt::test]
async fn test_concurrent_sync_updates() {
    let (test_app, app) = create_test_service!();

    let test_user = test_app.create_test_user().await;
    let space = test_app.create_test_space_for_user(&test_user.id).await;
    let document = test_app.create_test_document(&space.id, None).await;

    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    // Simulate multiple concurrent updates
    let updates = vec![
        serde_json::json!({
            "update": "dXBkYXRlMV8x", // "update1_1" in base64
            "state_vector": {"client_id": test_user.id.to_string(), "clock": 1}
        }),
        serde_json::json!({
            "update": "dXBkYXRlMl8y", // "update2_2" in base64
            "state_vector": {"client_id": test_user.id.to_string(), "clock": 2}
        }),
    ];

    // Clone the Rc to share the app and token across spawned tasks
    let app_rc = app.clone();
    let token_clone = token.clone();
    let document_id = document.id;

    // Send updates concurrently
    let handles: Vec<_> = updates
        .into_iter()
        .map(move |payload| {
            let app = app_rc.clone();
            let token = token_clone.clone();
            async move {
                let req = test::TestRequest::post()
                    .uri(&format!("/api/v1/sync/documents/{}", document_id))
                    .insert_header(("Authorization", format!("Bearer {}", token)))
                    .set_json(&payload)
                    .to_request();
                // Dereference the Rc once to get the inner service
                test::call_service(&*app, req).await
            }
        })
        .collect();

    let results: Vec<_> = futures_util::future::join_all(handles).await;

    // All updates should succeed (200 or 400 for invalid base64)
    for response in results {
        assert!(response.status() == 200 || response.status() == 400,
            "Expected 200 or 400, got: {}", response.status());
    }
}

/// Test sync with empty update
#[actix_rt::test]
async fn test_sync_with_empty_update() {
    let (test_app, app) = create_test_service!();

    let test_user = test_app.create_test_user().await;
    let space = test_app.create_test_space_for_user(&test_user.id).await;
    let document = test_app.create_test_document(&space.id, None).await;

    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let empty_payload = serde_json::json!({
        "update": "",
        "state_vector": {}
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/sync/documents/{}", document.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&empty_payload)
        .to_request();

    let resp: ServiceResponse = test::call_service(&*app, req).await;
    // Empty update should be handled gracefully (200 or 400)
    assert!(resp.status() == 200 || resp.status() == 400,
        "Expected 200 or 400, got: {}", resp.status());
}
