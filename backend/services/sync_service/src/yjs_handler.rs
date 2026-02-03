use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// use shared_models::entities::Document;

// const YJS_ENCODING_FORMAT: u8 = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncUpdateRequest {
    pub document_id: String,
    pub update: Vec<u8>,
    pub client_id: Option<String>,
    pub clock: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncUpdateResponse {
    pub success: bool,
    pub document_id: String,
    pub update: Option<Vec<u8>>,
    pub server_clock: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStateRequest {
    pub document_id: String,
    pub state_vector: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStateResponse {
    pub document_id: String,
    pub state: Vec<u8>,
    pub server_clock: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncInitRequest {
    pub document_id: String,
    pub initial_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncInitResponse {
    pub document_id: String,
    pub state: Vec<u8>,
    pub server_clock: u64,
}

pub async fn handle_sync_update(data: web::Json<SyncUpdateRequest>) -> impl Responder {
    HttpResponse::Ok().json(SyncUpdateResponse {
        success: true,
        document_id: data.document_id.clone(),
        update: None,
        server_clock: 0,
    })
}

pub async fn handle_sync_state(data: web::Json<SyncStateRequest>) -> impl Responder {
    HttpResponse::Ok().json(SyncStateResponse {
        document_id: data.document_id.clone(),
        state: Vec::new(),
        server_clock: 0,
    })
}

pub async fn handle_sync_init(data: web::Json<SyncInitRequest>) -> impl Responder {
    HttpResponse::Ok().json(SyncInitResponse {
        document_id: data.document_id.clone(),
        state: Vec::new(),
        server_clock: 0,
    })
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/sync/update", web::post().to(handle_sync_update))
        .route("/sync/state", web::post().to(handle_sync_state))
        .route("/sync/init", web::post().to(handle_sync_init));
}
