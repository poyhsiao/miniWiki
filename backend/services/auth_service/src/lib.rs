pub mod email_verification;
pub mod handlers;
pub mod jwt;
pub mod models;
pub mod password;
pub mod password_reset;
pub mod permissions;
pub mod rbac;
pub mod repository;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    use crate::handlers::*;

    cfg.service(
        actix_web::web::scope("/auth")
            .route("/register", actix_web::web::post().to(register))
            .route("/login", actix_web::web::post().to(login))
            .route("/logout", actix_web::web::post().to(logout))
            .route("/refresh", actix_web::web::post().to(refresh))
            .route("/me", actix_web::web::get().to(me)),
    );
}
