pub mod handlers;
pub mod models;
pub mod repository;
pub mod indexer;

use actix_web::web;
use crate::handlers::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/search")
            .route("", web::get().to(search_documents))
    );
}
