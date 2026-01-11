use config::Config;
use serde::Deserialize;
use std::env;

/// Application configuration loaded from environment variables.
///
/// This struct holds all configuration required by the application,
/// loaded from environment variables with sensible defaults.
#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    /// Application environment (development, staging, production)
    pub app_env: String,
    
    /// Server host to bind to
    pub app_host: String,
    
    /// Server port to listen on
    pub app_port: u16,
    
    /// PostgreSQL database URL
    pub database_url: String,
    
    /// PostgreSQL connection pool size
    pub db_pool_size: u32,
    
    /// Redis connection URL
    pub redis_url: String,
    
    /// MinIO/S3 endpoint
    pub minio_endpoint: String,
    
    /// MinIO access key
    pub minio_access_key: String,
    
    /// MinIO secret key
    pub minio_secret_key: String,
    
    /// MinIO bucket name
    pub minio_bucket: String,
    
    /// JWT secret key for token signing
    pub jwt_secret: String,
    
    /// JWT access token expiry in seconds
    pub jwt_access_expiry: i64,
    
    /// JWT refresh token expiry in seconds
    pub jwt_refresh_expiry: i64,
    
    /// JWT issuer
    pub jwt_issuer: String,
    
    /// JWT audience
    pub jwt_audience: String,
    
    /// Bcrypt cost factor for password hashing
    pub bcrypt_cost: u32,
    
    /// Rate limit for anonymous users (requests per hour)
    pub rate_limit_anonymous: String,
    
    /// Rate limit for authenticated users (requests per hour)
    pub rate_limit_authenticated: String,
    
    /// Maximum document size in bytes (10MB default)
    pub max_document_size: i64,
    
    /// Maximum file upload size in bytes (50MB default)
    pub max_file_size: i64,
    
    /// Allowed file types (comma-separated MIME types)
    pub allowed_file_types: String,
    
    /// SMTP host for email
    pub smtp_host: String,
    
    /// SMTP port
    pub smtp_port: u16,
    
    /// SMTP username
    pub smtp_user: String,
    
    /// SMTP password
    pub smtp_password: String,
    
    /// Email from address
    pub email_from: String,
    
    /// Whether to enable TLS for SMTP
    pub smtp_use_tls: bool,
    
    /// Log level (debug, info, warn, error)
    pub log_level: String,
    
    /// Whether to enable offline sync
    pub enable_offline_sync: bool,
    
    /// Whether to enable real-time collaboration
    pub enable_real_time_collaboration: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            app_env: "development".to_string(),
            app_host: "0.0.0.0".to_string(),
            app_port: 8080,
            database_url: "postgres://miniwiki:miniwiki@localhost:5432/miniwiki".to_string(),
            db_pool_size: 10,
            redis_url: "redis://localhost:6379".to_string(),
            minio_endpoint: "localhost:9000".to_string(),
            minio_access_key: "miniwiki_admin".to_string(),
            minio_secret_key: "miniwiki_secret_key".to_string(),
            minio_bucket: "miniwiki-files".to_string(),
            jwt_secret: "your-super-secret-jwt-key-minimum-256-bits-long".to_string(),
            jwt_access_expiry: 900,      // 15 minutes
            jwt_refresh_expiry: 604800,  // 7 days
            jwt_issuer: "miniwiki".to_string(),
            jwt_audience: "miniwiki-users".to_string(),
            bcrypt_cost: 12,
            rate_limit_anonymous: "100/hour".to_string(),
            rate_limit_authenticated: "1000/hour".to_string(),
            max_document_size: 10485760,  // 10MB
            max_file_size: 52428800,      // 50MB
            allowed_file_types: "image/*,application/pdf,text/*,video/*,audio/*".to_string(),
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_user: "".to_string(),
            smtp_password: "".to_string(),
            email_from: "noreply@miniwiki.local".to_string(),
            smtp_use_tls: true,
            log_level: "info".to_string(),
            enable_offline_sync: true,
            enable_real_time_collaboration: true,
        }
    }
}

impl AppSettings {
    /// Creates a new AppSettings instance by loading from environment variables.
    ///
    /// Environment variables are loaded with a "MINIWIKI_" prefix.
    /// For example, "MINIWIKI_DATABASE_URL" maps to `database_url`.
    ///
    /// # Returns
    ///
    /// A configured `AppSettings` instance.
    pub fn new() -> Self {
        load_config()
    }
    
    /// Validates the configuration settings.
    ///
    /// # Returns
    ///
    /// `Ok(())` if validation passes, or an error with a message if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("DATABASE_URL must be set".to_string());
        }
        
        if self.jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters".to_string());
        }
        
        if self.bcrypt_cost < 4 || self.bcrypt_cost > 31 {
            return Err("BCRYPT_COST must be between 4 and 31".to_string());
        }
        
        if self.max_document_size > 10485760 {
            return Err("MAX_DOCUMENT_SIZE cannot exceed 10MB".to_string());
        }
        
        if self.max_file_size > 52428800 {
            return Err("MAX_FILE_SIZE cannot exceed 50MB".to_string());
        }
        
        Ok(())
    }
}
