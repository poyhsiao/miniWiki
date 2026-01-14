pub mod handlers;
pub mod models;
pub mod storage;

use actix_web::web;
use crate::storage::S3Storage;

/// Configure file service routes
pub fn config(cfg: &mut web::ServiceConfig, pool: web::Data<sqlx::PgPool>, storage: web::Data<std::sync::Arc<S3Storage>>) {
    use crate::handlers::*;
    
    cfg.app_data(pool);
    cfg.app_data(storage);
    
    cfg.service(
        web::scope("/files")
            // Upload endpoints
            .route("/upload", web::post().to(upload_file))
            .route("/upload/chunked/init", web::post().to(init_chunked_upload))
            .route("/upload/chunked/{upload_id}", web::put().to(upload_chunk))
            .route("/upload/chunked/{upload_id}", web::post().to(complete_chunked_upload))
            .route("/upload/chunked/{upload_id}", web::delete().to(cancel_chunked_upload))
            .route("/upload/presigned-url", web::post().to(get_presigned_upload_url))
            
            // Download endpoints
            .route("/{file_id}/download", web::get().to(download_file))
            .route("/{file_id}/download/presigned-url", web::get().to(get_presigned_download_url))
            
            // Management endpoints
            .route("/{file_id}", web::get().to(get_file_metadata))
            .route("/{file_id}", web::delete().to(delete_file))
            .route("/{file_id}/restore", web::post().to(restore_file))
            .route("/{file_id}/permanent-delete", web::delete().to(permanent_delete_file))
            
            // Space files
            .route("/spaces/{space_id}/files", web::get().to(list_space_files))
            
            // Bulk operations
            .route("/bulk/delete", web::post().to(bulk_delete_files))
    );
}
