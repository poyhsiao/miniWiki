use actix_web::{web, App, HttpServer, Responder};
use dotenv::dotenv;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod routes;
mod config;
mod observability;
mod middleware;

use config::Config;
use shared_database::connection::init_database;
use file_service::storage::{S3Storage, S3StorageConfig};
use observability::RequestMetrics;
use middleware::SecurityHeaders;

/// Initialize structured logging with JSON formatting for production
fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,actix_web=info,sqlx=warn"));

    // In production, use JSON logging; in development, use pretty printing
    if std::env::var("RUST_LOG_JSON").is_ok() {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer()
                .json()
                .with_thread_names(true)
                .with_span_list(true))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer())
            .init();
    }

    info!("Structured logging initialized");
}

/// Create a detailed health check response including dependency status
async fn health_check(
    db: &web::Data<sqlx::PgPool>,
    metrics: &web::Data<Arc<RequestMetrics>>,
) -> impl Responder {
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_optional(db.as_ref())
        .await
        .is_ok();

    let metrics_snapshot = metrics.get_ref().snapshot();

    let status = if db_healthy { "healthy" } else { "degraded" };

    actix_web::web::Json(serde_json::json!({
        "status": status,
        "service": "miniwiki-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "dependencies": {
            "database": {
                "status": if db_healthy { "healthy" } else { "unhealthy" },
                "type": "postgresql"
            }
        },
        "metrics": {
            "total_requests": metrics_snapshot.total_requests,
            "successful_requests": metrics_snapshot.successful_requests,
            "failed_requests": metrics_snapshot.failed_requests,
            "avg_latency_ms": format!("{:.2}", metrics_snapshot.avg_latency_ms()),
            "error_rate_percent": format!("{:.2}", metrics_snapshot.error_rate_percent())
        }
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize structured logging first for proper startup logging
    init_logging();

    dotenv().ok();

    let config = Config::from_env().expect("Failed to load configuration");

    let host = config.host.clone();
    let port = config.port;

    info!(host = %host, port = %port, "Starting miniWiki API server");

    let db = Arc::new(
        init_database(&config.database_url)
            .await
            .expect("Failed to connect to database")
    );

    info!("Database connection established");

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

    info!("S3/MinIO storage initialized");

    let jwt_secret = Arc::new(config.jwt_secret.clone());

    // Initialize request metrics
    let metrics = Arc::new(RequestMetrics::new());

    info!("Request metrics initialized");

    HttpServer::new(move || {
        App::new()
            .wrap(SecurityHeaders)
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(s3_storage.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            .app_data(web::Data::new(metrics.clone()))
            .configure(routes::config)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
