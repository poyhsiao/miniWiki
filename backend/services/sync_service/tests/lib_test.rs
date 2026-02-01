//! Unit tests for sync_service
//!
//! This module contains tests for:
//! - StateVector operations
//! - Conflict resolution

use sync_service::conflict_resolver::{ConflictResolution, ConflictResolutionStrategy, ConflictResolver};
use sync_service::state_vector::{ClientId, Clock, StateVector};

// Test StateVector basic operations
#[test]
fn test_state_vector_new() {
    let sv = StateVector::new();
    assert!(sv.is_empty());
}

#[test]
fn test_state_vector_with_initial_client() {
    let mut sv = StateVector::new();
    sv.set(1, 0);
    assert_eq!(sv.get(1), Some(&0));
}

#[test]
fn test_state_vector_set() {
    let mut sv = StateVector::new();
    sv.set(1, 5);
    assert_eq!(sv.get(1), Some(&5));
}

#[test]
fn test_state_vector_increment() {
    let mut sv = StateVector::new();
    sv.increment(1);
    sv.increment(1);
    sv.increment(1);
    assert_eq!(sv.get(1), Some(&3));
}

#[test]
fn test_state_vector_update_multiple_clients() {
    let mut sv = StateVector::new();
    sv.set(1, 1);
    sv.set(2, 2);
    sv.set(3, 3);

    assert_eq!(sv.get(1), Some(&1));
    assert_eq!(sv.get(2), Some(&2));
    assert_eq!(sv.get(3), Some(&3));
}

#[test]
fn test_state_vector_get_nonexistent() {
    let sv = StateVector::new();
    assert_eq!(sv.get(999), None);
}

#[test]
fn test_state_vector_len() {
    let mut sv = StateVector::new();
    assert_eq!(sv.len(), 0);
    sv.set(1, 1);
    sv.set(2, 2);
    assert_eq!(sv.len(), 2);
}

#[test]
fn test_state_vector_inner() {
    let mut sv = StateVector::new();
    sv.set(1, 5);
    sv.set(2, 10);
    let inner = sv.inner();
    assert_eq!(inner.len(), 2);
    assert_eq!(inner.get(&1), Some(&5));
    assert_eq!(inner.get(&2), Some(&10));
}

// Test ConflictResolver
#[test]
fn test_conflict_resolver_new() {
    let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);
    let strategy = ConflictResolutionStrategy::Merge;
    let _ = strategy;
}

#[test]
fn test_conflict_resolver_resolve_state_vector() {
    let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

    let mut sv1 = StateVector::new();
    sv1.set(1, 10);

    let mut sv2 = StateVector::new();
    sv2.set(1, 5);
    sv2.set(2, 20);

    let (merged, resolution) = resolver.resolve_state_vector(&sv1, &sv2);

    assert_eq!(resolution, ConflictResolution::Merged);
    assert_eq!(merged.get(1), Some(&10));
    assert_eq!(merged.get(2), Some(&20));
}

#[test]
fn test_conflict_resolver_resolve_document_conflict_same() {
    let resolver = ConflictResolver::new(ConflictResolutionStrategy::Timestamp);

    let sv1 = StateVector::new();
    let sv2 = StateVector::new();

    let (result, resolution) = resolver.resolve_document_conflict(&"same", &"same", &sv1, &sv2);

    assert_eq!(resolution, ConflictResolution::NoConflict);
    assert_eq!(result, "same");
}

#[test]
fn test_conflict_resolver_resolve_by_timestamp() {
    let resolver = ConflictResolver::new(ConflictResolutionStrategy::Timestamp);

    let mut sv1 = StateVector::new();
    sv1.set(1, 10);

    let mut sv2 = StateVector::new();
    sv2.set(2, 20);

    let (result, resolution) = resolver.resolve_document_conflict(&"local", &"remote", &sv1, &sv2);

    assert_eq!(resolution, ConflictResolution::KeepSecond);
    assert_eq!(result, "remote");
}

#[test]
fn test_conflict_resolver_get_missing_updates() {
    let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

    let mut client_sv = StateVector::new();
    client_sv.set(1, 5);

    let mut server_sv = StateVector::new();
    server_sv.set(1, 10);
    server_sv.set(2, 5);

    let missing = resolver.calculate_missing_updates(&client_sv, &server_sv);

    assert_eq!(missing.len(), 2);
    assert!(missing.contains(&(1, 6, 10)));
    assert!(missing.contains(&(2, 0, 5)));
}

#[test]
fn test_conflict_resolver_can_merge() {
    let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

    let sv1 = StateVector::new();
    let sv2 = StateVector::new();

    assert!(resolver.can_merge(&sv1, &sv2));
}

#[test]
fn test_conflict_resolver_get_newer_state() {
    let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

    let mut sv1 = StateVector::new();
    sv1.set(1, 10);

    let mut sv2 = StateVector::new();
    sv2.set(1, 20);

    let (newer, older) = resolver.get_newer_state(&sv1, &sv2);

    assert_eq!(newer.get(1), Some(&20));
    assert_eq!(older.get(1), Some(&10));
}

// Test ConflictResolutionStrategy
#[test]
fn test_conflict_resolution_strategy_variants() {
    let _ = ConflictResolutionStrategy::Merge;
    let _ = ConflictResolutionStrategy::Timestamp;
    let _ = ConflictResolutionStrategy::ClientId;
    let _ = ConflictResolutionStrategy::Custom;
}

// Test ConflictResolution
#[test]
fn test_conflict_resolution_variants() {
    let _ = ConflictResolution::NoConflict;
    let _ = ConflictResolution::KeepFirst;
    let _ = ConflictResolution::KeepSecond;
    let _ = ConflictResolution::Merged;
    let _ = ConflictResolution::Unresolved;
}

// Test StateVector compare and ancestor operations
#[test]
fn test_state_vector_compare() {
    let mut sv1 = StateVector::new();
    sv1.set(1, 10);

    let mut sv2 = StateVector::new();
    sv2.set(1, 20);

    assert_eq!(sv1.compare(&sv2), std::cmp::Ordering::Less);
    assert_eq!(sv2.compare(&sv1), std::cmp::Ordering::Greater);
}

#[test]
fn test_state_vector_is_ancestor_of() {
    let mut ancestor = StateVector::new();
    ancestor.set(1, 10);
    ancestor.set(2, 5);

    let mut descendant = StateVector::new();
    descendant.set(1, 15);
    descendant.set(2, 10);

    assert!(ancestor.is_ancestor_of(&descendant));
    assert!(!descendant.is_ancestor_of(&ancestor));
}

#[test]
fn test_state_vector_get_missing() {
    let mut base = StateVector::new();
    base.set(1, 10);

    let mut target = StateVector::new();
    target.set(1, 20);
    target.set(2, 5);

    let missing = base.get_missing(&target);
    assert_eq!(missing.len(), 2);
    assert!(missing.contains(&(1, 11)));
    assert!(missing.contains(&(2, 0)));
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
        },
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
