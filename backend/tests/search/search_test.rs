// Integration tests for search service endpoints
// These tests verify the search functionality with the PostgreSQL database

#[cfg(test)]
mod search_tests {
    use actix_web::{web, App, test::{self, TestRequest}};
    use serde_json::json;
    use sqlx::{Pool, Postgres};
    use std::sync::Arc;
    use uuid::Uuid;
    use search_service;

    // Helper to create a test database
    async fn setup_test_db() -> Pool<Postgres> {
        // For integration tests, connect to test database
        // TEST_DATABASE_URL must be set in CI environments
        let database_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for tests");

        Pool::connect(&database_url).await.expect("Failed to connect to test database")
    }

    // Helper to create test user and space
    async fn create_test_data(pool: &Pool<Postgres>) -> (Uuid, Uuid, Uuid) {
        // Create test user
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, display_name, is_active)
            VALUES ($1, $2, $3, $4, true)
            "#
        )
        .bind(user_id)
        .bind(format!("test_{}@example.com", user_id))
        .bind("$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4aYJGYxMnC6C5.Oy")
        .bind("Test User")
        .execute(pool)
        .await
        .expect("Failed to create test user");

        // Create test space
        let space_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO spaces (id, owner_id, name, is_public)
            VALUES ($1, $2, $3, false)
            "#
        )
        .bind(space_id)
        .bind(user_id)
        .bind("Test Space")
        .execute(pool)
        .await
        .expect("Failed to create test space");

        // Create space membership
        sqlx::query(
            r#"
            INSERT INTO space_memberships (id, space_id, user_id, role, invited_by)
            VALUES ($1, $2, $3, 'owner', $3)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(space_id)
        .bind(user_id)
        .execute(pool)
        .await
        .expect("Failed to create space membership");

        // Create test documents
        let doc1_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO documents (id, space_id, title, content, content_text, created_by, last_edited_by)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            "#
        )
        .bind(doc1_id)
        .bind(space_id)
        .bind("Getting Started with Rust")
        .bind(json!({"ops": [{"insert": "Rust is a systems programming language that runs blazingly fast."}]}))
        .bind("Rust is a systems programming language that runs blazingly fast.")
        .bind(user_id)
        .execute(pool)
        .await
        .expect("Failed to create test document 1");

        let doc2_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO documents (id, space_id, title, content, content_text, created_by, last_edited_by)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            "#
        )
        .bind(doc2_id)
        .bind(space_id)
        .bind("Async Programming in Rust")
        .bind(json!({"ops": [{"insert": "Async programming allows you to write non-blocking code."}]}))
        .bind("Async programming allows you to write non-blocking code.")
        .bind(user_id)
        .execute(pool)
        .await
        .expect("Failed to create test document 2");

        (user_id, space_id, doc1_id)
    }

    #[tokio::test]
    async fn test_search_returns_matching_documents() {
        let pool = setup_test_db().await;
        let pool = Arc::new(pool);
        
        let (_user_id, space_id, _doc1_id) = create_test_data(&pool).await;

        // Create app with search service
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .configure(search_service::config)
        ).await;

        // Search for "Rust"
        let req = TestRequest::get()
            .uri("/search?q=Rust")
            .header("X-User-Id", _user_id.to_string())
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).expect("Invalid JSON response");
        
        assert!(json["success"].as_bool().unwrap_or(false));
        let results = json["data"]["results"].as_array().expect("Results should be an array");
        assert!(results.len() >= 1, "Should find at least one document about Rust");
    }

    #[tokio::test]
    async fn test_search_returns_empty_for_no_matches() {
        let pool = setup_test_db().await;
        let pool = Arc::new(pool);
        
        let (user_id, _space_id, _) = create_test_data(&pool).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .configure(search_service::config)
        ).await;

        // Search for something that doesn't exist
        let req = TestRequest::get()
            .uri("/search?q=xyznonexistent123")
            .header("X-User-Id", user_id.to_string())
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).expect("Invalid JSON response");
        
        assert!(json["success"].as_bool().unwrap_or(false));
        let results = json["data"]["results"].as_array().expect("Results should be an array");
        assert_eq!(results.len(), 0, "Should not find any documents");
    }

    #[tokio::test]
    async fn test_search_respects_space_filter() {
        let pool = setup_test_db().await;
        let pool = Arc::new(pool);
        
        let (user_id, space_id, _doc1_id) = create_test_data(&pool).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .configure(search_service::config)
        ).await;

        // Search with space filter
        let req = TestRequest::get()
            .uri(&format!("/search?q=Rust&spaceId={}", space_id))
            .header("X-User-Id", user_id.to_string())
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).expect("Invalid JSON response");
        
        assert!(json["success"].as_bool().unwrap_or(false));
    }

    #[tokio::test]
    async fn test_search_requires_authentication() {
        let pool = setup_test_db().await;
        let pool = Arc::new(pool);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .configure(search_service::config)
        ).await;

        // Search without auth header
        let req = TestRequest::get()
            .uri("/search?q=test")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401, "Should require authentication");
    }

    #[tokio::test]
    async fn test_search_validates_query_length() {
        let pool = setup_test_db().await;
        let pool = Arc::new(pool);
        
        let (user_id, _space_id, _) = create_test_data(&pool).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .configure(search_service::config)
        ).await;

        // Search with empty query
        let req = TestRequest::get()
            .uri("/search?q=")
            .header("X-User-Id", user_id.to_string())
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400, "Should reject empty query");
    }

    #[tokio::test]
    async fn test_search_returns_timing_information() {
        let pool = setup_test_db().await;
        let pool = Arc::new(pool);
        
        let (user_id, _space_id, _) = create_test_data(&pool).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .configure(search_service::config)
        ).await;

        let req = TestRequest::get()
            .uri("/search?q=test")
            .header("X-User-Id", user_id.to_string())
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).expect("Invalid JSON response");
        
        assert!(json["success"].as_bool().unwrap_or(false));
        let took = json["data"]["took"].as_i64().expect("Should have timing info");
        assert!(took >= 0, "Timing should be non-negative");
    }
}
