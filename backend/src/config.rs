use serde::Deserialize;
use std::time::Duration;

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
    #[serde(default)]
    pub csp_connect_src: Option<String>,
}

fn default_app_env() -> String {
    "development".to_string()
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let config: Self = config::Config::builder()
            .separator("__")
            .add_source(config::Environment::default())
            .build()?
            .try_deserialize()?;

        Ok(Config {
            database_url: config.database_url.clone(),
            redis_cache_ttl_default: Some(config.redis_cache_ttl_default.unwrap_or(3600)),
            redis_cache_ttl_short: Some(config.redis_cache_ttl_short.unwrap_or(300)),
            redis_cache_ttl_long: Some(config.redis_cache_ttl_long.unwrap_or(86400)),
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
                min_connections, max_connections, max_connections
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
            Ok(v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
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
        assert_eq!(config.origins, vec!["http://localhost:3000".to_string(), "http://localhost:8080".to_string()]);
    }

    #[test]
    fn test_deserialize_comma_separated_sequence() {
        let json = r#"{"origins": ["http://localhost:3000", "http://localhost:8080"]}"#;
        let config: TestConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.origins, vec!["http://localhost:3000".to_string(), "http://localhost:8080".to_string()]);
    }
}
