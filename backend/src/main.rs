use actix_web::{web, App, HttpServer, Responder};
use dotenv::dotenv;
use std::sync::Arc;

mod routes;
mod config;

use config::Config;
use shared_database::connection::init_database;
use file_service::storage::{S3Storage, S3StorageConfig};

async fn health() -> impl Responder {
    actix_web::web::Json(serde_json::json!({
        "status": "healthy",
        "service": "miniwiki-api",
        "version": "0.1.0"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::from_env().expect("Failed to load configuration");
    
    let host = config.host.clone();
    let port = config.port;

    println!("Starting miniWiki API server at http://{}:{}", host, port);

    let db = Arc::new(
        init_database(&config.database_url)
            .await
            .expect("Failed to connect to database")
    );

    // Initialize S3/MinIO storage
    let s3_config = S3StorageConfig {
        endpoint: config.minio_endpoint.clone(),
        access_key: config.minio_access_key.clone(),
        secret_key: config.minio_secret_key.clone(),
        bucket: config.minio_bucket.clone(),
        region: config.minio_region.clone(),
        use_ssl: config.minio_use_ssl,
    };
    
    let s3_storage = Arc::new(
        S3Storage::new(s3_config)
            .await
            .expect("Failed to connect to S3/MinIO storage")
    );

    let jwt_secret = Arc::new(config.jwt_secret.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(s3_storage.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            .configure(routes::config)
            .route("/health", web::get().to(health))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
