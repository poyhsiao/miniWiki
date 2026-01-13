pub mod handlers;
pub mod models;
pub mod yjs_handler;
pub mod state_vector;
pub mod sync_handler;
pub mod conflict_resolver;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    yjs_handler::config(cfg);
    sync_handler::config(cfg);
}
