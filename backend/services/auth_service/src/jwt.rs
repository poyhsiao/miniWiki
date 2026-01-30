use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Token generation error: {0}")]
    GenerationError(String),
    #[error("Token validation error: {0}")]
    ValidationError(String),
    #[error("Token decoding error: {0}")]
    DecodingError(String),
}

const BEARER_PREFIX: &str = "Bearer ";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>, // JWT ID for token uniqueness
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_expiry: i64,
    pub refresh_expiry: i64,
}

impl JwtConfig {
    pub fn new(secret: String, access_expiry: i64, refresh_expiry: i64) -> Self {
        Self {
            secret,
            access_expiry,
            refresh_expiry,
        }
    }
}

pub struct JwtService {
    pub config: JwtConfig,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    pub fn generate_access_token(&self, user_id: &str, email: &str, role: &str) -> Result<String, JwtError> {
        let now = Utc::now();
        let expiry = now + Duration::seconds(self.config.access_expiry);

        let claims = Claims {
            sub: user_id.to_string(),
            user_id: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            exp: expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: None, // Access tokens don't need JTI
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_bytes()),
        )
        .map_err(|e| JwtError::GenerationError(e.to_string()))
    }

    pub fn generate_refresh_token(&self, user_id: &str) -> Result<String, JwtError> {
        let now = Utc::now();
        let expiry = now + Duration::seconds(self.config.refresh_expiry);
        let jti = Uuid::new_v4().to_string(); // Generate unique ID for each token

        let claims = Claims {
            sub: user_id.to_string(),
            user_id: user_id.to_string(),
            email: String::new(),
            role: String::new(),
            exp: expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: Some(jti),
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_bytes()),
        )
        .map_err(|e| JwtError::GenerationError(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let decoding_key = DecodingKey::from_secret(self.config.secret.as_bytes());

        decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::HS256))
            .map(|data| data.claims)
            .map_err(|e| JwtError::ValidationError(e.to_string()))
    }

    pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
        if auth_header.starts_with(BEARER_PREFIX) {
            Some(&auth_header[BEARER_PREFIX.len()..])
        } else {
            None
        }
    }
}

/// Generate a JWT token for testing purposes.
/// Uses a hardcoded test secret for simplicity in tests.
pub fn generate_jwt_token(user_id: Uuid, email: &str) -> Result<String, JwtError> {
    let config = JwtConfig::new(
        "test-secret-key-for-testing-only-do-not-use-in-production".to_string(),
        3600,
        86400,
    );
    let service = JwtService::new(config);
    service.generate_access_token(&user_id.to_string(), email, "user")
}
