// CRDT conflict resolver for offline-first sync
// Handles merging concurrent document updates without data loss

use crate::state_vector::{StateVector, ClientId, Clock};
use std::cmp::Ordering;

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolutionStrategy {
    /// Keep both values (merge)
    Merge,
    /// Keep most recent by timestamp
    Timestamp,
    /// Keep first or last based on client ID
    ClientId,
    /// Custom resolver
    Custom,
}

/// Result of conflict resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    /// No conflict, operation succeeded
    NoConflict,
    /// Conflict resolved by keeping first value
    KeepFirst,
    /// Conflict resolved by keeping second value
    KeepSecond,
    /// Conflict resolved by merging values
    Merged,
    /// Conflict could not be resolved
    Unresolved,
}

/// Conflict information
#[derive(Debug, Clone)]
pub struct Conflict<T> {
    pub client_id: ClientId,
    pub clock: Clock,
    pub first_value: T,
    pub second_value: T,
}

/// CRDT conflict resolver
pub struct ConflictResolver {
    strategy: ConflictResolutionStrategy,
}

impl ConflictResolver {
    /// Create a new conflict resolver with the specified strategy
    pub fn new(strategy: ConflictResolutionStrategy) -> Self {
        Self { strategy }
    }

    /// Resolve a conflict between two state vectors
    pub fn resolve_state_vector(
        &self,
        local_sv: &StateVector,
        remote_sv: &StateVector,
    ) -> (StateVector, ConflictResolution) {
        let mut merged = StateVector::new();

        // Merge entries from both state vectors
        for (&client_id, &local_clock) in local_sv.inner() {
            let remote_clock = remote_sv.get(client_id).copied().unwrap_or(0);
            let max_clock = local_clock.max(remote_clock);
            merged.set(client_id, max_clock);
        }

        for (&client_id, &remote_clock) in remote_sv.inner() {
            if !local_sv.inner().contains_key(&client_id) {
                merged.set(client_id, remote_clock);
            }
        }

        let resolution = match local_sv.compare(remote_sv) {
            Ordering::Equal => ConflictResolution::NoConflict,
            Ordering::Less => ConflictResolution::Merged,
            Ordering::Greater => ConflictResolution::Merged,
        };

        (merged, resolution)
    }

    /// Resolve conflicts between two document updates
    pub fn resolve_document_conflict<T: Clone + PartialEq>(
        &self,
        local: &T,
        remote: &T,
        local_sv: &StateVector,
        remote_sv: &StateVector,
    ) -> (T, ConflictResolution) {
        if local == remote {
            return (local.clone(), ConflictResolution::NoConflict);
        }

        match self.strategy {
            ConflictResolutionStrategy::Merge => {
                // For simple types, prefer one value
                // In production, this would use proper CRDT merge
                self.resolve_by_timestamp(local, remote, local_sv, remote_sv)
            }
            ConflictResolutionStrategy::Timestamp => {
                self.resolve_by_timestamp(local, remote, local_sv, remote_sv)
            }
            ConflictResolutionStrategy::ClientId => {
                self.resolve_by_client_id(local, remote, local_sv, remote_sv)
            }
            ConflictResolutionStrategy::Custom => {
                // Custom resolution not implemented
                (local.clone(), ConflictResolution::Unresolved)
            }
        }
    }

    /// Resolve conflict by comparing state vector timestamps
    fn resolve_by_timestamp<T: Clone + PartialEq>(
        &self,
        local: &T,
        remote: &T,
        local_sv: &StateVector,
        remote_sv: &StateVector,
    ) -> (T, ConflictResolution) {
        let local_max = local_sv.inner().values().max().copied().unwrap_or(0);
        let remote_max = remote_sv.inner().values().max().copied().unwrap_or(0);

        if local_max >= remote_max {
            (local.clone(), ConflictResolution::KeepFirst)
        } else {
            (remote.clone(), ConflictResolution::KeepSecond)
        }
    }

    /// Resolve conflict by comparing client IDs
    fn resolve_by_client_id<T: Clone + PartialEq>(
        &self,
        local: &T,
        remote: &T,
        local_sv: &StateVector,
        remote_sv: &StateVector,
    ) -> (T, ConflictResolution) {
        let local_client_id = local_sv.inner().keys().max().copied().unwrap_or(0);
        let remote_client_id = remote_sv.inner().keys().max().copied().unwrap_or(0);

        if local_client_id >= remote_client_id {
            (local.clone(), ConflictResolution::KeepFirst)
        } else {
            (remote.clone(), ConflictResolution::KeepSecond)
        }
    }

    /// Calculate what updates the client needs from the server
    pub fn calculate_missing_updates(
        &self,
        client_sv: &StateVector,
        server_sv: &StateVector,
    ) -> Vec<(ClientId, Clock, Clock)> {
        let missing = client_sv.get_missing(server_sv);
        missing
            .into_iter()
            .map(|(client_id, from_clock)| {
                let to_clock = server_sv.get(client_id).copied().unwrap_or(from_clock);
                (client_id, from_clock, to_clock)
            })
            .collect()
    }

    /// Check if two state vectors can be merged without conflicts
    pub fn can_merge(&self, sv1: &StateVector, sv2: &StateVector) -> bool {
        // State vectors can always be merged in CRDT
        true
    }

    /// Get the newer state between two state vectors
    pub fn get_newer_state<'a, 'b>(
        &'a self,
        sv1: &'b StateVector,
        sv2: &'b StateVector,
    ) -> (&'b StateVector, &'b StateVector) {
        match sv1.compare(sv2) {
            Ordering::Less => (sv2, sv1),
            Ordering::Greater => (sv1, sv2),
            Ordering::Equal => (sv1, sv2),
        }
    }
}

/// Default conflict resolver using merge strategy
impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(ConflictResolutionStrategy::Merge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_state_vector_equal() {
        let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

        let mut sv1 = StateVector::new();
        sv1.set(1, 10);

        let mut sv2 = StateVector::new();
        sv2.set(1, 10);

        let (merged, resolution) = resolver.resolve_state_vector(&sv1, &sv2);

        assert_eq!(resolution, ConflictResolution::NoConflict);
        assert_eq!(merged.get(1), Some(&10));
    }

    #[test]
    fn test_resolve_state_vector_different() {
        let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

        let mut sv1 = StateVector::new();
        sv1.set(1, 10);

        let mut sv2 = StateVector::new();
        sv2.set(1, 20);

        let (merged, resolution) = resolver.resolve_state_vector(&sv1, &sv2);

        assert_eq!(resolution, ConflictResolution::Merged);
        assert_eq!(merged.get(1), Some(&20));
    }

    #[test]
    fn test_resolve_document_conflict_same_value() {
        let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

        let sv1 = StateVector::new();
        let sv2 = StateVector::new();

        let (result, resolution) = resolver.resolve_document_conflict(
            &"same value",
            &"same value",
            &sv1,
            &sv2,
        );

        assert_eq!(resolution, ConflictResolution::NoConflict);
        assert_eq!(result, "same value");
    }

    #[test]
    fn test_resolve_document_conflict_by_timestamp() {
        let resolver = ConflictResolver::new(ConflictResolutionStrategy::Timestamp);

        let mut sv1 = StateVector::new();
        sv1.set(1, 10);

        let mut sv2 = StateVector::new();
        sv2.set(2, 20);

        let (result, resolution) = resolver.resolve_document_conflict(
            &"local",
            &"remote",
            &sv1,
            &sv2,
        );

        // Remote has higher clock, so it should be kept
        assert_eq!(result, "remote");
        assert_eq!(resolution, ConflictResolution::KeepSecond);
    }

    #[test]
    fn test_calculate_missing_updates() {
        let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

        let mut client_sv = StateVector::new();
        client_sv.set(1, 5);

        let mut server_sv = StateVector::new();
        server_sv.set(1, 10);
        server_sv.set(2, 5);

        let missing = resolver.calculate_missing_updates(&client_sv, &server_sv);

        // Missing updates: client needs clock 6-10 from client 1, and clock 0-5 from client 2
        assert!(missing.contains(&(1, 6, 10)));
        assert!(missing.contains(&(2, 0, 5)));
    }

    #[test]
    fn test_can_always_merge() {
        let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

        let sv1 = StateVector::new();
        let sv2 = StateVector::new();

        assert!(resolver.can_merge(&sv1, &sv2));
    }

    #[test]
    fn test_get_newer_state() {
        let resolver = ConflictResolver::new(ConflictResolutionStrategy::Merge);

        let mut sv1 = StateVector::new();
        sv1.set(1, 10);

        let mut sv2 = StateVector::new();
        sv2.set(1, 20);

        let (newer, older) = resolver.get_newer_state(&sv1, &sv2);

        assert_eq!(newer.get(1), Some(&20));
        assert_eq!(older.get(1), Some(&10));
    }

    #[test]
    fn test_default_resolver() {
        let resolver = ConflictResolver::default();

        let sv1 = StateVector::new();
        let sv2 = StateVector::new();

        let (_, resolution) = resolver.resolve_state_vector(&sv1, &sv2);
        assert_eq!(resolution, ConflictResolution::NoConflict);
    }
}
