use crate::models::*;
use crate::helpers::*;
use actix_web::test;
use actix_web::web;
use uuid::Uuid;

#[actix_rt::test]
async fn test_list_spaces_empty() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let req = test::TestRequest::get()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    eprintln!("Status: {} (expected 200), Body: {}", status, String::from_utf8_lossy(&body));
    assert_eq!(status, 200);

    let spaces: Vec<Space> = serde_json::from_slice(&body).unwrap();
    assert!(spaces.is_empty());
}

#[actix_rt::test]
async fn test_get_spaces_unauthorized() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/spaces")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_create_space() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let create_req = CreateSpaceRequest {
        name: "My Workspace".to_string(),
        icon: Some("📚".to_string()),
        description: Some("Test description".to_string()),
        is_public: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    eprintln!("Status: {} (expected 201), Body: {}", status, String::from_utf8_lossy(&body));
    assert_eq!(status, 201);

    let space: Space = serde_json::from_slice(&body).unwrap();
    assert_eq!(space.name, "My Workspace");
    assert_eq!(space.owner_id.to_string(), test_user.id.to_string());
    assert!(!space.id.to_string().is_empty());
}

#[actix_rt::test]
async fn test_create_space_validation_empty_name() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let create_req = CreateSpaceRequest {
        name: "".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_create_space_validation_name_too_long() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let create_req = CreateSpaceRequest {
        name: "a".repeat(201),
        icon: None,
        description: None,
        is_public: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/spaces")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_rt::test]
async fn test_get_space() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

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

    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/spaces/{}", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, get_req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let retrieved_space: Space = serde_json::from_slice(&body).unwrap();
    assert_eq!(retrieved_space.id, space.id);
    assert_eq!(retrieved_space.name, "Test Space");
}

#[actix_rt::test]
async fn test_get_space_not_found() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let space_id = Uuid::new_v4().to_string();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/spaces/{}", space_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_rt::test]
async fn test_update_space() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let create_req = CreateSpaceRequest {
        name: "Original Name".to_string(),
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

    let update_req = UpdateSpaceRequest {
        name: Some("Updated Name".to_string()),
        icon: Some("🚀".to_string()),
        description: Some("Updated description".to_string()),
        is_public: Some(true),
    };

    let req = test::TestRequest::patch()
        .uri(&format!("/api/v1/spaces/{}", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let updated_space: Space = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_space.name, "Updated Name");
    assert_eq!(updated_space.icon, Some("🚀".to_string()));
    assert_eq!(updated_space.description, Some("Updated description".to_string()));
    assert!(updated_space.is_public);
}

#[actix_rt::test]
async fn test_delete_space() {
    let test_app = TestApp::create().await;
    let app = test::init_service(
        actix_web::App::new()
            .app_data(web::Data::new(test_app.pool.clone()))
            .configure(miniwiki_backend::routes::config)
    ).await;

    let test_user = test_app.create_test_user().await;
    let token = generate_test_jwt_token(test_user.id, &test_user.email);

    let create_req = CreateSpaceRequest {
        name: "Space to Delete".to_string(),
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

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/spaces/{}", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, delete_req).await;
    assert_eq!(resp.status(), 204);

    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/spaces/{}", space.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, get_req).await;
    assert_eq!(resp.status(), 404);
}
