pub mod models;
pub mod handlers;
pub mod repository;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/spaces")
            .route("", web::get().to(handlers::list_spaces))
            .route("", web::post().to(handlers::create_space))
            .route("/{id}", web::get().to(handlers::get_space))
            .route("/{id}", web::patch().to(handlers::update_space))
            .route("/{id}", web::delete().to(handlers::delete_space))
            .route("/{id}/members", web::get().to(handlers::list_space_members))
            .route("/{id}/members", web::post().to(handlers::add_space_member))
            .route("/{id}/members/{member_id}", web::patch().to(handlers::update_member_role))
            .route("/{id}/members/{member_id}", web::delete().to(handlers::remove_member))
    );
}
