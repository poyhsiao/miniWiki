pub mod handlers;
pub mod models;
pub mod yjs_handler;
pub mod state_vector;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    yjs_handler::config(cfg);
}
