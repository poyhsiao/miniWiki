pub mod export;
pub mod comments;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod validation;
pub mod sharing;

use actix_web::web;
use crate::handlers::*;
use crate::comments::*;
use crate::sharing::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Document-scoped endpoints
    cfg.service(
        web::scope("/documents")
            .route("/{documentId}", web::get().to(get_document))
            .route("/{documentId}", web::patch().to(update_document))
            .route("/{documentId}", web::delete().to(delete_document))
            .route("/{documentId}/children", web::get().to(get_document_children))
            .route("/{documentId}/path", web::get().to(get_document_path))
            // Export endpoint
            .route("/{documentId}/export", web::get().to(export_document))
            // Version endpoints
            .route("/{documentId}/versions", web::post().to(create_version))
            .route("/{documentId}/versions", web::get().to(list_versions))
            .route("/{documentId}/versions/{versionNumber}", web::get().to(get_version))
            .route("/{documentId}/versions/{versionNumber}/restore", web::post().to(restore_version))
            .route("/{documentId}/versions/diff", web::get().to(get_version_diff))
            // Comment endpoints
            .route("/{documentId}/comments", web::get().to(list_comments))
            .route("/{documentId}/comments", web::post().to(create_comment))
    );

    // Comment-scoped endpoints
    cfg.service(
        web::scope("/comments")
            .route("/{commentId}", web::patch().to(update_comment))
            .route("/{commentId}/resolve", web::post().to(resolve_comment))
            .route("/{commentId}/unresolve", web::post().to(unresolve_comment))
            .route("/{commentId}", web::delete().to(delete_comment))
    );

    // Share link endpoints
    cfg.service(
        web::scope("/documents/{documentId}/share")
            .route("", web::post().to(create_share_link))
            .route("", web::get().to(get_document_share_links))
            .route("/{token}", web::delete().to(delete_share_link))
    );
}
