use actix_web::web;
use actix_web::App;
use backend::services::auth_service::jwt::JwtConfig;
use backend::shared::database::db::DbPool;
use backend::shared::errors::AppError;
use sqlx::postgres::PgPoolOptions;
use std::sync::Mutex;

pub async fn test_app() -> impl actix_web::dev::AppEntry {
    let pool = create_test_pool().await;
    
    let jwt_config = JwtConfig {
        secret: "test-secret-key-for-testing-only".to_string(),
        access_token_expiry: 900,
        refresh_token_expiry: 604800,
    };
    
    let app_data = web::Data::new(pool);
    let jwt_data = web::Data::new(jwt_config);
    
    App::new()
        .app_data(app_data)
        .app_data(jwt_data)
        .wrap(actix_web::middleware::Logger::default())
        .configure(backend::services::auth_service::config)
        .configure(backend::services::document_service::config)
}

async fn create_test_pool() -> DbPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://miniwiki:miniwiki@localhost:5432/miniwiki_test".to_string());
    
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create test pool")
}
