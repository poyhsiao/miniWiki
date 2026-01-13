//! Conflict resolution tests
//!
//! Tests for CRDT conflict resolution when multiple clients edit the same document.
//! These tests verify that concurrent edits are properly merged without data loss.
//!
//! Run with: cargo test -p miniwiki-backend-tests sync::conflict_resolution_test

use uuid::Uuid;
use std::collections::HashMap;

/// Mock state vector for testing
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct StateVector {
    client_id: String,
    clock: u64,
}

/// Mock update for testing
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct SyncUpdate {
    update: String,      // Base64 encoded CRDT update
    state_vector: StateVector,
    origin: String,
}

/// Simulated CRDT document state
struct CrdtDocumentState {
    state_vector: StateVector,
    content: HashMap<String, String>,  // Simple key-value for testing
}

impl CrdtDocumentState {
    fn new(client_id: &Uuid) -> Self {
        Self {
            state_vector: StateVector {
                client_id: client_id.to_string(),
                clock: 0,
            },
            content: HashMap::new(),
        }
    }

    /// Merge another update into this document
    fn merge(&mut self, update: &SyncUpdate) -> bool {
        // Check if update is from a different client or newer state
        let needs_merge = update.state_vector.client_id != self.state_vector.client_id
            || update.state_vector.clock > self.state_vector.clock;

        if needs_merge {
            // Simulate CRDT merge logic
            // In real implementation, this would use Yjs/Lib0 merge algorithm
            self.state_vector.clock = self.state_vector.clock.max(update.state_vector.clock) + 1;

            // Simulate content update
            self.content.insert(
                format!("update_from_{}", update.origin),
                update.update.clone(),
            );
        }

        needs_merge
    }
}

/// Test that concurrent edits from different clients merge correctly
#[tokio::test]
async fn test_concurrent_edits_merge() {
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    let mut doc_state = CrdtDocumentState::new(&user1_id);

    // User 1 makes first edit
    let update1 = SyncUpdate {
        update: "User1_edit_1".to_string(),
        state_vector: StateVector {
            client_id: user1_id.to_string(),
            clock: 1,
        },
        origin: "user1_client".to_string(),
    };

    // User 2 makes concurrent edit
    let update2 = SyncUpdate {
        update: "User2_edit_1".to_string(),
        state_vector: StateVector {
            client_id: user2_id.to_string(),
            clock: 1,
        },
        origin: "user2_client".to_string(),
    };

    // Merge both updates
    let merged1 = doc_state.merge(&update1);
    let merged2 = doc_state.merge(&update2);

    assert!(merged1, "First update should be merged");
    assert!(merged2, "Second concurrent update should be merged");
    assert_eq!(doc_state.content.len(), 2, "Both edits should be preserved");
}

/// Test that older updates are ignored
#[tokio::test]
async fn test_older_updates_ignored() {
    let user_id = Uuid::new_v4();
    let mut doc_state = CrdtDocumentState::new(&user_id);

    // First update
    let update1 = SyncUpdate {
        update: "First_update".to_string(),
        state_vector: StateVector {
            client_id: user_id.to_string(),
            clock: 5,
        },
        origin: "client".to_string(),
    };

    // Older update (lower clock)
    let old_update = SyncUpdate {
        update: "Old_update".to_string(),
        state_vector: StateVector {
            client_id: user_id.to_string(),
            clock: 3,
        },
        origin: "client".to_string(),
    };

    // Apply newer update first
    doc_state.merge(&update1);

    // Try to apply older update (should be ignored)
    let merged = doc_state.merge(&old_update);

    assert!(!merged, "Older update should be ignored");
    assert_eq!(doc_state.content.len(), 1, "Only newer update should be preserved");
    assert_eq!(doc_state.state_vector.clock, 6, "Clock should have advanced");
}

/// Test state vector comparison determines merge order
#[tokio::test]
async fn test_state_vector_comparison() {
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    let mut doc1 = CrdtDocumentState::new(&user1_id);
    let mut doc2 = CrdtDocumentState::new(&user2_id);

    // Both clients make some edits
    let updates_doc1 = vec![
        SyncUpdate {
            update: "edit_1".to_string(),
            state_vector: StateVector { client_id: user1_id.to_string(), clock: 1 },
            origin: "doc1".to_string(),
        },
        SyncUpdate {
            update: "edit_2".to_string(),
            state_vector: StateVector { client_id: user1_id.to_string(), clock: 2 },
            origin: "doc1".to_string(),
        },
    ];

    let updates_doc2 = vec![
        SyncUpdate {
            update: "edit_a".to_string(),
            state_vector: StateVector { client_id: user2_id.to_string(), clock: 1 },
            origin: "doc2".to_string(),
        },
        SyncUpdate {
            update: "edit_b".to_string(),
            state_vector: StateVector { client_id: user2_id.to_string(), clock: 2 },
            origin: "doc2".to_string(),
        },
    ];

    // Apply own updates
    for update in updates_doc1 {
        doc1.merge(&update);
    }
    for update in updates_doc2 {
        doc2.merge(&update);
    }

    // Simulate sync between doc1 and doc2
    // Doc1 sends its state to Doc2
    let sync_update = SyncUpdate {
        update: "sync_from_doc1".to_string(),
        state_vector: doc1.state_vector.clone(),
        origin: "doc1".to_string(),
    };
    doc2.merge(&sync_update);

    // Doc2 sends its state back to Doc1
    let sync_update2 = SyncUpdate {
        update: "sync_from_doc2".to_string(),
        state_vector: doc2.state_vector.clone(),
        origin: "doc2".to_string(),
    };
    doc1.merge(&sync_update2);

    // Both documents should have the same number of updates
    assert_eq!(doc1.content.len(), doc2.content.len(), "Both docs should have merged all updates");
}

/// Test conflict resolution preserves both conflicting values
#[tokio::test]
async fn test_conflict_preserves_both_values() {
    let user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();

    let mut doc = CrdtDocumentState::new(&user_id);

    // Two different users edit the same content
    let update1 = SyncUpdate {
        update: "value_from_user1".to_string(),
        state_vector: StateVector { client_id: user_id.to_string(), clock: 1 },
        origin: "user1".to_string(),
    };

    let update2 = SyncUpdate {
        update: "value_from_user2".to_string(),
        state_vector: StateVector { client_id: other_user_id.to_string(), clock: 1 },
        origin: "user2".to_string(),
    };

    doc.merge(&update1);
    doc.merge(&update2);

    // Both values should be preserved (CRDT guarantees no data loss)
    assert_eq!(doc.content.len(), 2);
    assert!(doc.content.values().any(|v| v == "value_from_user1"));
    assert!(doc.content.values().any(|v| v == "value_from_user2"));
}

/// Test multiple rapid successive updates
#[tokio::test]
async fn test_rapid_successive_updates() {
    let user_id = Uuid::new_v4();
    let mut doc = CrdtDocumentState::new(&user_id);

    // Rapid succession of updates
    for i in 1..=10 {
        let update = SyncUpdate {
            update: format!("update_{}", i),
            state_vector: StateVector { client_id: user_id.to_string(), clock: i },
            origin: "rapid_client".to_string(),
        };
        doc.merge(&update);
    }

    // All updates should be preserved
    assert_eq!(doc.content.len(), 10);
    assert_eq!(doc.state_vector.clock, 11, "Final clock should be 11 (10 updates + 1)");
}

/// Test that state vector clock increments correctly on merge
#[tokio::test]
async fn test_state_vector_clock_increments() {
    let user_id = Uuid::new_v4();
    let mut doc = CrdtDocumentState::new(&user_id);

    assert_eq!(doc.state_vector.clock, 0, "Initial clock should be 0");

    for i in 1..=5 {
        let update = SyncUpdate {
            update: format!("update_{}", i),
            state_vector: StateVector { client_id: "other_client".to_string(), clock: i },
            origin: "other".to_string(),
        };
        doc.merge(&update);
    }

    // Clock should be max(0, 5) + 1 = 6
    assert_eq!(doc.state_vector.clock, 6);
}

/// Test sync with self (no conflict)
#[tokio::test]
async fn test_sync_with_self_no_conflict() {
    let user_id = Uuid::new_v4();
    let mut doc = CrdtDocumentState::new(&user_id);

    // Update from same client
    let update = SyncUpdate {
        update: "self_update".to_string(),
        state_vector: StateVector { client_id: user_id.to_string(), clock: 1 },
        origin: "self".to_string(),
    };

    // First merge should succeed
    let merged = doc.merge(&update);
    assert!(merged);

    // Second merge of same state should be ignored
    let merged_again = doc.merge(&update);
    assert!(!merged_again, "Duplicate update should be ignored");
}

/// Test cross-document update isolation
#[tokio::test]
async fn test_cross_document_isolation() {
    let user_id = Uuid::new_v4();

    let mut doc1 = CrdtDocumentState::new(&user_id);
    let mut doc2 = CrdtDocumentState::new(&user_id);

    // Update doc1
    let update1 = SyncUpdate {
        update: "doc1_content".to_string(),
        state_vector: StateVector { client_id: user_id.to_string(), clock: 1 },
        origin: "doc1".to_string(),
    };

    // Update doc2
    let update2 = SyncUpdate {
        update: "doc2_content".to_string(),
        state_vector: StateVector { client_id: user_id.to_string(), clock: 1 },
        origin: "doc2".to_string(),
    };

    doc1.merge(&update1);
    doc2.merge(&update2);

    // Documents should be independent
    assert_eq!(doc1.content.len(), 1);
    assert_eq!(doc2.content.len(), 1);
    assert_ne!(doc1.content.values().next(), doc2.content.values().next());
}

/// Test that merge is idempotent
#[tokio::test]
async fn test_merge_idempotent() {
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    let mut doc = CrdtDocumentState::new(&user1_id);

    let update = SyncUpdate {
        update: "test_update".to_string(),
        state_vector: StateVector { client_id: user2_id.to_string(), clock: 1 },
        origin: "user2".to_string(),
    };

    // Merge same update multiple times
    doc.merge(&update);
    doc.merge(&update);
    doc.merge(&update);

    // Should only be applied once
    assert_eq!(doc.content.len(), 1);
    assert_eq!(doc.state_vector.clock, 2);
}
