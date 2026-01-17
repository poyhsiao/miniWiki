use actix_web::web;
use std::sync::Arc;
use document_service::sharing::{get_share_link_by_token, verify_share_link_access_code};

const DEFAULT_JWT_SECRET: &str = "test-secret-key-for-testing-only-do-not-use-in-production";

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

    cfg.app_data(web::Data::new(Arc::new(DEFAULT_JWT_SECRET.to_string())));

    // file_service::config will use pool and storage from app_data (registered in main.rs)
    // Handlers will extract them directly via web::Data<T>
    cfg.service(
        web::scope("/api/v1")
            .configure(space_service::config)
            .configure(document_service::config)
            .configure(file_service::config)
    );

    cfg.configure(websocket_service::config);
}
