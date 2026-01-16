pub mod handlers;
pub mod models;
pub mod storage;

/// Configure file service routes
/// Pool and storage will be extracted by handlers from app_data
pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    use crate::handlers::*;

    cfg.service(
        actix_web::web::scope("/files")
            // Upload endpoints
            .route("/upload", actix_web::web::post().to(upload_file))
            .route("/upload/chunked/init", actix_web::web::post().to(init_chunked_upload))
            .route("/upload/chunked/{upload_id}", actix_web::web::put().to(upload_chunk))
            .route("/upload/chunked/{upload_id}", actix_web::web::post().to(complete_chunked_upload))
            .route("/upload/chunked/{upload_id}", actix_web::web::delete().to(cancel_chunked_upload))
            .route("/upload/presigned-url", actix_web::web::post().to(get_presigned_upload_url))

            // Download endpoints
            .route("/{file_id}/download", actix_web::web::get().to(download_file))
            .route("/{file_id}/download/presigned-url", actix_web::web::get().to(get_presigned_download_url))

            // Management endpoints
            .route("/{file_id}", actix_web::web::get().to(get_file_metadata))
            .route("/{file_id}", actix_web::web::delete().to(delete_file))
            .route("/{file_id}/restore", actix_web::web::post().to(restore_file))
            .route("/{file_id}/permanent-delete", actix_web::web::delete().to(permanent_delete_file))

            // Space files
            .route("/spaces/{space_id}/files", actix_web::web::get().to(list_space_files))

            // Bulk operations
            .route("/bulk/delete", actix_web::web::post().to(bulk_delete_files))
    );
}
