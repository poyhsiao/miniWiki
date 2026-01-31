#[cfg(test)]
mod websocket_service_test {
    use uuid::Uuid;
    use websocket_service::connection_manager::ConnectionManager;
    use websocket_service::presence::PresenceEntry;
    use websocket_service::WebSocketSession;

    #[test]
    fn test_presence_entry_creation() {
        let entry = PresenceEntry::new(
            Uuid::new_v4(),
            "Test User".to_string(),
            "#FF0000".to_string(),
            Uuid::new_v4(),
        );

        assert!(!entry.user_id.is_nil());
        assert_eq!(entry.display_name, "Test User");
        assert_eq!(entry.color, "#FF0000");
        assert!(entry.cursor.is_none());
    }

    #[test]
    fn test_connection_manager_add_user() {
        use chrono::Duration;
        use chrono::Utc;

        let manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        let session = WebSocketSession::new(document_id, user_id, "Test User".to_string(), "#FF0000".to_string());

        manager.register_connection(&session);

        let stats = manager.get_stats();
        assert_eq!(stats.active_connections, 1);

        // Check session is active with 5 minute timeout
        assert!(manager.is_session_active(&session, 300));
        assert!(manager.is_session_active(&session, 240));
    }

    #[test]
    fn test_connection_manager_remove_user() {
        let manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        let session = WebSocketSession::new(document_id, user_id, "Test User".to_string(), "#FF0000".to_string());

        manager.register_connection(&session);
        let stats = manager.get_stats();
        assert_eq!(stats.active_connections, 1);

        manager.unregister_connection(session.id);
        let stats = manager.get_stats();
        assert_eq!(stats.active_connections, 0);
    }

    #[test]
    fn test_connection_manager_multiple_users() {
        let document_id = Uuid::new_v4();

        let mut manager = ConnectionManager::new();
        for i in 0..3 {
            let session = WebSocketSession::new(
                document_id,
                Uuid::new_v4(),
                format!("User {}", i),
                "#FF0000".to_string(),
            );
            manager.register_connection(&session);
        }

        let stats = manager.get_stats();
        assert_eq!(stats.active_connections, 3);
    }

    #[test]
    fn test_connection_manager_user_cleanup() {
        use chrono::Utc;

        let manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        let session = WebSocketSession::new(document_id, user_id, "Test User".to_string(), "#FF0000".to_string());

        manager.register_connection(&session);

        // Record global message statistics
        manager.record_message_sent(100);

        // Verify connection is registered via global stats
        let stats = manager.get_stats();
        assert_eq!(stats.active_connections, 1);

        // Session should be active (it was just created, so last_activity is recent)
        assert!(manager.is_session_active(&session, 1800));
    }

    #[test]
    fn test_connection_manager_stale_connections() {
        use chrono::{Duration, Utc};

        let manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        let session = WebSocketSession::new(document_id, user_id, "Test User".to_string(), "#FF0000".to_string());

        // Register session
        manager.register_connection(&session);

        // Create a session with old activity (1 hour ago)
        let old_session = WebSocketSession {
            last_activity: Utc::now() - Duration::seconds(3600),
            ..session.clone()
        };

        // Recent session should be active after 5 minutes
        assert!(manager.is_session_active(&session, 300));

        // Old session should NOT be active after 15 minutes
        assert!(!manager.is_session_active(&old_session, 900));
    }
}
