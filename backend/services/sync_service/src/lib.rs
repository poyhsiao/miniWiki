pub mod conflict_resolver;
pub mod state_vector;
pub mod sync_handler;
pub mod yjs_handler;

pub use conflict_resolver::{
    Awareness, ConflictResolution, ConflictResolutionStrategy, ConflictResolver, ConnectionState, MergeResult,
    SyncError,
};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    yjs_handler::config(cfg);
    sync_handler::config(cfg);
}
