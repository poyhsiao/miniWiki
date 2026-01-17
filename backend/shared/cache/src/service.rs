use redis::AsyncCommands;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;
use tracing::{error, warn};

use crate::error::{CacheEntry, CacheError, TTL_DEFAULT};

struct InMemoryCache {
    data: RwLock<std::collections::HashMap<String, CacheEntry<String>>>,
}

impl InMemoryCache {
    fn new() -> Self {
        Self {
            data: RwLock::new(std::collections::HashMap::new()),
        }
    }

    async fn get(&self, key: &str) -> Option<String> {
        let entry = {
            let data = self.data.read().await;
            data.get(key).cloned()
        };

        if let Some(entry) = entry {
            if entry.is_expired() {
                // Drop read lock and acquire write lock to evict
                let mut data = self.data.write().await;
                // Double check if it's still there and still expired
                if let Some(entry) = data.get(key) {
                    if entry.is_expired() {
                        data.remove(key);
                    }
                }
                None
            } else {
                Some(entry.data.clone())
            }
        } else {
            None
        }
    }

    async fn set(&self, key: &str, value: String, ttl: u64) {
        let entry = CacheEntry::new(value.clone(), ttl);
        self.data.write().await.insert(key.to_string(), entry);
    }

    async fn delete(&self, key: &str) {
        self.data.write().await.remove(key);
    }
}

pub struct CacheService {
    redis: Option<redis::aio::MultiplexedConnection>,
    fallback: InMemoryCache,
    is_redis_available: RwLock<bool>,
}

impl CacheService {
    pub async fn new(redis_url: Option<String>) -> Result<Self, CacheError> {
        let (redis, available) = if let Some(url) = redis_url {
            match redis::Client::open(url) {
                Ok(client) => {
                    match client.get_multiplexed_async_connection().await {
                        Ok(conn) => (Some(conn), true),
                        Err(_) => (None, false),
                    }
                },
                Err(_) => (None, false),
            }
        } else {
            (None, false)
        };

        Ok(Self {
            redis,
            fallback: InMemoryCache::new(),
            is_redis_available: RwLock::new(available),
        })
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: DeserializeOwned + 'static,
    {
        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            match conn.get::<_, Option<String>>(key).await {
                Ok(Some(data)) => {
                    match serde_json::from_str::<CacheEntry<T>>(&data) {
                        Ok(entry) => {
                            if !entry.is_expired() {
                                return Ok(Some(entry.data));
                            } else {
                                return Ok(None);
                            }
                        }
                        Err(e) => {
                            error!("Redis cache deserialization failed for key {}: {}", key, e);
                            // Treat deserialization failure as a cache miss, like Redis failures
                            return Ok(None);
                        }
                    }
                }
                Ok(None) => {
                    // Cache miss in Redis, proceed to fallback
                }
                Err(e) => {
                    error!("Redis 'GET' operation failed for key {}: {}", key, e);
                    // Do not return error, proceed to fallback
                }
            }
        }

        let value = self.fallback.get(key).await;
        if let Some(json) = value {
            match serde_json::from_str::<CacheEntry<T>>(&json) {
                Ok(entry) => {
                    if entry.is_expired() {
                        Ok(None)
                    } else {
                        Ok(Some(entry.data))
                    }
                }
                Err(e) => {
                    error!("In-memory cache deserialization failed for key {}: {}", key, e);
                    // Treat deserialization failure as a cache miss
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T) -> Result<(), CacheError>
    where
        T: Serialize + Clone + 'static,
    {
        self.set_with_ttl(key, value, TTL_DEFAULT).await
    }

    pub async fn set_with_ttl<T>(&self, key: &str, value: &T, ttl: u64) -> Result<(), CacheError>
    where
        T: Serialize + Clone + 'static,
    {
        let entry = CacheEntry::new(value.clone(), ttl);
        let entry_json = serde_json::to_string(&entry)
            .map_err(CacheError::Serialization)?;

        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            match conn.set_ex::<_, _, ()>(key, &entry_json, ttl).await {
                Ok(_) => {
                    // Redis write succeeded, also write to fallback for consistency
                    self.fallback.set(key, entry_json.clone(), ttl).await;
                }
                Err(e) => {
                    // Redis write failed, log warning (silent degradation) and write to fallback
                    warn!("Redis 'SET_EX' operation failed for key {}: {}", key, e);
                    self.fallback.set(key, entry_json.clone(), ttl).await;
                    // Do not propagate error, return success to caller
                }
            }
        } else {
            self.fallback.set(key, entry_json, ttl).await;
        }

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            if let Err(e) = conn.del::<_, ()>(key).await {
                error!("Redis 'DEL' operation failed for key {}: {}", key, e);
            }
        }

        self.fallback.delete(key).await;
        Ok(())
    }

    pub async fn clear_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        // Validate pattern is not empty - empty patterns match nothing
        if pattern.is_empty() {
            return Ok(());
        }

        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            let mut cursor = 0u64;
            loop {
                let res: Result<(u64, Vec<String>), _> = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(pattern)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(&mut conn)
                    .await;

                match res {
                    Ok(result) => {
                        cursor = result.0;
                        let keys = result.1;

                        if !keys.is_empty() {
                            if let Err(e) = conn.del::<_, ()>(&keys).await {
                                tracing::warn!("Redis 'DEL' operation failed during clear_pattern: {}", e);
                            }
                        }

                        if cursor == 0 {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Redis 'SCAN' operation failed in clear_pattern: {}", e);
                        // Continue to clear in-memory fallback despite Redis error
                        break;
                    }
                }
            }
        }

        // Also clear in-memory cache
        let mut data = self.fallback.data.write().await;

        // Use a simple prefix match if it's just a "prefix*" pattern
        if pattern.ends_with('*') && !pattern[..pattern.len()-1].contains(|c| c == '*' || c == '?') {
            let prefix = &pattern[..pattern.len() - 1];
            data.retain(|k, _| !k.starts_with(prefix));
        } else {
            // Complex pattern: escape regex metacharacters, then convert glob wildcards
            let escaped = regex::escape(pattern);
            let regex_pattern = escaped.replace("\\*", ".*").replace("\\?", ".");
            let re = regex::Regex::new(&format!("^{}$", regex_pattern))
                .map_err(CacheError::InvalidPattern)?;
            data.retain(|k, _| !re.is_match(k));
        }

        Ok(())
    }

    pub async fn is_redis_available(&self) -> bool {
        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            // Perform a lightweight PING to check connection health
            let result: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;
            let is_alive = result.is_ok();

            // Update the state
            let mut available = self.is_redis_available.write().await;
            *available = is_alive;

            is_alive
        } else {
            false
        }
    }

    pub fn build_key(prefix: &str, id: &str) -> String {
        format!("{}{}", prefix, id)
    }

    pub fn build_composite_key(prefix: &str, parts: &[&str]) -> String {
        format!("{}{}", prefix, parts.join(":"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_cache_service_basic() {
        let service = CacheService::new(None).await.unwrap();
        service.set("test_key", &"test_value").await.unwrap();
        let result: Option<String> = service.get("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        service.delete("test_key").await.unwrap();
        let result: Option<String> = service.get("test_key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let service = CacheService::new(None).await.unwrap();
        service.set_with_ttl("ttl_key", &"will_expire", 1).await.unwrap();

        let result: Option<String> = service.get("ttl_key").await.unwrap();
        assert!(result.is_some());

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let result: Option<String> = service.get("ttl_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_service_complex_type() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
        struct Complex {
            name: String,
            count: i32,
        }
        let service = CacheService::new(None).await.unwrap();
        let value = Complex { name: "test".to_string(), count: 42 };
        service.set("complex_key", &value).await.unwrap();
        let result: Option<Complex> = service.get("complex_key").await.unwrap();
        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn test_in_memory_eviction() {
        let service = CacheService::new(None).await.unwrap();
        service.set_with_ttl("expire_me", &"value", 1).await.unwrap();

        // Wait for it to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // This should return None AND trigger eviction
        let result: Option<String> = service.get("expire_me").await.unwrap();
        assert!(result.is_none());

        // Verify it's actually gone from the map
        let data = service.fallback.data.read().await;
        assert!(!data.contains_key("expire_me"), "Expired key should be evicted from the map");
    }

    #[tokio::test]
    async fn test_consistent_serialization_error() {
        let service = CacheService::new(None).await.unwrap();
        // Manually insert malformed JSON into fallback
        {
            let mut data = service.fallback.data.write().await;
            let now = Utc::now().timestamp();
            data.insert("bad_json".to_string(), CacheEntry {
                // We use a raw string that is definitely NOT a valid serialized CacheEntry
                // The CacheService::get will try to deserialize this string as CacheEntry<T>
                // "{ invalid json }" is treated as JSON object start but invalid syntax
                data: "{ invalid json }".to_string(),
                expires_at: now + 3600,
                cached_at: now,
            });
        }

        // get<T> with a type that deserialization fails for should treat it as a cache miss
        // and return Ok(None) instead of an error. Requesting u32 will fail since the stored
        // data is a plain string, not a serialized CacheEntry<u32>.
        let result: Result<Option<u32>, _> = service.get("bad_json").await;
        assert!(result.is_ok(), "Should return Ok, not an error");
        assert_eq!(result.unwrap(), None, "Deserialization failure should be treated as cache miss");
    }

    #[tokio::test]
    async fn test_redis_failure_fallback() {
        // We'll simulate Redis failure by providing a definitely invalid URL,
        // though our current 'new' implementation already handles connection failure
        // by setting self.redis to None.
        // To really test the write error handling when self.redis is Some but fails,
        // we'd need a mock Redis connection, but we can at least verify basic flow.

        let service = CacheService::new(None).await.unwrap();
        // Even without Redis, set should succeed using fallback
        let res = service.set("fallback_key", &"val").await;
        assert!(res.is_ok());

        let val: Option<String> = service.get("fallback_key").await.unwrap();
        assert_eq!(val, Some("val".to_string()));
    }

    #[tokio::test]
    async fn test_clear_pattern_complex() {
        let service = CacheService::new(None).await.unwrap();
        service.set("user:1:profile", &"data1").await.unwrap();
        service.set("user:2:profile", &"data2").await.unwrap();
        service.set("admin:1:profile", &"admin_data").await.unwrap();

        // Clear all user profiles using glob
        service.clear_pattern("user:*:profile").await.unwrap();

        let val1: Option<String> = service.get("user:1:profile").await.unwrap();
        let val2: Option<String> = service.get("user:2:profile").await.unwrap();
        let val3: Option<String> = service.get("admin:1:profile").await.unwrap();

        assert_eq!(val1, None);
        assert_eq!(val2, None);
        assert_eq!(val3, Some("admin_data".to_string()));
    }

    #[tokio::test]
    async fn test_set_writes_to_both_stores() {
        // Create a mock Redis service (we'll simulate with in-memory for this test)
        // In a real scenario, we'd use a mock Redis connection
        let service = CacheService::new(None).await.unwrap();

        // Set a value
        service.set_with_ttl("test_key", &"test_value", 3600).await.unwrap();

        // Verify it's in the fallback store
        let fallback_value = service.fallback.get("test_key").await;
        assert!(fallback_value.is_some(), "Value should be in fallback store");

        // Verify we can retrieve it
        let retrieved: Option<String> = service.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_delete_removes_from_both_stores() {
        let service = CacheService::new(None).await.unwrap();

        // Set a value
        service.set("test_key", &"test_value").await.unwrap();

        // Verify it exists
        let before: Option<String> = service.get("test_key").await.unwrap();
        assert_eq!(before, Some("test_value".to_string()));

        // Delete it
        service.delete("test_key").await.unwrap();

        // Verify it's gone from fallback
        let fallback_value = service.fallback.get("test_key").await;
        assert!(fallback_value.is_none(), "Value should be removed from fallback store");

        // Verify we can't retrieve it
        let after: Option<String> = service.get("test_key").await.unwrap();
        assert_eq!(after, None);
    }
}
