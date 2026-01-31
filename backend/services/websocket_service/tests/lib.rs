//! Integration tests for websocket_service

use serde_json::json;
use uuid::Uuid;

use websocket_service::{
    AwarenessMessage, ClientMessage, ConnectionInfo, CursorPosition, DocumentAwareness, DocumentState, ErrorResponse,
    MessageType, ServerMessage, SessionStore, SyncMessage, UserPresence, UserState, WebSocketMessage,
    WebSocketMessageType, WebSocketSession,
};

// ========================================
// Cursor Position Tests
// ========================================

#[test]
fn test_cursor_position_creation() {
    let cursor = CursorPosition {
        x: 100.5,
        y: 200.0,
        selection_start: Some(10),
        selection_end: Some(20),
    };
    assert_eq!(cursor.x, 100.5);
    assert_eq!(cursor.y, 200.0);
}

#[test]
fn test_cursor_position_partial_selection() {
    let cursor = CursorPosition {
        x: 50.0,
        y: 75.0,
        selection_start: Some(5),
        selection_end: None,
    };
    assert!(cursor.selection_start.is_some());
    assert!(cursor.selection_end.is_none());
}

#[test]
fn test_cursor_position_clone() {
    let cursor = CursorPosition {
        x: 123.456,
        y: 789.012,
        selection_start: Some(0),
        selection_end: Some(100),
    };
    let cloned = cursor.clone();
    assert_eq!(cloned.x, cursor.x);
    assert_eq!(cloned.y, cursor.y);
}

// ========================================
// User Presence Tests
// ========================================

#[test]
fn test_user_presence_creation() {
    let presence = UserPresence {
        user_id: Uuid::new_v4(),
        display_name: "Test User".to_string(),
        color: "#FF5733".to_string(),
        cursor: None,
        last_active: chrono::Utc::now(),
    };
    assert_eq!(presence.display_name, "Test User");
    assert_eq!(presence.color, "#FF5733");
}

#[test]
fn test_user_presence_with_cursor() {
    let cursor = CursorPosition {
        x: 100.0,
        y: 200.0,
        selection_start: None,
        selection_end: None,
    };
    let presence = UserPresence {
        user_id: Uuid::new_v4(),
        display_name: "Active User".to_string(),
        color: "#33FF57".to_string(),
        cursor: Some(cursor),
        last_active: chrono::Utc::now(),
    };
    assert!(presence.cursor.is_some());
}

#[test]
fn test_user_presence_default() {
    let presence = UserPresence::default();
    assert_eq!(presence.user_id, Uuid::nil());
    assert!(presence.display_name.is_empty());
}

// ========================================
// WebSocket Message Tests
// ========================================

#[test]
fn test_websocket_message_creation() {
    let message = WebSocketMessage::new(
        WebSocketMessageType::Sync,
        Uuid::new_v4(),
        Uuid::new_v4(),
        json!({"data": "test"}),
    );
    assert!(matches!(message.type_, WebSocketMessageType::Sync));
    assert!(message.document_id != Uuid::nil());
}

#[test]
fn test_websocket_message_json_serialization() {
    let message = WebSocketMessage::new(
        WebSocketMessageType::DocumentUpdate,
        Uuid::new_v4(),
        Uuid::new_v4(),
        json!({"update": [1, 2, 3]}),
    );
    let json_str = message.to_json().expect("Serialize");
    let deserialized = WebSocketMessage::from_json(&json_str).expect("Deserialize");
    assert!(matches!(deserialized.type_, WebSocketMessageType::DocumentUpdate));
}

#[test]
fn test_websocket_message_types() {
    for mt in [
        WebSocketMessageType::Sync,
        WebSocketMessageType::Awareness,
        WebSocketMessageType::Cursor,
        WebSocketMessageType::DocumentUpdate,
        WebSocketMessageType::UserJoin,
        WebSocketMessageType::UserLeave,
        WebSocketMessageType::Ping,
        WebSocketMessageType::Pong,
    ] {
        let message = WebSocketMessage::new(mt, Uuid::new_v4(), Uuid::new_v4(), json!({}));
        assert_eq!(message.type_, mt);
    }
}

// ========================================
// WebSocket Session Tests
// ========================================

#[test]
fn test_websocket_session_creation() {
    let session = WebSocketSession::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "Test User".to_string(),
        "#FF5733".to_string(),
    );
    assert!(session.id != Uuid::nil());
    assert_eq!(session.display_name, "Test User");
}

#[test]
fn test_websocket_session_update_activity() {
    let mut session = WebSocketSession::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "Test User".to_string(),
        "#FF5733".to_string(),
    );
    let original = session.last_activity;
    std::thread::sleep(std::time::Duration::from_millis(10));
    session.update_activity();
    assert!(session.last_activity >= original);
}

// ========================================
// Session Store Tests
// ========================================

#[test]
fn test_session_store_add_and_get() {
    let store = SessionStore::new();
    let document_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let session = WebSocketSession::new(document_id, user_id, "Test".to_string(), "#FFF".to_string());
    let session_id = session.id;
    store.add_session(session);
    let retrieved = store.get_session(session_id);
    assert!(retrieved.is_some());
}

#[test]
fn test_session_store_remove() {
    let store = SessionStore::new();
    let session = WebSocketSession::new(Uuid::new_v4(), Uuid::new_v4(), "Test".to_string(), "#FFF".to_string());
    let session_id = session.id;
    store.add_session(session.clone());
    assert!(store.get_session(session_id).is_some());
    store.remove_session(session_id);
    assert!(store.get_session(session_id).is_none());
}

#[test]
fn test_session_store_get_document_sessions() {
    let store = SessionStore::new();
    let document_id = Uuid::new_v4();
    for i in 0..3 {
        store.add_session(WebSocketSession::new(
            document_id,
            Uuid::new_v4(),
            format!("U{}", i),
            "#FFF".to_string(),
        ));
    }
    let sessions = store.get_document_sessions(document_id);
    assert_eq!(sessions.len(), 3);
}

// ========================================
// Document Awareness Tests
// ========================================

#[test]
fn test_document_awareness_creation() {
    let awareness = DocumentAwareness::new(Uuid::new_v4());
    assert!(awareness.users.is_empty());
}

#[test]
fn test_document_awareness_add_user() {
    let mut awareness = DocumentAwareness::new(Uuid::new_v4());
    let user = UserState {
        user_id: Uuid::new_v4(),
        display_name: "Test".to_string(),
        color: "#FFF".to_string(),
        cursor: None,
        last_active: chrono::Utc::now(),
    };
    awareness.add_user(user.clone());
    assert_eq!(awareness.users.len(), 1);
}

#[test]
fn test_document_awareness_remove_user() {
    let mut awareness = DocumentAwareness::new(Uuid::new_v4());
    let user_id = Uuid::new_v4();
    let user = UserState {
        user_id,
        display_name: "Test".to_string(),
        color: "#FFF".to_string(),
        cursor: None,
        last_active: chrono::Utc::now(),
    };
    awareness.add_user(user);
    assert_eq!(awareness.users.len(), 1);
    awareness.remove_user(user_id);
    assert!(awareness.users.is_empty());
}

#[test]
fn test_document_awareness_update_cursor() {
    let mut awareness = DocumentAwareness::new(Uuid::new_v4());
    let user_id = Uuid::new_v4();
    let user = UserState {
        user_id,
        display_name: "Test".to_string(),
        color: "#FFF".to_string(),
        cursor: None,
        last_active: chrono::Utc::now(),
    };
    awareness.add_user(user);
    let cursor = CursorPosition {
        x: 100.0,
        y: 200.0,
        selection_start: None,
        selection_end: None,
    };
    awareness.update_cursor(user_id, cursor);
    assert!(awareness.users.get(&user_id).unwrap().cursor.is_some());
}

#[test]
fn test_document_awareness_get_users() {
    let mut awareness = DocumentAwareness::new(Uuid::new_v4());
    for i in 0..3 {
        awareness.add_user(UserState {
            user_id: Uuid::new_v4(),
            display_name: format!("U{}", i),
            color: "#FFF".to_string(),
            cursor: None,
            last_active: chrono::Utc::now(),
        });
    }
    assert_eq!(awareness.get_users().len(), 3);
}

// ========================================
// Document State Tests
// ========================================

#[test]
fn test_document_state_creation() {
    let state = DocumentState {
        document_id: Uuid::new_v4(),
        state_vector: vec![],
        update: vec![],
        timestamp: chrono::Utc::now(),
    };
    assert!(state.state_vector.is_empty());
    assert!(state.update.is_empty());
}

#[test]
fn test_document_state_with_content() {
    let document_id = Uuid::new_v4();
    let state = DocumentState {
        document_id,
        state_vector: vec![1, 2, 3, 4, 5],
        update: vec![10, 20, 30],
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(state.state_vector.len(), 5);
    assert_eq!(state.update.len(), 3);
}

// ========================================
// Connection Info Tests
// ========================================

#[test]
fn test_connection_info_creation() {
    let info = ConnectionInfo {
        session_id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        connected_at: chrono::Utc::now(),
        last_ping: chrono::Utc::now(),
    };
    assert!(info.session_id != Uuid::nil());
}

// ========================================
// Error Response Tests
// ========================================

#[test]
fn test_error_response_creation() {
    let error = ErrorResponse::new("INVALID_TOKEN", "The provided token is invalid");
    assert_eq!(error.code, "INVALID_TOKEN");
    assert_eq!(error.message, "The provided token is invalid");
}

#[test]
fn test_error_response_variants() {
    for (code, msg) in [
        ("AUTH_REQUIRED", "Authentication is required"),
        ("DOCUMENT_NOT_FOUND", "Document does not exist"),
        ("CONNECTION_LIMIT", "Maximum connections reached"),
    ] {
        let error = ErrorResponse::new(code, msg);
        assert!(!error.code.is_empty());
        assert!(!error.message.is_empty());
    }
}

// ========================================
// Message Type Tests
// ========================================

#[test]
fn test_message_type_variants() {
    for mt in [
        MessageType::Sync,
        MessageType::Awareness,
        MessageType::Cursor,
        MessageType::DocumentUpdate,
        MessageType::UserJoin,
        MessageType::UserLeave,
        MessageType::Ping,
        MessageType::Pong,
        MessageType::Error,
    ] {
        let _ = mt;
    }
}

// ========================================
// Client/Server Message Tests
// ========================================

#[test]
fn test_client_message_creation() {
    let message = ClientMessage {
        type_: MessageType::Sync,
        document_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        payload: json!({"state_vector": []}),
    };
    assert!(matches!(message.type_, MessageType::Sync));
}

#[test]
fn test_server_message_creation() {
    let message = ServerMessage {
        type_: MessageType::Sync,
        document_id: Uuid::new_v4(),
        payload: json!({"update": []}),
        timestamp: chrono::Utc::now(),
    };
    assert!(matches!(message.type_, MessageType::Sync));
}

#[test]
fn test_sync_message_variants() {
    let sync1 = SyncMessage {
        state_vector: Some(vec![1, 2, 3]),
        update: None,
    };
    assert!(sync1.state_vector.is_some());
    let sync2 = SyncMessage {
        state_vector: None,
        update: Some(vec![4, 5, 6]),
    };
    assert!(sync2.update.is_some());
    let sync3 = SyncMessage {
        state_vector: Some(vec![1]),
        update: Some(vec![2]),
    };
    assert!(sync3.state_vector.is_some() && sync3.update.is_some());
}

#[test]
fn test_awareness_message_creation() {
    let message = AwarenessMessage {
        state: json!({"user": {"name": "Test", "color": "#FF5733"}}),
    };
    assert!(message.state.is_object());
}

// ========================================
// User State Tests
// ========================================

#[test]
fn test_user_state_creation() {
    let state = UserState {
        user_id: Uuid::new_v4(),
        display_name: "Test User".to_string(),
        color: "#FF5733".to_string(),
        cursor: None,
        last_active: chrono::Utc::now(),
    };
    assert_eq!(state.display_name, "Test User");
}

#[test]
fn test_user_state_default() {
    let state = UserState::default();
    assert_eq!(state.user_id, Uuid::nil());
    assert!(state.display_name.is_empty());
}

// ========================================
// JSON Serialization Tests
// ========================================

#[test]
fn test_message_json_roundtrip() {
    let original = ClientMessage {
        type_: MessageType::Awareness,
        document_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        payload: json!({"presence": {"x": 100, "y": 200}}),
    };
    let json_str = serde_json::to_string(&original).expect("Serialize");
    let deserialized: ClientMessage = serde_json::from_str(&json_str).expect("Deserialize");
    assert!(matches!(deserialized.type_, MessageType::Awareness));
}

#[test]
fn test_sync_message_json_roundtrip() {
    let original = SyncMessage {
        state_vector: Some(vec![1, 2, 3, 4, 5]),
        update: Some(vec![10, 20, 30]),
    };
    let json_str = serde_json::to_string(&original).expect("Serialize");
    let deserialized: SyncMessage = serde_json::from_str(&json_str).expect("Deserialize");
    assert_eq!(original.state_vector, deserialized.state_vector);
}
