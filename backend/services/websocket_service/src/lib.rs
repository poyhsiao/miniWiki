use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod handlers;
pub mod models;
pub mod presence;
pub mod connection_manager;
pub mod actor;
pub mod redis_pubsub;

pub use handlers::*;
pub use models::*;
pub use presence::*;
pub use actor::*;
pub use redis_pubsub::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: Uuid,
    pub display_name: String,
    pub color: String,
    pub cursor: Option<CursorPosition>,
    pub last_active: DateTime<Utc>,
}

impl Default for UserPresence {
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
pub enum WebSocketMessageType {
    Sync,
    Awareness,
    Cursor,
    DocumentUpdate,
    UserJoin,
    UserLeave,
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub type_: WebSocketMessageType,
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl WebSocketMessage {
    pub fn new(
        type_: WebSocketMessageType,
        document_id: Uuid,
        user_id: Uuid,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            type_,
            document_id,
            user_id,
            payload,
            timestamp: Utc::now(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentState {
    pub document_id: Uuid,
    pub state_vector: Vec<u8>,
    pub update: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwarenessUpdate {
    pub user_id: Uuid,
    pub state: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub state_vector: Option<Vec<u8>>,
    pub update: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct WebSocketSession {
    pub id: Uuid,
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub color: String,
    pub last_activity: DateTime<Utc>,
}

impl WebSocketSession {
    pub fn new(document_id: Uuid, user_id: Uuid, display_name: String, color: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            document_id,
            user_id,
            display_name,
            color,
            last_activity: Utc::now(),
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }
}

#[derive(Default)]
pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<Uuid, Arc<Mutex<WebSocketSession>>>>>,
    document_sessions: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
    user_sessions: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            document_sessions: Arc::new(Mutex::new(HashMap::new())),
            user_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_session(&self, session: WebSocketSession) {
        let mut sessions = self.sessions.lock().unwrap();
        let mut document_sessions = self.document_sessions.lock().unwrap();
        let mut user_sessions = self.user_sessions.lock().unwrap();

        let session_id = session.id;
        let document_id = session.document_id;
        let user_id = session.user_id;

        sessions.insert(session_id, Arc::new(Mutex::new(session)));

        document_sessions
            .entry(document_id)
            .or_insert_with(Vec::new)
            .push(session_id);

        user_sessions
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(session_id);
    }

    pub fn remove_session(&self, session_id: Uuid) {
        let mut sessions = self.sessions.lock().unwrap();
        let mut document_sessions = self.document_sessions.lock().unwrap();
        let mut user_sessions = self.user_sessions.lock().unwrap();

        if let Some(session_arc) = sessions.remove(&session_id) {
            let session = session_arc.lock().unwrap();
            let document_id = session.document_id;
            let user_id = session.user_id;

            if let Some(doc_sessions) = document_sessions.get_mut(&document_id) {
                doc_sessions.retain(|id| *id != session_id);
            }

            if let Some(user_session_list) = user_sessions.get_mut(&user_id) {
                user_session_list.retain(|id| *id != session_id);
            }
        }
    }

    pub fn get_session(&self, session_id: Uuid) -> Option<Arc<Mutex<WebSocketSession>>> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(&session_id).cloned()
    }

    pub fn get_document_sessions(&self, document_id: Uuid) -> Vec<Arc<Mutex<WebSocketSession>>> {
        let sessions = self.sessions.lock().unwrap();
        let document_sessions = self.document_sessions.lock().unwrap();

        if let Some(session_ids) = document_sessions.get(&document_id) {
            session_ids
                .iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_user_sessions(&self, user_id: Uuid) -> Vec<Arc<Mutex<WebSocketSession>>> {
        let sessions = self.sessions.lock().unwrap();
        let user_sessions = self.user_sessions.lock().unwrap();

        if let Some(session_ids) = user_sessions.get(&user_id) {
            session_ids
                .iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub static SESSION_STORE: once_cell::sync::Lazy<SessionStore> =
    once_cell::sync::Lazy::new(SessionStore::new);
