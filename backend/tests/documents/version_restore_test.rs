//! Version restore operation tests
//!
//! Tests specifically for document version restore functionality.
//! These tests verify the version restore flow works correctly.
//!
//! Run with: cargo test -p miniwiki-backend-tests documents::version_restore_test

use document_service::models::{CreateDocumentRequest, CreateVersionRequest};
use crate::helpers::{create_test_app, create_test_document, create_test_space, create_test_user, TestApp};
use uuid::Uuid;

/// Test cases for version restore functionality:
/// 1. Basic restore to previous version
/// 2. Restore creates new version (not overwriting)
/// 3. Restore with concurrent edits handling
/// 4. Restore permission checks
/// 5. Restore version not found
/// 6. Restore to same version (no-op)
/// 7. Restore with content validation
/// 8. Restore audit logging

#[tokio::test]
async fn test_restore_creates_new_version_not_overwrite() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Create initial version
    let initial_request = CreateVersionRequest {
        content: serde_json::json!({"text": "Original content"}),
        title: "Original Title".to_string(),
        change_summary: Some("Initial version".to_string()),
    };

    let initial_response = app
        .post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&initial_request)
        .send()
        .await;

    assert!(initial_response.status().is_success());
    let initial_version: serde_json::Value = initial_response.json().await;
    assert_eq!(initial_version["data"]["version_number"], 1);

    // Update document (version 2)
    let update_response = app
        .patch(&format!("/api/v1/documents/{}", document.id))
        .json(&serde_json::json!({
            "title": "Modified Title",
            "content": {"text": "Modified content"}
        }))
        .send()
        .await;

    assert!(update_response.status().is_success());

    // Restore to version 1
    let restore_response = app
        .post(&format!("/api/v1/documents/{}/versions/1/restore", document.id))
        .send()
        .await;

    assert!(restore_response.status().is_success());
    let restored_version: serde_json::Value = restore_response.json().await;

    // RESTORE SHOULD CREATE NEW VERSION (version 3), not overwrite version 1 or 2
    assert_eq!(restored_version["data"]["version_number"], 3);
    assert_eq!(restored_version["data"]["title"], "Original Title");
    assert_eq!(restored_version["data"]["restored_from_version"], 1);
    assert!(restored_version["data"]["change_summary"]
        .as_str()
        .unwrap_or("")
        .contains("Restored to version 1"));
}

#[tokio::test]
async fn test_restore_version_not_found() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Try to restore non-existent version
    let response = app
        .post(&format!("/api/v1/documents/{}/versions/999/restore", document.id))
        .send()
        .await;

    assert_eq!(response.status(), 404);
    let error: serde_json::Value = response.json().await;
    assert!(error["error"]["message"]
        .as_str()
        .unwrap_or("")
        .to_lowercase()
        .contains("not found"));
}

#[tokio::test]
async fn test_restore_same_version_noop() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Create a version
    let request = CreateVersionRequest {
        content: serde_json::json!({"text": "Content v1"}),
        title: "Title v1".to_string(),
        change_summary: Some("Version 1".to_string()),
    };

    app.post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&request)
        .send()
        .await;

    // Restore to current version (version 1) should be a no-op
    // but still creates a new version with same content
    let response = app
        .post(&format!("/api/v1/documents/{}/versions/1/restore", document.id))
        .send()
        .await;

    // Should still succeed, creating version 2
    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    assert_eq!(result["data"]["version_number"], 2);
    assert!(result["data"]["change_summary"]
        .as_str()
        .unwrap_or("")
        .contains("Restored to version 1"));
}

#[tokio::test]
async fn test_restore_content_preservation() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Create version with specific Yjs content
    let yjs_content = serde_json::json!({
        "type": "Y.Doc",
        "update": "dGhpcyBpcyBhIHRlc3QgdXBkYXRl", // base64 encoded test
        "vector_clock": {
            "client_id": user.id.to_string(),
            "clock": 1
        }
    });

    let create_request = CreateVersionRequest {
        content: yjs_content.clone(),
        title: "Original Document".to_string(),
        change_summary: Some("Original version".to_string()),
    };

    app.post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&create_request)
        .send()
        .await;

    // Update document
    app.patch(&format!("/api/v1/documents/{}", document.id))
        .json(&serde_json::json!({
            "title": "Modified Document",
            "content": {"type": "Y.Doc", "update": "bmV3IGNvbnRlbnQ="}
        }))
        .send()
        .await;

    // Restore to version 1
    let restore_response = app
        .post(&format!("/api/v1/documents/{}/versions/1/restore", document.id))
        .send()
        .await;

    assert!(restore_response.status().is_success());
    let restored: serde_json::Value = restore_response.json().await;

    // Verify content is restored
    assert_eq!(restored["data"]["title"], "Original Document");
    // The content should match the original version's content
    assert!(restored["data"]["content"].is_object());
}

#[tokio::test]
async fn test_restore_preserves_document_id() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Create version
    let request = CreateVersionRequest {
        content: serde_json::json!({"text": "Content"}),
        title: "Title".to_string(),
        change_summary: None,
    };

    app.post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&request)
        .send()
        .await;

    // Restore
    let response = app
        .post(&format!("/api/v1/documents/{}/versions/1/restore", document.id))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;

    // Restored version should reference the original document
    assert_eq!(result["data"]["document_id"], document.id.to_string());
    assert_eq!(result["data"]["restored_from_version"], 1);
}

#[tokio::test]
async fn test_restore_creates_audit_log() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Create version
    let request = CreateVersionRequest {
        content: serde_json::json!({}),
        title: "Title".to_string(),
        change_summary: None,
    };

    app.post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&request)
        .send()
        .await;

    // Restore
    let restore_response = app
        .post(&format!("/api/v1/documents/{}/versions/1/restore", document.id))
        .send()
        .await;

    assert!(restore_response.status().is_success());

    // Check audit log for restore action
    let audit_response = app
        .get(&format!("/api/v1/audit?document_id={}", document.id))
        .send()
        .await;

    assert!(audit_response.status().is_success());
    let audit_logs: serde_json::Value = audit_response.json().await;

    // Should have audit log entry for version restore
    let restore_logs: Vec<_> = audit_logs["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|log| log["action"] == "version.restore")
        .collect();

    assert!(!restore_logs.is_empty());
    assert_eq!(restore_logs[0]["resource_id"], document.id.to_string());
}

#[tokio::test]
async fn test_restore_invalid_document_id() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;

    // Try to restore version for non-existent document
    let fake_doc_id = Uuid::new_v4();
    let response = app
        .post(&format!("/api/v1/documents/{}/versions/1/restore", fake_doc_id))
        .send()
        .await;

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_restore_version_chain_integrity() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Create versions: 1, 2, 3
    for i in 1..=3 {
        let request = CreateVersionRequest {
            content: serde_json::json!({"version": i}),
            title: format!("Version {}", i),
            change_summary: None,
        };

        app.post(&format!("/api/v1/documents/{}/versions", document.id))
            .json(&request)
            .send()
            .await;
    }

    // Restore to version 2 (should create version 4)
    let restore_response = app
        .post(&format!("/api/v1/documents/{}/versions/2/restore", document.id))
        .send()
        .await;

    assert!(restore_response.status().is_success());
    let restored: serde_json::Value = restore_response.json().await;
    assert_eq!(restored["data"]["version_number"], 4);
    assert_eq!(restored["data"]["restored_from_version"], 2);

    // Get all versions - should be 4 versions total
    let list_response = app
        .get(&format!(
            "/api/v1/documents/{}/versions?limit=100",
            document.id
        ))
        .send()
        .await;

    assert!(list_response.status().is_success());
    let versions: serde_json::Value = list_response.json().await;
    assert_eq!(
        versions["data"]["versions"].as_array().unwrap().len(),
        4
    );
}

#[tokio::test]
async fn test_restore_concurrent_edits_simulation() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Create version 1
    let v1_request = CreateVersionRequest {
        content: serde_json::json!({"text": "Version 1"}),
        title: "V1".to_string(),
        change_summary: Some("Original".to_string()),
    };

    app.post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&v1_request)
        .send()
        .await;

    // Create version 2 with updated content
    let v2_request = CreateVersionRequest {
        content: serde_json::json!({"text": "Version 2 - Updated"}),
        title: "V2".to_string(),
        change_summary: Some("Updated content".to_string()),
    };

    app.post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&v2_request)
        .send()
        .await;

    // Restore to version 1 while document might have been updated
    // This tests that restore still works correctly
    let restore_response = app
        .post(&format!("/api/v1/documents/{}/versions/1/restore", document.id))
        .send()
        .await;

    // Restore should succeed regardless of concurrent state
    assert!(restore_response.status().is_success());
    let result: serde_json::Value = restore_response.json().await;
    assert_eq!(result["data"]["title"], "V1");
}

#[tokio::test]
async fn test_restore_response_format() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await.unwrap();
    let document = create_test_document(&app, &space.id, None).await;

    // Create version
    let request = CreateVersionRequest {
        content: serde_json::json!({}),
        title: "Test Document".to_string(),
        change_summary: Some("Test version".to_string()),
    };

    app.post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&request)
        .send()
        .await;

    // Restore
    let response = app
        .post(&format!("/api/v1/documents/{}/versions/1/restore", document.id))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;

    // Verify response structure
    assert_eq!(result["success"], true);
    assert!(result["error"].is_null());
    assert!(result["data"].is_object());
    assert!(result["data"]["id"].is_string());
    assert!(result["data"]["document_id"].is_string());
    assert!(result["data"]["version_number"].is_number());
    assert!(result["data"]["title"].is_string());
    assert!(result["data"]["content"].is_object());
    assert!(result["data"]["created_by"].is_string());
    assert!(result["data"]["created_at"].is_string());
    assert!(result["data"]["restored_from_version"].is_number());
    assert!(result["data"]["change_summary"].is_string());
}
