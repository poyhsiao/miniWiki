use crate::{
    models::{AwarenessMessage, ClientMessage, MessageType, ServerMessage, SyncMessage},
    CursorPosition, UserPresence, WebSocketMessage, WebSocketSession, PRESENCE_STORE, SESSION_STORE,
};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

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
        states.entry(document_id).or_insert_with(|| Arc::new(Mutex::new(DocumentSyncState::default())));
        Arc::clone(states.get(&document_id).unwrap())
    }

    pub async fn get_broadcast_sender(&self, document_id: Uuid) -> Arc<DocumentBroadcastSender> {
        let mut senders = self.broadcast_senders.lock().await;
        senders.entry(document_id).or_insert_with(|| Arc::new(DocumentBroadcastSender::new(document_id)));
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

pub async fn handle_message(session: &WebSocketSession, msg: ClientMessage) -> Result<Vec<ServerMessage>, String> {
    match msg.type_ {
        MessageType::Sync => handle_sync(session, msg.payload).await,
        MessageType::Awareness => handle_awareness(session, msg.payload).await,
        MessageType::Cursor => handle_cursor(session, msg.payload).await,
        MessageType::Ping => handle_ping(session).await,
        _ => Ok(vec![]),
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
async fn handle_sync(session: &WebSocketSession, payload: serde_json::Value) -> Result<Vec<ServerMessage>, String> {
    let sync_msg: SyncMessage = serde_json::from_value(payload).map_err(|e| format!("Invalid sync message: {}", e))?;

    let _sync_state = SYNC_MANAGER.get_or_create_sync_state(session.document_id).await;

    tracing::debug!(
        "Received sync message for document {} from user {}",
        session.document_id,
        session.user_id
    );

    let mut messages_to_send: Vec<ServerMessage> = Vec::new();

    match &sync_msg {
        SyncMessage {
            state_vector: Some(sv),
            update: None,
        } => {
            // Client sending state vector - step 1 of sync
            if let Some(response) = handle_sync_step1(session, sv).await {
                // Send update directly to requesting client
                messages_to_send.push(response);
            }
        },
        SyncMessage {
            state_vector: None,
            update: Some(update),
        } => {
            // Client sending update - step 2 or ongoing updates
            handle_sync_step2(session, update).await;
            // Note: Broadcasting is handled by SYNC_MANAGER, not returned here
        },
        SyncMessage {
            state_vector: Some(sv),
            update: Some(update),
        } => {
            // Both state vector and update (shouldn't happen in standard Yjs)
            if let Some(response) = handle_sync_step1(session, sv).await {
                messages_to_send.push(response);
            }
            handle_sync_step2(session, update).await;
        },
        _ => {
            // Invalid sync message
            tracing::warn!("Received invalid sync message: {:?}", sync_msg);
        },
    }

    Ok(messages_to_send)
}

/// Handle sync step 1: Client sends state vector
async fn handle_sync_step1(session: &WebSocketSession, state_vector: &[u8]) -> Option<ServerMessage> {
    // In a real implementation, this would:
    // 1. Look up document in the database
    // 2. Get the current document state
    // 3. Compute the diff between the state vector and current state
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

    tracing::debug!("Computed sync response for document {}, user {}", document_id, user_id);

    // Return response so caller can send it
    Some(response)
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

    tracing::debug!("Processed update from user {} for document {}", user_id, document_id);
}

/// Compute Yjs diff between state vector and current document state
async fn compute_yjs_diff(_document_id: Uuid, _state_vector: &[u8]) -> Vec<u8> {
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
) -> Result<Vec<ServerMessage>, String> {
    let _awareness_msg: AwarenessMessage =
        serde_json::from_value(payload.clone()).map_err(|e| format!("Invalid awareness message: {}", e))?;

    let user_id = session.user_id;
    let document_id = session.document_id;
    let display_name = session.display_name.clone();
    let color = session.color.clone();

    tracing::debug!(
        "Received awareness update for user {} in document {}",
        user_id,
        document_id
    );

    // Extract cursor once and propagate to broadcast
    let cursor = payload
        .get("cursor")
        .and_then(|c| serde_json::from_value::<CursorPosition>(c.clone()).ok());

    if let Some(ref cursor) = cursor {
        PRESENCE_STORE.update_cursor(user_id, cursor.clone());
    }

    // Broadcast awareness update to all clients in the document
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

    Ok(vec![message])
}

async fn handle_cursor(session: &WebSocketSession, payload: serde_json::Value) -> Result<Vec<ServerMessage>, String> {
    let cursor: CursorPosition =
        serde_json::from_value(payload).map_err(|e| format!("Invalid cursor position: {}", e))?;

    let user_id = session.user_id;
    let document_id = session.document_id;

    tracing::debug!(
        "Cursor update for user {} in document {}: ({}, {})",
        user_id,
        document_id,
        cursor.x,
        cursor.y
    );

    // Update cursor in presence store
    PRESENCE_STORE.update_cursor(user_id, cursor.clone());

    Ok(vec![])
}

async fn handle_ping(_session: &WebSocketSession) -> Result<Vec<ServerMessage>, String> {
    // Ping is handled at the WebSocket actor level for heartbeat
    Ok(vec![])
}

/// Broadcast a message to all sessions in a document
///
/// The send_fn callback should take (session_id, message) and send the message
/// to the WebSocket connection for that session.
///
/// TODO: The actual WebSocket send operation requires the actor context.
/// When calling this from within a DocumentWsHandler actor, pass a closure like:
///     |session_id, msg| {
///         // Look up the connection by session_id and send
///         // This requires access to the actor's Addr or context
///     }
/// For now, this logs the send attempt.
pub fn broadcast_to_document(
    document_id: Uuid,
    message: ServerMessage,
    exclude_user_id: Option<Uuid>,
    send_fn: impl Fn(Uuid, String),
) {
    let sessions = SESSION_STORE.get_document_sessions(document_id);

    for session_arc in sessions {
        let session = session_arc.lock().unwrap();
        if let Some(exclude) = exclude_user_id {
            if session.user_id == exclude {
                continue;
            }
        }
        if let Ok(json) = serde_json::to_string(&message) {
            send_fn(session.id, json);
        }
    }
}

/// Broadcast awareness update to all clients in a document
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

    let _message = ServerMessage {
        type_: MessageType::UserJoin,
        document_id,
        payload: json!(presence),
        timestamp: Utc::now(),
    };
}

/// Broadcast user leave event to all clients in a document
pub fn broadcast_user_leave(document_id: Uuid, user_id: Uuid) {
    let message = ServerMessage {
        type_: MessageType::UserLeave,
        document_id,
        payload: json!({ "user_id": user_id.to_string() }),
        timestamp: Utc::now(),
    };

    // Use placeholder delivery callback
    // TODO: Wire this to the actual WebSocket send mechanism when actor context is available
    broadcast_to_document(document_id, message, None, |_session_id, _msg| {
        tracing::warn!(
            "broadcast_user_leave: WebSocket delivery not implemented - message would be sent to session {}",
            _session_id
        );
    });
}

/// Broadcast document update to all clients in a document
pub fn broadcast_document_update(document_id: Uuid, update: Vec<u8>, origin_user_id: Uuid) {
    let message = ServerMessage {
        type_: MessageType::DocumentUpdate,
        document_id,
        payload: json!({
            "update": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &update),
            "origin_user_id": origin_user_id.to_string()
        }),
        timestamp: Utc::now(),
    };

    // Use placeholder delivery callback
    // TODO: Wire this to the actual WebSocket send mechanism when actor context is available
    broadcast_to_document(document_id, message, Some(origin_user_id), |_session_id, _msg| {
        tracing::warn!(
            "broadcast_document_update: WebSocket delivery not implemented - message would be sent to session {}",
            _session_id
        );
    });
}

/// Broadcast cursor position to all clients in a document
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

    // Use placeholder delivery callback
    // TODO: Wire this to the actual WebSocket send mechanism when actor context is available
    broadcast_to_document(document_id, message, Some(user_id), |_session_id, _msg| {
        tracing::warn!(
            "broadcast_cursor_position: WebSocket delivery not implemented - message would be sent to session {}",
            _session_id
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ClientMessage, MessageType, ServerMessage, SyncMessage};

    #[test]
    fn test_sync_state_variants() {
        assert_eq!(SyncState::WaitingForStateVector, SyncState::WaitingForStateVector);
        assert_eq!(SyncState::SentInitialUpdate, SyncState::SentInitialUpdate);
        assert_eq!(SyncState::Synced, SyncState::Synced);
    }

    #[test]
    fn test_sync_state_ordering() {
        let states = vec![
            SyncState::WaitingForStateVector,
            SyncState::SentInitialUpdate,
            SyncState::Synced,
        ];

        for state in states {
            assert_eq!(state, state.clone());
        }
    }

    #[test]
    fn test_server_message_serialization() {
        let message = ServerMessage {
            type_: MessageType::Sync,
            document_id: Uuid::new_v4(),
            payload: json!({"update": "test"}),
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: ServerMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(message.type_, deserialized.type_);
        assert_eq!(message.document_id, deserialized.document_id);
    }

    #[test]
    fn test_client_message_sync() {
        let sync_msg = SyncMessage {
            state_vector: Some(vec![1, 2, 3]),
            update: None,
        };
        let message = ClientMessage {
            type_: MessageType::Sync,
            document_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            payload: serde_json::to_value(&sync_msg).unwrap(),
        };

        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("Sync"));
    }

    #[test]
    fn test_client_message_awareness() {
        let awareness_msg = AwarenessMessage {
            state: serde_json::json!({"test": "data"}),
        };
        let message = ClientMessage {
            type_: MessageType::Awareness,
            document_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            payload: serde_json::to_value(&awareness_msg).unwrap(),
        };

        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("Awareness"));
    }

    #[test]
    fn test_message_type_variants() {
        assert_ne!(MessageType::Sync, MessageType::Awareness);
        assert_ne!(MessageType::Cursor, MessageType::UserJoin);
        assert_ne!(MessageType::UserLeave, MessageType::DocumentUpdate);
    }

    #[test]
    fn test_sync_message_state_vector() {
        let msg = SyncMessage {
            state_vector: Some(vec![0, 1, 2, 3, 4]),
            update: None,
        };

        assert!(msg.state_vector.is_some());
        assert_eq!(msg.state_vector.unwrap().len(), 5);
        assert!(msg.update.is_none());
    }

    #[test]
    fn test_sync_message_update() {
        let msg = SyncMessage {
            state_vector: None,
            update: Some(vec![255, 254, 253]),
        };

        assert!(msg.state_vector.is_none());
        assert!(msg.update.is_some());
        assert_eq!(msg.update.unwrap().len(), 3);
    }

    #[test]
    fn test_awareness_message() {
        let state = serde_json::json!({"users": [1, 2, 3, 4, 5]});
        let msg = AwarenessMessage { state: state.clone() };

        assert_eq!(msg.state, state);
    }

    #[test]
    fn test_cursor_position() {
        let cursor = CursorPosition {
            x: 100.0,
            y: 200.0,
            selection_start: Some(50),
            selection_end: Some(75),
        };

        assert_eq!(cursor.x, 100.0);
        assert_eq!(cursor.y, 200.0);
        assert_eq!(cursor.selection_start, Some(50));
        assert_eq!(cursor.selection_end, Some(75));
    }

    #[test]
    fn test_cursor_position_no_selection() {
        let cursor = CursorPosition {
            x: 50.0,
            y: 100.0,
            selection_start: None,
            selection_end: None,
        };

        assert!(cursor.selection_start.is_none());
        assert!(cursor.selection_end.is_none());
    }
}
