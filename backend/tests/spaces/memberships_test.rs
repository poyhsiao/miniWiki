use crate::models::*;
use crate::helpers::*;
use actix_web::test;
use actix_web::web;
use auth_service::jwt::generate_jwt_token;

#[actix_rt::test]
async fn test_list_space_members() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;
    
    eprintln!("DEBUG: Creating test user...");
    let owner = test_app.create_test_user().await;
    eprintln!("DEBUG: Created test user: id={}, email={}", owner.id, owner.email);
    
    let token = generate_jwt_token(owner.id, &owner.email).unwrap();
    eprintln!("DEBUG: Generated token for user");
    
    let create_req = CreateSpaceRequest {
        name: "Test Space".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };
    
    eprintln!("DEBUG: Creating space...");
    let create_resp = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();
    let resp = test::call_service(&app, create_resp).await;
    let status = resp.status();
    eprintln!("DEBUG: Space creation response status: {}", status);
    
    let body = test::read_body(resp).await;
    
    if !status.is_success() {
        eprintln!("DEBUG: Error body: {}", String::from_utf8_lossy(&body));
    }
    assert_eq!(status, 201);
    
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
    assert_eq!(members[0].user_id, owner.id.to_string());
    assert_eq!(members[0].role, "owner");
}

#[actix_rt::test]
async fn test_add_space_member() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;
    
    let owner = test_app.create_test_user().await;
    let member = test_app.create_test_user().await;
    let token = generate_jwt_token(owner.id, &owner.email).unwrap();
    
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
        user_id: member.id.to_string(),
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
    assert_eq!(membership.user_id, member.id.to_string());
    assert_eq!(membership.role, "editor");
}

#[actix_rt::test]
async fn test_add_member_validation_invalid_role() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;
    
    let owner = test_app.create_test_user().await;
    let member = test_app.create_test_user().await;
    let token = generate_jwt_token(owner.id, &owner.email).unwrap();
    
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
        user_id: member.id.to_string(),
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
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;
    
    let owner = test_app.create_test_user().await;
    let member = test_app.create_test_user().await;
    let token = generate_jwt_token(owner.id, &owner.email).unwrap();
    
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
        user_id: member.id.to_string(),
        role: "editor".to_string(),
    };
    
    let add_resp = test::TestRequest::post()
        .uri(&format!("/api/v1/spaces/{}/members", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&add_req)
        .to_request();
    let resp = test::call_service(&app, add_resp).await;
    assert_eq!(resp.status(), 201);
    
    let body = test::read_body(resp).await;
    let membership: SpaceMembership = serde_json::from_slice(&body).unwrap();
    
    let update_req = UpdateMemberRequest {
        role: "viewer".to_string(),
    };
    
    let req = test::TestRequest::patch()
        .uri(&format!("/api/v1/spaces/{}/members/{}", space.id, membership.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_req)
        .to_request();
    
    eprintln!("DEBUG: Sending PATCH request to /api/v1/spaces/{}/members/{}", space.id, membership.id);
    let resp = test::call_service(&app, req).await;
    eprintln!("DEBUG: Response status: {}", resp.status());
    
    let status = resp.status();
    let body = test::read_body(resp).await;
    eprintln!("DEBUG: Response body: {}", String::from_utf8_lossy(&body));
    
    assert_eq!(status, 200);
    
    let updated: SpaceMembership = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated.role, "viewer");
}