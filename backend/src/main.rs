use actix_web::{App, HttpServer, middleware as actix_middleware, web};
use actix_cors::Cors;
use dotenv::dotenv;
use tracing::{info, warn, error};
use std::sync::Arc;

// Use symbols from the library crate
use miniwiki_backend::{
    config::Config,
    middleware::{
        error_handler::ErrorHandler,
        security_headers::SecurityHeaders,
        csrf::{CsrfMiddleware, CsrfConfig, CsrfStore, InMemoryCsrfStore, RedisCsrfStore},
    },
    routes,
    observability::RequestMetrics,
};
use auth_service::repository::AuthRepository;
use tokio::sync::Mutex;
use sync_service::sync_handler::SyncAppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
            )
        )
        .init();

    info!("Starting miniWiki backend...");

    let config = Config::from_env().unwrap_or_else(|e| {
        error!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    let csrf_config = CsrfConfig {
        cookie_name: std::env::var("CSRF_COOKIE_NAME")
            .unwrap_or_else(|_| "csrf_token".to_string()),
        cookie_max_age: std::env::var("CSRF_COOKIE_MAX_AGE")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .unwrap_or(3600),
        header_name: std::env::var("CSRF_HEADER_NAME")
            .unwrap_or_else(|_| "X-CSRF-Token".to_string()),
        secure_cookie: std::env::var("CSRF_SECURE_COOKIE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(config.app_env != "development"),
    };


    // Initialize CSRF Store (Redis if configured, otherwise In-Memory)
    let csrf_store: Arc<dyn CsrfStore> = if !config.redis_url.is_empty() {
        match redis::Client::open(config.redis_url.as_str()) {
            Ok(client) => match client.get_multiplexed_async_connection().await {
                Ok(conn) => Arc::new(RedisCsrfStore::new(Arc::new(conn))),
                Err(e) => {
                    let error_msg = format!("Failed to connect to Redis for CSRF store: {}", e);
                    if config.csrf_strict_redis || config.app_env != "development" {
                        tracing::error!("{}", error_msg);
                        std::process::exit(1);
                    } else {
                        warn!("{}. Falling back to in-memory for development.", error_msg);
                        Arc::new(InMemoryCsrfStore::new())
                    }
                }
            },
            Err(e) => {
                let error_msg = format!("Failed to open Redis client for CSRF store: {}", e);
                if config.csrf_strict_redis || config.app_env != "development" {
                    tracing::error!("{}", error_msg);
                    std::process::exit(1);
                } else {
                    warn!("{}. Falling back to in-memory for development.", error_msg);
                    Arc::new(InMemoryCsrfStore::new())
                }
            }
        }
    } else {
        info!("Redis URL not configured, using in-memory CSRF store.");
        Arc::new(InMemoryCsrfStore::new())
    };

    // Spawn background cleanup task for CSRF store
    // This is especially important for InMemoryCsrfStore which doesn't have auto-expiry like Redis
    let store_for_cleanup = csrf_store.clone();
    tokio::spawn(async move {
        // Run cleanup every hour
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            tracing::debug!("Running scheduled CSRF token cleanup");
            store_for_cleanup.cleanup_expired().await;
        }
    });

    let metrics = Arc::new(RequestMetrics::new());
    let pool = match config.create_pool().await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to create database pool: {}", e);
            std::process::exit(1);
        }
    };

    let port = config.port;

    let allow_all_origins = std::env::var("ALLOW_ALL_ORIGINS").unwrap_or_default() == "true";

    let server = HttpServer::new(move || {
        let cors_config = config.clone();
        let cors = Cors::default()
            .allowed_origin_fn(move |origin, _req_head| {
                let origin_str = origin.to_str().unwrap_or("");

                // Allow all ONLY in development AND if explicitly allowed via env var
                if cors_config.app_env == "development" && allow_all_origins {
                    return true;
                }

                // Check against configured allowlist
                cors_config.api_cors_origins.iter().any(|o| o == origin_str)
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::HeaderName::from_static("x-csrf-token"),
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(AuthRepository::new(pool.clone())))
            .app_data(web::Data::new(SyncAppState {
                pool: pool.clone(),
                server_clock: Arc::new(Mutex::new(0)),
            }))
            .app_data(web::Data::new(metrics.clone()))
            .app_data(web::Data::new(csrf_config.clone()))
            .app_data(web::Data::new(csrf_store.clone()))
            .wrap(actix_middleware::Logger::default())
            .wrap(ErrorHandler)
            .wrap(SecurityHeaders)
            .wrap(CsrfMiddleware::new(csrf_config.clone(), csrf_store.clone()))
            .wrap(cors)
            .configure(routes::config)
    })
    .bind(("0.0.0.0", port))?
    .run();

    info!("Server listening on http://0.0.0.0:{}", port);

    server.await
}
