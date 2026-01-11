use config::Config;
use serde::Deserialize;
use std::env;

/// Loads configuration from environment variables.
///
/// Environment variables are expected to have a "MINIWIKI_" prefix.
/// For example, "MINIWIKI_DATABASE_URL" maps to `database_url`.
///
/// If an environment variable is not set, the default value from `AppSettings::default()` is used.
pub fn load_config() -> super::AppSettings {
    let mut settings = Config::default();
    
    // Add default values first
    settings.set_default("app_env", "development").unwrap();
    settings.set_default("app_host", "0.0.0.0").unwrap();
    settings.set_default("app_port", 8080).unwrap();
    settings.set_default("db_pool_size", 10).unwrap();
    settings.set_default("jwt_access_expiry", 900).unwrap();
    settings.set_default("jwt_refresh_expiry", 604800).unwrap();
    settings.set_default("bcrypt_cost", 12).unwrap();
    settings.set_default("max_document_size", 10485760).unwrap();
    settings.set_default("max_file_size", 52428800).unwrap();
    settings.set_default("smtp_port", 587).unwrap();
    settings.set_default("smtp_use_tls", true).unwrap();
    settings.set_default("enable_offline_sync", true).unwrap();
    settings.set_default("enable_real_time_collaboration", true).unwrap();
    
    // Load from environment with MINIWIKI_ prefix
    settings.merge(config::Environment::with_prefix("MINIWIKI")).unwrap();
    
    // Try to parse the configuration
    settings.try_into::<super::AppSettings>().unwrap_or_default()
}

/// Gets an environment variable, returning a default if not set.
fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Gets a required environment variable.
///
/// # Returns
///
/// The environment variable value, or an error if not set.
fn get_required_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Required environment variable {} is not set", key))
}
