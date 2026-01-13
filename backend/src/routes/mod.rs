use actix_web::web;
use std::sync::Arc;

const DEFAULT_JWT_SECRET: &str = "test-secret-key-for-testing-only-do-not-use-in-production";

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(|| async {
        actix_web::web::Json(serde_json::json!({
            "status": "healthy",
            "service": "miniwiki-api",
            "version": "0.1.0"
        }))
    }));
    
    cfg.app_data(web::Data::new(Arc::new(DEFAULT_JWT_SECRET.to_string())));
    
    cfg.service(
        web::scope("/api/v1")
            .configure(space_service::config)
            .configure(document_service::config)
    );
    
    cfg.configure(websocket_service::config);
}
