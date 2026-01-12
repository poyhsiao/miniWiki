use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(|| async {
        actix_web::web::Json(serde_json::json!({
            "status": "healthy",
            "service": "miniwiki-api",
            "version": "0.1.0"
        }))
    }));
    
    // Document routes
    cfg.service(document_service::config);
    
    // Auth routes will be mounted here
    // cfg.service(web::scope("/auth").configure(auth_routes::config));
    
    // Space routes will be mounted here
    // cfg.service(web::scope("/spaces").configure(space_routes::config));
    
    // Sync routes will be mounted here
    // cfg.service(web::scope("/sync").configure(sync_routes::config));
}
