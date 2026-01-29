use actix_web::http::header::HeaderValue;
use serde::Deserialize;
use std::time::Duration;

/// Intermediate struct for deserializing security headers config from environment variables
#[derive(Debug, Clone, Deserialize)]
struct SecurityHeadersConfigRaw {
    #[serde(default)]
    api_origin: Option<String>,

    #[serde(default = "default_csp")]
    content_security_policy: String,

    #[serde(default = "default_hsts")]
    strict_transport_security: String,

    #[serde(default = "default_frame_options")]
    x_frame_options: String,

    #[serde(default = "default_content_type_options")]
    x_content_type_options: String,

    #[serde(default = "default_referrer_policy")]
    referrer_policy: String,

    #[serde(default = "default_permissions_policy")]
    permissions_policy: String,

    #[serde(default = "default_cache_control")]
    cache_control: String,

    #[serde(default = "default_pragma")]
    pragma: String,
}

/// Configuration for security-related HTTP headers
///
/// This struct defines all security headers that can be configured
/// for application. All fields have sensible defaults and can
/// be overridden via environment variables or config files.
/// The header values are parsed and validated at startup time for better performance.
///
/// # Example (Environment Variables)
///
/// ```ignore
/// export SECURITY_HEADERS__CONTENT_SECURITY_POLICY="default-src 'self'"
/// export SECURITY_HEADERS__STRICT_TRANSPORT_SECURITY="max-age=31536000"
/// export SECURITY_HEADERS__API_ORIGIN="https://api.example.com"
/// ```
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    /// API origin for CSP connect-src directive
    ///
    /// If set, this will be added to connect-src directive in the CSP.
    /// Default: None (only 'self' will be used)
    pub api_origin: Option<String>,

    /// Content-Security-Policy header value (pre-validated HeaderValue)
    pub content_security_policy: Option<HeaderValue>,

    /// Strict-Transport-Security header value (pre-validated HeaderValue)
    pub strict_transport_security: Option<HeaderValue>,

    /// X-Frame-Options header value (pre-validated HeaderValue)
    pub x_frame_options: Option<HeaderValue>,

    /// X-Content-Type-Options header value (pre-validated HeaderValue)
    pub x_content_type_options: Option<HeaderValue>,

    /// Referrer-Policy header value (pre-validated HeaderValue)
    pub referrer_policy: Option<HeaderValue>,

    /// Permissions-Policy header value (pre-validated HeaderValue)
    pub permissions_policy: Option<HeaderValue>,

    /// Cache-Control header value (pre-validated HeaderValue)
    pub cache_control: Option<HeaderValue>,

    /// Pragma header value (pre-validated HeaderValue)
    pub pragma: Option<HeaderValue>,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        let csp_string = default_csp();
        Self::from_raw(SecurityHeadersConfigRaw {
            api_origin: None,
            content_security_policy: csp_string.clone(),
            strict_transport_security: default_hsts(),
            x_frame_options: default_frame_options(),
            x_content_type_options: default_content_type_options(),
            referrer_policy: default_referrer_policy(),
            permissions_policy: default_permissions_policy(),
            cache_control: default_cache_control(),
            pragma: default_pragma(),
        }, &csp_string)
    }
}

impl SecurityHeadersConfig {
    /// Create SecurityHeadersConfig from raw config by parsing and validating header values
    fn from_raw(raw: SecurityHeadersConfigRaw, csp_string: &str) -> Self {
        Self {
            api_origin: raw.api_origin,
            content_security_policy: parse_header("Content-Security-Policy", csp_string),
            strict_transport_security: parse_header("Strict-Transport-Security", &raw.strict_transport_security),
            x_frame_options: parse_header("X-Frame-Options", &raw.x_frame_options),
            x_content_type_options: parse_header("X-Content-Type-Options", &raw.x_content_type_options),
            referrer_policy: parse_header("Referrer-Policy", &raw.referrer_policy),
            permissions_policy: parse_header("Permissions-Policy", &raw.permissions_policy),
            cache_control: parse_header("Cache-Control", &raw.cache_control),
            pragma: parse_header("Pragma", &raw.pragma),
        }
    }

    /// Update CSP to include API origin if configured
    ///
    /// This method modifies the `content_security_policy` field to include
    /// the configured `api_origin` in the connect-src directive. If no
    /// api_origin is set, the CSP remains unchanged.
    pub fn update_csp(&mut self) {
        if let Some(ref origin) = self.api_origin {
            if !origin.is_empty() {
                let csp = self.content_security_policy
                    .as_ref()
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");

                let updated_csp = csp
                    .replace("connect-src 'self'", &format!("connect-src 'self' {}", origin));

                self.content_security_policy = parse_header("Content-Security-Policy", &updated_csp);
            }
        }
    }
}

/// Parse a header value and log warnings if invalid
fn parse_header(header_name: &str, value: &str) -> Option<HeaderValue> {
    if value.is_empty() {
        tracing::warn!(
            "Security header '{}' is empty, skipping",
            header_name
        );
        return None;
    }
    match HeaderValue::from_str(value) {
        Ok(header_value) => Some(header_value),
        Err(e) => {
            tracing::warn!(
                "Invalid security header value for '{}': '{}'. Error: {}. Header will not be set.",
                header_name,
                value,
                e
            );
            None
        }
    }
}
    }
}

impl SecurityHeadersConfig {
    /// Update the CSP to include the API origin if configured
    ///
    /// This method modifies the `content_security_policy` field to include
    /// the configured `api_origin` in the connect-src directive. If no
    /// api_origin is set, the CSP remains unchanged.
    pub fn update_csp(&mut self) {
        if let Some(ref origin) = self.api_origin {
            if !origin.is_empty() {
                // Replace 'self' with 'self <origin>' in connect-src
                self.content_security_policy = self
                    .content_security_policy
                    .replace("connect-src 'self'", &format!("connect-src 'self' {}", origin));
            }
        }
    }
}

fn default_csp() -> String {
    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'".to_string()
}

fn default_hsts() -> String {
    "max-age=31536000; includeSubDomains; preload".to_string()
}

fn default_frame_options() -> String {
    "DENY".to_string()
}

fn default_content_type_options() -> String {
    "nosniff".to_string()
}

fn default_referrer_policy() -> String {
    "strict-origin-when-cross-origin".to_string()
}

fn default_permissions_policy() -> String {
    "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
        .to_string()
}

fn default_cache_control() -> String {
    "no-store, no-cache, must-revalidate, private".to_string()
}

fn default_pragma() -> String {
    "no-cache".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    #[serde(default)]
    pub db_min_connections: Option<u32>,
    #[serde(default)]
    pub db_max_connections: Option<u32>,
    #[serde(default)]
    pub db_connection_timeout: Option<u64>,
    pub jwt_secret: String,
    pub jwt_access_expiry: i64,
    pub jwt_refresh_expiry: i64,
    pub redis_url: String,
    #[serde(default)]
    pub redis_cache_ttl_default: Option<u64>,
    #[serde(default)]
    pub redis_cache_ttl_short: Option<u64>,
    #[serde(default)]
    pub redis_cache_ttl_long: Option<u64>,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_bucket: String,
    pub minio_region: String,
    pub minio_use_ssl: bool,
    #[serde(default = "default_app_env")]
    pub app_env: String,
    #[serde(deserialize_with = "deserialize_comma_separated", default)]
    pub api_cors_origins: Vec<String>,
    #[serde(default)]
    pub csrf_strict_redis: bool,
    /// Security headers configuration (raw, will be parsed)
    #[serde(default)]
    security_headers_raw: SecurityHeadersConfigRaw,
}

impl Config {
    pub fn security_headers(&self) -> &SecurityHeadersConfig {
        &self.security_headers
    }
}

fn default_app_env() -> String {
    "development".to_string()
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let config: Self = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()?;

        let mut security_headers = SecurityHeadersConfig::from_raw(
            config.security_headers_raw.clone(),
            &default_csp()
        );
        security_headers.update_csp();

        Ok(Config {
            database_url: config.database_url.clone(),
            redis_cache_ttl_default: Some(config.redis_cache_ttl_default.unwrap_or(3600)),
            redis_cache_ttl_short: Some(config.redis_cache_ttl_short.unwrap_or(300)),
            redis_cache_ttl_long: Some(config.redis_cache_ttl_long.unwrap_or(86400)),
            security_headers,
            security_headers_raw: config.security_headers_raw,
            ..config
        })
    }

    pub async fn create_pool(&self) -> Result<sqlx::PgPool, sqlx::Error> {
        // Read connection count configurations with defaults
        let min_connections = self.db_min_connections.unwrap_or(5);
        let max_connections = self.db_max_connections.unwrap_or(20);

        // Validate and clamp: min should not exceed max
        let validated_min = if min_connections > max_connections {
            // Adjust min to max and log a warning
            tracing::warn!(
                "db_min_connections ({}) > db_max_connections ({}), adjusting min to {}",
                min_connections,
                max_connections,
                max_connections
            );
            max_connections
        } else {
            min_connections
        };

        sqlx::postgres::PgPoolOptions::new()
            .min_connections(validated_min)
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(self.db_connection_timeout.unwrap_or(30)))
            .connect(&self.database_url)
            .await
    }
}
use serde::Deserializer;

pub fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct CommaSeparatedVisitor;

    impl<'de> serde::de::Visitor<'de> for CommaSeparatedVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a comma-separated string or a sequence of strings")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element::<String>()? {
                vec.push(elem);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(CommaSeparatedVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestConfig {
        #[serde(deserialize_with = "deserialize_comma_separated")]
        origins: Vec<String>,
    }

    #[test]
    fn test_deserialize_comma_separated_string() {
        let json = r#"{"origins": "http://localhost:3000, http://localhost:8080"}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.origins,
            vec!["http://localhost:3000".to_string(), "http://localhost:8080".to_string()]
        );
    }

    #[test]
    fn test_deserialize_comma_separated_sequence() {
        let json = r#"{"origins": ["http://localhost:3000", "http://localhost:8080"]}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.origins,
            vec!["http://localhost:3000".to_string(), "http://localhost:8080".to_string()]
        );
    }

    // SecurityHeadersConfig tests
    #[test]
    fn test_security_headers_update_csp_with_api_origin() {
        let mut config = SecurityHeadersConfig::default();
        config.api_origin = Some("https://api.example.com".to_string());
        config.update_csp();

        assert!(config.content_security_policy
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .contains("connect-src 'self' https://api.example.com"));
    }

    #[test]
    fn test_security_headers_update_csp_without_api_origin() {
        let mut config = SecurityHeadersConfig::default();
        config.api_origin = None;
        config.update_csp();

        // Should remain with only 'self'
        let csp = config.content_security_policy
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(csp.contains("connect-src 'self'"));
        assert!(!csp.contains("connect-src 'self' https://"));
    }

    #[test]
    fn test_security_headers_update_csp_with_empty_api_origin() {
        let mut config = SecurityHeadersConfig::default();
        config.api_origin = Some("".to_string());
        config.update_csp();

        // Should remain unchanged with only 'self'
        let csp = config.content_security_policy
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(csp.contains("connect-src 'self'"));
        assert!(!csp.contains("connect-src 'self' https://"));
    }

    #[test]
    fn test_security_headers_update_csp_without_api_origin() {
        let mut config = SecurityHeadersConfig::default();
        config.api_origin = None;
        config.update_csp();

        // Should remain with only 'self'
        assert!(config.content_security_policy.contains("connect-src 'self'"));
        assert!(!config.content_security_policy.contains("connect-src 'self' https://"));
    }

    #[test]
    fn test_security_headers_update_csp_with_empty_api_origin() {
        let mut config = SecurityHeadersConfig::default();
        config.api_origin = Some("".to_string());
        config.update_csp();

        // Should remain unchanged with only 'self'
        assert!(config.content_security_policy.contains("connect-src 'self'"));
        assert!(!config.content_security_policy.contains("connect-src 'self' https://"));
    }
}
