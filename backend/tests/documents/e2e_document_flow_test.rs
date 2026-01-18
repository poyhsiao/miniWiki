//! End-to-end document flow tests
//!
//! Tests complete document operations including CRUD, versions, and export.
//! These tests verify the full document lifecycle.
//!
//! Run with: cargo test

use crate::helpers::TestApp;
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

    // Create a new document
    let create_response = app
        .client
        .post(&format!(
            "http://localhost:{}/v1/spaces/{}/documents",
            app.port, space.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .json(&json!({
            "title": "E2E Test Document",
            "content": {
                "type": "Y.Doc",
                "update": "dGVzdCB1cGRhdGU=",
                "vector_clock": {"client_id": Uuid::new_v4().to_string(), "clock": 1}
            },
            "icon": "📝"
        }))
        .send()
        .await
        .expect("Document creation request failed");

    assert!(
        create_response.status() == 200 || create_response.status() == 201,
        "Expected 200 or 201 for document creation, got: {}",
        create_response.status()
    );

    if create_response.status() == 200 || create_response.status() == 201 {
        let body: serde_json::Value = create_response.json().await.expect("Parse create response failed");
        let doc_id = body.get("data").and_then(|d| d.get("id")).or_else(|| body.get("id"));

        if let Some(doc_id_val) = doc_id {
            println!("Created document: {:?}", doc_id_val);
        }
    }
}

/// E2E document retrieval flow test
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
        .get(&format!(
            "http://localhost:{}/v1/documents/{}",
            app.port, document.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Document retrieval request failed");

    assert!(
        get_response.status() == 200,
        "Expected 200 for document retrieval, got: {}",
        get_response.status()
    );

    // Verify response structure
    let body: serde_json::Value = get_response.json().await.expect("Parse get response failed");
    assert!(
        body.get("data").is_some() || body.get("document").is_some(),
        "Response should contain document data"
    );
}

/// E2E document update flow test
#[tokio::test]
async fn test_e2e_document_update_flow() {
    let app = TestApp::create().await;

    // Create test user, space, and document
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;
    let document = app.create_test_document(&space.id, None).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Update the document
    let update_response = app
        .client
        .patch(&format!(
            "http://localhost:{}/v1/documents/{}",
            app.port, document.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .json(&json!({
            "title": "Updated E2E Test Document",
            "content": {
                "type": "Y.Doc",
                "update": "dXBkYXRlZCB1cGRhdGU=",
                "vector_clock": {"client_id": document.id.to_string(), "clock": 2}
            }
        }))
        .send()
        .await
        .expect("Document update request failed");

    assert!(
        update_response.status() == 200,
        "Expected 200 for document update, got: {}",
        update_response.status()
    );

    // Verify the update
    let body: serde_json::Value = update_response.json().await.expect("Parse update response failed");
    let updated_title = body.get("data").and_then(|d| d.get("title"))
        .or_else(|| body.get("title"))
        .and_then(|t| t.as_str());

    assert!(
        updated_title == Some("Updated E2E Test Document"),
        "Document title should be updated"
    );
}

/// E2E document deletion flow test
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
        .client
        .delete(&format!(
            "http://localhost:{}/v1/documents/{}",
            app.port, document.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Document deletion request failed");

    assert!(
        delete_response.status() == 200,
        "Expected 200 for document deletion, got: {}",
        delete_response.status()
    );

    // Verify document is deleted (should return 404)
    let get_response = app
        .client
        .get(&format!(
            "http://localhost:{}/v1/documents/{}",
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

/// E2E document list flow test
#[tokio::test]
async fn test_e2e_document_list_flow() {
    let app = TestApp::create().await;

    // Create test user and space
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;

    // Create multiple documents
    let _doc1 = app.create_test_document(&space.id, None).await;
    let _doc2 = app.create_test_document(&space.id, None).await;
    let _doc3 = app.create_test_document(&space.id, None).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // List documents
    let list_response = app
        .client
        .get(&format!(
            "http://localhost:{}/v1/space-docs/{}/documents",
            app.port, space.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Document list request failed");

    assert!(
        list_response.status() == 200,
        "Expected 200 for document list, got: {}",
        list_response.status()
    );

    // Verify response contains documents
    let body: serde_json::Value = list_response.json().await.expect("Parse list response failed");
    let documents = body.get("data").and_then(|d| d.get("documents"))
        .or_else(|| body.get("documents"))
        .or_else(|| body.get("data"));

    assert!(
        documents.is_some(),
        "Response should contain documents array"
    );
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

    // Create a version
    let create_version_response = app
        .client
        .post(&format!(
            "http://localhost:{}/v1/documents/{}/versions",
            app.port, document.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .json(&json!({
            "name": "E2E Test Version",
            "description": "Created by E2E test"
        }))
        .send()
        .await
        .expect("Version creation request failed");

    // Version creation may be automatic or manual depending on implementation
    assert!(
        create_version_response.status() == 200 || create_version_response.status() == 201,
        "Expected 200 or 201 for version creation, got: {}",
        create_version_response.status()
    );

    // List versions
    let list_versions_response = app
        .client
        .get(&format!(
            "http://localhost:{}/v1/documents/{}/versions",
            app.port, document.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Version list request failed");

    assert!(
        list_versions_response.status() == 200,
        "Expected 200 for version list, got: {}",
        list_versions_response.status()
    );
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
        .client
        .get(&format!(
            "http://localhost:{}/v1/documents/{}/export?format=markdown",
            app.port, document.id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Export request failed");

    assert!(
        export_response.status() == 200,
        "Expected 200 for export, got: {}",
        export_response.status()
    );

    // Verify content type
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
}

/// E2E document search flow test
#[tokio::test]
async fn test_e2e_document_search_flow() {
    let app = TestApp::create().await;

    // Create test user and space
    let test_user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&test_user.id).await;

    // Create documents with specific content matching search query
    let doc = app.create_test_document(&space.id, Some("Test Document".to_string())).await;

    // Get auth token
    let (token, user_id_str) = app.get_auth_data(Some(test_user.id), None).await;

    // Search for the document
    let search_response = app
        .client
        .get(&format!(
            "http://localhost:{}/v1/search?q=Test%20Document",
            app.port
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-User-Id", user_id_str.clone())
        .send()
        .await
        .expect("Search request failed");

    assert!(
        search_response.status() == 200,
        "Expected 200 for search, got: {}",
        search_response.status()
    );

    // Verify response contains results
    let body: serde_json::Value = search_response.json().await.expect("Parse search response failed");
    let results = body.get("data").and_then(|d| d.get("results"))
        .or_else(|| body.get("results"))
        .or_else(|| body.get("data"));

    assert!(
        results.is_some(),
        "Response should contain search results"
    );

    // Verify results are non-empty and contain the created document
    let results_array = results.and_then(|r| r.as_array()).expect("Results should be an array");
    assert!(
        !results_array.is_empty(),
        "Search results should not be empty"
    );

    // Verify the created document is in the results
    let found_doc = results_array.iter().any(|item| {
        let item_id = item.get("id").and_then(|id| id.as_str());
        let item_title = item.get("title").and_then(|t| t.as_str());
        item_id == Some(doc.id.to_string().as_str()) ||
        item_title.map_or(false, |t| t.contains("Test Document"))
    });

    assert!(
        found_doc,
        "Created document should be found in search results"
    );
}
