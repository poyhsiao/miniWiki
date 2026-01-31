//! Redis Pub/Sub for Multi-Instance Presence
//!
//! This module provides Redis-based pub/sub for synchronizing presence
//! across multiple backend instances. This is essential for real-time
//! collaboration when the backend is horizontally scaled.

use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};
use uuid::Uuid;

use crate::{CursorPosition, PRESENCE_STORE};

/// Channel prefix for Redis pub/sub
const REDIS_CHANNEL_PREFIX: &str = "miniwiki:ws:";

/// Redis connection configuration
#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>,
    pub db: i64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            password: std::env::var("REDIS_PASSWORD").ok(),
            db: std::env::var("REDIS_DB")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
        }
    }
}

/// Redis message types for pub/sub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedisMessage {
    /// Broadcast presence update to all instances
    PresenceUpdate {
        document_id: Uuid,
        user_id: Uuid,
        display_name: String,
        color: String,
        cursor: Option<CursorPosition>,
    },
    /// Notify other instances that a user joined
    UserJoin {
        document_id: Uuid,
        user_id: Uuid,
        display_name: String,
        color: String,
    },
    /// Notify other instances that a user left
    UserLeave { document_id: Uuid, user_id: Uuid },
    /// Broadcast document update to all instances
    DocumentUpdate {
        document_id: Uuid,
        user_id: Uuid,
        update: Vec<u8>,
    },
    /// Cursor position update
    CursorUpdate {
        document_id: Uuid,
        user_id: Uuid,
        cursor: CursorPosition,
    },
}

impl RedisMessage {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn channel(&self) -> String {
        match self {
            RedisMessage::PresenceUpdate { document_id, .. } => {
                format!("{}presence:{}", REDIS_CHANNEL_PREFIX, document_id)
            },
            RedisMessage::UserJoin { document_id, .. } => format!("{}join:{}", REDIS_CHANNEL_PREFIX, document_id),
            RedisMessage::UserLeave { document_id, .. } => format!("{}leave:{}", REDIS_CHANNEL_PREFIX, document_id),
            RedisMessage::DocumentUpdate { document_id, .. } => format!("{}doc:{}", REDIS_CHANNEL_PREFIX, document_id),
            RedisMessage::CursorUpdate { document_id, .. } => format!("{}cursor:{}", REDIS_CHANNEL_PREFIX, document_id),
        }
    }
}

use redis::aio::MultiplexedConnection;

/// Redis Pub/Sub Manager for WebSocket presence
#[derive(Clone)]
pub struct RedisPubSubManager {
    client: Arc<RedisClient>,
    _config: RedisConfig,
    connection: Arc<tokio::sync::Mutex<Option<MultiplexedConnection>>>,
    local_sender: Arc<tokio::sync::Mutex<Option<broadcast::Sender<RedisMessage>>>>,
    subscribed_channels: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl RedisPubSubManager {
    pub async fn new(config: Option<RedisConfig>) -> Result<Self, redis::RedisError> {
        let config = config.unwrap_or_default();

        let client = redis::Client::open(config.url.clone())?;
        // Initialize the connection immediately - this will fail startup if Redis is unavailable
        let connection = client.get_multiplexed_async_connection().await?;

        info!("Connected to Redis at {}", config.url);

        Ok(Self {
            client: Arc::new(client),
            _config: config,
            connection: Arc::new(tokio::sync::Mutex::new(Some(connection))),
            local_sender: Arc::new(tokio::sync::Mutex::new(None)),
            subscribed_channels: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        })
    }

    async fn get_connection(&self) -> Result<MultiplexedConnection, redis::RedisError> {
        let mut guard = self.connection.lock().await;

        if let Some(conn) = guard.as_ref() {
            // Cloning MultiplexedConnection is cheap and is the intended way to share it
            return Ok(conn.clone());
        }

        // Try to reconnect
        match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => {
                *guard = Some(conn.clone());
                Ok(conn)
            },
            Err(e) => Err(e),
        }
    }

    pub async fn get_local_receiver(&self) -> broadcast::Receiver<RedisMessage> {
        let mut guard = self.local_sender.lock().await;

        if guard.is_none() {
            let (sender, _receiver) = broadcast::channel(1000);
            *guard = Some(sender);
        }

        guard.as_ref().unwrap().subscribe()
    }

    async fn subscribe_to_channel(&self, channel: &str) -> Result<(), redis::RedisError> {
        let mut subscribed = self.subscribed_channels.lock().await;

        if subscribed.contains(&channel.to_string()) {
            return Ok(());
        }

        subscribed.push(channel.to_string());
        info!("Subscribed to Redis channel: {}", channel);

        Ok(())
    }

    pub async fn subscribe_to_document(&self, document_id: Uuid) {
        let channels = vec![
            format!("{}presence:{}", REDIS_CHANNEL_PREFIX, document_id),
            format!("{}join:{}", REDIS_CHANNEL_PREFIX, document_id),
            format!("{}leave:{}", REDIS_CHANNEL_PREFIX, document_id),
            format!("{}cursor:{}", REDIS_CHANNEL_PREFIX, document_id),
            format!("{}doc:{}", REDIS_CHANNEL_PREFIX, document_id),
        ];

        for channel in channels {
            if let Err(e) = self.subscribe_to_channel(&channel).await {
                error!("Failed to subscribe to channel {}: {}", channel, e);
            }
        }
    }

    pub async fn publish(&self, message: &RedisMessage) -> Result<(), redis::RedisError> {
        let channel = message.channel();
        let json = match message.to_json() {
            Ok(j) => j,
            Err(e) => {
                // In redis 1.0.2, ResponseError moved to ServerErrorKind
                return Err(redis::RedisError::from((
                    redis::ErrorKind::Server(redis::ServerErrorKind::ResponseError),
                    "Failed to serialize message",
                    e.to_string(),
                )));
            },
        };

        // Get the cached connection or reconnect
        let mut connection = self.get_connection().await?;

        // Attempt to publish
        if let Err(e) = connection.publish::<&str, &str, ()>(&channel, &json).await {
            // If publish fails, it might be because the connection is dead.
            // Clear the cached connection and try one more time.
            error!("Redis publish failed: {}. Attempting to reconnect...", e);

            {
                let mut guard = self.connection.lock().await;
                *guard = None;
            }

            let mut connection = self.get_connection().await?;
            connection.publish::<&str, &str, ()>(&channel, &json).await?;
        }

        Ok(())
    }

    pub async fn broadcast_user_join(&self, document_id: Uuid, user_id: Uuid, display_name: String, color: String) {
        let message = RedisMessage::UserJoin {
            document_id,
            user_id,
            display_name,
            color,
        };

        if let Err(e) = self.publish(&message).await {
            error!("Failed to broadcast user join: {}", e);
        }
    }

    pub async fn broadcast_user_leave(&self, document_id: Uuid, user_id: Uuid) {
        let message = RedisMessage::UserLeave { document_id, user_id };

        if let Err(e) = self.publish(&message).await {
            error!("Failed to broadcast user leave: {}", e);
        }
    }

    pub async fn broadcast_cursor_update(&self, document_id: Uuid, user_id: Uuid, cursor: CursorPosition) {
        let message = RedisMessage::CursorUpdate {
            document_id,
            user_id,
            cursor,
        };

        if let Err(e) = self.publish(&message).await {
            error!("Failed to broadcast cursor update: {}", e);
        }
    }

    pub async fn broadcast_document_update(&self, document_id: Uuid, user_id: Uuid, update: Vec<u8>) {
        let message = RedisMessage::DocumentUpdate {
            document_id,
            user_id,
            update,
        };

        if let Err(e) = self.publish(&message).await {
            error!("Failed to broadcast document update: {}", e);
        }
    }

    pub async fn handle_redis_message(&self, message: RedisMessage) {
        match message {
            RedisMessage::UserJoin {
                document_id,
                user_id,
                display_name,
                color,
            } => {
                let entry = crate::presence::PresenceEntry::new(user_id, display_name, color, document_id);
                PRESENCE_STORE.set_presence(entry);
            },
            RedisMessage::UserLeave {
                document_id: _,
                user_id,
            } => {
                PRESENCE_STORE.remove_presence(user_id);
            },
            RedisMessage::CursorUpdate {
                document_id: _,
                user_id,
                cursor,
            } => {
                PRESENCE_STORE.update_cursor(user_id, cursor);
            },
            RedisMessage::PresenceUpdate {
                document_id,
                user_id,
                display_name,
                color,
                cursor,
            } => {
                let mut entry = crate::presence::PresenceEntry::new(user_id, display_name, color, document_id);
                entry.cursor = cursor;
                PRESENCE_STORE.set_presence(entry);
            },
            RedisMessage::DocumentUpdate {
                document_id: _,
                user_id: _,
                update: _,
            } => {
                // Document update handling would trigger sync with connected clients
            },
        }
    }
}

/// Global Redis pub/sub manager instance
pub static REDIS_PUBSUB: once_cell::sync::Lazy<tokio::sync::RwLock<Option<Arc<RedisPubSubManager>>>> =
    once_cell::sync::Lazy::new(|| tokio::sync::RwLock::new(None));

/// Initialize the Redis pub/sub manager
pub async fn init_redis_pubsub() -> Result<(), redis::RedisError> {
    let manager = RedisPubSubManager::new(None).await?;
    let mut guard = REDIS_PUBSUB.write().await;
    *guard = Some(Arc::new(manager));
    info!("Redis pub/sub manager initialized");
    Ok(())
}

/// Get the Redis pub/sub manager
pub async fn get_redis_pubsub() -> Option<Arc<RedisPubSubManager>> {
    let guard = REDIS_PUBSUB.read().await;
    guard.clone()
}

/// Shutdown Redis pub/sub manager
pub async fn shutdown_redis_pubsub() {
    let mut guard = REDIS_PUBSUB.write().await;
    *guard = None;
    info!("Redis pub/sub manager shutdown");
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_redis_message_channel() {
        let doc_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let join_msg = RedisMessage::UserJoin {
            document_id: doc_id,
            user_id,
            display_name: "Test User".to_string(),
            color: "#FF0000".to_string(),
        };
        assert!(join_msg.channel().contains(&doc_id.to_string()));

        let leave_msg = RedisMessage::UserLeave {
            document_id: doc_id,
            user_id,
        };
        assert!(leave_msg.channel().contains(&doc_id.to_string()));

        let cursor_msg = RedisMessage::CursorUpdate {
            document_id: doc_id,
            user_id,
            cursor: CursorPosition {
                x: 100.0,
                y: 200.0,
                selection_start: None,
                selection_end: None,
            },
        };
        assert!(cursor_msg.channel().contains(&doc_id.to_string()));
    }

    #[test]
    fn test_redis_message_serialization() {
        let doc_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let join_msg = RedisMessage::UserJoin {
            document_id: doc_id,
            user_id,
            display_name: "Test User".to_string(),
            color: "#FF0000".to_string(),
        };

        let json = join_msg.to_json().expect("Failed to serialize");
        let decoded = RedisMessage::from_json(&json).expect("Failed to deserialize");

        match decoded {
            RedisMessage::UserJoin {
                document_id: decoded_doc_id,
                user_id: decoded_user_id,
                display_name: decoded_display_name,
                color: decoded_color,
            } => {
                assert_eq!(decoded_doc_id, doc_id);
                assert_eq!(decoded_user_id, user_id);
                assert_eq!(decoded_display_name, "Test User");
                assert_eq!(decoded_color, "#FF0000");
            },
            _ => panic!("Wrong message type"),
        }
    }
}
