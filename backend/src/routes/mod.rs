use actix_web::web;
use document_service::sharing::{get_share_link_by_token, verify_share_link_access_code};
use auth_service::jwt::JwtService;

const DEFAULT_JWT_SECRET: &str = "test-secret-key-for-testing-only-do-not-use-in-production";

/// Get JWT secret from environment variable or fall back to test secret in dev/test mode
fn get_jwt_secret() -> String {
    // Try to get from environment first
    if let Ok(secret) = std::env::var("JWT_SECRET") {
        return secret;
    }

    // Only allow fallback to test secret in development/test mode
    #[cfg(any(debug_assertions, test))]
    {
        eprintln!("WARNING: Using default JWT secret. Set JWT_SECRET environment variable in production!");
        return DEFAULT_JWT_SECRET.to_string();
    }

    // In release mode without JWT_SECRET, panic to prevent insecure startup
    #[cfg(not(any(debug_assertions, test)))]
    {
        panic!("JWT_SECRET environment variable must be set in production mode!");
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(|| async {
        actix_web::web::Json(serde_json::json!({
            "status": "healthy",
            "service": "miniwiki-api",
            "version": "0.1.0"
        }))
    }));

    // Public share link endpoints (no auth required)
    cfg.service(
        web::scope("/share")
            .route("/{token}", web::get().to(get_share_link_by_token))
            .route("/{token}/verify", web::post().to(verify_share_link_access_code))
    );

    // Configure auth service with required data
    // The pool is already registered in main.rs, but we need to create JwtService and register it
    cfg.app_data(web::Data::new(JwtService::new(auth_service::jwt::JwtConfig {
        secret: get_jwt_secret(),
        access_expiry: 3600,
        refresh_expiry: 86400,
    })));

    // Register auth service routes (under /api/v1/auth)
    cfg.service(
        web::scope("/api/v1")
            // Auth endpoints first to ensure they're available
            .configure(auth_service::config)
            // Document endpoints
            .configure(document_service::configure)
            // Space endpoints
            .configure(space_service::config)
            // Space-scoped document endpoints as a separate scope with a different prefix
            // Using /space-docs instead of /spaces to avoid conflict with space_service's /spaces/{id}
            .service(
                web::scope("/space-docs")
                    .route("/{spaceId}/documents", web::post().to(document_service::handlers::create_document))
                    .route("/{spaceId}/documents", web::get().to(document_service::handlers::list_documents))
            )
            .configure(file_service::config)
            .configure(sync_service::config)
    );

    cfg.configure(websocket_service::config);
}
