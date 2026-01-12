pub mod handlers;
pub mod models;
pub mod repository;
pub mod validation;

use actix_web::web;
use crate::handlers::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    // Document CRUD endpoints
    cfg.service(
        web::scope("/api/v1")
            // Space-scoped document endpoints
            .service(
                web::scope("/spaces/{spaceId}")
                    .route("/documents", web::post().to(create_document))
                    .route("/documents", web::get().to(list_documents))
            )
            // Document-scoped endpoints
            .service(
                web::scope("/documents")
                    .route("/{documentId}", web::get().to(get_document))
                    .route("/{documentId}", web::patch().to(update_document))
                    .route("/{documentId}", web::delete().to(delete_document))
                    .route("/{documentId}/children", web::get().to(get_document_children))
                    .route("/{documentId}/path", web::get().to(get_document_path))
                    // Version endpoints
                    .route("/{documentId}/versions", web::post().to(create_version))
                    .route("/{documentId}/versions", web::get().to(list_versions))
                    .route("/{documentId}/versions/{versionNumber}", web::get().to(get_version))
                    .route("/{documentId}/versions/{versionNumber}/restore", web::post().to(restore_version))
                    .route("/{documentId}/versions/diff", web::get().to(get_version_diff))
            )
    );
}
