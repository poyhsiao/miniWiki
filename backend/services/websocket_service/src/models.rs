use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Sync,
    Awareness,
    Cursor,
    DocumentUpdate,
    UserJoin,
    UserLeave,
    Ping,
    Pong,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub type_: MessageType,
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMessage {
    pub type_: MessageType,
    pub document_id: Uuid,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMessage {
    pub state_vector: Option<Vec<u8>>,
    pub update: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwarenessMessage {
    pub state: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    pub user_id: Uuid,
    pub display_name: String,
    pub color: String,
    pub cursor: Option<super::CursorPosition>,
    pub last_active: DateTime<Utc>,
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            user_id: Uuid::nil(),
            display_name: String::new(),
            color: String::new(),
            cursor: None,
            last_active: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAwareness {
    pub document_id: Uuid,
    pub users: HashMap<Uuid, UserState>,
    pub local_state: serde_json::Value,
}

impl DocumentAwareness {
    pub fn new(document_id: Uuid) -> Self {
        Self {
            document_id,
            users: HashMap::new(),
            local_state: serde_json::Value::Null,
        }
    }

    pub fn add_user(&mut self, user: UserState) {
        self.users.insert(user.user_id, user);
    }

    pub fn remove_user(&mut self, user_id: Uuid) {
        self.users.remove(&user_id);
    }

    pub fn update_cursor(&mut self, user_id: Uuid, cursor: super::CursorPosition) {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.cursor = Some(cursor);
            user.last_active = Utc::now();
        }
    }

    pub fn get_users(&self) -> Vec<&UserState> {
        self.users.values().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    pub document_id: Uuid,
    pub state: Vec<u8>,
    pub vector: Vec<u8>,
    pub last_modified: DateTime<Utc>,
}

impl DocumentState {
    pub fn new(document_id: Uuid) -> Self {
        Self {
            document_id,
            state: Vec::new(),
            vector: Vec::new(),
            last_modified: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub session_id: Uuid,
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub connected_at: DateTime<Utc>,
    pub last_ping: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}
