use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::CursorPosition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEntry {
    pub user_id: Uuid,
    pub display_name: String,
    pub color: String,
    pub cursor: Option<CursorPosition>,
    pub last_active: DateTime<Utc>,
    pub document_id: Uuid,
}

impl PresenceEntry {
    pub fn new(
        user_id: Uuid,
        display_name: String,
        color: String,
        document_id: Uuid,
    ) -> Self {
        Self {
            user_id,
            display_name,
            color,
            cursor: None,
            last_active: Utc::now(),
            document_id,
        }
    }

    pub fn update_cursor(&mut self, cursor: CursorPosition) {
        self.cursor = Some(cursor);
        self.last_active = Utc::now();
    }

    pub fn is_active(&self) -> bool {
        let timeout = chrono::Duration::seconds(30);
        Utc::now() - self.last_active < timeout
    }
}

#[derive(Default)]
pub struct PresenceStore {
    entries: Arc<Mutex<HashMap<Uuid, PresenceEntry>>>,
}

impl PresenceStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_presence(&self, entry: PresenceEntry) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(entry.user_id, entry);
    }

    pub fn remove_presence(&self, user_id: Uuid) {
        let mut entries = self.entries.lock().unwrap();
        entries.remove(&user_id);
    }

    pub fn get_presence(&self, user_id: Uuid) -> Option<PresenceEntry> {
        let entries = self.entries.lock().unwrap();
        entries.get(&user_id).cloned()
    }

    pub fn get_document_presence(&self, document_id: Uuid) -> Vec<PresenceEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .values()
            .filter(|e| e.document_id == document_id && e.is_active())
            .cloned()
            .collect()
    }

    pub fn update_cursor(&self, user_id: Uuid, cursor: CursorPosition) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get_mut(&user_id) {
            entry.update_cursor(cursor);
        }
    }

    pub fn cleanup_stale_entries(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|_, entry| entry.is_active());
    }
}

pub static PRESENCE_STORE: once_cell::sync::Lazy<PresenceStore> =
    once_cell::sync::Lazy::new(PresenceStore::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_entry_is_active() {
        let mut entry = PresenceEntry::new(
            Uuid::new_v4(),
            "Test User".to_string(),
            "#FF0000".to_string(),
            Uuid::new_v4(),
        );

        assert!(entry.is_active());

        entry.last_active = Utc::now() - chrono::Duration::seconds(60);
        assert!(!entry.is_active());
    }

    #[test]
    fn test_presence_store_operations() {
        let store = PresenceStore::new();
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        let entry = PresenceEntry::new(
            user_id,
            "Test User".to_string(),
            "#FF0000".to_string(),
            document_id,
        );

        store.set_presence(entry.clone());

        assert_eq!(store.get_presence(user_id), Some(entry));

        store.remove_presence(user_id);
        assert_eq!(store.get_presence(user_id), None);
    }

    #[test]
    fn test_get_document_presence() {
        let store = PresenceStore::new();
        let document_id = Uuid::new_v4();

        for i in 0..3 {
            let entry = PresenceEntry::new(
                Uuid::new_v4(),
                format!("User {}", i),
                format!("#{:02X}00", i * 50),
                document_id,
            );
            store.set_presence(entry);
        }

        let other_doc_entry = PresenceEntry::new(
            Uuid::new_v4(),
            "Other User".to_string(),
            "#000000".to_string(),
            Uuid::new_v4(),
        );
        store.set_presence(other_doc_entry);

        let doc_presence = store.get_document_presence(document_id);
        assert_eq!(doc_presence.len(), 3);
    }
}
