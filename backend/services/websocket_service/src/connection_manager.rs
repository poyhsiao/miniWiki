use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::WebSocketSession;

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_activity: Option<DateTime<Utc>>,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            last_activity: None,
        }
    }
}

impl ConnectionStats {
    pub fn increment_connections(&mut self) {
        self.total_connections += 1;
        self.active_connections += 1;
    }

    pub fn decrement_connections(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    pub fn record_sent(&mut self, size: usize) {
        self.messages_sent += 1;
        self.bytes_sent += size as u64;
        self.last_activity = Some(Utc::now());
    }

    pub fn record_received(&mut self, size: usize) {
        self.messages_received += 1;
        self.bytes_received += size as u64;
        self.last_activity = Some(Utc::now());
    }
}

pub struct ConnectionManager {
    connections: Arc<Mutex<HashMap<Uuid, Arc<Mutex<WebSocketSession>>>>>,
    document_connections: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
    user_connections: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
    stats: Arc<Mutex<ConnectionStats>>,
    last_cleanup: Arc<Mutex<Instant>>,
    inactivity_timeout: Duration,
}

impl ConnectionManager {
    pub fn new(inactivity_timeout: Option<Duration>) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            document_connections: Arc::new(Mutex::new(HashMap::new())),
            user_connections: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ConnectionStats::default())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
            inactivity_timeout: inactivity_timeout.unwrap_or(Duration::from_secs(300)),
        }
    }

    pub fn add_connection(&self, session: WebSocketSession) {
        let mut connections = self.connections.lock().unwrap();
        let mut document_connections = self.document_connections.lock().unwrap();
        let mut user_connections = self.user_connections.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let session_id = session.id;
        let document_id = session.document_id;
        let user_id = session.user_id;

        connections.insert(session_id, Arc::new(Mutex::new(session)));

        document_connections
            .entry(document_id)
            .or_insert_with(Vec::new)
            .push(session_id);

        user_connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(session_id);

        stats.increment_connections();

        tracing::info!(
            "Added connection {} for user {} in document {}",
            session_id,
            user_id,
            document_id
        );
    }

    pub fn remove_connection(&self, session_id: Uuid) {
        let mut connections = self.connections.lock().unwrap();
        let mut document_connections = self.document_connections.lock().unwrap();
        let mut user_connections = self.user_connections.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if let Some(session_arc) = connections.remove(&session_id) {
            let session = session_arc.lock().unwrap();
            let document_id = session.document_id;
            let user_id = session.user_id;

            if let Some(conns) = document_connections.get_mut(&document_id) {
                conns.retain(|id| *id != session_id);
            }

            if let Some(conns) = user_connections.get_mut(&user_id) {
                conns.retain(|id| *id != session_id);
            }

            stats.decrement_connections();

            tracing::info!(
                "Removed connection {} for user {} in document {}",
                session_id,
                user_id,
                document_id
            );
        }
    }

    pub fn get_connection(&self, session_id: Uuid) -> Option<Arc<Mutex<WebSocketSession>>> {
        let connections = self.connections.lock().unwrap();
        connections.get(&session_id).cloned()
    }

    pub fn get_document_connections(&self, document_id: Uuid) -> Vec<Arc<Mutex<WebSocketSession>>> {
        let connections = self.connections.lock().unwrap();
        let document_connections = self.document_connections.lock().unwrap();

        if let Some(session_ids) = document_connections.get(&document_id) {
            session_ids
                .iter()
                .filter_map(|id| connections.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_user_connections(&self, user_id: Uuid) -> Vec<Arc<Mutex<WebSocketSession>>> {
        let connections = self.connections.lock().unwrap();
        let user_connections = self.user_connections.lock().unwrap();

        if let Some(session_ids) = user_connections.get(&user_id) {
            session_ids
                .iter()
                .filter_map(|id| connections.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_stats(&self) -> ConnectionStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    pub fn record_message_sent(&self, _session_id: Uuid, size: usize) {
        let mut stats = self.stats.lock().unwrap();
        stats.record_sent(size);
    }

    pub fn record_message_received(&self, _session_id: Uuid, size: usize) {
        let mut stats = self.stats.lock().unwrap();
        stats.record_received(size);
    }

    pub fn cleanup_stale_connections(&self) {
        let mut last_cleanup_guard = self.last_cleanup.lock().unwrap();
        let now = Instant::now();

        if last_cleanup_guard.duration_since(now) < Duration::from_secs(60) {
            return;
        }

        let mut connections = self.connections.lock().unwrap();
        let mut stale_ids = Vec::new();

        for (session_id, session_arc) in connections.iter() {
            let session = session_arc.lock().unwrap();
            let last_activity = session.last_activity;
            let elapsed = Utc::now().signed_duration_since(last_activity);
            let timeout_seconds = self.inactivity_timeout.as_secs() as i64;

            if elapsed.num_seconds() > timeout_seconds {
                stale_ids.push(*session_id);
            }
        }

        drop(connections);

        for session_id in &stale_ids {
            self.remove_connection(*session_id);
        }

        tracing::debug!("Cleaned up {} stale connections", stale_ids.len());

        *last_cleanup_guard = now;
    }

    pub fn get_active_document_count(&self) -> usize {
        let document_connections = self.document_connections.lock().unwrap();
        document_connections.len()
    }

    pub fn get_total_connection_count(&self) -> usize {
        let connections = self.connections.lock().unwrap();
        connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use uuid::Uuid;

    #[test]
    fn test_connection_manager_add_remove() {
        let manager = ConnectionManager::new(None);
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        let session = WebSocketSession::new(
            document_id,
            user_id,
            "Test User".to_string(),
            "#FF0000".to_string(),
        );

        let session_id = session.id;
        assert_eq!(manager.get_total_connection_count(), 0);

        manager.add_connection(session);
        assert_eq!(manager.get_total_connection_count(), 1);

        manager.remove_connection(session_id);
        assert_eq!(manager.get_total_connection_count(), 0);
    }

    #[test]
    fn test_connection_stats() {
        let mut stats = ConnectionStats::default();

        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);

        stats.increment_connections();
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.active_connections, 1);

        stats.increment_connections();
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.active_connections, 2);

        stats.decrement_connections();
        assert_eq!(stats.active_connections, 1);

        stats.record_sent(100);
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.bytes_sent, 100);

        stats.record_received(50);
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.bytes_received, 50);
    }

    #[test]
    fn test_get_document_connections() {
        let manager = ConnectionManager::new(None);
        let document_id = Uuid::new_v4();

        for i in 0..3 {
            let session = WebSocketSession::new(
                document_id,
                Uuid::new_v4(),
                format!("User {}", i),
                format!("#{:02X}0000", i * 50),
            );
            manager.add_connection(session);
        }

        let other_session = WebSocketSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Other User".to_string(),
            "#000000".to_string(),
        );
        let other_doc_id = other_session.document_id;
        manager.add_connection(other_session);
        let other_connections = manager.get_document_connections(other_doc_id);
        assert_eq!(other_connections.len(), 1);
    }

    #[test]
    fn test_time_delta_duration() {
        let duration = TimeDelta::try_seconds(300).unwrap();
        assert_eq!(duration.num_seconds(), 300);
    }
}
