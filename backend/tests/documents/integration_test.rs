//! Integration tests for Document Service
//!
//! Tests T095-T098: Document CRUD with PostgreSQL, Yjs state handling,
//! and complete document lifecycle flow.
//!
//! Run with: cargo test -p miniwiki-backend-tests documents::integration

use crate::helpers::{TestApp, TestUser, TestSpace, TestDocument};
use document_service::models::{CreateDocumentRequest, UpdateDocumentRequest};
use uuid::Uuid;

#[tokio::test]
async fn test_t095_document_crud_with_postgresql() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;

    let create_response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&serde_json::json!({
            "title": "Integration Test Document",
            "icon": "📄",
            "parent_id": null,
            "content": {"type": "Y.Doc", "update": "aGVsbG8gd29ybGQ="}
        }))
        .send()
        .await
        .expect("Create document request failed");

    assert!(create_response.status().is_success(), "Create should succeed");
    let create_data: serde_json::Value = create_response.json().await.expect("Parse create response");
    assert!(create_data["success"].as_bool().unwrap_or(false));
    let document_id = create_data["data"]["document"]["id"].as_str().unwrap().to_string();

    let read_response = app
        .get(&format!("/api/v1/documents/{}", document_id))
        .send()
        .await
        .expect("Read document request failed");

    assert!(read_response.status().is_success(), "Read should succeed");
    let read_data: serde_json::Value = read_response.json().await.expect("Parse read response");
    assert_eq!(read_data["data"]["title"], "Integration Test Document");

    let update_response = app
        .patch(&format!("/api/v1/documents/{}", document_id))
        .json(&serde_json::json!({
            "title": "Updated Integration Test Document",
            "content": {"type": "Y.Doc", "update": "updated_content"}
        }))
        .send()
        .await
        .expect("Update document request failed");

    assert!(update_response.status().is_success(), "Update should succeed");
    let update_data: serde_json::Value = update_response.json().await.expect("Parse update response");
    assert_eq!(update_data["data"]["title"], "Updated Integration Test Document");

    let list_response = app
        .get(&format!("/api/v1/spaces/{}/documents", space.id))
        .send()
        .await
        .expect("List documents request failed");

    assert!(list_response.status().is_success(), "List should succeed");
    let list_data: serde_json::Value = list_response.json().await.expect("Parse list response");
    let documents = list_data["data"]["documents"].as_array().unwrap();
    assert!(documents.len() >= 1);

    let delete_response = app
        .delete(&format!("/api/v1/documents/{}", document_id))
        .send()
        .await
        .expect("Delete document request failed");

    assert!(delete_response.status().is_success(), "Delete should succeed");

    let verify_response = app
        .get(&format!("/api/v1/documents/{}", document_id))
        .send()
        .await
        .expect("Verify document request failed");

    assert!(verify_response.status().is_success());
    let verify_data: serde_json::Value = verify_response.json().await.expect("Parse verify response");
    assert!(verify_data["data"]["is_archived"].as_bool().unwrap_or(false));
}

#[tokio::test]
async fn test_t096_yjs_crdt_state_storage_retrieval() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;

    let yjs_content = serde_json::json!({
        "type": "Y.Doc",
        "update": "YWN0dWFsIHlqcyB1cGRhdGUgZGF0YQ==",
        "vector_clock": {
            "client_id": user.id,
            "clock": 42
        }
    });

    let create_response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&serde_json::json!({
            "title": "Yjs CRDT Test Document",
            "content": yjs_content
        }))
        .send()
        .await
        .expect("Create document request failed");

    assert!(create_response.status().is_success());
    let create_data: serde_json::Value = create_response.json().await.expect("Parse create response");
    let document_id = create_data["data"]["document"]["id"].as_str().unwrap().to_string();

    let retrieve_response = app
        .get(&format!("/api/v1/documents/{}", document_id))
        .send()
        .await
        .expect("Retrieve document request failed");

    assert!(retrieve_response.status().is_success());
    let retrieve_data: serde_json::Value = retrieve_response.json().await.expect("Parse retrieve response");
    let retrieved_content = &retrieve_data["data"]["content"];

    assert_eq!(retrieved_content["type"], "Y.Doc");
    assert_eq!(retrieved_content["update"], "YWN0dWFsIHlqcyB1cGRhdGUgZGF0YQ==");

    let new_yjs_content = serde_json::json!({
        "type": "Y.Doc",
        "update": "bmV3IHlqcyB1cGRhdGUgZGF0YQ==",
        "vector_clock": {
            "client_id": user.id,
            "clock": 43
        }
    });

    let update_response = app
        .patch(&format!("/api/v1/documents/{}", document_id))
        .json(&serde_json::json!({
            "content": new_yjs_content
        }))
        .send()
        .await
        .expect("Update document request failed");

    assert!(update_response.status().is_success());
    let update_data: serde_json::Value = update_response.json().await.expect("Parse update response");
    let updated_content = &update_data["data"]["content"];

    assert_eq!(updated_content["type"], "Y.Doc");
    assert_eq!(updated_content["update"], "bmV3IHlqcyB1cGRhdGUgZGF0YQ==");
    assert_eq!(updated_content["vector_clock"]["clock"], 43);

    let final_response = app
        .get(&format!("/api/v1/documents/{}", document_id))
        .send()
        .await
        .expect("Get final document request failed");

    assert!(final_response.status().is_success());
    let final_data: serde_json::Value = final_response.json().await.expect("Parse final response");
    let content_size = final_data["data"]["content_size"].as_i64().unwrap_or(0);
    assert!(content_size > 0, "Content size should be calculated");
}

#[tokio::test]
async fn test_t097_document_lifecycle_flow() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;

    let create_response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&serde_json::json!({
            "title": "Lifecycle Test Document",
            "icon": "📝"
        }))
        .send()
        .await
        .expect("Create document request failed");

    assert!(create_response.status().is_success());
    let create_data: serde_json::Value = create_response.json().await.expect("Parse create response");
    let document_id = create_data["data"]["document"]["id"].as_str().unwrap().to_string();
    let created_at = create_data["data"]["document"]["created_at"].as_str().unwrap().to_string();

    for i in 1..=3 {
        let edit_response = app
            .patch(&format!("/api/v1/documents/{}", document_id))
            .json(&serde_json::json!({
                "title": format!("Lifecycle Test Document v{}", i),
                "content": {
                    "type": "Y.Doc",
                    "update": format!("ZXh0cmEgdXBkYXRlIHBhdGgg{}", i),
                    "vector_clock": {"client_id": user.id, "clock": i}
                }
            }))
            .send()
            .await
            .expect("Edit document request failed");

        assert!(edit_response.status().is_success(), "Edit {} should succeed", i);
    }

    let save_response = app
        .get(&format!("/api/v1/documents/{}", document_id))
        .send()
        .await
        .expect("Get document request failed");

    assert!(save_response.status().is_success());
    let save_data: serde_json::Value = save_response.json().await.expect("Parse save response");
    assert_eq!(save_data["data"]["title"], "Lifecycle Test Document v3");
    assert_eq!(save_data["data"]["last_edited_by"].as_str().unwrap(), user.id.to_string());

    let retrieve_response = app
        .get(&format!("/api/v1/documents/{}", document_id))
        .send()
        .await
        .expect("Retrieve document request failed");

    assert!(retrieve_response.status().is_success());
    let retrieve_data: serde_json::Value = retrieve_response.json().await.expect("Parse retrieve response");

    assert_eq!(retrieve_data["data"]["id"], document_id);
    assert_eq!(retrieve_data["data"]["created_at"], created_at);
    let updated_at = retrieve_data["data"]["updated_at"].as_str().unwrap().to_string();
    assert!(updated_at > created_at, "Updated time should be after created time");
    let content = &retrieve_data["data"]["content"];
    assert_eq!(content["vector_clock"]["clock"], 3);

    let child_response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&serde_json::json!({
            "title": "Child Document",
            "parent_id": document_id
        }))
        .send()
        .await
        .expect("Create child document request failed");

    assert!(child_response.status().is_success());
    let child_data: serde_json::Value = child_response.json().await.expect("Parse child response");
    let child_id = child_data["data"]["document"]["id"].as_str().unwrap().to_string();
    assert_eq!(child_data["data"]["document"]["parent_id"], document_id);

    let children_response = app
        .get(&format!("/api/v1/documents/{}/children", document_id))
        .send()
        .await
        .expect("Get children request failed");

    assert!(children_response.status().is_success());
    let children_data: serde_json::Value = children_response.json().await.expect("Parse children response");
    let children = children_data["data"]["documents"].as_array().unwrap();
    assert!(children.len() == 1);
    assert_eq!(children[0]["id"], child_id);
}

#[tokio::test]
async fn test_t098_flutter_editor_integration() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;

    let flutter_content = serde_json::json!({
        "type": "Y.Doc",
        "update": "Zmx1dHRlciBlZGl0b3IgY29udGVudA==",
        "vector_clock": {"client_id": user.id, "clock": 1}
    });

    let create_response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&serde_json::json!({
            "title": "Flutter Editor Integration Test",
            "icon": "🎨",
            "content": flutter_content
        }))
        .send()
        .await
        .expect("Create document request failed");

    assert!(create_response.status().is_success());
    let create_data: serde_json::Value = create_response.json().await.expect("Parse create response");
    let document_id = create_data["data"]["document"]["id"].as_str().unwrap().to_string();

    let document = &create_data["data"]["document"];
    assert!(document["id"].is_string());
    assert!(document["space_id"].is_string());
    assert!(document["title"].is_string());
    assert!(document["created_by"].is_string());
    assert!(document["last_edited_by"].is_string());
    assert!(document["created_at"].is_string());
    assert!(document["updated_at"].is_string());
    assert!(document["content"].is_object());
    assert!(document["content_size"].is_number());

    let rich_text_content = serde_json::json!({
        "type": "Y.Doc",
        "update": "cmljaCB0ZXh0IHdpdGggZmx1dHRlciBxdWlsbA==",
        "vector_clock": {"client_id": user.id, "clock": 2}
    });

    let update_response = app
        .patch(&format!("/api/v1/documents/{}", document_id))
        .json(&serde_json::json!({
            "title": "Updated from Flutter Quill",
            "content": rich_text_content
        }))
        .send()
        .await
        .expect("Update document request failed");

    assert!(update_response.status().is_success());
    let update_data: serde_json::Value = update_response.json().await.expect("Parse update response");

    let updated_document = &update_data["data"]["document"];
    assert_eq!(updated_document["title"], "Updated from Flutter Quill");
    assert_eq!(updated_document["content"]["type"], "Y.Doc");

    let list_response = app
        .get(&format!("/api/v1/spaces/{}/documents", space.id))
        .send()
        .await
        .expect("List documents request failed");

    assert!(list_response.status().is_success());
    let list_data: serde_json::Value = list_response.json().await.expect("Parse list response");

    assert!(list_data["data"]["documents"].is_array());
    assert!(list_data["data"]["total"].is_number());
    assert!(list_data["data"]["limit"].is_number());
    assert!(list_data["data"]["offset"].is_number());

    let children_response = app
        .get(&format!("/api/v1/documents/{}/children", document_id))
        .send()
        .await
        .expect("Get children request failed");

    assert!(children_response.status().is_success());
    let children_data: serde_json::Value = children_response.json().await.expect("Parse children response");
    assert!(children_data["data"]["documents"].is_array());
    assert!(children_data["data"]["total"].is_number());
}

#[tokio::test]
async fn test_document_hierarchy_integration() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;

    let parent_response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&serde_json::json!({
            "title": "Parent Document"
        }))
        .send()
        .await
        .expect("Create parent document request failed");

    assert!(parent_response.status().is_success());
    let parent_data: serde_json::Value = parent_response.json().await.expect("Parse parent response");
    let parent_id = parent_data["data"]["document"]["id"].as_str().unwrap().to_string();

    let child_response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&serde_json::json!({
            "title": "Child Document",
            "parent_id": parent_id
        }))
        .send()
        .await
        .expect("Create child document request failed");

    assert!(child_response.status().is_success());

    let grandchild_response = app
        .post(&format!("/api/v1/spaces/{}/documents", space.id))
        .json(&serde_json::json!({
            "title": "Grandchild Document",
            "parent_id": parent_id
        }))
        .send()
        .await
        .expect("Create grandchild document request failed");

    assert!(grandchild_response.status().is_success());

    let children_response = app
        .get(&format!("/api/v1/documents/{}/children", parent_id))
        .send()
        .await
        .expect("Get children request failed");

    assert!(children_response.status().is_success());
    let children_data: serde_json::Value = children_response.json().await.expect("Parse children response");
    let children = children_data["data"]["documents"].as_array().unwrap();
    assert_eq!(children.len(), 2);

    let grandchild_response_json: serde_json::Value = grandchild_response.json().await.expect("Parse grandchild response");
    let grandchild_id = grandchild_response_json["data"]["document"]["id"].as_str().unwrap();
    let path_response = app
        .get(&format!("/api/v1/documents/{}/path", grandchild_id))
        .send()
        .await
        .expect("Get document path request failed");

    assert!(path_response.status().is_success());
    let path_data: serde_json::Value = path_response.json().await.expect("Parse path response");
    let path = path_data["data"]["path"].as_array().unwrap();
    assert!(path.len() >= 1);
}

#[tokio::test]
async fn test_document_pagination_integration() {
    let app = TestApp::create().await;
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;

    for i in 0..15 {
        app.create_test_document(&space.id, None).await;
    }

    let page1_response = app
        .get(&format!("/api/v1/spaces/{}/documents?limit=5&offset=0", space.id))
        .send()
        .await
        .expect("Get page 1 request failed");

    assert!(page1_response.status().is_success());
    let page1_data: serde_json::Value = page1_response.json().await.expect("Parse page 1 response");
    let page1_docs = page1_data["data"]["documents"].as_array().unwrap();
    assert_eq!(page1_docs.len(), 5);
    assert_eq!(page1_data["data"]["total"], 15);
    assert_eq!(page1_data["data"]["limit"], 5);
    assert_eq!(page1_data["data"]["offset"], 0);

    let page2_response = app
        .get(&format!("/api/v1/spaces/{}/documents?limit=5&offset=5", space.id))
        .send()
        .await
        .expect("Get page 2 request failed");

    assert!(page2_response.status().is_success());
    let page2_data: serde_json::Value = page2_response.json().await.expect("Parse page 2 response");
    let page2_docs = page2_data["data"]["documents"].as_array().unwrap();
    assert_eq!(page2_docs.len(), 5);
    assert_eq!(page2_data["data"]["offset"], 5);

    let page3_response = app
        .get(&format!("/api/v1/spaces/{}/documents?limit=5&offset=10", space.id))
        .send()
        .await
        .expect("Get page 3 request failed");

    assert!(page3_response.status().is_success());
    let page3_data: serde_json::Value = page3_response.json().await.expect("Parse page 3 response");
    let page3_docs = page3_data["data"]["documents"].as_array().unwrap();
    assert_eq!(page3_docs.len(), 5);
    assert_eq!(page3_data["data"]["offset"], 10);

    let page4_response = app
        .get(&format!("/api/v1/spaces/{}/documents?limit=5&offset=20", space.id))
        .send()
        .await
        .expect("Get page 4 request failed");

    assert!(page4_response.status().is_success());
    let page4_data: serde_json::Value = page4_response.json().await.expect("Parse page 4 response");
    let page4_docs = page4_data["data"]["documents"].as_array().unwrap();
    assert_eq!(page4_docs.len(), 0);
}
