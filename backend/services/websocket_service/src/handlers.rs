use crate::{
    models::{ClientMessage, ServerMessage, MessageType, SyncMessage, AwarenessMessage},
    WebSocketSession, SESSION_STORE, UserPresence, WebSocketMessage, PRESENCE_STORE, CursorPosition,
};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, broadcast};

/// Maximum number of sync updates to buffer per document
const SYNC_BUFFER_SIZE: usize = 1000;

/// Yjs Sync Protocol State Machine
///
/// The Yjs sync protocol consists of three steps:
/// 1. SyncStep1: Client sends state vector, server responds with update
/// 2. SyncStep2: Client sends update based on server's response
/// 3. Update: Both sides exchange incremental updates
#[derive(Debug, Clone, PartialEq)]
pub enum SyncState {
    /// Initial state - waiting for client's state vector
    WaitingForStateVector,
    /// Received state vector, sent initial update
    SentInitialUpdate,
    /// Sync complete - normal update exchange
    Synced,
}

/// Sync state per document session
pub struct DocumentSyncState {
    pub state: SyncState,
    pub last_update: Vec<u8>,
    pub pending_updates: Vec<Vec<u8>>,
}

impl Default for DocumentSyncState {
    fn default() -> Self {
        Self {
            state: SyncState::WaitingForStateVector,
            last_update: Vec::new(),
            pending_updates: Vec::new(),
        }
    }
}

/// Broadcast sender for document updates (used for Redis pub/sub fallback)
pub struct DocumentBroadcastSender {
    document_id: Uuid,
    sender: broadcast::Sender<WebSocketMessage>,
}

impl DocumentBroadcastSender {
    pub fn new(document_id: Uuid) -> Self {
        let (sender, _) = broadcast::channel(SYNC_BUFFER_SIZE);
        Self { document_id, sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WebSocketMessage> {
        self.sender.subscribe()
    }

    pub fn send(&self, message: WebSocketMessage) -> Result<usize, broadcast::error::SendError<WebSocketMessage>> {
        self.sender.send(message)
    }

    pub fn document_id(&self) -> Uuid {
        self.document_id
    }
}

/// Thread-safe document sync state manager
#[derive(Clone, Default)]
pub struct DocumentSyncManager {
    states: Arc<Mutex<HashMap<Uuid, Arc<Mutex<DocumentSyncState>>>>>,
    broadcast_senders: Arc<Mutex<HashMap<Uuid, Arc<DocumentBroadcastSender>>>>,
}

impl DocumentSyncManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            broadcast_senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_or_create_sync_state(&self, document_id: Uuid) -> Arc<Mutex<DocumentSyncState>> {
        let mut states = self.states.lock().await;
        if !states.contains_key(&document_id) {
            states.insert(document_id, Arc::new(Mutex::new(DocumentSyncState::default())));
        }
        Arc::clone(states.get(&document_id).unwrap())
    }

    pub async fn get_broadcast_sender(&self, document_id: Uuid) -> Arc<DocumentBroadcastSender> {
        let mut senders = self.broadcast_senders.lock().await;
        if !senders.contains_key(&document_id) {
            senders.insert(document_id, Arc::new(DocumentBroadcastSender::new(document_id)));
        }
        Arc::clone(senders.get(&document_id).unwrap())
    }

    pub async fn remove_document_state(&self, document_id: Uuid) {
        let mut states = self.states.lock().await;
        let mut senders = self.broadcast_senders.lock().await;
        states.remove(&document_id);
        senders.remove(&document_id);
    }
}

/// Global sync manager instance
pub static SYNC_MANAGER: once_cell::sync::Lazy<DocumentSyncManager> =
    once_cell::sync::Lazy::new(DocumentSyncManager::new);

pub async fn handle_message(
    session: &WebSocketSession,
    msg: ClientMessage,
) -> Result<(), String> {
    match msg.type_ {
        MessageType::Sync => handle_sync(session, msg.payload).await,
        MessageType::Awareness => handle_awareness(session, msg.payload).await,
        MessageType::Cursor => handle_cursor(session, msg.payload).await,
        MessageType::Ping => handle_ping(session).await,
        _ => Ok(()),
    }
}

/// Handle Yjs sync protocol messages
///
/// Implements the Yjs sync algorithm:
/// 1. Client sends state vector (step 1)
/// 2. Server computes diff and sends update (step 1)
/// 3. Client applies update and sends its own update (step 2)
/// 4. Server applies update and broadcasts to other clients
/// 5. Ongoing updates are exchanged via the Update message type
async fn handle_sync(
    session: &WebSocketSession,
    payload: serde_json::Value,
) -> Result<(), String> {
    let sync_msg: SyncMessage = serde_json::from_value(payload)
        .map_err(|e| format!("Invalid sync message: {}", e))?;

    let sync_state = SYNC_MANAGER.get_or_create_sync_state(session.document_id).await;

    tracing::debug!(
        "Received sync message for document {} from user {}",
        session.document_id,
        session.user_id
    );

    match &sync_msg {
        SyncMessage { state_vector: Some(sv), update: None } => {
            // Client sending state vector - step 1 of sync
            handle_sync_step1(session, sv).await;
        }
        SyncMessage { state_vector: None, update: Some(update) } => {
            // Client sending update - step 2 or ongoing updates
            handle_sync_step2(session, update).await;
        }
        SyncMessage { state_vector: Some(sv), update: Some(update) } => {
            // Both state vector and update (shouldn't happen in standard Yjs)
            handle_sync_step1(session, sv).await;
            handle_sync_step2(session, update).await;
        }
        _ => {
            // Invalid sync message
            tracing::warn!("Received invalid sync message: {:?}", sync_msg);
        }
    }

    Ok(())
}

/// Handle sync step 1: Client sends state vector
async fn handle_sync_step1(session: &WebSocketSession, state_vector: &[u8]) {
    // In a real implementation, this would:
    // 1. Look up the document in the database
    // 2. Get the current document state
    // 3. Compute the diff between state vector and current state
    // 4. Send the diff update to the client

    let document_id = session.document_id;
    let user_id = session.user_id;

    // Placeholder: Generate a mock update response
    // In production, this would compute the actual Yjs diff
    let update = compute_yjs_diff(document_id, state_vector).await;

    let response = ServerMessage {
        type_: MessageType::Sync,
        document_id,
        payload: json!({
            "update": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, update)
        }),
        timestamp: Utc::now(),
    };

    // Send update to requesting client
    // TODO: Implement actual WebSocket message sending to session
    // Currently, this is a placeholder - messages are serialized but not transmitted
    broadcast_to_document(
        document_id,
        response.clone(),
        Some(session.id),
        |_session_id, _msg| {
            tracing::warn!("TODO: WebSocket message sending not yet implemented - send_fn closure is empty placeholder");
            // In real implementation, this would send to the specific WebSocket connection
        },
    );

    tracing::debug!(
        "Sent sync update to client for document {}, user {}",
        document_id,
        user_id
    );
}

/// Handle sync step 2: Client sends update
async fn handle_sync_step2(session: &WebSocketSession, update: &[u8]) {
    let document_id = session.document_id;
    let user_id = session.user_id;

    let sync_state = SYNC_MANAGER.get_or_create_sync_state(document_id).await;
    let mut state_guard = sync_state.lock().await;
    state_guard.last_update = update.to_vec();

    // Broadcast update to other clients in the same document
    broadcast_document_update(document_id, update.to_vec(), user_id);

    tracing::debug!(
        "Processed update from user {} for document {}",
        user_id,
        document_id
    );
}

/// Compute Yjs diff between state vector and current document state
async fn compute_yjs_diff(document_id: Uuid, _state_vector: &[u8]) -> Vec<u8> {
    // Placeholder for Yjs diff computation
    // In production, this would:
    // 1. Load document state from database
    // 2. Use Yjs library to compute diff
    // 3. Return the encoded update

    // Mock return for now
    Vec::new()
}

async fn handle_awareness(
    session: &WebSocketSession,
    payload: serde_json::Value,
) -> Result<(), String> {
    let _awareness_msg: AwarenessMessage = serde_json::from_value(payload.clone())
        .map_err(|e| format!("Invalid awareness message: {}", e))?;

    let user_id = session.user_id;
    let document_id = session.document_id;
    let display_name = session.display_name.clone();
    let color = session.color.clone();

    tracing::debug!(
        "Received awareness update for user {} in document {}",
        user_id,
        document_id
    );

    // Update presence in the presence store
    PRESENCE_STORE.update_cursor(
        user_id,
        CursorPosition {
            x: payload.get("cursor")
                .and_then(|c| c.get("x"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0),
            y: payload.get("cursor")
                .and_then(|c| c.get("y"))
                .and_then(|y| y.as_f64())
                .unwrap_or(0.0),
            selection_start: payload.get("cursor")
                .and_then(|c| c.get("selection_start"))
                .and_then(|s| s.as_u64())
                .map(|v| v as usize),
            selection_end: payload.get("cursor")
                .and_then(|c| c.get("selection_end"))
                .and_then(|s| s.as_u64())
                .map(|v| v as usize),
        },
    );

    // Broadcast awareness update to all clients in the document
    broadcast_awareness_update(
        document_id,
        user_id,
        display_name,
        color,
        None, // Will be filled from the message
    );

    Ok(())
}

async fn handle_cursor(
    session: &WebSocketSession,
    payload: serde_json::Value,
) -> Result<(), String> {
    let cursor: CursorPosition = serde_json::from_value(payload)
        .map_err(|e| format!("Invalid cursor position: {}", e))?;

    let user_id = session.user_id;
    let document_id = session.document_id;
    let display_name = session.display_name.clone();
    let color = session.color.clone();

    tracing::debug!(
        "Cursor update for user {} in document {}: ({}, {})",
        user_id,
        document_id,
        cursor.x,
        cursor.y
    );

    // Update cursor in presence store
    PRESENCE_STORE.update_cursor(user_id, cursor.clone());

    // Broadcast cursor position to all other clients in the document
    broadcast_cursor_position(
        document_id,
        user_id,
        display_name,
        color,
        cursor,
    );

    Ok(())
}

async fn handle_ping(_session: &WebSocketSession) -> Result<(), String> {
    // Ping is handled at the WebSocket actor level for heartbeat
    Ok(())
}

pub fn broadcast_to_document(
    document_id: Uuid,
    message: ServerMessage,
    exclude_session: Option<Uuid>,
    send_fn: impl Fn(Uuid, String),
) {
    let sessions = SESSION_STORE.get_document_sessions(document_id);

    for session_arc in sessions {
        let session = session_arc.lock().unwrap();
        if let Some(exclude) = exclude_session {
            if session.user_id == exclude {
                continue;
            }
        }
        if let Ok(json) = serde_json::to_string(&message) {
            send_fn(session.id, json);
        }
    }
}

pub fn broadcast_awareness_update(
    document_id: Uuid,
    user_id: Uuid,
    display_name: String,
    color: String,
    cursor: Option<CursorPosition>,
) {
    let presence = UserPresence {
        user_id,
        display_name,
        color,
        cursor,
        last_active: Utc::now(),
    };

    let message = ServerMessage {
        type_: MessageType::UserJoin,
        document_id,
        payload: json!(presence),
        timestamp: Utc::now(),
    };

    broadcast_to_document(
        document_id,
        message,
        None,
        |_session_id, _msg| {},
    );
}

pub fn broadcast_user_leave(document_id: Uuid, user_id: Uuid) {
    let message = ServerMessage {
        type_: MessageType::UserLeave,
        document_id,
        payload: json!({ "user_id": user_id.to_string() }),
        timestamp: Utc::now(),
    };

    broadcast_to_document(
        document_id,
        message,
        None,
        |_session_id, _msg| {},
    );
}

pub fn broadcast_document_update(
    document_id: Uuid,
    update: Vec<u8>,
    origin_user_id: Uuid,
) {
    let message = ServerMessage {
        type_: MessageType::DocumentUpdate,
        document_id,
        payload: json!({
            "update": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &update),
            "origin_user_id": origin_user_id.to_string()
        }),
        timestamp: Utc::now(),
    };

    broadcast_to_document(
        document_id,
        message,
        Some(origin_user_id),
        |_session_id, _msg| {},
    );
}

pub fn broadcast_cursor_position(
    document_id: Uuid,
    user_id: Uuid,
    display_name: String,
    color: String,
    cursor: CursorPosition,
) {
    let message = ServerMessage {
        type_: MessageType::Cursor,
        document_id,
        payload: json!({
            "user_id": user_id.to_string(),
            "display_name": display_name,
            "color": color,
            "cursor": {
                "x": cursor.x,
                "y": cursor.y,
                "selection_start": cursor.selection_start,
                "selection_end": cursor.selection_end
            }
        }),
        timestamp: Utc::now(),
    };

    broadcast_to_document(
        document_id,
        message,
        Some(user_id),
        |_session_id, _msg| {},
    );
}
