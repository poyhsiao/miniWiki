pub mod handlers;
pub mod models;
pub mod repository;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/documents")
            .route("", actix_web::web::get().to(handlers::list_documents))
            .route("", actix_web::web::post().to(handlers::create_document))
    );
}
