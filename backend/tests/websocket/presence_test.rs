//! WebSocket Presence Tests
//!
//! Tests for WebSocket presence tracking functionality including:
//! - User presence updates
//! - Cursor position broadcasting
//! - Awareness state management
//! - Connection lifecycle

use std::time::Duration;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::json;

// Test modules for presence service testing
#[cfg(test)]
mod presence_test {
    use super::*;
    use tokio::sync::broadcast;
    use actix_web_actors::ws;

    // =========================================================================
    // Message Protocol Tests
    // =========================================================================

    /// Test WebSocket message serialization and deserialization
    #[test]
    fn test_message_serialization() {
        let message = ProtocolMessage {
            type_: MessageType::Sync,
            document_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            payload: json!({
                "update": [1, 2, 3, 4],
                "vector": [5, 6, 7, 8]
            }),
        };

        let serialized = serde_json::to_string(&message).expect("Failed to serialize");
        let deserialized: ProtocolMessage = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(message.type_, deserialized.type_);
        assert_eq!(message.document_id, deserialized.document_id);
        assert_eq!(message.user_id, deserialized.user_id);
    }

    /// Test awareness message structure
    #[test]
    fn test_awareness_message() {
        let awareness = AwarenessState {
            user: UserAwareness {
                user_id: Uuid::new_v4(),
                display_name: "Test User".to_string(),
                color: "#FF6B6B".to_string(),
            },
            cursor: Some(CursorState {
                x: 100.5,
                y: 200.75,
                selection: Some(SelectionState {
                    anchor: 50,
                    head: 75,
                }),
            }),
            local_state: json!({"focused": true}),
        };

        let serialized = serde_json::to_string(&awareness).expect("Failed to serialize");
        let deserialized: AwarenessState = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(awareness.user.display_name, deserialized.user.display_name);
        assert_eq!(awareness.cursor.as_ref().unwrap().x, deserialized.cursor.as_ref().unwrap().x);
        assert_eq!(
            awareness.cursor.as_ref().unwrap().selection.as_ref().unwrap().anchor,
            deserialized.cursor.as_ref().unwrap().selection.as_ref().unwrap().anchor
        );
    }

    /// Test sync message with Yjs update format
    #[test]
    fn test_sync_message() {
        let sync = SyncMessage {
            state_vector: Some(vec![1, 0, 1, 0]),
            update: Some(vec![0, 1, 0, 1, 1, 0]),
        };

        let serialized = serde_json::to_string(&sync).expect("Failed to serialize");
        let deserialized: SyncMessage = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(sync.state_vector, deserialized.state_vector);
        assert_eq!(sync.update, deserialized.update);
    }

    // =========================================================================
    // Presence State Tests
    // =========================================================================

    /// Test presence entry creation and activity tracking
    #[test]
    fn test_presence_entry_activity() {
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        let mut entry = PresenceEntry {
            user_id,
            display_name: "Test User".to_string(),
            color: "#4ECDC4".to_string(),
            cursor: None,
            last_active: Utc::now(),
            document_id,
        };

        // Initially active
        assert!(entry.is_active(30));

        // After 60 seconds, should be inactive
        entry.last_active = Utc::now() - chrono::Duration::seconds(60);
        assert!(!entry.is_active(30));
    }

    /// Test presence store operations
    #[test]
    fn test_presence_store_operations() {
        let store = PresenceStore::new();
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        // Set presence
        let entry = PresenceEntry {
            user_id,
            display_name: "Test User".to_string(),
            color: "#45B7D1".to_string(),
            cursor: None,
            last_active: Utc::now(),
            document_id,
        };
        store.set_presence(entry.clone());

        // Verify presence was set
        let retrieved = store.get_presence(&user_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().display_name, "Test User");

        // Update cursor
        let cursor = CursorState {
            x: 150.0,
            y: 200.0,
            selection: None,
        };
        store.update_cursor(&user_id, &cursor);

        let updated = store.get_presence(&user_id).unwrap();
        assert!(updated.cursor.is_some());
        assert_eq!(updated.cursor.unwrap().x, 150.0);

        // Remove presence
        store.remove_presence(&user_id);
        assert!(store.get_presence(&user_id).is_none());
    }

    /// Test document-specific presence filtering
    #[test]
    fn test_document_presence_filtering() {
        let store = PresenceStore::new();
        let doc_id_1 = Uuid::new_v4();
        let doc_id_2 = Uuid::new_v4();

        // Add users to document 1
        for i in 0..3 {
            let entry = PresenceEntry {
                user_id: Uuid::new_v4(),
                display_name: format!("User {}", i),
                color: format!("#{:02X}0000", i * 50),
                cursor: None,
                last_active: Utc::now(),
                document_id: doc_id_1,
            };
            store.set_presence(entry);
        }

        // Add user to document 2
        let entry = PresenceEntry {
            user_id: Uuid::new_v4(),
            display_name: "Other User".to_string(),
            color: "#000000".to_string(),
            cursor: None,
            last_active: Utc::now(),
            document_id: doc_id_2,
        };
        store.set_presence(entry);

        // Verify filtering
        let doc1_users = store.get_document_presence(&doc_id_1);
        assert_eq!(doc1_users.len(), 3);

        let doc2_users = store.get_document_presence(&doc_id_2);
        assert_eq!(doc2_users.len(), 1);

        let all_users = store.get_all_presence();
        assert_eq!(all_users.len(), 4);
    }

    /// Test stale entry cleanup
    #[test]
    fn test_stale_entry_cleanup() {
        let store = PresenceStore::new();
        let doc_id = Uuid::new_v4();

        // Add active user
        let active_entry = PresenceEntry {
            user_id: Uuid::new_v4(),
            display_name: "Active User".to_string(),
            color: "#00FF00".to_string(),
            cursor: None,
            last_active: Utc::now(),
            document_id: doc_id,
        };
        store.set_presence(active_entry);

        // Add stale user (inactive for 60 seconds)
        let stale_entry = PresenceEntry {
            user_id: Uuid::new_v4(),
            display_name: "Stale User".to_string(),
            color: "#FF0000".to_string(),
            cursor: None,
            last_active: Utc::now() - chrono::Duration::seconds(60),
            document_id: doc_id,
        };
        store.set_presence(stale_entry);

        // Cleanup with 30-second timeout
        store.cleanup_stale_entries(30);

        let remaining = store.get_document_presence(&doc_id);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].display_name, "Active User");
    }

    // =========================================================================
    // WebSocket Connection Tests
    // =========================================================================

    /// Test WebSocket frame handling
    #[test]
    fn test_websocket_frame_handling() {
        let test_data = b"Hello, WebSocket!";
        let frame = ws::Message::Text(String::from_utf8_lossy(test_data).to_string());

        match frame {
            ws::Message::Text(text) => {
                assert_eq!(text, "Hello, WebSocket!");
            }
            _ => panic!("Expected Text message"),
        }
    }

    /// Test ping/pong message handling
    #[test]
    fn test_ping_pong_messages() {
        let ping = ProtocolMessage {
            type_: MessageType::Ping,
            document_id: Uuid::nil(),
            user_id: Uuid::nil(),
            payload: json!({"timestamp": Utc::now().timestamp()}),
        };

        let pong_response = ProtocolMessage {
            type_: MessageType::Pong,
            document_id: ping.document_id,
            user_id: ping.user_id,
            payload: json!({
                "original_timestamp": ping.payload["timestamp"],
                "response_timestamp": Utc::now().timestamp()
            }),
        };

        assert_ne!(ping.type_, pong_response.type_);
        assert!(pong_response.type_ == MessageType::Pong);
    }

    // =========================================================================
    // Broadcast Channel Tests
    // =========================================================================

    #[tokio::test]
    async fn test_broadcast_channel() {
        let (tx, _rx) = broadcast::channel::<ProtocolMessage>(100);

        let message = ProtocolMessage {
            type_: MessageType::UserJoin,
            document_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            payload: json!({"display_name": "New User"}),
        };

        // Send message
        tx.send(message.clone()).expect("Failed to send");

        // Receive message
        let mut rx = tx.subscribe();
        let received = rx.recv().await.expect("Failed to receive");

        assert_eq!(message.type_, received.type_);
        assert_eq!(message.user_id, received.user_id);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let (tx, _rx) = broadcast::channel::<ProtocolMessage>(100);

        let message = ProtocolMessage {
            type_: MessageType::DocumentUpdate,
            document_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            payload: json!({"update": [1, 2, 3]}),
        };

        tx.send(message.clone()).expect("Failed to send");

        // Multiple subscribers should all receive
        let mut rx1 = tx.subscribe();
        let mut rx2 = tx.subscribe();

        let received1 = rx1.recv().await.expect("Subscriber 1 failed");
        let received2 = rx2.recv().await.expect("Subscriber 2 failed");

        assert_eq!(received1, received2);
    }

    // =========================================================================
    // Integration Scenarios
    // =========================================================================

    /// Test complete user join flow
    #[tokio::test]
    async fn test_user_join_flow() {
        let store = PresenceStore::new();
        let document_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Simulate user joining
        let join_message = ProtocolMessage {
            type_: MessageType::UserJoin,
            document_id,
            user_id,
            payload: json!({
                "display_name": "John Doe",
                "color": "#FF6B6B"
            }),
        };

        // Create presence entry
        let entry = PresenceEntry {
            user_id,
            display_name: "John Doe".to_string(),
            color: "#FF6B6B".to_string(),
            cursor: None,
            last_active: Utc::now(),
            document_id,
        };
        store.set_presence(entry.clone());

        // Verify user is now in document presence
        let presence = store.get_document_presence(&document_id);
        assert_eq!(presence.len(), 1);
        assert_eq!(presence[0].display_name, "John Doe");
    }

    /// Test cursor movement broadcast flow
    #[tokio::test]
    async fn test_cursor_broadcast_flow() {
        let store = PresenceStore::new();
        let document_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Set initial presence
        let entry = PresenceEntry {
            user_id,
            display_name: "Test User".to_string(),
            color: "#4ECDC4".to_string(),
            cursor: None,
            last_active: Utc::now(),
            document_id,
        };
        store.set_presence(entry);

        // Simulate cursor update
        let cursor = CursorState {
            x: 250.0,
            y: 350.0,
            selection: Some(SelectionState {
                anchor: 100,
                head: 150,
            }),
        };

        store.update_cursor(&user_id, &cursor);

        // Verify cursor was updated
        let presence = store.get_presence(&user_id).unwrap();
        assert!(presence.cursor.is_some());
        assert_eq!(presence.cursor.unwrap().x, 250.0);
    }

    /// Test user leave cleanup flow
    #[tokio::test]
    async fn test_user_leave_flow() {
        let store = PresenceStore::new();
        let document_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // User joins
        let entry = PresenceEntry {
            user_id,
            display_name: "Leaving User".to_string(),
            color: "#FF6B6B".to_string(),
            cursor: None,
            last_active: Utc::now(),
            document_id,
        };
        store.set_presence(entry.clone());

        assert!(store.get_presence(&user_id).is_some());

        // User leaves
        store.remove_presence(&user_id);

        // Verify user removed
        assert!(store.get_presence(&user_id).is_none());
        let presence = store.get_document_presence(&document_id);
        assert!(presence.is_empty());
    }

    /// Test concurrent updates safety
    #[test]
    fn test_concurrent_updates() {
        let store = PresenceStore::new();
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        // Simulate concurrent updates
        let mut handles = Vec::new();

        for i in 0..10 {
            let store_clone = store.clone();
            let handle = std::thread::spawn(move || {
                let entry = PresenceEntry {
                    user_id,
                    display_name: format!("User Update {}", i),
                    color: format!("#{:02X}0000", i * 20),
                    cursor: None,
                    last_active: Utc::now(),
                    document_id,
                };
                store_clone.set_presence(entry);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify final state is consistent
        let presence = store.get_presence(&user_id);
        assert!(presence.is_some());
        // Last write wins - any of the 10 updates is acceptable
        assert!(presence.unwrap().display_name.starts_with("User Update"));
    }
}

// =========================================================================
// Data Structures
// =========================================================================

/// WebSocket protocol message types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "sync")]
    Sync,
    #[serde(rename = "awareness")]
    Awareness,
    #[serde(rename = "cursor")]
    Cursor,
    #[serde(rename = "document_update")]
    DocumentUpdate,
    #[serde(rename = "user_join")]
    UserJoin,
    #[serde(rename = "user_leave")]
    UserLeave,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "error")]
    Error,
}

/// Protocol message structure for WebSocket communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub type_: MessageType,
    pub document_id: Uuid,
    pub user_id: Uuid,
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Yjs sync message with state vector and update
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncMessage {
    pub state_vector: Option<Vec<u8>>,
    pub update: Option<Vec<u8>>,
}

/// User awareness state for presence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwarenessState {
    pub user: UserAwareness,
    pub cursor: Option<CursorState>,
    #[serde(default)]
    pub local_state: serde_json::Value,
}

/// User information for awareness
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAwareness {
    pub user_id: Uuid,
    pub display_name: String,
    pub color: String,
}

/// Cursor position and selection state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorState {
    pub x: f64,
    pub y: f64,
    pub selection: Option<SelectionState>,
}

/// Text selection range
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionState {
    pub anchor: usize,
    pub head: usize,
}

/// Presence entry for tracking online users
#[derive(Debug, Clone, PartialEq)]
pub struct PresenceEntry {
    pub user_id: Uuid,
    pub display_name: String,
    pub color: String,
    pub cursor: Option<CursorState>,
    pub last_active: DateTime<Utc>,
    pub document_id: Uuid,
}

impl PresenceEntry {
    /// Check if entry is active within timeout seconds
    pub fn is_active(&self, timeout_secs: i64) -> bool {
        let timeout = chrono::Duration::seconds(timeout_secs);
        Utc::now() - self.last_active < timeout
    }
}

/// Thread-safe presence store
#[derive(Clone, Default)]
pub struct PresenceStore {
    entries: Arc<Mutex<HashMap<Uuid, PresenceEntry>>>,
}

impl PresenceStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_presence(&self, entry: PresenceEntry) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(entry.user_id, entry);
    }

    pub fn remove_presence(&self, user_id: &Uuid) {
        let mut entries = self.entries.lock().unwrap();
        entries.remove(user_id);
    }

    pub fn get_presence(&self, user_id: &Uuid) -> Option<PresenceEntry> {
        let entries = self.entries.lock().unwrap();
        entries.get(user_id).cloned()
    }

    pub fn get_document_presence(&self, document_id: &Uuid) -> Vec<PresenceEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .values()
            .filter(|e| e.document_id == *document_id)
            .cloned()
            .collect()
    }

    pub fn update_cursor(&self, user_id: &Uuid, cursor: &CursorState) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get_mut(user_id) {
            entry.cursor = Some(cursor.clone());
            entry.last_active = Utc::now();
        }
    }

    pub fn cleanup_stale_entries(&self, timeout_secs: i64) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|_, entry| entry.is_active(timeout_secs));
    }

    pub fn get_all_presence(&self) -> Vec<PresenceEntry> {
        let entries = self.entries.lock().unwrap();
        entries.values().cloned().collect()
    }
}
