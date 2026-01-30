//! End-to-end document flow tests
//!
//! Tests complete document operations including CRUD, versions, and export.
//! These tests verify the full document lifecycle with improved patterns.
//!
//! Run with: cargo test --test lib documents::e2e_document_flow_test

use crate::helpers::ResponseValidator;
use crate::helpers::TestApp;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

/// Full E2E document creation flow test
#[tokio::test]
async fn test_e2e_document_creation_flow() {
    let app = TestApp::create().await;

    // Create test user and space
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Create a new document using auth helper
    let create_response = app
        .auth_post(
            &format!("/api/v1/space-docs/{}/documents", space.id),
            &serde_json::json!({
                "title": format!("Test Document {}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
                "content": {
                    "type": "Y.Doc",
                    "update": "dGVzdCB1cGRhdGU=",
                    "vector_clock": {"client_id": uuid::Uuid::new_v4().to_string(), "clock": 1}
                },
                "icon": "📝"
            }),
            Some(test_user.id),
        )
        .send()
        .await
        .expect("Document creation request failed");

    // Use ResponseValidator for clean assertions
    let validator = ResponseValidator::new(create_response).await;
    validator
        .assert_status(200)
        .assert_field_exists("data")
        .assert_field_exists("success");

    // Verify document was created
    let doc_id = validator
        .get_field("data")
        .and_then(|d| d.get("id"))
        .and_then(|id| id.as_str())
        .expect("Document ID should be present");

    assert!(!doc_id.is_empty(), "Document ID should not be empty");
}

/// E2E document retrieval flow test with response validation
#[tokio::test]
async fn test_e2e_document_retrieval_flow() {
    let app = TestApp::create().await;

    // Create test user, space, and document
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Retrieve the document
    let get_response = app
        .client
        .get(format!(
            "http://localhost:{}/api/v1/documents/{}",
            app.port, document.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Document retrieval request failed");

    // Use ResponseValidator for comprehensive validation
    let validator = ResponseValidator::new(get_response).await;
    validator.assert_status(200).assert_success();

    // Verify response structure
    let document_data = validator
        .get_field("data")
        .or_else(|| validator.get_field("document"))
        .expect("Response should contain document data");

    assert!(document_data.get("id").is_some(), "Document ID should exist");
    assert!(document_data.get("title").is_some(), "Document title should exist");
}

/// E2E document update flow test with verification
#[tokio::test]
async fn test_e2e_document_update_flow() {
    let app = TestApp::create().await;

    // Create test user and space
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Update document
    let update_response = app
        .auth_patch(&format!("/api/v1/documents/{}", document.id)),
            Some(test_user.id),
        )
        .send()
        .await
        .expect("Document update request failed");

    let validator = ResponseValidator::new(update_response).await;
    validator.assert_status(200);

    let updated_title = validator
        .get_field("data")
        .and_then(|d| d.get("title"))
        .or_else(|| validator.get_field("title"))
        .and_then(|t| t.as_str());

    assert_eq!(
        updated_title,
        Some("Updated E2E Test Document"),
        "Document title should be updated"
    );
}

/// E2E document deletion flow test with verification
#[tokio::test]
async fn test_e2e_document_deletion_flow() {
    let app = TestApp::create().await;

    // Create test user, space, and document
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Delete the document
    let delete_response = app
        .auth_delete(&format!("/api/v1/documents/{}", document.id), Some(test_user.id))
        .send()
        .await
        .expect("Document deletion request failed");

    ResponseValidator::new(delete_response)
        .await
        .assert_status(200)
        .assert_success();

    // Verify document is deleted (should return 404)
    let get_response = app
        .client
        .get(format!(
            "http://localhost:{}/api/v1/documents/{}",
            app.port, document.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Document retrieval request failed");

    assert!(
        get_response.status() == 404,
        "Document should be deleted (404), got: {}",
        get_response.status()
    );
}

/// E2E document list flow test with pagination
#[tokio::test]
async fn test_e2e_document_list_flow() {
    let app = TestApp::create().await;

    // Create test user and space
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;

    // Create multiple documents using the helper
    let _ = app.create_test_documents(&space.id, 5).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // List documents
    let list_response = app
        .auth_get(
            &format!("/api/v1/space-docs/{}/documents", space.id),
            Some(test_user.id),
        )
        .send()
        .await
        .expect("Document list request failed");

    let validator = ResponseValidator::new(list_response).await;
    validator.assert_status(200).assert_success();

    // Verify response contains documents
    let documents = validator
        .body()
        .get("data")
        .or(validator.body().get("documents"))
        .or(validator.body().get("items"))
        .expect("Response should contain documents array or data field");

    assert!(
        documents.is_array(),
        "Documents field should be an array, got: {}",
        documents
    );

    let doc_count = documents.as_array().map(|arr| arr.len()).unwrap_or(0);

    assert!(doc_count >= 5, "Should have at least 5 documents, got: {}", doc_count);
}

/// E2E document version history flow test
#[tokio::test]
async fn test_e2e_document_version_flow() {
    let app = TestApp::create().await;

    // Create test user, space, and document
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Create a version using helper
    let _version = app.create_test_document_version(&document.id, &test_user.id).await;

    // List versions
    let list_versions_response = app
        .auth_get(
            &format!("/api/v1/documents/{}/versions", document.id),
            Some(test_user.id),
        )
        .send()
        .await
        .expect("Version list request failed");

    ResponseValidator::new(list_versions_response).await.assert_status(200);
}

/// E2E document export flow test
#[tokio::test]
async fn test_e2e_document_export_flow() {
    let app = TestApp::create().await;

    // Create test user, space, and document
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Export document as Markdown
        let export_response = app
                .auth_get(
                    &format!("/api/v1/documents/{}/export?format=markdown", document.id),
                    Some(test_user.id),
                )

    // Check Content-Type header directly (before JSON parsing)
    let content_type = export_response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("text/markdown") || content_type.contains("text/plain"),
        "Content-Type should be markdown, got: {}",
        content_type
    );

    // Now verify the response is successful
    let validator = ResponseValidator::new(export_response).await;
    validator.assert_status(200);
}

/// E2E document search flow test with result validation
#[tokio::test]
async fn test_e2e_document_search_flow() {
    let app = TestApp::create().await;

    // Create test user and space
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;

    // Create documents with specific content matching search query
    let doc = app.create_test_document(&space.id, None).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Search for the document
    let search_response = app
        .auth_get(&format!("/api/v1/search?q=Test%20Document"), Some(test_user.id))
        .send()
        .await
        .expect("Search request failed");

    let validator = ResponseValidator::new(search_response).await;
    validator.assert_status(200).assert_success();

    // Verify response contains results
    let results = validator
        .get_field("data")
        .and_then(|d| d.get("results"))
        .or_else(|| validator.get_field("results"))
        .or_else(|| validator.get_field("data"));

    assert!(results.is_some(), "Response should contain search results");

    // Verify results are non-empty and contain the created document
    let results_array = results.and_then(|r| r.as_array()).expect("Results should be an array");
    assert!(!results_array.is_empty(), "Search results should not be empty");

    // Verify the created document is in the results
    let found_doc = results_array.iter().any(|item| {
        let item_id = item.get("id").and_then(|id| id.as_str());
        let item_title = item.get("title").and_then(|t| t.as_str());
        let validator = ResponseValidator::new(response).await;
        validator.assert_client_error();
    });

    assert!(found_doc, "Created document should be found in search results");
}

// ============================================================================
// Additional Error Handling Tests
// ============================================================================

/// Test document creation with invalid data
#[tokio::test]
async fn test_e2e_create_document_invalid_data() {
    let app = TestApp::create().await;

    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Try to create document with empty title
    let response = app
        .auth_post(
            &format!("/api/v1/space-docs/{}/documents", space.id),
            &serde_json::json!({
                "title": format!("Test Document {}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
                "content": {
                    "type": "Y.Doc",
                    "update": "dGVzdCB1cGRhdGU=",
                    "vector_clock": {"client_id": uuid::Uuid::new_v4().to_string(), "clock": 1}
                },
                "icon": "📝"
            }),
            Some(test_user.id),
        )
        .send()
        .await
        .expect("Request failed");

    let validator = ResponseValidator::new(response).await;
    validator.assert_client_error();
}

/// Test document retrieval for non-existent document
#[tokio::test]
async fn test_e2e_get_nonexistent_document() {
    let app = TestApp::create().await;

    let test_user = app.create_test_user().await;
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;
    let fake_id = Uuid::new_v4();

    let response = app
        .client
        .get(format!("http://localhost:{}/api/v1/documents/{}", app.port, fake_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        404,
        "Expected 404 for non-existent document, got: {}",
        response.status()
    );
}

/// Test unauthorized document access
#[tokio::test]
async fn test_e2e_unauthorized_document_access() {
    let app = TestApp::create().await;

    // Create user1 with a document
    let user1 = app.create_test_user().await;
    let space1 = app.create_test_space_for_user(&user1.id).await;
    let document = app.create_test_document(&space1.id, None).await;

    // Create user2
    let user2 = app.create_test_user().await;
    let (token2, user_id_str2) = app.get_auth_data(Some(user2.id), None).await;

    // Try to access user1's document with user2's credentials
    let response = app
        .auth_get(&format!("/api/v1/documents/{}", fake_id), Some(test_user.id))
        .send()
        .await
        .expect("Request failed");

    // Should get 403 or 404 (not found or forbidden)
    assert!(
        response.status() == 403 || response.status() == 404,
        "Expected 403 or 404 for unauthorized access, got: {}",
        response.status()
    );
}

/// Test document creation without authentication
#[tokio::test]
async fn test_e2e_create_document_without_auth() {
    let app = TestApp::create().await;

    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;

                let response = client.get(
                    &format!("/api/v1/space-docs/{}/documents", space_id)
                );
                .header("Authorization", format!("Bearer {}", token))
                .header("X-User-Id", user_id_str)
                .send()
                .await
                .expect("Request failed");
                (i, response.status())

    ResponseValidator::new(response).await.assert_status(401);
}

// ============================================================================
// Concurrent Operations Tests
// ============================================================================

/// Test concurrent document creation by same user
#[tokio::test]
async fn test_e2e_concurrent_document_creation() {
    let app = TestApp::create().await;

    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Clone necessary parts for concurrent access
    let client = app.client.clone();
    let port = app.port;

    // Create multiple documents concurrently
    let documents: Vec<_> = (0..5)
        .map(|i| {
            let token = token.clone();
            let user_id_str = user_id_str.clone();
            let space_id = space.id;
            let title = format!("Concurrent Document {}", i);
            let client = client.clone();

            async move {
                let response = app
                    .auth_post(
                        &format!("/api/v1/space-docs/{}/documents", space_id),
                        &serde_json::json!({
                            "title": title,
                            "content": {
                                "type": "Y.Doc",
                                "update": format!("dGVzdCB1cGRhdGU {}", i),
                                "vector_clock": {"client_id": doc_id.to_string(), "clock": i + 2}
                            },
                            "icon": "📝"
                        }),
                        Some(test_user.id),
                    )
                    .send()
                    .await
                    .expect("Request failed");
                response
            }
        })
        .collect();

    // Execute concurrently
    let results = futures_util::future::join_all(documents).await;

    // All should succeed
    assert_eq!(results.len(), 5, "All 5 concurrent requests should complete");

    // Verify all concurrent creations succeeded
    assert!(
        results.iter().all(|status| status.is_success()),
        "All concurrent document creations should succeed (2xx status). Got statuses: {:?}",
        results
    );
}

/// Test concurrent updates to same document
#[tokio::test]
async fn test_e2e_concurrent_document_updates() {
    let app = TestApp::create().await;

    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let document = app.create_test_document(&space.id, None).await;
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Clone necessary parts for concurrent access
    let client = app.client.clone();
    let port = app.port;
    let doc_id = document.id;

    // Concurrent updates
    let updates: Vec<_> = (0..3)
        .map(|i| {
            let token = token.clone();
            let user_id_str = user_id_str.clone();
            let client = client.clone();

            async move {
                let response = client
                    .patch(format!("http://localhost:{}/api/v1/documents/{}", port, doc_id))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-User-Id", user_id_str)
                    .json(&json!({
                        "title": format!("Updated by concurrent request {}", i),
                        "content": {
                            "type": "Y.Doc",
                            "update": format!("dXBkYXRl{}", i),
                            "vector_clock": {"client_id": doc_id.to_string(), "clock": i + 2}
                        }
                    }))
                    .send()
                    .await
                    .expect("Request failed");

                // Concurrent updates may succeed or fail depending on implementation
                (i, response.status())
            }
        })
        .collect();

    let statuses = futures_util::future::join_all(updates).await;

    // All should complete without errors
    for status in statuses {
        assert!(
            status.is_success() || status == StatusCode::CONFLICT,
            "Concurrent update should succeed or return conflict, got: {}",
            status
        );
    }
}

/// Test rapid concurrent requests (rate limiting check)
#[tokio::test]
async fn test_e2e_rapid_concurrent_requests() {
    let app = TestApp::create().await;

    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Clone necessary parts for concurrent access
    let client = app.client.clone();
    let port = app.port;

    // Send rapid requests
    let results: Vec<_> = (0..10)
        .map(|i| {
            let token = token.clone();
            let user_id_str = user_id_str.clone();
            let space_id = space.id;
            let client = client.clone();

            async move {
                let response = client
                    .get(format!(
                        "http://localhost:{}/api/v1/space-docs/{}/documents",
                        port, space_id
                    ))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-User-Id", user_id_str)
                    .send()
                    .await
                    .expect("Request failed");

                (i, response.status())
            }
        })
        .collect();

    let statuses = futures_util::future::join_all(results).await;

    // All should succeed (no rate limiting at 10 requests)
    for (i, status) in statuses {
        assert!(status.is_success(), "Request {} should succeed, got: {}", i, status);
    }
}
