//! Document CRUD operation tests
//!
//! Tests for document creation, reading, updating, and deletion operations.
//! These tests verify the document service handlers work correctly.
//!
//! Run with: cargo test -p miniwiki-backend-tests documents::crud_test

use actix_web::web;
use chrono::Utc;
use miniwiki_backend::services::document_service::{CreateDocumentRequest, UpdateDocumentRequest};
use miniwiki_backend::tests::helpers::{create_test_app, create_test_document, create_test_space, create_test_user, TestApp};
use shared_errors::error::ApiError;
use uuid::Uuid;

#[tokio::test]
async fn test_create_document_success() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;

    let request = CreateDocumentRequest {
        space_id: space.id,
        parent_id: None,
        title: "Test Document".to_string(),
        icon: Some("📝".to_string()),
        content: serde_json::json!({
            "type": "Y.Doc",
            "update": "dGVzdCB1cGRhdGU=",
            "vector_clock": {
                "client_id": user.id.to_string(),
                "clock": 1
            }
        }),
    };

    let response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&request)
        .send()
        .await;

    assert!(response.status().is_success());
    let document: serde_json::Value = response.json().await;
    assert_eq!(document["title"], "Test Document");
    assert_eq!(document["space_id"], space.id.to_string());
    assert!(document["id"].is_string());
}

#[tokio::test]
async fn test_create_document_with_parent() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let parent_doc = create_test_document(&app, &space.id, None).await;

    let request = CreateDocumentRequest {
        space_id: space.id,
        parent_id: Some(parent_doc.id),
        title: "Child Document".to_string(),
        icon: None,
        content: serde_json::json!({}),
    };

    let response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&request)
        .send()
        .await;

    assert!(response.status().is_success());
    let document: serde_json::Value = response.json().await;
    assert_eq!(document["parent_id"], parent_doc.id.to_string());
}

#[tokio::test]
async fn test_create_document_empty_title_fails() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;

    let request = CreateDocumentRequest {
        space_id: space.id,
        parent_id: None,
        title: "".to_string(),
        icon: None,
        content: serde_json::json!({}),
    };

    let response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&request)
        .send()
        .await;

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_get_document_success() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    let response = app
        .get(&format!("/api/v1/documents/{}", document.id))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    assert_eq!(result["id"], document.id.to_string());
    assert_eq!(result["title"], document.title);
}

#[tokio::test]
async fn test_get_document_not_found() {
    let app = create_test_app().await;
    let fake_id = Uuid::new_v4();

    let response = app
        .get(&format!("/api/v1/documents/{}", fake_id))
        .send()
        .await;

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_update_document_title() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    let request = UpdateDocumentRequest {
        title: Some("Updated Title".to_string()),
        icon: None,
        content: None,
    };

    let response = app
        .patch(&format!("/api/v1/documents/{}", document.id))
        .json(&request)
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    assert_eq!(result["title"], "Updated Title");
}

#[tokio::test]
async fn test_update_document_content() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    let new_content = serde_json::json!({
        "type": "Y.Doc",
        "update": "new_base64_content",
        "vector_clock": {
            "client_id": user.id.to_string(),
            "clock": 2
        }
    });

    let request = UpdateDocumentRequest {
        title: None,
        icon: None,
        content: Some(new_content),
    };

    let response = app
        .patch(&format!("/api/v1/documents/{}", document.id))
        .json(&request)
        .send()
        .await;

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_delete_document_soft_delete() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;
    let document = create_test_document(&app, &space.id, None).await;

    let response = app
        .delete(&format!("/api/v1/documents/{}", document.id))
        .send()
        .await;

    assert!(response.status().is_success());

    // Verify document is soft deleted (is_archived = true)
    let get_response = app
        .get(&format!("/api/v1/documents/{}", document.id))
        .send()
        .await;

    assert!(get_response.status().is_success());
    let result: serde_json::Value = get_response.json().await;
    assert_eq!(result["is_archived"], true);
}

#[tokio::test]
async fn test_list_documents_in_space() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;

    // Create multiple documents
    create_test_document(&app, &space.id, None).await;
    create_test_document(&app, &space.id, None).await;
    create_test_document(&app, &space.id, None).await;

    let response = app
        .get(&format!("/api/v1/spaces/{}/documents", space.id))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    assert!(result["documents"].is_array());
    assert_eq!(result["documents"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_list_documents_with_pagination() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;

    // Create documents
    for i in 0..10 {
        create_test_document(&app, &space.id, None).await;
    }

    let response = app
        .get(&format!("/api/v1/spaces/{}/documents?limit=5&offset=0", space.id))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    assert_eq!(result["documents"].as_array().unwrap().len(), 5);
    assert_eq!(result["total"], 10);
    assert_eq!(result["limit"], 5);
    assert_eq!(result["offset"], 0);
}

#[tokio::test]
async fn test_document_hierarchy_nested() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;
    let space = create_test_space(&app, &user.id).await;

    // Create nested hierarchy: parent -> child -> grandchild
    let parent = create_test_document(&app, &space.id, None).await;
    let child = create_test_document(&app, &space.id, Some(parent.id)).await;
    let _grandchild = create_test_document(&app, &space.id, Some(child.id)).await;

    // Get children of parent
    let response = app
        .get(&format!("/api/v1/documents/{}/children", parent.id))
        .send()
        .await;

    assert!(response.status().is_success());
    let result: serde_json::Value = response.json().await;
    assert_eq!(result["documents"].as_array().unwrap().len(), 1);
}
