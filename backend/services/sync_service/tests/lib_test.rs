//! Unit tests for sync_service
//!
//! This module contains tests for:
//! - StateVector operations
//! - Conflict resolution
//! - Sync state management

use std::collections::HashMap;
use sync_service::conflict_resolver::{ConflictResolver, ConflictType, ResolutionStrategy};
use sync_service::state_vector::{Clock, HlcTimestamp, StateVector, VectorClock};

// Test StateVector basic operations
#[test]
fn test_state_vector_new() {
    let sv = StateVector::new();
    assert!(sv.clocks.is_empty());
}

#[test]
fn test_state_vector_with_initial_peer() {
    let mut sv = StateVector::new();
    sv.update("peer-1".to_string(), 0);
    assert_eq!(sv.get("peer-1"), Some(&0));
}

#[test]
fn test_state_vector_update() {
    let mut sv = StateVector::new();
    sv.update("peer-1".to_string(), 5);
    assert_eq!(sv.get("peer-1"), Some(&5));
}

#[test]
fn test_state_vector_update_incremental() {
    let mut sv = StateVector::new();
    sv.update("peer-1".to_string(), 1);
    sv.update("peer-1".to_string(), 2);
    sv.update("peer-1".to_string(), 3);
    assert_eq!(sv.get("peer-1"), Some(&3));
}

#[test]
fn test_state_vector_update_multiple_peers() {
    let mut sv = StateVector::new();
    sv.update("peer-1".to_string(), 1);
    sv.update("peer-2".to_string(), 2);
    sv.update("peer-3".to_string(), 3);

    assert_eq!(sv.get("peer-1"), Some(&1));
    assert_eq!(sv.get("peer-2"), Some(&2));
    assert_eq!(sv.get("peer-3"), Some(&3));
}

#[test]
fn test_state_vector_get_nonexistent() {
    let sv = StateVector::new();
    assert_eq!(sv.get("nonexistent"), None);
}

#[test]
fn test_state_vector_merge() {
    let mut sv1 = StateVector::new();
    sv1.update("peer-1".to_string(), 5);

    let mut sv2 = StateVector::new();
    sv2.update("peer-1".to_string(), 3);
    sv2.update("peer-2".to_string(), 10);

    sv1.merge(&sv2);

    assert_eq!(sv1.get("peer-1"), Some(&5)); // Max of 5 and 3
    assert_eq!(sv1.get("peer-2"), Some(&10));
}

#[test]
fn test_state_vector_causally_ready() {
    let mut sv_local = StateVector::new();
    sv_local.update("local".to_string(), 5);

    let mut sv_remote = StateVector::new();
    sv_remote.update("local".to_string(), 3);
    sv_remote.update("remote".to_string(), 10);

    assert!(sv_local.causally_ready(&sv_remote));
}

#[test]
fn test_state_vector_not_causally_ready() {
    let mut sv_local = StateVector::new();
    sv_local.update("peer-1".to_string(), 5);

    let mut sv_remote = StateVector::new();
    sv_remote.update("peer-1".to_string(), 10);

    assert!(!sv_local.causally_ready(&sv_remote));
}

// Test VectorClock
#[test]
fn test_vector_clock_new() {
    let vc = VectorClock::new();
    assert!(vc.0.is_empty());
}

#[test]
fn test_vector_clock_increment() {
    let mut vc = VectorClock::new();
    vc.increment("peer-1".to_string());
    assert_eq!(vc.get("peer-1"), Some(&1));
}

#[test]
fn test_vector_clock_increment_multiple() {
    let mut vc = VectorClock::new();
    vc.increment("peer-1".to_string());
    vc.increment("peer-1".to_string());
    vc.increment("peer-1".to_string());
    assert_eq!(vc.get("peer-1"), Some(&3));
}

#[test]
fn test_vector_clock_compare_less_than() {
    let mut vc1 = VectorClock::new();
    vc1.increment("peer-1".to_string());

    let mut vc2 = VectorClock::new();
    vc2.increment("peer-1".to_string());
    vc2.increment("peer-1".to_string());

    assert_eq!(vc1.partial_cmp(&vc2), Some(std::cmp::Ordering::Less));
}

#[test]
fn test_vector_clock_compare_greater_than() {
    let mut vc1 = VectorClock::new();
    vc1.increment("peer-1".to_string());
    vc1.increment("peer-1".to_string());

    let mut vc2 = VectorClock::new();
    vc2.increment("peer-1".to_string());

    assert_eq!(vc1.partial_cmp(&vc2), Some(std::cmp::Ordering::Greater));
}

#[test]
fn test_vector_clock_compare_concurrent() {
    let mut vc1 = VectorClock::new();
    vc1.increment("peer-1".to_string());

    let mut vc2 = VectorClock::new();
    vc2.increment("peer-2".to_string());

    assert_eq!(vc1.partial_cmp(&vc2), None); // Concurrent
}

// Test HlcTimestamp
#[test]
fn test_hlc_timestamp_new() {
    let hlc = HlcTimestamp::new();
    assert!(hlc.logical > 0 || hlc.physical > 0);
}

#[test]
fn test_hlc_timestamp_increment() {
    let mut hlc1 = HlcTimestamp::new();
    let hlc2 = hlc1.increment();

    assert!(hlc2.logical >= hlc1.logical);
}

#[test]
fn test_hlc_timestamp_update() {
    let mut hlc = HlcTimestamp::new();
    hlc.update(100, "other-1".to_string());
    assert!(hlc.logical >= 100);
}

#[test]
fn test_hlc_timestamp_causal_order() {
    let mut hlc1 = HlcTimestamp::new();
    let hlc2 = hlc1.increment();
    let hlc3 = hlc2.update(50, "other".to_string());

    assert!(hlc3 >= hlc1);
    assert!(hlc2 >= hlc1);
}

// Test ConflictResolver
#[test]
fn test_conflict_resolver_new() {
    let resolver = ConflictResolver::new();
    assert!(resolver.strategies.is_empty());
}

#[test]
fn test_conflict_resolver_add_strategy() {
    let mut resolver = ConflictResolver::new();
    resolver.add_strategy(ConflictType::Content, ResolutionStrategy::AcceptIncoming);
    assert_eq!(resolver.strategies.len(), 1);
}

#[test]
fn test_conflict_resolver_get_strategy() {
    let mut resolver = ConflictResolver::new();
    resolver.add_strategy(ConflictType::Content, ResolutionStrategy::AcceptIncoming);

    let strategy = resolver.get_strategy(&ConflictType::Content);
    assert_eq!(strategy, Some(&ResolutionStrategy::AcceptIncoming));
}

#[test]
fn test_conflict_resolver_unknown_conflict() {
    let resolver = ConflictResolver::new();
    let strategy = resolver.get_strategy(&ConflictType::ConcurrentEdits);
    assert_eq!(strategy, None); // Default fallback
}

// Test ConflictType
#[test]
fn test_conflict_type_variants() {
    let _ = ConflictType::Content;
    let _ = ConflictType::Metadata;
    let _ = ConflictType::Delete;
    let _ = ConflictType::ConcurrentEdits;
}

// Test ResolutionStrategy
#[test]
fn test_resolution_strategy_variants() {
    let _ = ResolutionStrategy::AcceptIncoming;
    let _ = ResolutionStrategy::KeepBoth;
    let _ = ResolutionStrategy::Manual;
    let _ = ResolutionStrategy::TimestampBased;
    let _ = ResolutionStrategy::UserPriority;
}

// Test sync state operations
#[tokio::test]
async fn test_sync_state_new() {
    let state = sync_service::SyncState::new();
    assert!(state.pending_updates.is_empty());
}

#[test]
fn test_sync_message_serialization() {
    use sync_service::sync_handler::SyncMessage;

    let msg = SyncMessage::Update {
        document_id: "doc-123".to_string(),
        content: "test content".to_string(),
        version: 1,
    };

    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: SyncMessage = serde_json::from_str(&json).unwrap();

    match deserialized {
        SyncMessage::Update {
            document_id,
            content,
            version,
        } => {
            assert_eq!(document_id, "doc-123");
            assert_eq!(content, "test content");
            assert_eq!(version, 1);
        },
        _ => panic!("Wrong message type"),
    }
}

// Test sync connection state
#[test]
fn test_connection_state_connected() {
    let state = sync_service::ConnectionState::Connected;
    assert!(matches!(state, sync_service::ConnectionState::Connected));
}

#[test]
fn test_connection_state_disconnected() {
    let state = sync_service::ConnectionState::Disconnected(None);
    assert!(matches!(state, sync_service::ConnectionState::Disconnected(None)));

    if let sync_service::ConnectionState::Disconnected(reason) = state {
        assert!(reason.is_none());
    } else {
        panic!("Expected Disconnected variant");
    }
}

#[test]
fn test_connection_state_with_reason() {
    let state = sync_service::ConnectionState::Disconnected(Some("Network error".to_string()));
    assert!(matches!(state, sync_service::ConnectionState::Disconnected(Some(_))));

    match state {
        sync_service::ConnectionState::Disconnected(reason) => {
            assert_eq!(reason, Some("Network error".to_string()));
        }
        other => panic!("Expected Disconnected variant, got {:?}", other),
    }
}

// Test sync error variants
#[test]
fn test_sync_error_conflict() {
    let error = sync_service::SyncError::Conflict {
        document_id: "doc-123".to_string(),
        details: "Concurrent modification detected".to_string(),
    };

    match error {
        sync_service::SyncError::Conflict { document_id, details } => {
            assert_eq!(document_id, "doc-123");
            assert!(details.contains("Concurrent"));
        },
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_sync_error_version_mismatch() {
    let error = sync_service::SyncError::VersionMismatch {
        document_id: "doc-456".to_string(),
        expected: 5,
        actual: 3,
    };

    match error {
        sync_service::SyncError::VersionMismatch {
            document_id,
            expected,
            actual,
        } => {
            assert_eq!(document_id, "doc-456");
            assert_eq!(expected, 5);
            assert_eq!(actual, 3);
        },
        _ => panic!("Wrong error type"),
    }
}

// Test awareness state
#[test]
fn test_awareness_new() {
    let awareness = sync_service::Awareness::new();
    assert!(awareness.states.is_empty());
}

#[test]
fn test_awareness_set_local_state() {
    let mut awareness = sync_service::Awareness::new();
    let state = serde_json::json!({"user": "test-user", "cursor": {"line": 10, "column": 5}});
    awareness.set_local_state("client-1".to_string(), state.clone());

    assert_eq!(awareness.states.len(), 1);
    assert_eq!(awareness.get("client-1"), Some(&state));
}

#[test]
fn test_awareness_remove_state() {
    let mut awareness = sync_service::Awareness::new();
    let state = serde_json::json!({"user": "test-user"});
    awareness.set_local_state("client-1".to_string(), state);
    awareness.remove_state("client-1".to_string());

    assert!(awareness.states.is_empty());
}

// Test document merge result
#[test]
fn test_merge_result_success() {
    let result = sync_service::MergeResult::Success {
        content: "merged content".to_string(),
        version: 5,
    };

    match result {
        sync_service::MergeResult::Success { content, version } => {
            assert_eq!(content, "merged content");
            assert_eq!(version, 5);
        },
        _ => panic!("Wrong result type"),
    }
}

#[test]
fn test_merge_result_conflict() {
    let result = sync_service::MergeResult::Conflict {
        local_content: "local".to_string(),
        remote_content: "remote".to_string(),
        resolution: "Manual resolution required".to_string(),
    };

    match result {
        sync_service::MergeResult::Conflict {
            local_content,
            remote_content,
            resolution,
        } => {
            assert_eq!(local_content, "local");
            assert_eq!(remote_content, "remote");
            assert_eq!(resolution, "Manual resolution required");
        },
        _ => panic!("Wrong result type"),
    }
}
