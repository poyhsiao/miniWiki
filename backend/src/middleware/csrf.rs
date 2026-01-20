use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, error::Error, http::{header, Method}, HttpRequest};
use std::future::{ready, Ready};
use std::pin::Pin;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use redis::AsyncCommands;

/// CSRF token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub token: String,
    pub expires_at: i64, // Unix timestamp
}

impl CsrfToken {
    pub fn new(ttl_seconds: i64) -> Self {
        let token = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let expires_at = now.saturating_add(ttl_seconds);
        Self { token, expires_at }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.expires_at
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CsrfConfig {
    #[serde(default = "default_cookie_name")]
    pub cookie_name: String,
    #[serde(default = "default_cookie_max_age")]
    pub cookie_max_age: u64,
    #[serde(default = "default_header_name")]
    pub header_name: String,
    #[serde(default = "default_secure_cookie")]
    pub secure_cookie: bool,
}

fn default_cookie_name() -> String { "csrf_token".to_string() }
fn default_cookie_max_age() -> u64 { 3600 }
fn default_header_name() -> String { "X-CSRF-Token".to_string() }
fn default_secure_cookie() -> bool { true }

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            cookie_name: default_cookie_name(),
            cookie_max_age: default_cookie_max_age(),
            header_name: default_header_name(),
            secure_cookie: default_secure_cookie(),
        }
    }
}

#[async_trait]
pub trait CsrfStore: Send + Sync {
    async fn generate(&self, session_id: &str, ttl: i64) -> Result<String, actix_web::Error>;
    async fn validate_and_consume(&self, session_id: &str, token: &str) -> bool;
    async fn cleanup_expired(&self);
}

pub struct InMemoryCsrfStore {
    // Changed from HashMap<String, CsrfToken> to HashMap<String, Vec<CsrfToken>>
    // to support multiple concurrent tokens per session (multi-tab scenarios)
    tokens: tokio::sync::RwLock<std::collections::HashMap<String, Vec<CsrfToken>>>,
    // Maximum number of tokens per session to prevent unbounded growth
    max_tokens_per_session: usize,
}

impl InMemoryCsrfStore {
    pub fn new() -> Self {
        Self {
            tokens: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            max_tokens_per_session: 5, // Allow up to 5 concurrent tabs
        }
    }

    pub fn with_max_tokens(max_tokens: usize) -> Self {
        Self {
            tokens: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            max_tokens_per_session: max_tokens,
        }
    }
}

#[async_trait]
impl CsrfStore for InMemoryCsrfStore {
    async fn generate(&self, session_id: &str, ttl: i64) -> Result<String, actix_web::Error> {
        let token = CsrfToken::new(ttl);
        let token_str = token.token.clone();

        let mut tokens_map = self.tokens.write().await;
        let session_tokens = tokens_map.entry(session_id.to_string()).or_insert_with(Vec::new);

        // Add new token to the collection
        session_tokens.push(token);

        // Enforce limit: remove oldest tokens if we exceed max_tokens_per_session
        if session_tokens.len() > self.max_tokens_per_session {
            let excess = session_tokens.len() - self.max_tokens_per_session;
            session_tokens.drain(0..excess);
            tracing::debug!(
                "Removed {} oldest CSRF tokens for session {} (limit: {})",
                excess,
                session_id,
                self.max_tokens_per_session
            );
        }

        Ok(token_str)
    }

    async fn validate_and_consume(&self, session_id: &str, token: &str) -> bool {
        // Removed inline cleanup_expired() call to avoid global write lock contention.
        // Cleanup is now handled by a background task.

        let mut tokens_map = self.tokens.write().await;

        if let Some(session_tokens) = tokens_map.get_mut(session_id) {
            // Find the matching, non-expired token
            if let Some(pos) = session_tokens.iter().position(|t| t.token == token && !t.is_expired()) {
                // Remove only the matched token (consume it)
                session_tokens.remove(pos);

                // Clean up empty session entries
                if session_tokens.is_empty() {
                    tokens_map.remove(session_id);
                }

                return true;
            }
        }

        false
    }

    async fn cleanup_expired(&self) {
        let mut tokens_map = self.tokens.write().await;

        // Remove expired tokens from each session's collection
        for (session_id, session_tokens) in tokens_map.iter_mut() {
            session_tokens.retain(|token| !token.is_expired());

            if session_tokens.is_empty() {
                tracing::debug!("All CSRF tokens expired for session {}", session_id);
            }
        }

        // Remove sessions with no valid tokens
        tokens_map.retain(|_, tokens| !tokens.is_empty());
    }
}

#[async_trait]
pub trait RedisConnection: Send + Sync {
    async fn add_token(&self, key: String, token: String, ttl: u64) -> Result<(), redis::RedisError>;
    async fn remove_token(&self, key: String, token: String) -> Result<bool, redis::RedisError>;
}

#[async_trait]
impl RedisConnection for redis::aio::MultiplexedConnection {
    async fn add_token(&self, key: String, token: String, ttl: u64) -> Result<(), redis::RedisError> {
        let mut conn = self.clone();
        let ttl_secs = ttl.try_into().unwrap_or(3600);
        
        // Add token to set; refresh TTL only when token is newly added
        // This prevents resetting the TTL on every token addition
        let added: bool = conn.sadd(&key, &token).await?;
        
        if added {
            // Token was newly added, refresh TTL to full duration
            // This extends the session window for all tokens in the set
            let _: () = conn.expire(&key, ttl_secs).await?;
        }
        
        Ok(())
    }

    async fn remove_token(&self, key: String, token: String) -> Result<bool, redis::RedisError> {
        let mut conn = self.clone();
        let removed: bool = conn.srem(key, token).await?;
        Ok(removed)
    }
}

pub struct RedisCsrfStore {
    redis: Arc<dyn RedisConnection>,
    prefix: String,
}

impl RedisCsrfStore {
    pub fn new(redis: Arc<dyn RedisConnection>) -> Self {
        Self {
            redis,
            prefix: "csrf:".to_string(),
        }
    }

    fn key(&self, session_id: &str) -> String {
        format!("{}{}", self.prefix, session_id)
    }
}

#[async_trait]
impl CsrfStore for RedisCsrfStore {
    async fn generate(&self, session_id: &str, ttl: i64) -> Result<String, actix_web::Error> {
        let token = Uuid::new_v4().to_string();

        let key = self.key(session_id);
        let u_ttl: u64 = if ttl > 0 {
            ttl.try_into().unwrap_or(3600)
        } else {
            log::warn!("Non-positive TTL provided: {}, using default", ttl);
            3600
        };

        self.redis.add_token(key, token.clone(), u_ttl)
            .await
            .map_err(|e| {
                log::error!("Failed to store CSRF token in Redis: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to store CSRF token")
            })?;

        Ok(token)
    }

    async fn validate_and_consume(&self, session_id: &str, token: &str) -> bool {
        let key = self.key(session_id);
        match self.redis.remove_token(key, token.to_string()).await {
            Ok(removed) => removed,
            Err(e) => {
                log::error!("Redis error during CSRF validation: {}", e);
                false
            }
        }
    }

    async fn cleanup_expired(&self) {
        // Redis handles TTL automatically
    }
}

pub struct CsrfMiddleware {
    pub config: CsrfConfig,
    pub store: Arc<dyn CsrfStore>,
}

impl CsrfMiddleware {
    pub fn new(config: CsrfConfig, store: Arc<dyn CsrfStore>) -> Self {
        Self { config, store }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CsrfMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfMiddlewareService {
            service: Arc::new(Mutex::new(service)),
            config: self.config.clone(),
            store: self.store.clone(),
        }))
    }
}

pub struct CsrfMiddlewareService<S> {
    service: Arc<Mutex<S>>,
    config: CsrfConfig,
    store: Arc<dyn CsrfStore>,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        if let Ok(mut svc) = self.service.try_lock() {
            svc.poll_ready(cx)
        } else {
            std::task::Poll::Pending
        }
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();
        let store = self.store.clone();

        Box::pin(async move {
            let method = req.method().clone();
            let session_id = get_session_id_from_request(req.request());

            if method == Method::OPTIONS {
                let mut svc = service.lock().await;
                return svc.call(req).await;
            }

            if method == Method::POST || method == Method::PUT || method == Method::PATCH || method == Method::DELETE {
                if let Some(ref sid) = session_id {
                    let token = get_csrf_token_from_request(req.request(), &config)?;
                    if !store.validate_and_consume(sid, &token).await {
                        return Err(actix_web::error::ErrorForbidden("Invalid or expired CSRF token"));
                    }
                }
            }

            let mut svc = service.lock().await;
            let mut res = svc.call(req).await;
            drop(svc); // Release lock before token generation

            if matches!(method, Method::POST | Method::PUT | Method::PATCH | Method::DELETE) {
                if session_id.is_some() {
                    if let Ok(ref mut response) = res {
                        if response.status().is_success() {
                            let ttl_i64 = i64::try_from(config.cookie_max_age).unwrap_or(i64::MAX);
                            if let Ok(token) = store.generate(session_id.as_ref().unwrap(), ttl_i64).await {
                                let display_ttl = config.cookie_max_age.min(i64::MAX as u64);
                                let mut cookie = format!("{}={}; SameSite=Strict; Path=/; Max-Age={}", config.cookie_name, token, display_ttl);
                                if config.secure_cookie {
                                    cookie.push_str("; Secure");
                                }
                                response.headers_mut().append(
                                    header::SET_COOKIE,
                                    header::HeaderValue::from_str(&cookie).map_err(actix_web::error::ErrorInternalServerError)?
                                );
                            } else {
                                tracing::error!("Failed to generate CSRF token");
                            }
                        }
                    }
                }
            }

            res
        })
    }
}

fn get_session_id_from_request(req: &HttpRequest) -> Option<String> {
    if let Some(cookie_header) = req.headers().get("Cookie").and_then(|h| h.to_str().ok()) {
        for part in cookie_header.split(';') {
            let part = part.trim();
            if part.starts_with("session_id=") {
                return part.strip_prefix("session_id=").map(|v| v.to_string());
            }
        }
    }
    None
}

fn get_csrf_token_from_request(req: &HttpRequest, config: &CsrfConfig) -> Result<String, Error> {
    if let Some(header) = req.headers().get(&config.header_name) {
        return header.to_str().map(|s| s.to_string()).map_err(|_| actix_web::error::ErrorBadRequest("Invalid CSRF header"));
    }
    Err(actix_web::error::ErrorBadRequest("Missing CSRF token"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App, web, HttpResponse};
    use actix_web::http::Method;

    struct MockStore;
    #[async_trait]
    impl CsrfStore for MockStore {
        async fn generate(&self, _sid: &str, _ttl: i64) -> Result<String, actix_web::Error> { Ok("new-token".to_string()) }
        async fn validate_and_consume(&self, _sid: &str, _token: &str) -> bool { true }
        async fn cleanup_expired(&self) {}
    }

    #[actix_web::test]
    async fn test_csrf_cookie_secure_flag() {
        let config = CsrfConfig {
            cookie_name: "csrf".to_string(),
            cookie_max_age: 3600,
            header_name: "X-CSRF".to_string(),
            secure_cookie: false, // Comp-error here initially
        };

        let store = Arc::new(MockStore);
        let middleware = CsrfMiddleware::new(config, store);

        let srv = test::init_service(
            App::new()
                .wrap(middleware)
                .default_service(web::to(|| async { HttpResponse::Ok().finish() }))
        ).await;

        let req = test::TestRequest::with_uri("/")
            .method(Method::POST)
            .insert_header(("Cookie", "session_id=123"))
            .insert_header(("X-CSRF", "old-token"))
            .to_request();

        let resp = test::call_service(&srv, req).await;
        let cookie = resp.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();

        assert!(!cookie.contains("Secure"), "Cookie should not contain 'Secure' when secure_cookie is false");
    }

    #[actix_web::test]
    async fn test_csrf_cookie_max_age_overflow() {
        let config = CsrfConfig {
            cookie_name: "csrf".to_string(),
            cookie_max_age: u64::MAX, // Extremely large value
            header_name: "X-CSRF".to_string(),
            secure_cookie: true,
        };

        let store = Arc::new(MockStore);
        let middleware = CsrfMiddleware::new(config, store);

        let srv = test::init_service(
            App::new()
                .wrap(middleware)
                .default_service(web::to(|| async { HttpResponse::Ok().finish() }))
        ).await;

        let req = test::TestRequest::with_uri("/")
            .method(Method::POST)
            .insert_header(("Cookie", "session_id=123"))
            .insert_header(("X-CSRF", "old-token"))
            .to_request();

        let resp = test::call_service(&srv, req).await;
        let cookie = resp.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();

        // If it overflows to negative, it might be a small number or negative.
        // We want to ensure it is correctly clamped or handled.
        assert!(cookie.contains("Max-Age=9223372036854775807"), "Cookie Max-Age should be clamped to i64::MAX");
    }

    struct MockRedis {
        // key -> (Set of tokens, ttl)
        data: tokio::sync::RwLock<std::collections::HashMap<String, (std::collections::HashSet<String>, u64)>>,
    }

    #[async_trait]
    impl RedisConnection for MockRedis {
        async fn add_token(&self, key: String, token: String, ttl: u64) -> Result<(), redis::RedisError> {
            let mut data = self.data.write().await;
            // Insert or update
            let entry = data.entry(key).or_insert((std::collections::HashSet::new(), ttl));
            let inserted = entry.0.insert(token);
            // Only refresh TTL when token was newly inserted
            if inserted {
                entry.1 = ttl;
            }
            Ok(())
        }
        async fn remove_token(&self, key: String, token: String) -> Result<bool, redis::RedisError> {
            let mut data = self.data.write().await;
            if let Some((set, _)) = data.get_mut(&key) {
                let removed = set.remove(&token);
                // Optional: clean up empty sets
                if set.is_empty() {
                    data.remove(&key);
                }
                Ok(removed)
            } else {
                Ok(false)
            }
        }
    }

    #[tokio::test]
    async fn test_redis_store_positive_ttl() {
        let mock_redis = Arc::new(MockRedis {
            data: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        });
        let store = RedisCsrfStore::new(mock_redis.clone());

        let session_id = "test-session";
        let ttl = 300;

        // Generate token
        let token = store.generate(session_id, ttl).await.unwrap();

        // Verify it's in "Redis"
        let data = mock_redis.data.read().await;
        let key = format!("csrf:{}", session_id);
        assert!(data.contains_key(&key));
        let (stored_set, stored_ttl) = data.get(&key).unwrap();
        assert!(stored_set.contains(&token), "Set should contain the generated token");
        assert_eq!(stored_ttl, &(ttl as u64));

        drop(data);

        // Validate and consume
        let is_valid = store.validate_and_consume(session_id, &token).await;
        assert!(is_valid);

        // Verify it's removed
        let data_after = mock_redis.data.read().await;
        // Depending on implementation, key might remain with empty set or be removed
        if let Some((set, _)) = data_after.get(&key) {
            assert!(!set.contains(&token), "Token should be removed from set");
        }
        // If the key is removed entire, that's also fine (my implementation does generic cleanup)
    }

    #[tokio::test]
    async fn test_redis_store_negative_ttl() {
        let mock_redis = Arc::new(MockRedis {
            data: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        });
        let store = RedisCsrfStore::new(mock_redis.clone());

        let session_id = "test-session-neg";
        let ttl = -100; // Negative TTL

        // Generate token
        let token = store.generate(session_id, ttl).await.unwrap();

        // Verify it uses a positive fallback (0 or max(0, ttl))
        let data = mock_redis.data.read().await;
        let key = format!("csrf:{}", session_id);
        assert!(data.contains_key(&key));
        let (stored_set, stored_ttl) = data.get(&key).unwrap();
        assert!(stored_set.contains(&token));
        assert_eq!(stored_ttl, &3600); // Should be 3600 based on default fallback
    }

    #[tokio::test]
    async fn test_csrf_token_no_overflow() {
        // Test with extremely large TTL that would overflow with normal addition
        let large_ttl = i64::MAX;
        let token = CsrfToken::new(large_ttl);

        // expires_at should be saturated to i64::MAX, not overflow to negative
        assert!(token.expires_at > 0, "expires_at should not overflow to negative");
        assert_eq!(token.expires_at, i64::MAX, "expires_at should saturate at i64::MAX");
    }

    #[tokio::test]
    async fn test_csrf_token_normal_ttl() {
        // Test with normal TTL
        let now = Utc::now().timestamp();
        let ttl = 3600;
        let token = CsrfToken::new(ttl);

        // Should be approximately now + ttl (within a few seconds tolerance)
        let expected = now + ttl;
        assert!((token.expires_at - expected).abs() <= 2, "expires_at should be approximately now + ttl");
    }

    /// RED TEST: Verify that expired tokens are NOT cleaned up automatically
    /// This test documents that InMemoryCsrfStore does not perform automatic cleanup.
    /// The validate_and_consume method does not call cleanup_expired - expired tokens
    /// accumulate unless explicitly cleaned up via cleanup_expired().
    #[tokio::test]
    async fn test_expired_tokens_accumulate_without_cleanup() {
        let store = InMemoryCsrfStore::new();

        // Generate tokens with very short TTL (already expired)
        let past_time = Utc::now().timestamp() - 3600; // 1 hour ago
        let expired_token = CsrfToken {
            token: "expired-token".to_string(),
            expires_at: past_time,
        };

        // Manually insert expired token (as a Vec)
        store.tokens.write().await.insert("session1".to_string(), vec![expired_token]);

        // Generate a new token for a different session
        let _new_token = store.generate("session2", 3600).await.unwrap();

        // Both sessions exist because cleanup_expired is never called automatically
        let tokens = store.tokens.read().await;
        assert_eq!(tokens.len(), 2, "Both sessions exist - expired tokens are not cleaned up");
    }

    /// GREEN TEST: Verify that multiple tokens can coexist for multi-tab scenarios
    /// This test verifies the FIX where generate() no longer overwrites previous tokens
    #[tokio::test]
    async fn test_multiple_tokens_for_same_session() {
        let store = InMemoryCsrfStore::new();
        let session_id = "multi-tab-session";

        // Tab 1: Generate first token
        let token1 = store.generate(session_id, 3600).await.unwrap();

        // Tab 2: Generate second token (no longer overwrites token1!)
        let token2 = store.generate(session_id, 3600).await.unwrap();

        // Tab 3: Generate third token (no longer overwrites token2!)
        let token3 = store.generate(session_id, 3600).await.unwrap();

        // FIXED: All three tokens are now valid independently
        assert!(store.validate_and_consume(session_id, &token1).await, "Token1 should be valid");
        assert!(store.validate_and_consume(session_id, &token2).await, "Token2 should be valid");
        assert!(store.validate_and_consume(session_id, &token3).await, "Token3 should be valid");

        // Verify all tokens were consumed
        let tokens = store.tokens.read().await;
        assert!(!tokens.contains_key(session_id), "All tokens should be consumed");
    }


    /// Test that cleanup_expired method works correctly when called manually
    #[tokio::test]
    async fn test_cleanup_expired_works_when_called() {
        let store = InMemoryCsrfStore::new();

        // Insert expired token (as Vec)
        let expired = CsrfToken {
            token: "expired".to_string(),
            expires_at: Utc::now().timestamp() - 100,
        };
        store.tokens.write().await.insert("session1".to_string(), vec![expired]);

        // Insert valid token (as Vec)
        let valid = CsrfToken::new(3600);
        store.tokens.write().await.insert("session2".to_string(), vec![valid]);

        // Manually call cleanup
        store.cleanup_expired().await;

        // Verify expired token was removed
        let tokens = store.tokens.read().await;
        assert_eq!(tokens.len(), 1, "Only valid token should remain");
        assert!(tokens.contains_key("session2"), "Valid token should still exist");
        assert!(!tokens.contains_key("session1"), "Expired token should be removed");
    }
}

