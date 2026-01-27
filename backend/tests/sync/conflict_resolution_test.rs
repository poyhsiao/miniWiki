//! Conflict resolution tests
//!
//! Tests for CRDT conflict resolution when multiple clients edit the same document.
//! These tests verify that concurrent edits are properly merged without data loss.
//!
//! Run with: cargo test -p miniwiki-backend-tests sync::conflict_resolution_test

use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Mock state vector for testing
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct StateVector {
    client_id: String,
    clock: u64,
}

/// Mock update for testing
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct SyncUpdate {
    update: String, // Base64 encoded CRDT update
    state_vector: StateVector,
    origin: String,
}

/// Simulated CRDT document state
struct CrdtDocumentState {
    state_vector: StateVector,
    content: HashMap<String, String>,        // Simple key-value for testing
    applied_updates: HashSet<(String, u64)>, // Track (origin, clock) of applied updates (HashSet for O(1) lookups)
    client_clocks: HashMap<String, u64>,     // Track max clock seen per client
}

impl CrdtDocumentState {
    fn new(client_id: &Uuid) -> Self {
        Self {
            state_vector: StateVector {
                client_id: client_id.to_string(),
                clock: 0,
            },
            content: HashMap::new(),
            applied_updates: HashSet::new(),
            client_clocks: HashMap::new(),
        }
    }

    /// Merge another update into this document
    fn merge(&mut self, update: &SyncUpdate) -> bool {
        // Check if this specific update has already been applied
        let update_key = (update.origin.clone(), update.state_vector.clock);
        let was_applied = self.applied_updates.contains(&update_key);

        if was_applied {
            // Already applied, return false (idempotent)
            return false;
        }

        // Determine if this update should be merged
        let needs_merge = if update.state_vector.client_id != self.state_vector.client_id {
            // From different client: always merge
            true
        } else {
            // From same client: check if this is a new update (not outdated)
            let max_clock = self.client_clocks.entry(update.state_vector.client_id.clone()).or_insert(0);
            update.state_vector.clock > *max_clock
        };

        if needs_merge {
            // Update client clock tracking - insert new clients or max existing clocks
            self.client_clocks
                .entry(update.state_vector.client_id.clone())
                .and_modify(|clock| *clock = (*clock).max(update.state_vector.clock))
                .or_insert(update.state_vector.clock);

            // For same client updates, adopt the sender's clock
            // For different client updates, merge clocks (take max)
            if update.state_vector.client_id == self.state_vector.client_id {
                self.state_vector.clock = update.state_vector.clock;
            } else {
                self.state_vector.clock = self.state_vector.clock.max(update.state_vector.clock);
            }

            // Increment our local clock to account for this merge operation
            self.state_vector.clock += 1;

            // Simulate content update - use unique key per update
            let content_key = format!("{}_{}_{}", update.origin, update.state_vector.clock, update.update);
            self.content.insert(content_key, update.update.clone());

            // Mark this update as applied (HashSet.insert for O(1) operation)
            self.applied_updates.insert(update_key);

            true
        } else {
            false
        }
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
            state_vector: StateVector {
                client_id: user1_id.to_string(),
                clock: 1,
            },
            origin: "doc1".to_string(),
        },
        SyncUpdate {
            update: "edit_2".to_string(),
            state_vector: StateVector {
                client_id: user1_id.to_string(),
                clock: 2,
            },
            origin: "doc1".to_string(),
        },
    ];

    let updates_doc2 = vec![
        SyncUpdate {
            update: "edit_a".to_string(),
            state_vector: StateVector {
                client_id: user2_id.to_string(),
                clock: 1,
            },
            origin: "doc2".to_string(),
        },
        SyncUpdate {
            update: "edit_b".to_string(),
            state_vector: StateVector {
                client_id: user2_id.to_string(),
                clock: 2,
            },
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
    assert_eq!(
        doc1.content.len(),
        doc2.content.len(),
        "Both docs should have merged all updates"
    );
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
        state_vector: StateVector {
            client_id: user_id.to_string(),
            clock: 1,
        },
        origin: "user1".to_string(),
    };

    let update2 = SyncUpdate {
        update: "value_from_user2".to_string(),
        state_vector: StateVector {
            client_id: other_user_id.to_string(),
            clock: 1,
        },
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
            state_vector: StateVector {
                client_id: user_id.to_string(),
                clock: i,
            },
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
            state_vector: StateVector {
                client_id: "other_client".to_string(),
                clock: i,
            },
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
        state_vector: StateVector {
            client_id: user_id.to_string(),
            clock: 1,
        },
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
        state_vector: StateVector {
            client_id: user_id.to_string(),
            clock: 1,
        },
        origin: "doc1".to_string(),
    };

    // Update doc2
    let update2 = SyncUpdate {
        update: "doc2_content".to_string(),
        state_vector: StateVector {
            client_id: user_id.to_string(),
            clock: 1,
        },
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
        state_vector: StateVector {
            client_id: user2_id.to_string(),
            clock: 1,
        },
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

/// Test that merge is idempotent for the same update applied twice
#[tokio::test]
async fn merge_is_idempotent_for_same_update() {
    let doc_client_id = Uuid::new_v4();
    let mut doc = CrdtDocumentState::new(&doc_client_id);

    let origin = "client-a".to_string();
    let initial_clock = 1;

    // Construct a SyncUpdate that changes the document
    let update = SyncUpdate {
        update: "test_content".to_string(),
        state_vector: StateVector {
            client_id: origin.clone(),
            clock: initial_clock,
        },
        origin: origin.clone(),
    };

    // Snapshot the pre-merge state
    let before_state_vector = doc.state_vector.clone();
    let before_content = doc.content.clone();
    let before_client_clocks = doc.client_clocks.clone();

    // First application should apply the update
    let applied_first = doc.merge(&update);

    // Assert: first call should report success and mutate the state
    assert!(
        applied_first,
        "first application of a new SyncUpdate should return true"
    );
    assert!(
        doc.state_vector.clock >= before_state_vector.clock,
        "state_vector.clock should advance after first merge"
    );
    assert_ne!(
        doc.content, before_content,
        "content should change after first merge to reflect the update"
    );
    assert_ne!(
        doc.client_clocks, before_client_clocks,
        "client_clocks should be updated after first merge"
    );

    // Snapshot the post-first-merge state
    let after_first_state_vector = doc.state_vector.clone();
    let after_first_content = doc.content.clone();
    let after_first_client_clocks = doc.client_clocks.clone();

    // Second application of the same update
    let applied_second = doc.merge(&update);

    // Assert: second call should be a no-op and return false
    assert!(
        !applied_second,
        "second application of the same SyncUpdate should return false (idempotent behavior)"
    );
    assert_eq!(
        doc.state_vector, after_first_state_vector,
        "state_vector should not change on re-applying the same update"
    );
    assert_eq!(
        doc.content, after_first_content,
        "content should not change on re-applying the same update"
    );
    assert_eq!(
        doc.client_clocks, after_first_client_clocks,
        "client_clocks should not change on re-applying the same update"
    );
}

/// Test that idempotency is per (origin, clock) key
#[tokio::test]
async fn merge_idempotency_is_per_update_key() {
    let doc_client_id = Uuid::new_v4();
    let mut doc = CrdtDocumentState::new(&doc_client_id);

    let origin = "client-a".to_string();
    let clock = 1;

    // First update: (origin, clock)
    let update1 = SyncUpdate {
        update: "update1_content".to_string(),
        state_vector: StateVector {
            client_id: origin.clone(),
            clock,
        },
        origin: origin.clone(),
    };

    // Second update: different clock (should *not* be considered a duplicate)
    let update2 = SyncUpdate {
        update: "update2_content".to_string(),
        state_vector: StateVector {
            client_id: origin.clone(),
            clock: clock + 1,
        },
        origin: origin.clone(),
    };

    // First update applied once
    assert!(doc.merge(&update1));
    let after_update1_state = doc.state_vector.clone();
    let after_update1_content = doc.content.clone();
    let after_update1_client_clocks = doc.client_clocks.clone();

    // Reapplying the same logical update (same origin + same clock) is a no-op
    assert!(!doc.merge(&update1));
    assert_eq!(doc.state_vector, after_update1_state);
    assert_eq!(doc.content, after_update1_content);
    assert_eq!(doc.client_clocks, after_update1_client_clocks);

    // Applying a *different* update (different clock) should be accepted
    assert!(doc.merge(&update2));
}

/// Test that same-client outdated update with lower clock is ignored
#[tokio::test]
async fn same_client_outdated_update_with_lower_clock_is_ignored() {
    let doc_client_id = Uuid::new_v4();
    let mut doc = CrdtDocumentState::new(&doc_client_id);

    let client_id = "client-a".to_string();
    let initial_clock = 5;

    let first_update = SyncUpdate {
        update: "first_content".to_string(),
        state_vector: StateVector {
            client_id: client_id.clone(),
            clock: initial_clock,
        },
        origin: "client-a".to_string(),
    };

    // Apply the initial update so that client_clocks[client_id] = N
    let merged_first = doc.merge(&first_update);
    assert!(merged_first, "first update from client should be merged");

    // Capture state after first merge
    let clock_before = doc.state_vector.clock;
    let client_clock_before = *doc
        .client_clocks
        .get(&client_id)
        .expect("client clock should be tracked after first merge");
    let content_count_before = doc.content.len();

    // Now create an outdated update from the same client with clock < N
    let outdated_update = SyncUpdate {
        update: "outdated_content".to_string(),
        state_vector: StateVector {
            client_id: client_id.clone(),
            clock: initial_clock - 1,
        },
        origin: "client-a".to_string(),
    };

    // Merge the outdated update
    let merged_outdated = doc.merge(&outdated_update);

    // Assert: merge returns false and no additional content is created
    assert!(
        !merged_outdated,
        "outdated same-client update with lower clock should be ignored"
    );

    assert_eq!(
        content_count_before,
        doc.content.len(),
        "no new content entries should be created when outdated updates are ignored"
    );

    // Verify clocks are unchanged
    assert_eq!(
        clock_before, doc.state_vector.clock,
        "global state_vector.clock should remain unchanged for ignored updates"
    );
    assert_eq!(
        client_clock_before,
        *doc.client_clocks.get(&client_id).expect("client clock should still be present"),
        "client_clocks entry should not be modified for ignored outdated updates"
    );
}

/// Test that same-client outdated update with equal clock is ignored
#[tokio::test]
async fn same_client_outdated_update_with_equal_clock_is_ignored() {
    let doc_client_id = Uuid::new_v4();
    let mut doc = CrdtDocumentState::new(&doc_client_id);

    let client_id = "client-a".to_string();
    let initial_clock = 5;

    let first_update = SyncUpdate {
        update: "first_content".to_string(),
        state_vector: StateVector {
            client_id: client_id.clone(),
            clock: initial_clock,
        },
        origin: "client-a".to_string(),
    };

    // Apply the initial update so that client_clocks[client_id] = N
    let merged_first = doc.merge(&first_update);
    assert!(merged_first, "first update from client should be merged");

    // Capture state after first merge
    let clock_before = doc.state_vector.clock;
    let client_clock_before = *doc
        .client_clocks
        .get(&client_id)
        .expect("client clock should be tracked after first merge");
    let content_count_before = doc.content.len();

    // Now create an outdated update from the same client with clock == N
    let equal_clock_update = SyncUpdate {
        update: "equal_clock_content".to_string(),
        state_vector: StateVector {
            client_id: client_id.clone(),
            clock: initial_clock,
        },
        origin: "client-a".to_string(),
    };

    // Merge the equal-clock update
    let merged_equal = doc.merge(&equal_clock_update);

    // Assert: merge returns false and no additional content is created
    assert!(
        !merged_equal,
        "same-client update with equal clock should be treated as outdated and ignored"
    );

    assert_eq!(
        content_count_before,
        doc.content.len(),
        "no new content entries should be created when equal-clock updates are ignored"
    );

    // Verify clocks are unchanged
    assert_eq!(
        clock_before, doc.state_vector.clock,
        "global state_vector.clock should remain unchanged for equal-clock ignored updates"
    );
    assert_eq!(
        client_clock_before,
        *doc.client_clocks.get(&client_id).expect("client clock should still be present"),
        "client_clocks entry should not be modified for equal-clock ignored updates"
    );
}

// Tests for how distinct updates are reflected in `content` keys.
// These specifically validate that the `content_key` encoding of
// `origin`, `clock`, and `update` yields unique keys for each
// accepted update.
#[cfg(test)]
mod content_key_distinctness_tests {
    use super::*;

    /// Multiple updates from the same client, with monotonic clocks and
    /// different payloads, must produce distinct `content` keys.
    #[tokio::test]
    async fn content_keys_are_distinct_for_same_client_monotonic_clocks() {
        let doc_client_id = Uuid::new_v4();
        let mut doc = CrdtDocumentState::new(&doc_client_id);

        // First update from client 1.
        let update1 = SyncUpdate {
            update: "payload-1".to_string(),
            state_vector: StateVector {
                client_id: doc_client_id.to_string(),
                clock: doc.state_vector.clock,
            },
            origin: "client-1".to_string(),
        };
        doc.merge(&update1);

        let keys_after_first: HashSet<String> = doc.content.keys().cloned().collect();
        assert_eq!(keys_after_first.len(), 1);

        // Second update from the same client with a different payload.
        // The merge logic will advance the local clock.
        let update2 = SyncUpdate {
            update: "payload-2".to_string(),
            state_vector: StateVector {
                client_id: doc_client_id.to_string(),
                clock: doc.state_vector.clock,
            },
            origin: "client-1".to_string(),
        };
        doc.merge(&update2);

        let keys_after_second: HashSet<String> = doc.content.keys().cloned().collect();

        // We should have one additional key and the sets should differ.
        assert_eq!(keys_after_second.len(), 2);
        assert_ne!(keys_after_first, keys_after_second);
    }

    /// Updates from different clients with overlapping clocks must also
    /// produce distinct `content` keys, ensuring that conflict resolution
    /// preserves all accepted updates in the `content` map.
    #[tokio::test]
    async fn content_keys_are_distinct_for_different_clients_overlapping_clocks() {
        let doc_client_id = Uuid::new_v4();
        let client1_id = Uuid::new_v4();
        let client2_id = Uuid::new_v4();
        let mut doc = CrdtDocumentState::new(&doc_client_id);

        // Update from client 1 at logical clock 1.
        let update_from_client1 = SyncUpdate {
            update: "client-1-payload".to_string(),
            state_vector: StateVector {
                client_id: client1_id.to_string(),
                clock: 1,
            },
            origin: "client-1".to_string(),
        };
        doc.merge(&update_from_client1);

        // Update from client 2 at the same logical clock 1.
        let update_from_client2 = SyncUpdate {
            update: "client-2-payload".to_string(),
            state_vector: StateVector {
                client_id: client2_id.to_string(),
                clock: 1,
            },
            origin: "client-2".to_string(),
        };
        doc.merge(&update_from_client2);

        // Both updates should be represented by distinct keys.
        assert_eq!(
            doc.content.len(),
            2,
            "expected two distinct content entries for overlapping clocks from different origins"
        );

        let keys: Vec<String> = doc.content.keys().cloned().collect();
        assert_eq!(keys.len(), 2);
        assert_ne!(keys[0], keys[1], "content keys must be distinct");
    }
}
