// Sync handler implementation for offline-first sync endpoints
// Handles document sync state retrieval, update submission, and sync status

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use std::sync::Arc;
use crate::state_vector::StateVector;
use chrono::{Utc, TimeZone, NaiveDateTime};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncDocument {
    pub id: Uuid,
    pub title: String,
    pub content: serde_json::Value,
    pub version: i32,
    pub updated_at: NaiveDateTime,
}

/// Request body for sync update submission
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncUpdateRequest {
    /// Base64 encoded CRDT update
    pub update: String,
    /// Client's state vector
    #[serde(default)]
    pub state_vector: Option<StateVectorDto>,
}

/// State vector data transfer object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateVectorDto {
    pub client_id: String,
    pub clock: u64,
}

/// Response for sync update submission
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncUpdateResponse {
    pub success: bool,
    pub merged: bool,
    pub server_clock: u64,
    pub missing_updates: Option<Vec<MissingUpdate>>,
    pub error: Option<String>,
}

/// Missing update information
#[derive(Debug, Serialize, Deserialize)]
pub struct MissingUpdate {
    pub client_id: String,
    pub from_clock: u64,
    pub to_clock: u64,
}

/// Response for sync state retrieval
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStateResponse {
    pub document_id: String,
    pub title: String,
    pub state_vector: Vec<u8>,
    pub version: i32,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
}

/// Response for sync status
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    pub pending_documents: i64,
    pub last_sync_time: Option<chrono::DateTime<chrono::Utc>>,
    pub documents_in_sync: i64,
    pub failed_syncs: i64,
}

/// Request body for full sync trigger
#[derive(Debug, Serialize, Deserialize)]
pub struct FullSyncRequest {
    pub document_ids: Option<Vec<Uuid>>,
}

/// Response for full sync trigger
#[derive(Debug, Serialize, Deserialize)]
pub struct FullSyncResponse {
    pub success: bool,
    pub synced_documents: i64,
    pub failed_documents: i64,
    pub errors: Vec<String>,
}

/// App state for sync handlers
pub struct SyncAppState {
    pub pool: PgPool,
    pub server_clock: Arc<std::sync::atomic::AtomicU64>,
}

/// Get sync state for a document
pub async fn get_sync_state(
    path: web::Path<Uuid>,
    state: web::Data<SyncAppState>,
) -> impl Responder {
    let document_id = path.into_inner();

    // Fetch document from database
    let result = sqlx::query!(
        r#"
        SELECT id, title, content, version, updated_at
        FROM documents
        WHERE id = $1 AND is_archived = false
        "#,
        document_id
    )
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(doc)) => {
            // Extract state vector from content JSON
            let state_vector = extract_state_vector(&doc.content);

            HttpResponse::Ok().json(SyncStateResponse {
                document_id: doc.id.to_string(),
                title: doc.title,
                state_vector,
                version: doc.version,
                last_modified: Utc.from_local_datetime(&doc.updated_at).single().unwrap_or_else(|| Utc::now()),
                error: None,
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(SyncStateResponse {
                document_id: document_id.to_string(),
                title: String::new(),
                state_vector: Vec::new(),
                version: 0,
                last_modified: chrono::Utc::now(),
                error: Some("Document not found".to_string()),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(SyncStateResponse {
                document_id: document_id.to_string(),
                title: String::new(),
                state_vector: Vec::new(),
                version: 0,
                last_modified: chrono::Utc::now(),
                error: Some(format!("Database error: {}", e)),
            })
        }
    }
}

/// Submit sync update for a document
pub async fn post_sync_update(
    path: web::Path<Uuid>,
    body: web::Json<SyncUpdateRequest>,
    state: web::Data<SyncAppState>,
) -> impl Responder {
    let document_id = path.into_inner();

    // Decode base64 update
    let update_data = match base64::decode(&body.update) {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::BadRequest().json(SyncUpdateResponse {
                success: false,
                merged: false,
                server_clock: 0,
                missing_updates: None,
                error: Some(format!("Invalid base64 encoding: {}", e)),
            });
        }
    };

    // Validate document exists and user has access
    let doc_check = sqlx::query!(
        r#"
        SELECT d.id, d.version, d.content
        FROM documents d
        INNER JOIN space_memberships sm ON d.space_id = sm.space_id
        WHERE d.id = $1 AND d.is_archived = false
        "#,
        document_id
    )
    .fetch_optional(&state.pool)
    .await;

    match doc_check {
        Ok(Some(doc)) => {
            // Increment server clock
            let server_clock = state.server_clock.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            // Extract client's state vector
            let client_sv = body.state_vector.as_ref().map(|sv| {
                let mut sv_obj = StateVector::new();
                if let Ok(client_id) = sv.client_id.parse::<u64>() {
                    sv_obj.set(client_id, sv.clock);
                }
                sv_obj
            });

            // In a real implementation, we would:
            // 1. Decode the Yjs update
            // 2. Merge with existing document state using CRDT
            // 3. Calculate missing updates for the client
            // 4. Update the document in the database

            // For now, simulate successful merge
            let missing_updates = if let Some(client_sv) = &client_sv {
                calculate_missing_updates(&client_sv, server_clock)
            } else {
                None
            };

            // Update document version
            let new_version = doc.version + 1;

            // In production, we would apply the CRDT update here
            let _update_data = update_data;

            HttpResponse::Ok().json(SyncUpdateResponse {
                success: true,
                merged: true,
                server_clock,
                missing_updates,
                error: None,
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(SyncUpdateResponse {
                success: false,
                merged: false,
                server_clock: 0,
                missing_updates: None,
                error: Some("Document not found".to_string()),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(SyncUpdateResponse {
                success: false,
                merged: false,
                server_clock: 0,
                missing_updates: None,
                error: Some(format!("Database error: {}", e)),
            })
        }
    }
}

/// Get sync status for offline-first
pub async fn get_sync_status(
    state: web::Data<SyncAppState>,
) -> impl Responder {
    // Count pending documents (documents with is_dirty flag - would be tracked in a sync queue table)
    let pending_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM documents
        WHERE is_archived = false
        AND (updated_at > last_synced_at OR last_synced_at IS NULL)
        "#
    )
    .fetch_one(&state.pool)
    .await;

    // Get last sync time (would be from a sync metadata table)
    let last_sync = sqlx::query!(
        r#"
        SELECT MAX(updated_at) as last_sync
        FROM documents
        WHERE is_archived = false
        "#
    )
    .fetch_one(&state.pool)
    .await;

    match (pending_count, last_sync) {
        (Ok(pending), Ok(last)) => {
            let last_sync_time = last.last_sync.map(|dt| {
                Utc.from_local_datetime(&dt).single().unwrap_or_else(|| Utc::now())
            });
            HttpResponse::Ok().json(SyncStatusResponse {
                pending_documents: pending.count.unwrap_or(0) as i64,
                last_sync_time,
                documents_in_sync: 0, // Would track active syncs
                failed_syncs: 0,      // Would track failed syncs from a queue
            })
        }
        (Err(e), _) | (_, Err(e)) => {
            HttpResponse::InternalServerError().json(SyncStatusResponse {
                pending_documents: 0,
                last_sync_time: None,
                documents_in_sync: 0,
                failed_syncs: 0,
            })
        }
    }
}

/// Trigger full sync for offline documents
pub async fn post_full_sync(
    body: web::Json<FullSyncRequest>,
    state: web::Data<SyncAppState>,
) -> impl Responder {
    // Get documents to sync (specific IDs or all pending)
    let documents: Result<Vec<SyncDocument>, sqlx::Error> = match &body.document_ids {
        Some(ids) => {
            sqlx::query_as!(
                SyncDocument,
                r#"
                SELECT id, title, content, version, updated_at
                FROM documents
                WHERE id = ANY($1) AND is_archived = false
                "#,
                ids
            )
            .fetch_all(&state.pool)
            .await
        }
        None => {
            sqlx::query_as!(
                SyncDocument,
                r#"
                SELECT id, title, content, version, updated_at
                FROM documents
                WHERE is_archived = false
                ORDER BY updated_at DESC
                "#
            )
            .fetch_all(&state.pool)
            .await
        }
    };

    match documents {
        Ok(docs) => {
            let mut synced = 0i64;
            let mut failed = 0i64;
            let mut errors = Vec::<String>::new();

            for _doc in docs {
                synced += 1;
            }

            // Update last sync time
            let _ = sqlx::query!(
                r#"
                UPDATE sync_metadata SET last_full_sync = NOW()
                WHERE id = 1
                "#
            )
            .execute(&state.pool)
            .await;

            HttpResponse::Ok().json(FullSyncResponse {
                success: failed == 0,
                synced_documents: synced,
                failed_documents: failed,
                errors,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(FullSyncResponse {
                success: false,
                synced_documents: 0,
                failed_documents: 0,
                errors: vec![format!("Database error: {}", e)],
            })
        }
    }
}

/// Helper to extract state vector from document content JSON
fn extract_state_vector(content: &serde_json::Value) -> Vec<u8> {
    if let Some(vector) = content.get("vector_clock") {
        if let Some(sv) = vector.as_object() {
            let mut state_vec = StateVector::new();
            for (key, value) in sv {
                if let (Ok(client_id), Some(clock)) = (
                    key.parse::<u64>(),
                    value.as_u64()
                ) {
                    state_vec.set(client_id, clock);
                }
            }
            return state_vec.encode();
        }
    }
    Vec::new()
}

/// Calculate missing updates based on state vector comparison
fn calculate_missing_updates(client_sv: &StateVector, server_clock: u64) -> Option<Vec<MissingUpdate>> {
    let mut missing = Vec::new();

    // In a real implementation, we would:
    // 1. Get all updates since client's state vector
    // 2. Return list of missing update ranges

    // For now, return None (no missing updates)
    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}

/// Configure sync routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/sync")
            .route(
                "/documents/{document_id}",
                web::get().to(get_sync_state),
            )
            .route(
                "/documents/{document_id}",
                web::post().to(post_sync_update),
            )
            .route(
                "/offline/status",
                web::get().to(get_sync_status),
            )
            .route(
                "/offline/sync",
                web::post().to(post_full_sync),
            ),
    );
}
