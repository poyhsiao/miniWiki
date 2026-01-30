#[cfg(test)]
mod websocket_service_test {
    use uuid::Uuid;
    use websocket_service::connection_manager::ConnectionManager;
    use websocket_service::models::*;
    use websocket_service::presence::Presence;

    #[test]
    fn test_presence_creation() {
        let presence = Presence {
            user_id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            file_id: Uuid::new_v4(),
            is_typing: false,
            last_seen_at: None,
        };

        assert!(!presence.user_id.is_nil());
        assert_eq!(presence.typing, false);
        assert!(presence.last_seen_at.is_none());
    }

    #[test]
    fn test_presence_last_seen_update() {
        use chrono::Utc;

        let mut presence = Presence {
            user_id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            file_id: Uuid::new_v4(),
            is_typing: false,
            last_seen_at: None,
        };

        let now = Utc::now();
        presence.update_last_seen(now);

        assert_eq!(presence.last_seen_at, Some(now));
    }

    #[test]
    fn test_presence_typing_state() {
        let mut presence = Presence {
            user_id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            file_id: Uuid::new_v4(),
            is_typing: true,
            last_seen_at: Some(chrono::Utc::now()),
        };

        presence.stop_typing();

        assert_eq!(presence.is_typing, false);
    }

    #[test]
    fn test_connection_manager_multiple_users() {
        let user_ids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ];

        let mut manager = ConnectionManager::new();
        for user_id in &user_ids {
            manager.add_user(user_id.clone());
        }

        assert_eq!(manager.connection_count(), 3);
    }

    #[test]
    fn test_connection_manager_user_cleanup() {
        use std::time::Duration;
        use std::time::Instant;

        let mut manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let now = Instant::now();

        manager.add_user(user_id);
        
        let activity = manager.update_user_activity(user_id, now);
        assert!(activity.is_some());

        let connections = manager.get_active_connections(&user_id);
        assert_eq!(connections.len(), 1);

        let stale_time = now + Duration::from_secs(1800);
        assert!(manager.is_connection_stale(user_id, stale_time));
    }

    #[test]
    fn test_connection_manager_stale_connections() {
        use std::time::Duration;
        use std::time::Instant;

        let mut manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let old_time = Instant::now() - Duration::from_secs(3600);
        let now = Instant::now();

        manager.add_user(user_id);
        manager.update_user_activity(user_id, old_time);

        let recent = now - Duration::from_secs(300);
        assert!(!manager.is_connection_stale(user_id, recent));

        let stale = now - Duration::from_secs(900);
        assert!(manager.is_connection_stale(user_id, stale));
    }

        assert_eq!(manager.connection_count(), 3);
    }

    #[test]
    fn test_connection_manager_user_cleanup() {
        use std::time::Duration;
        use std::time::Instant;

        let mut manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let now = Instant::now();

        manager.add_user(user_id);
        
        // Mark as active
        let activity = manager.update_user_activity(user_id, now);
        assert!(activity.is_some());

        // Check active state
        let connections = manager.get_active_connections(&user_id);
        assert_eq!(connections.len(), 1);

        // After cleanup (30 minutes), should be removed
        let stale_time = now + Duration::from_secs(1800);
        assert!(manager.is_connection_stale(user_id, stale_time));
    }

    #[test]
    fn test_connection_manager_stale_connections() {
        use std::time::Duration;
        use std::time::Instant;

        let mut manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let old_time = Instant::now() - Duration::from_secs(3600); // 10 minutes ago
        let now = Instant::now();

        manager.add_user(user_id);
        manager.update_user_activity(user_id, old_time);

        // Should not be stale after 5 minutes
        let recent = now - Duration::from_secs(300);
        assert!(!manager.is_connection_stale(user_id, recent));

        // Should be stale after 15 minutes
        let stale = now - Duration::from_secs(900);
        assert!(manager.is_connection_stale(user_id, stale));
    }
}

    #[test]
    fn test_connection_manager_add_user() {
        use std::time::Duration;
        use std::time::Instant;

        let mut manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();
        let now = Instant::now();

        manager.add_user(user_id);

        assert!(manager.get_user(user_id).is_some());

        // After 5 minutes, connection should be considered stale
        assert!(manager.is_connection_stale(user_id, Duration::from_secs(300)));
        assert!(!manager.is_connection_stale(user_id, Duration::from_secs(240)));
    }

    #[test]
    fn test_connection_manager_remove_user() {
        use std::time::Duration;

        let mut manager = ConnectionManager::new();
        let user_id = Uuid::new_v4();

        manager.add_user(user_id);
        assert!(manager.get_user(user_id).is_some());

        manager.remove_user(&user_id);
        assert!(manager.get_user(user_id).is_none());
    }

    #[test]
    fn test_connection_manager_multiple_users() {
        let user_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

        let mut manager = ConnectionManager::new();
        for user_id in &user_ids {
            manager.add_user(user_id.clone());
        }

        assert_eq!(manager.connection_count(), 3);
    }
}
