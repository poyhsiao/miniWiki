pub mod handlers;
pub mod models;
pub mod repository;
pub mod validation;

use actix_web::web;
use crate::handlers::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    // Space endpoints
    cfg.service(
        web::scope("/api/v1/spaces")
            .route("", web::get().to(list_spaces))
            .route("", web::post().to(create_space))
            .route("/{spaceId}", web::get().to(get_space))
            .route("/{spaceId}", web::patch().to(update_space))
            .route("/{spaceId}", web::delete().to(delete_space))
            .route("/{spaceId}/members", web::get().to(list_space_members))
            .route("/{spaceId}/members", web::post().to(add_space_member))
            .route("/{spaceId}/members/{userId}", web::patch().to(update_space_member))
            .route("/{spaceId}/members/{userId}", web::delete().to(remove_space_member))
    );

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
