use crate::models::*;
use crate::helpers::*;
use actix_web::test;
use backend::services::auth_service::jwt::generate_jwt_token;
use uuid::Uuid;

mod helpers;

#[actix_rt::test]
async fn test_list_space_members() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Test Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/spaces/{}/members", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);
    
    let body = test::read_body(resp).await;
    let members: Vec<SpaceMembership> = serde_json::from_slice(&body).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].user_id, owner_id.to_string());
    assert_eq!(members[0].role, "owner");
}

#[actix_rt::test]
async fn test_add_space_member() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Test Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let add_req = AddMemberRequest {
        user_id: member_id.to_string(),
        role: "editor".to_string(),
    };
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/spaces/{}/members", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&add_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let membership: SpaceMembership = serde_json::from_slice(&body).unwrap();
    assert_eq!(membership.user_id, member_id.to_string());
    assert_eq!(membership.role, "editor");
}

#[actix_rt::test]
async fn test_add_member_validation_invalid_role() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Test Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let add_req = AddMemberRequest {
        user_id: member_id.to_string(),
        role: "invalid_role".to_string(),
    };
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/spaces/{}/members", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&add_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_update_member_role() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Test Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let add_req = AddMemberRequest {
        user_id: member_id.to_string(),
        role: "editor".to_string(),
    };
    
    let add_resp = test::TestRequest::post()
        .uri(&format!("/api/v1/spaces/{}/members", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&add_req)
        .to_request();
    let resp = test::call_service(&app, add_resp).await;
    assert_eq!(resp.status(), 201);
    
    let update_req = UpdateMemberRequest {
        role: "viewer".to_string(),
    };
    
    let req = test::TestRequest::patch()
        .uri(&format!("/api/v1/spaces/{}/members/{}", space.id, member_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    
    let body = test::read_body(resp).await;
    let membership: SpaceMembership = serde_json::from_slice(&body).unwrap();
    assert_eq!(membership.role, "viewer");
}

#[actix_rt::test]
async fn test_cannot_update_owner_role() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Test Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let update_req = UpdateMemberRequest {
        role: "viewer".to_string(),
    };
    
    let req = test::TestRequest::patch()
        .uri(&format!("/api/v1/spaces/{}/members/{}", space.id, owner_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_remove_member() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Test Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let add_req = AddMemberRequest {
        user_id: member_id.to_string(),
        role: "editor".to_string(),
    };
    
    let add_resp = test::TestRequest::post()
        .uri(&format!("/api/v1/spaces/{}/members", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&add_req)
        .to_request();
    let resp = test::call_service(&app, add_resp).await;
    assert_eq!(resp.status(), 201);
    
    let remove_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/spaces/{}/members/{}", space.id, member_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, remove_req).await;
    assert_eq!(resp.status(), 204);
    
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/spaces/{}/members", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);
    
    let body = test::read_body(resp).await;
    let members: Vec<SpaceMembership> = serde_json::from_slice(&body).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].user_id, owner_id.to_string());
}

#[actix_rt::test]
async fn test_cannot_remove_owner() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Test Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let remove_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/spaces/{}/members/{}", space.id, owner_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, remove_req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_non_member_cannot_access_space() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    let owner_token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    let other_token = generate_jwt_token(other_user_id, "other@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Private Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", owner_token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/spaces/{}", space.id))
        .insert_header(("Authorization", format!("Bearer {}", other_token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

#[actix_rt::test]
async fn test_public_space_accessible() {
    let app = test::init_service(test_app().await).await;
    
    let owner_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    let owner_token = generate_jwt_token(owner_id, "owner@example.com").unwrap();
    let other_token = generate_jwt_token(other_user_id, "other@example.com").unwrap();
    
    let create_req = CreateSpaceRequest {
        name: "Public Space".to_string(),
        icon: None,
        description: None,
        is_public: true,
    };
    
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", owner_token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let space: Space = serde_json::from_slice(&body).unwrap();
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/spaces/{}", space.id))
        .insert_header(("Authorization", format!("Bearer {}", other_token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}
