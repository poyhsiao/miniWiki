//! Integration tests for User Story 2 - Document Organization
//!
//! These tests verify the complete flow of space creation, member invitation,
//! and document organization with PostgreSQL integration.
//!
//! Tests: T126, T127, T128

use crate::helpers::generate_test_jwt_token;

use crate::helpers::TestApp;

/// T126: Verify space CRUD endpoints work with PostgreSQL
///
/// This integration test verifies that all space CRUD operations
/// work correctly with the actual PostgreSQL database.
#[tokio::test]
async fn test_space_crud_with_postgres() {
    let app = TestApp::create().await;

    // Create a test user
    let user = app.create_test_user().await;
    let token = generate_test_jwt_token(user.id, &user.email);

    // Test CREATE space
    let create_req = serde_json::json!({
        "name": "Integration Test Space",
        "icon": "📁",
        "description": "Space for integration testing",
        "is_public": false
    });

    let resp = app.post("/api/v1/spaces")
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_req)
        .send()
        .await
        .expect("Create space request failed");

    assert_eq!(resp.status(), 201, "Space creation should return 201");
    let created: serde_json::Value = resp.json().await.expect("Parse create response");
    let space_id = created["id"].as_str().expect("Space ID should be string");

    // Test READ space
    let resp = app.get(&format!("/api/v1/spaces/{}", space_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Get space request failed");

    assert_eq!(resp.status(), 200, "Get space should return 200");
    let retrieved: serde_json::Value = resp.json().await.expect("Parse get response");
    assert_eq!(retrieved["name"], "Integration Test Space");

    // Test UPDATE space
    let update_req = serde_json::json!({
        "name": "Updated Integration Test Space",
        "icon": "📂"
    });

    let resp = app.patch(&format!("/api/v1/spaces/{}", space_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_req)
        .send()
        .await
        .expect("Update space request failed");

    assert_eq!(resp.status(), 200, "Update space should return 200");
    let updated: serde_json::Value = resp.json().await.expect("Parse update response");
    assert_eq!(updated["name"], "Updated Integration Test Space");

    // Test LIST spaces
    let resp = app.get("/api/v1/spaces")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("List spaces request failed");

    assert_eq!(resp.status(), 200, "List spaces should return 200");
    let spaces: Vec<serde_json::Value> = resp.json().await.expect("Parse list response");
    assert!(spaces.iter().any(|s| s["id"] == space_id), "Created space should appear in list");

    // Test DELETE space
    let resp = app.delete(&format!("/api/v1/spaces/{}", space_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Delete space request failed");

    assert_eq!(resp.status(), 204, "Delete space should return 204");

    // Verify space is soft-deleted (not found)
    let resp = app.get(&format!("/api/v1/spaces/{}", space_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Get deleted space request failed");

    assert_eq!(resp.status(), 404, "Deleted space should return 404");

    // Cleanup
    app.cleanup_test_user(&user.id).await;
}

/// T127: Verify hierarchical document queries work correctly
///
/// This integration test verifies that hierarchical document queries
/// (parent-child relationships) work correctly with PostgreSQL.
#[tokio::test]
async fn test_hierarchical_document_queries() {
    let app = TestApp::create().await;

    // Create a test user and space
    let user = app.create_test_user().await;
    let space = app.create_test_space_for_user(&user.id).await;
    let token = generate_test_jwt_token(user.id, &user.email);

    // Create parent document
    let parent_doc_req = serde_json::json!({
        "title": "Parent Document",
        "icon": "📘",
        "content": {"type": "Y.Doc", "update": "dGVzdCB1cGRhdGU=", "vector_clock": {"client_id": "parent", "clock": 1}}
    });

    let resp = app.post(&format!("/api/v1/space-docs/{}/documents", space.id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&parent_doc_req)
        .send()
        .await
        .expect("Create parent document failed");

    assert_eq!(resp.status(), 201, "Parent document creation should return 201");
    let parent: serde_json::Value = resp.json().await.expect("Parse parent response");
    let parent_id = parent["data"]["document"]["id"].as_str().expect("Parent ID should be string");

    // Create child document under parent
    let child_doc_req = serde_json::json!({
        "title": "Child Document",
        "icon": "📄",
        "parent_id": parent_id,
        "content": {"type": "Y.Doc", "update": "dGVzdCB1cGRhdGU=", "vector_clock": {"client_id": "child", "clock": 1}}
    });

    let resp = app.post(&format!("/api/v1/space-docs/{}/documents", space.id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&child_doc_req)
        .send()
        .await
        .expect("Create child document failed");

    assert_eq!(resp.status(), 201, "Child document creation should return 201");
    let child: serde_json::Value = resp.json().await.expect("Parse child response");
    let child_id = child["data"]["document"]["id"].as_str().expect("Child ID should be string");

    // Verify parent-child relationship
    let resp = app.get(&format!("/api/v1/documents/{}", child_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Get child document failed");

    assert_eq!(resp.status(), 200, "Get child should return 200");
    let child_retrieved: serde_json::Value = resp.json().await.expect("Parse child response");
    assert_eq!(child_retrieved["data"]["parent_id"], parent_id, "Child should have correct parent_id");

    // Create grandchild document
    let grandchild_doc_req = serde_json::json!({
        "title": "Grandchild Document",
        "icon": "📝",
        "parent_id": child_id,
        "content": {"type": "Y.Doc", "update": "dGVzdCB1cGRhdGU=", "vector_clock": {"client_id": "grandchild", "clock": 1}}
    });

    let resp = app.post(&format!("/api/v1/space-docs/{}/documents", space.id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&grandchild_doc_req)
        .send()
        .await
        .expect("Create grandchild document failed");

    assert_eq!(resp.status(), 201, "Grandchild document creation should return 201");

    // List documents in space (should include all)
    let resp = app.get(&format!("/api/v1/space-docs/{}/documents", space.id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("List documents failed");

    assert_eq!(resp.status(), 200, "List documents should return 200");
    let response: serde_json::Value = resp.json().await.expect("Parse documents list");
    let documents = response["data"]["documents"].as_array().expect("Parse documents list");
    assert_eq!(documents.len(), 3, "Should have 3 documents (parent, child, grandchild)");

    // Cleanup
    app.cleanup_test_user(&user.id).await;
}

/// T128: Test space creation → member invitation → document organization flow
///
/// This integration test verifies the complete user story flow:
/// 1. Create a space
/// 2. Invite members with different roles
/// 3. Organize documents with proper permissions
#[tokio::test]
async fn test_complete_space_member_document_flow() {
    let app = TestApp::create().await;

    // Create space owner
    let owner = app.create_test_user().await;
    let owner_token = generate_test_jwt_token(owner.id, &owner.email);

    // Create another user to be invited
    let member_user = app.create_test_user().await;

    // Step 1: Create a space
    let create_space_req = serde_json::json!({
        "name": "Team Workspace",
        "icon": "👥",
        "description": "A workspace for team collaboration",
        "is_public": false
    });

    let resp = app.post("/api/v1/spaces")
        .header("Authorization", format!("Bearer {}", owner_token))
        .json(&create_space_req)
        .send()
        .await
        .expect("Create space failed");

    assert_eq!(resp.status(), 201, "Space creation should return 201");
    let space: serde_json::Value = resp.json().await.expect("Parse space response");
    let space_id = space["id"].as_str().expect("Space ID should be string");

    // Step 2: Invite member as editor
    let invite_req = serde_json::json!({
        "user_id": member_user.id.to_string(),
        "role": "editor"
    });

    let resp = app.post(&format!("/api/v1/spaces/{}/members", space_id))
        .header("Authorization", format!("Bearer {}", owner_token))
        .json(&invite_req)
        .send()
        .await
        .expect("Invite member failed");

    assert_eq!(resp.status(), 201, "Member invitation should return 201");
    let membership: serde_json::Value = resp.json().await.expect("Parse membership response");
    assert_eq!(membership["role"], "editor");

    // Verify member can list spaces
    let member_token = generate_test_jwt_token(member_user.id, &member_user.email);
    let resp = app.get("/api/v1/spaces")
        .header("Authorization", format!("Bearer {}", member_token))
        .send()
        .await
        .expect("Member list spaces failed");

    assert_eq!(resp.status(), 200, "Member should see their spaces");
    let spaces: Vec<serde_json::Value> = resp.json().await.expect("Parse spaces list");
    assert!(spaces.iter().any(|s| s["id"] == space_id), "Member should see the space");

    // Step 3: Member creates a document in the space
    let doc_req = serde_json::json!({
        "title": "Member's First Document",
        "icon": "📄",
        "content": {"type": "Y.Doc", "update": "dGVzdCB1cGRhdGU=", "vector_clock": {"client_id": "member", "clock": 1}}
    });

    let resp = app.post(&format!("/api/v1/space-docs/{}/documents", space_id))
        .header("Authorization", format!("Bearer {}", member_token))
        .json(&doc_req)
        .send()
        .await
        .expect("Member create document failed");

    assert_eq!(resp.status(), 201, "Editor should be able to create documents");
    let document: serde_json::Value = resp.json().await.expect("Parse document response");
    let _doc_id = document["data"]["document"]["id"].as_str().expect("Document ID should be string");

    // Owner updates member role to viewer
    let update_role_req = serde_json::json!({
        "role": "viewer"
    });

    let resp = app.patch(&format!("/api/v1/spaces/{}/members/{}", space_id, membership["id"].as_str().expect("Membership ID")))
        .header("Authorization", format!("Bearer {}", owner_token))
        .json(&update_role_req)
        .send()
        .await
        .expect("Update member role failed");

    assert_eq!(resp.status(), 200, "Role update should return 200");

    // Verify viewer cannot create documents
    let doc_req = serde_json::json!({
        "title": "Should Fail",
        "content": {"type": "Y.Doc"}
    });

    let resp = app.post(&format!("/api/v1/space-docs/{}/documents", space_id))
        .header("Authorization", format!("Bearer {}", member_token))
        .json(&doc_req)
        .send()
        .await
        .expect("Viewer create document request failed");

    assert_eq!(resp.status(), 403, "Viewer should not be able to create documents");

    // Owner can still create documents
    let doc_req = serde_json::json!({
        "title": "Owner's Document",
        "content": {"type": "Y.Doc", "update": "dGVzdCB1cGRhdGU=", "vector_clock": {"client_id": "owner", "clock": 1}}
    });

    let resp = app.post(&format!("/api/v1/space-docs/{}/documents", space_id))
        .header("Authorization", format!("Bearer {}", owner_token))
        .json(&doc_req)
        .send()
        .await
        .expect("Owner create document failed");

    assert_eq!(resp.status(), 201, "Owner should be able to create documents");

    // Cleanup
    app.cleanup_test_user(&member_user.id).await;
    app.cleanup_test_user(&owner.id).await;
}
