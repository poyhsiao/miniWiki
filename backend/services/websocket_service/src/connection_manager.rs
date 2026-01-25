//! Connection Manager for WebSocket sessions
//!
//! This module provides:
//! - Connection tracking and management
//! - WebSocket session lifecycle
//! - Message broadcasting to connections
//! - Statistics tracking

use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;
use chrono::Utc;
use crate::{WebSocketSession, SESSION_STORE};

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    pub total_connections: u64,
    pub active_connections: u64,
    pub messages_sent: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub last_activity: Option<Instant>,
}

/// Connection manager for WebSocket sessions
#[derive(Clone)]
pub struct ConnectionManager {
    stats: Arc<Mutex<ConnectionStats>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(ConnectionStats::default())),
        }
    }

    pub fn register_connection(&self, session: &WebSocketSession) {
        let mut stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
        stats.total_connections += 1;
        stats.active_connections += 1;
        stats.last_activity = Some(Instant::now());
        SESSION_STORE.add_session(session.clone());
    }

    pub fn unregister_connection(&self, session_id: Uuid) {
        let mut stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
        stats.active_connections = stats.active_connections.saturating_sub(1);
        stats.last_activity = Some(Instant::now());
        SESSION_STORE.remove_session(session_id);
    }

    pub fn get_stats(&self) -> ConnectionStats {
        self.stats.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn record_message_sent(&self, bytes: u64) {
        let mut stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
        stats.messages_sent += 1;
        stats.bytes_sent += bytes;
        stats.last_activity = Some(Instant::now());
    }

    pub fn record_bytes_received(&self, bytes: u64) {
        let mut stats = self.stats.lock().unwrap_or_else(|e| e.into_inner());
        stats.bytes_received += bytes;
        stats.last_activity = Some(Instant::now());
    }

    pub fn is_session_active(&self, session: &WebSocketSession, timeout_secs: u64) -> bool {
        let elapsed = Utc::now().signed_duration_since(session.last_activity).num_seconds();
        elapsed < timeout_secs as i64
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global connection manager instance
pub static CONNECTION_MANAGER: once_cell::sync::Lazy<ConnectionManager> =
    once_cell::sync::Lazy::new(ConnectionManager::new);

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use chrono::Utc;

    // Test: ConnectionStats initialization
    #[test]
    fn test_connection_stats_initialization() {
        let stats = ConnectionStats::default();

        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.last_activity, None);
    }

    // Test: ConnectionManager registration
    #[test]
    fn test_connection_registration() {
        let manager = ConnectionManager::new();
        let session = WebSocketSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test User".to_string(),
            "#FF0000".to_string(),
        );

        manager.register_connection(&session);

        let stats = manager.get_stats();
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 1);
    }

    // Test: ConnectionManager unregistration
    #[test]
    fn test_connection_unregistration() {
        let manager = ConnectionManager::new();
        let session = WebSocketSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test User".to_string(),
            "#FF0000".to_string(),
        );

        manager.register_connection(&session);
        let session_id = session.id;

        manager.unregister_connection(session_id);

        let stats = manager.get_stats();
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 0);
    }

    // Test: message sending updates stats
    #[test]
    fn test_message_sending_updates_stats() {
        let manager = ConnectionManager::new();

        manager.record_message_sent(100);

        let stats = manager.get_stats();
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.bytes_sent, 100);
    }

    // Test: multiple messages increment correctly
    #[test]
    fn test_multiple_messages() {
        let manager = ConnectionManager::new();

        for _ in 0..5 {
            manager.record_message_sent(60);
        }

        let stats = manager.get_stats();
        assert_eq!(stats.messages_sent, 5);
        assert_eq!(stats.bytes_sent, 300);
    }

    // Test: statistics tracking
    #[test]
    fn test_statistics_tracking() {
        let manager = ConnectionManager::new();

        manager.record_bytes_received(100);

        let stats = manager.get_stats();
        assert_eq!(stats.bytes_received, 100);
    }

    // Test: last activity tracking
    #[test]
    fn test_last_activity_tracking() {
        let manager = ConnectionManager::new();

        manager.record_message_sent(50);

        let stats = manager.get_stats();
        assert!(stats.last_activity.is_some());
    }

    // Test: concurrent connection management
    #[test]
    fn test_concurrent_connections() {
        let manager = ConnectionManager::new();

        for _ in 0..4 {
            let session = WebSocketSession::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Test User".to_string(),
                "#FF0000".to_string(),
            );
            manager.register_connection(&session);
        }

        let stats = manager.get_stats();
        assert_eq!(stats.total_connections, 4);
        assert_eq!(stats.active_connections, 4);
    }

    // Test: connection disconnection reduces active count
    #[test]
    fn test_connection_disconnection() {
        let manager = ConnectionManager::new();
        let session1 = WebSocketSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "User 1".to_string(),
            "#FF0000".to_string(),
        );
        let session2 = WebSocketSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "User 2".to_string(),
            "#00FF00".to_string(),
        );

        manager.register_connection(&session1);
        manager.register_connection(&session2);

        let stats = manager.get_stats();
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.active_connections, 2);

        manager.unregister_connection(session1.id);

        let stats = manager.get_stats();
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.active_connections, 1);
    }

    // Test: session timeout tracking
    #[test]
    fn test_session_timeout_tracking() {
        let manager = ConnectionManager::new();
        let session = WebSocketSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test User".to_string(),
            "#FF0000".to_string(),
        );

        // Session should be active
        assert!(manager.is_session_active(&session, 300));

        // Simulate old activity
        let old_session = WebSocketSession {
            last_activity: Utc::now() - chrono::Duration::seconds(400),
            ..session
        };

        assert!(!manager.is_session_active(&old_session, 300));
    }

    // Test: byte tracking accuracy
    #[test]
    fn test_byte_tracking_accuracy() {
        let manager = ConnectionManager::new();

        let message_size = 1024;
        for _ in 0..10 {
            manager.record_message_sent(message_size);
        }

        let stats = manager.get_stats();
        assert_eq!(stats.bytes_sent, 10240);
    }

    // Test: connection pool management
    #[test]
    fn test_connection_pool() {
        let pool = Arc::new(Mutex::new(ConnectionStats::default()));

        // Lock the pool
        let _guard = pool.lock().unwrap();
        drop(_guard);

        // Verify we can lock again
        let _guard2 = pool.lock().unwrap();
    }

    // Test: default connection stats
    #[test]
    fn test_default_connection_stats() {
        let stats = ConnectionStats::default();

        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.last_activity, None);
    }

    // Test: large number of connections
    #[test]
    fn test_large_connection_count() {
        let manager = ConnectionManager::new();

        let sessions: Vec<_> = (0..1000)
            .map(|_| {
                WebSocketSession::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    "Test".to_string(),
                    "#FF0000".to_string(),
                )
            })
            .collect();

        for session in &sessions {
            manager.register_connection(session);
        }

        let stats = manager.get_stats();
        assert_eq!(stats.total_connections, 1000);
        assert_eq!(stats.active_connections, 1000);
    }

    // Test: message count increment
    #[test]
    fn test_message_count_increment() {
        let manager = ConnectionManager::new();

        for _ in 0..100 {
            manager.record_message_sent(60);
        }

        let stats = manager.get_stats();
        assert_eq!(stats.messages_sent, 100);
    }

    // Test: activity timestamp updates
    #[test]
    fn test_activity_timestamp_updates() {
        let manager = ConnectionManager::new();

        manager.record_message_sent(100);
        let time1 = manager.get_stats().last_activity.unwrap();

        std::thread::sleep(Duration::from_millis(10));

        manager.record_message_sent(200);
        let time2 = manager.get_stats().last_activity.unwrap();

        assert_ne!(time1, time2);
        assert!(time2 > time1);
    }

    // Test: ConnectionManager default implementation
    #[test]
    fn test_connection_manager_default() {
        let manager = ConnectionManager::default();

        let stats = manager.get_stats();
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
    }
}
