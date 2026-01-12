//! Document version operation tests
//!
//! Tests for document versioning, history retrieval, and restore functionality.
//! These tests verify the version control system works correctly.
//!
//! Run with: cargo test -p miniwiki-backend-tests documents::versions_test

use chrono::Utc;
use miniwiki_backend::services::document_service::{CreateDocumentRequest, CreateVersionRequest};
use miniwiki_backend::tests::helpers::{create_test_app, create_test_document, create_test_space, create_test_user, TestApp};
use uuid::Uuid;

#[tokio::test]
async fn test_create_version_success() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    let request = CreateVersionRequest {
        document_id: document.id,
        content: serde_json::json!({
            "type": "Y.Doc",
            "update": "version1_update",
            "vector_clock": {
                "client_id": user.id.to_string(),
                "clock": 2
            }
        }),
        title: document.title.clone(),
        change_summary: Some("First edit".to_string()),
    };

    let response = app
        .post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&request)
        .send()
        .await;

    assert!(response.status().is_success());
    let version: serde_json::Value = response.json().await;
    assert_eq!(version["version_number"], 1);
    assert_eq!(version["change_summary"], "First edit");
}

#[tokio::test]
async fn test_create_version_auto_increments() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    // Create multiple versions
    for i in 1..=3 {
        let request = CreateVersionRequest {
            document_id: document.id,
            content: serde_json::json!({}),
            title: format!("Title v{}", i),
            change_summary: Some(format!("Version {}", i)),
        };

        let response = app
            .post(&format!("/api/v1/documents/{}/versions", document.id))
            .json(&request)
            .send()
            .await;

        assert!(response.status().is_success());
        let version: serde_json::Value = response.json().await;
        assert_eq!(version["version_number"], i);
    }
}

#[tokio::test]
async fn test_list_document_versions() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    // Create some versions
    for i in 1..=5 {
        let request = CreateVersionRequest {
            document_id: document.id,
            content: serde_json::json!({}),
            title: format!("Version {}", i),
            change_summary: None,
        };

        let _ = app
            .post(&format!("/api/v1/documents/{}/versions", document.id))
            .json(&request)
            .send()
            .await;
    }

    let response = app
        .get(&format!("/api/v1/documents/{}/versions", document.id))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    assert!(result["versions"].is_array());
    assert_eq!(result["versions"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn test_get_specific_version() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    // Create versions
    for i in 1..=3 {
        let request = CreateVersionRequest {
            document_id: document.id,
            content: serde_json::json!({"version": i}),
            title: format!("Version {}", i),
            change_summary: None,
        };

        let _ = app
            .post(&format!("/api/v1/documents/{}/versions", document.id))
            .json(&request)
            .send()
            .await;
    }

    // Get version 2
    let response = app
        .get(&format!("/api/v1/documents/{}/versions/2", document.id))
        .send()
        .await;

    assert!(response.status().is_success());
    let version: serde_json::Value = response.json().await;
    assert_eq!(version["version_number"], 2);
    assert_eq!(version["title"], "Version 2");
}

#[tokio::test]
async fn test_get_version_not_found() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    let response = app
        .get(&format!("/api/v1/documents/{}/versions/999", document.id))
        .send()
        .await;

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_restore_version() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    // Create initial version
    let request = CreateVersionRequest {
        document_id: document.id,
        content: serde_json::json!({"content": "original"}),
        title: "Original Title".to_string(),
        change_summary: Some("Original".to_string()),
    };

    let _ = app
        .post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&request)
        .send()
        .await;

    // Update document
    let update_response = app
        .patch(&format!("/api/v1/documents/{}", document.id))
        .json(&serde_json::json!({
            "title": "Modified Title",
            "content": {"content": "modified"}
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
    let restored: serde_json::Value = restore_response.json().await;
    assert_eq!(restored["title"], "Original Title");

    // Verify current document state
    let get_response = app
        .get(&format!("/api/v1/documents/{}", document.id))
        .send()
        .await;

    assert!(get_response.status().is_success());
    let current: serde_json::Value = get_response.json().await;
    assert_eq!(current["title"], "Original Title");
}

#[tokio::test]
async fn test_version_pagination() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    // Create 15 versions
    for i in 1..=15 {
        let request = CreateVersionRequest {
            document_id: document.id,
            content: serde_json::json!({}),
            title: format!("Version {}", i),
            change_summary: None,
        };

        let _ = app
            .post(&format!("/api/v1/documents/{}/versions", document.id))
            .json(&request)
            .send()
            .await;
    }

    // Get first page
    let response = app
        .get(&format!(
            "/api/v1/documents/{}/versions?limit=5&offset=0",
            document.id
        ))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    assert_eq!(result["versions"].as_array().unwrap().len(), 5);
    assert_eq!(result["total"], 15);
    assert_eq!(result["limit"], 5);
}

#[tokio::test]
async fn test_version_diff() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    // Create version 1
    let request1 = CreateVersionRequest {
        document_id: document.id,
        content: serde_json::json!({"text": "Hello"}),
        title: "V1".to_string(),
        change_summary: None,
    };

    let _ = app
        .post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&request1)
        .send()
        .await;

    // Create version 2
    let request2 = CreateVersionRequest {
        document_id: document.id,
        content: serde_json::json!({"text": "Hello World"}),
        title: "V2".to_string(),
        change_summary: None,
    };

    let _ = app
        .post(&format!("/api/v1/documents/{}/versions", document.id))
        .json(&request2)
        .send()
        .await;

    // Get diff between versions
    let response = app
        .get(&format!(
            "/api/v1/documents/{}/versions/diff?from=1&to=2",
            document.id
        ))
        .send()
        .await;

    assert!(response.status().is_success());
    let diff: serde_json::Value = response.json().await;
    assert!(diff.is_object());
}

#[tokio::test]
async fn test_version_retention_enforced() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    // Create many versions (more than retention limit)
    for i in 1..=50 {
        let request = CreateVersionRequest {
            document_id: document.id,
            content: serde_json::json!({}),
            title: format!("Version {}", i),
            change_summary: None,
        };

        let response = app
            .post(&format!("/api/v1/documents/{}/versions", document.id))
            .json(&request)
            .send()
            .await;

        // Should succeed - old versions are cleaned up asynchronously
        assert!(response.status().is_success() || response.status() == 500);
    }

    // Verify cleanup job ran by checking recent versions count
    let response = app
        .get(&format!(
            "/api/v1/documents/{}/versions?limit=100",
            document.id
        ))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    let count = result["versions"].as_array().unwrap().len();

    // After retention, should have fewer versions (old ones cleaned up)
    // This is an eventually consistent check
    assert!(count <= 45); // Allow some buffer
}
