use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cmp::Ordering;

pub type ClientId = u64;
pub type Clock = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVector(HashMap<ClientId, Clock>);

impl StateVector {
    pub fn new() -> Self {
        StateVector(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        StateVector(HashMap::with_capacity(capacity))
    }

    pub fn get(&self, client_id: ClientId) -> Option<&Clock> {
        self.0.get(&client_id)
    }

    pub fn set(&mut self, client_id: ClientId, clock: Clock) {
        self.0.insert(client_id, clock);
    }

    pub fn increment(&mut self, client_id: ClientId) -> Clock {
        let new_clock = self.0.get(&client_id).map(|c| *c + 1).unwrap_or(1);
        self.0.insert(client_id, new_clock);
        new_clock
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a reference to the internal HashMap (for internal use)
    pub fn inner(&self) -> &HashMap<ClientId, Clock> {
        &self.0
    }

    /// Get a mutable reference to the internal HashMap (for internal use)
    pub fn inner_mut(&mut self) -> &mut HashMap<ClientId, Clock> {
        &mut self.0
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by_key(|(&k, _)| k);

        for (&client_id, &clock) in entries {
            bytes.extend_from_slice(&client_id.to_le_bytes());
            bytes.extend_from_slice(&clock.to_le_bytes());
        }

        bytes
    }

    pub fn decode(data: &[u8]) -> Result<Self, StateVectorError> {
        let mut sv = StateVector::with_capacity(data.len() / 16);

        let mut pos = 0;
        while pos + 16 <= data.len() {
            let client_id = u64::from_le_bytes(
                data[pos..pos + 8].try_into().map_err(|_| StateVectorError)?
            );
            let clock = u64::from_le_bytes(
                data[pos + 8..pos + 16].try_into().map_err(|_| StateVectorError)?
            );
            sv.set(client_id, clock);
            pos += 16;
        }

        if pos != data.len() {
            return Err(StateVectorError);
        }

        Ok(sv)
    }

    /// Compares this state vector with another to determine their ordering.
    ///
    /// Returns `Ordering::Greater` if this vector has entries the other lacks
    /// (indicating this is newer), `Ordering::Less` if the other has higher
    /// clock values for shared entries, or `Ordering::Equal` if they are
    /// equivalent.
    ///
    /// This follows CRDT semantics where a vector with more information is
    /// considered "greater" for synchronization purposes.
    pub fn compare(&self, other: &StateVector) -> Ordering {
        for (&client_id, &clock) in &self.0 {
            match other.get(client_id) {
                Some(other_clock) if *other_clock > clock => return Ordering::Less,
                Some(other_clock) if *other_clock < clock => return Ordering::Greater,
                None => return Ordering::Greater,
                _ => {}
            }
        }
        Ordering::Equal
    }

    pub fn is_ancestor_of(&self, other: &StateVector) -> bool {
        for (&client_id, &clock) in &self.0 {
            match other.get(client_id) {
                Some(other_clock) if *other_clock >= clock => {}
                Some(_) => return false,
                None => return false,
            }
        }
        true
    }

    /// Returns entries that are missing or have lower clocks in this vector
    /// compared to the other vector.
    ///
    /// For each entry in `other`, if `self` doesn't have it or has a lower
    /// clock value, returns a tuple of (client_id, starting_clock) indicating
    /// the client needs to sync from `starting_clock`.
    ///
    /// # Arguments
    /// * `other` - The state vector to compare against
    ///
    /// # Returns
    /// A vector of (client_id, from_clock) tuples representing missing entries
    pub fn get_missing(&self, other: &StateVector) -> Vec<(ClientId, Clock)> {
        let mut missing = Vec::new();
        // Check entries in other that self doesn't have or has lower clock
        for (&client_id, &clock) in other.0.iter() {
            match self.get(client_id) {
                Some(self_clock) if *self_clock < clock => {
                    missing.push((client_id, *self_clock + 1));
                }
                None => {
                    missing.push((client_id, 0));
                }
                _ => {}
            }
        }
        missing
    }
}

impl Default for StateVector {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for StateVector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for StateVector {}

#[derive(Debug, Clone)]
pub struct StateVectorError;

impl std::fmt::Display for StateVectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid state vector encoding")
    }
}

impl std::error::Error for StateVectorError {}

impl From<std::array::TryFromSliceError> for StateVectorError {
    fn from(_: std::array::TryFromSliceError) -> Self {
        StateVectorError
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_vector_encode_decode() {
        let mut sv = StateVector::new();
        sv.set(1, 10);
        sv.set(2, 20);
        sv.set(3, 30);

        let encoded = sv.encode();
        let decoded = StateVector::decode(&encoded).unwrap();

        assert_eq!(sv, decoded);
        assert_eq!(decoded.get(1), Some(&10));
        assert_eq!(decoded.get(2), Some(&20));
        assert_eq!(decoded.get(3), Some(&30));
    }

    #[test]
    fn test_state_vector_compare() {
        let mut sv1 = StateVector::new();
        sv1.set(1, 10);

        let mut sv2 = StateVector::new();
        sv2.set(1, 20);

        assert_eq!(sv1.compare(&sv2), Ordering::Less);
        assert_eq!(sv2.compare(&sv1), Ordering::Greater);

        let sv3 = StateVector::new();
        assert_eq!(sv1.compare(&sv3), Ordering::Greater);
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

        let mut unrelated = StateVector::new();
        unrelated.set(3, 1);

        assert!(!ancestor.is_ancestor_of(&unrelated));
        assert!(!unrelated.is_ancestor_of(&ancestor));
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
}
