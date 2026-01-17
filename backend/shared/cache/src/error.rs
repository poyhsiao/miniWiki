// use thiserror::Error;
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// Cache key prefix constants
pub const CACHE_PREFIX_USER: &str = "user:";
pub const CACHE_PREFIX_DOCUMENT: &str = "doc:";
pub const CACHE_PREFIX_SPACE: &str = "space:";
pub const CACHE_PREFIX_SESSION: &str = "session:";

/// TTL for different cache types (in seconds)
pub const TTL_DEFAULT: u64 = 3600;  // 1 hour
pub const TTL_SHORT: u64 = 300;      // 5 minutes
pub const TTL_LONG: u64 = 86400;    // 24 hours

/// Cache error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    RedisConnection(#[source] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache key not found")]
    NotFound,

    #[error("Invalid cache key: {0}")]
    InvalidKey(String),

    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(#[from] regex::Error),
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub expires_at: i64,  // Unix timestamp
    pub cached_at: i64,  // Unix timestamp
}

impl<T: Serialize> CacheEntry<T> {
    pub fn new(data: T, ttl_seconds: u64) -> Self {
        let cached_at = Utc::now().timestamp();
        let safe_ttl = i64::try_from(ttl_seconds).unwrap_or(i64::MAX);
        let expires_at = cached_at.saturating_add(safe_ttl);
        Self {
            data,
            expires_at,
            cached_at,
        }
    }
}

impl<T> CacheEntry<T> {
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_overflow() {
        let data = "test".to_string();
        // Use a very large TTL
        let entry = CacheEntry::new(data, u64::MAX);
        assert!(entry.expires_at >= entry.cached_at);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_error_display() {
        let err = CacheError::InvalidKey("foo".to_string());
        assert_eq!(err.to_string(), "Invalid cache key: foo");
    }
}
