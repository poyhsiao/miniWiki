use crate::{
    models::{ClientMessage, ServerMessage, MessageType, SyncMessage, AwarenessMessage},
    WebSocketSession, SESSION_STORE, UserPresence, CursorPosition,
};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

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

async fn handle_sync(
    session: &WebSocketSession,
    payload: serde_json::Value,
) -> Result<(), String> {
    let _sync_msg: SyncMessage = serde_json::from_value(payload)
        .map_err(|e| format!("Invalid sync message: {}", e))?;

    tracing::debug!(
        "Received sync message for document {} from user {}",
        session.document_id,
        session.user_id
    );

    Ok(())
}

async fn handle_awareness(
    session: &WebSocketSession,
    payload: serde_json::Value,
) -> Result<(), String> {
    let _awareness_msg: AwarenessMessage = serde_json::from_value(payload)
        .map_err(|e| format!("Invalid awareness message: {}", e))?;

    tracing::debug!(
        "Received awareness update for user {} in document {}",
        session.user_id,
        session.document_id
    );

    Ok(())
}

async fn handle_cursor(
    session: &WebSocketSession,
    payload: serde_json::Value,
) -> Result<(), String> {
    let _cursor: CursorPosition = serde_json::from_value(payload)
        .map_err(|e| format!("Invalid cursor position: {}", e))?;

    tracing::debug!(
        "Cursor update for user {} in document {}",
        session.user_id,
        session.document_id
    );

    Ok(())
}

async fn handle_ping(session: &WebSocketSession) -> Result<(), String> {
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
            if session.id == exclude {
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

    let _sessions = SESSION_STORE.get_document_sessions(document_id);
}

pub fn broadcast_user_leave(document_id: Uuid, user_id: Uuid) {
    let message = ServerMessage {
        type_: MessageType::UserLeave,
        document_id,
        payload: json!({ "user_id": user_id.to_string() }),
        timestamp: Utc::now(),
    };

    let _sessions = SESSION_STORE.get_document_sessions(document_id);
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
            "update": base64::encode(&update),
            "origin_user_id": origin_user_id.to_string()
        }),
        timestamp: Utc::now(),
    };

    let _sessions = SESSION_STORE.get_document_sessions(document_id);
}
