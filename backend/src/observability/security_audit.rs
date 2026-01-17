//! Security Audit System
//!
//! Provides comprehensive security event logging and audit trail for compliance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use shared_errors::AppError;
use tracing::{error, info};
use uuid::Uuid;
use std::sync::Arc;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_event_type", rename_all = "snake_case")]
pub enum AuditEventType {
    #[serde(rename = "authentication_success")]
    AuthenticationSuccess,
    #[serde(rename = "authentication_failure")]
    AuthenticationFailure,
    #[serde(rename = "authorization_success")]
    AuthorizationSuccess,
    #[serde(rename = "authorization_failure")]
    AuthorizationFailure,
    #[serde(rename = "csrf_failure")]
    CsrfFailure,
    #[serde(rename = "rate_limit_exceeded")]
    RateLimitExceeded,
    #[serde(rename = "suspicious_activity")]
    SuspiciousActivity,
    #[serde(rename = "data_access_attempt")]
    DataAccessAttempt,
    #[serde(rename = "session_created")]
    SessionCreated,
    #[serde(rename = "session_destroyed")]
    SessionDestroyed,
    #[serde(rename = "password_reset_request")]
    PasswordResetRequest,
    #[serde(rename = "permission_denied")]
    PermissionDenied,
}

/// Security event with full context
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AuditEvent {
    pub id: Uuid,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub success: bool,
    pub created_at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        event_type: AuditEventType,
        resource_type: String,
        action: String,
        user_id: Option<String>,
        resource_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        details: Option<serde_json::Value>,
        success: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            resource_type,
            action,
            user_id,
            resource_id,
            ip_address,
            user_agent,
            details,
            success,
            created_at: Utc::now(),
        }
    }

    /// Mask PII (User ID and IP Address) for logging or long-term storage
    pub fn mask_pii(&mut self) {
        if let Some(uid) = self.user_id.as_mut() {
            let char_count = uid.chars().count();
            if char_count > 4 {
                // Use character-aware iteration to get first 4 characters safely
                let first_four: String = uid.chars().take(4).collect();
                *uid = format!("{}***", first_four);
            } else {
                *uid = "****".to_string();
            }
        }
        if let Some(ip) = self.ip_address.as_mut() {
            if ip.contains('.') {
                // IPv4 masking: 192.168.1.1 -> 192.168.***.***
                let parts: Vec<&str> = ip.split('.').collect();
                if parts.len() == 4 {
                    *ip = format!("{}.{}.***.***", parts[0], parts[1]);
                } else {
                    *ip = "0.0.***.***".to_string();
                }
            } else if ip.contains(':') {
                // IPv6 masking: 2001:0db8:85a3:0000:0000:8a2e:0370:7334 -> 2001:0db8:****:****:****:****:****:****
                let parts: Vec<&str> = ip.split(':').collect();
                if parts.len() >= 2 {
                    *ip = format!("{}:{}:****:****:****:****:****:****", parts[0], parts[1]);
                } else {
                    *ip = "****:****:****:****:****:****:****:****".to_string();
                }
            } else {
                *ip = "anonymized".to_string();
            }
        }
    }
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Info,
    Warning,
    Critical,
}

/// Summary statistics for security report
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SecurityReportSummary {
    pub total_events: usize,
    pub authentication_events: usize,
    pub authorization_events: usize,
    pub csrf_failures: usize,
    pub rate_limit_violations: usize,
    pub suspicious_activities: usize,
    pub data_access_attempts: usize,
    pub permission_denials: usize,
    pub session_events: usize,
    pub password_reset_events: usize,
    pub critical_severity_count: usize,
    pub warning_severity_count: usize,
    pub info_severity_count: usize,
}

/// Audit logger for recording security events
pub struct AuditLogger {
    db: Arc<PgPool>,
}

impl AuditLogger {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    pub async fn log_event(&self, event: &AuditEvent) -> Result<(), AppError> {
        let mut masked_event = event.clone();
        masked_event.mask_pii();

        sqlx::query(
            "INSERT INTO audit_logs (
                id, event_type, user_id, action, resource_type, resource_id,
                details, ip_address, user_agent, success, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(masked_event.id)
        .bind(&masked_event.event_type)
        .bind(&masked_event.user_id)
        .bind(&masked_event.action)
        .bind(&masked_event.resource_type)
        .bind(&masked_event.resource_id)
        .bind(&masked_event.details)
        .bind(&masked_event.ip_address)
        .bind(&masked_event.user_agent)
        .bind(masked_event.success)
        .bind(masked_event.created_at)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to log audit event: {}", e);
            AppError::internal("Failed to log audit event")
        })?;

        info!(
            "Security event logged: {:?} for user {:?}",
            masked_event.event_type,
            masked_event.user_id.as_deref().unwrap_or("none")
        );
        Ok(())
    }

    /// Anonymize old audit logs containing PII
    ///
    /// Policy: Logs older than the retention period have their raw IP and User ID truncated.
    pub async fn purge_old_pii(&self, older_than_days: i64) -> Result<u64, AppError> {
        let delta = chrono::TimeDelta::try_days(older_than_days)
            .ok_or_else(|| AppError::validation("Days value out of range"))?;
        let cutoff = Utc::now() - delta;

        let result = sqlx::query(
            "UPDATE audit_logs
             SET ip_address = 'anonymized', user_id = 'anonymized'
             WHERE created_at < $1
             AND (ip_address IS DISTINCT FROM 'anonymized' OR user_id IS DISTINCT FROM 'anonymized')"
        )
        .bind(cutoff)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to purge old audit logs: {}", e);
            AppError::internal("Failed to purge old audit logs")
        })?;

        let affected = result.rows_affected();
        info!("Anonymized {} old security audit logs older than {} days", affected, older_than_days);
        Ok(affected)
    }

    pub async fn get_statistics(&self, days: i64) -> Result<SecurityReportSummary, AppError> {
        let delta = chrono::TimeDelta::try_days(days)
            .ok_or_else(|| AppError::validation("Days value out of range"))?;
        let since = Utc::now() - delta;

        let rows: Vec<(AuditEventType, i64)> = sqlx::query_as(
            "SELECT event_type, COUNT(*) FROM audit_logs WHERE created_at >= $1 GROUP BY event_type"
        )
        .bind(since)
        .fetch_all(self.db.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch statistics: {:?}", e);
            AppError::internal("Failed to fetch statistics")
        })?;

        let mut summary = SecurityReportSummary::default();
        for (evt, count) in rows {
            let count = usize::try_from(count).map_err(|_| AppError::internal("event count overflow"))?;
            summary.total_events += count;

            // Classify event to severity level
            let level = match evt {
                AuditEventType::AuthenticationFailure
                | AuditEventType::AuthorizationFailure
                | AuditEventType::CsrfFailure
                | AuditEventType::SuspiciousActivity => SecurityLevel::Critical,
                AuditEventType::RateLimitExceeded | AuditEventType::PermissionDenied => SecurityLevel::Warning,
                _ => SecurityLevel::Info,
            };

            match level {
                SecurityLevel::Critical => summary.critical_severity_count += count,
                SecurityLevel::Warning => summary.warning_severity_count += count,
                SecurityLevel::Info => summary.info_severity_count += count,
            }

            match evt {
                AuditEventType::AuthenticationSuccess | AuditEventType::AuthenticationFailure => {
                    summary.authentication_events += count;
                }
                AuditEventType::AuthorizationSuccess | AuditEventType::AuthorizationFailure => {
                    summary.authorization_events += count;
                }
                AuditEventType::CsrfFailure => summary.csrf_failures += count,
                AuditEventType::RateLimitExceeded => summary.rate_limit_violations += count,
                AuditEventType::SuspiciousActivity => summary.suspicious_activities += count,
                AuditEventType::DataAccessAttempt => summary.data_access_attempts += count,
                AuditEventType::PermissionDenied => summary.permission_denials += count,
                AuditEventType::SessionCreated | AuditEventType::SessionDestroyed => {
                    summary.session_events += count;
                }
                AuditEventType::PasswordResetRequest => {
                    summary.password_reset_events += count;
                }
            }
        }
        Ok(summary)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_anonymization() {
        let mut event = AuditEvent::new(
            AuditEventType::AuthenticationSuccess,
            "user".to_string(),
            "login".to_string(),
            Some("user-123456".to_string()),
            None,
            Some("192.168.1.1".to_string()),
            Some("Mozilla".to_string()),
            None,
            true,
        );

        event.mask_pii();

        assert!(event.user_id.unwrap().ends_with("***"));
        assert_eq!(event.ip_address.unwrap(), "192.168.***.***");

        // IPv6 test
        let mut ipv6_event = AuditEvent::new(
            AuditEventType::AuthenticationSuccess,
            "user".to_string(),
            "login".to_string(),
            None,
            None,
            Some("2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string()),
            None,
            None,
            true,
        );
        ipv6_event.mask_pii();
        assert_eq!(ipv6_event.ip_address.unwrap(), "2001:0db8:****:****:****:****:****:****");
    }

    // Integration test for purge_old_pii with NULL values
    // This test requires a database connection and should be run with `cargo test --features integration`
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn test_purge_old_pii_handles_null_values() {
        use sqlx::PgPool;
        use std::env;

        // Setup test database connection
        let database_url = env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for integration tests");
        let pool = PgPool::connect(&database_url).await.unwrap();
        let logger = AuditLogger::new(Arc::new(pool.clone()));

        // Clean up test data
        sqlx::query("DELETE FROM audit_logs WHERE action = 'test_null_purge'")
            .execute(&pool)
            .await
            .unwrap();

        // Insert test records with NULL ip_address and user_id
        let old_timestamp = Utc::now() - chrono::TimeDelta::try_days(31).unwrap();

        // Record 1: NULL ip_address, valid user_id
        sqlx::query(
            "INSERT INTO audit_logs (
                id, event_type, user_id, action, resource_type, resource_id,
                details, ip_address, user_agent, success, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(Uuid::new_v4())
        .bind(&AuditEventType::AuthenticationSuccess)
        .bind(Some("user-123"))
        .bind("test_null_purge")
        .bind("test")
        .bind(None::<String>)
        .bind(None::<serde_json::Value>)
        .bind(None::<String>) // NULL ip_address
        .bind(Some("TestAgent"))
        .bind(true)
        .bind(old_timestamp)
        .execute(&pool)
        .await
        .unwrap();

        // Record 2: valid ip_address, NULL user_id
        sqlx::query(
            "INSERT INTO audit_logs (
                id, event_type, user_id, action, resource_type, resource_id,
                details, ip_address, user_agent, success, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(Uuid::new_v4())
        .bind(&AuditEventType::AuthenticationSuccess)
        .bind(None::<String>) // NULL user_id
        .bind("test_null_purge")
        .bind("test")
        .bind(None::<String>)
        .bind(None::<serde_json::Value>)
        .bind(Some("192.168.1.1"))
        .bind(Some("TestAgent"))
        .bind(true)
        .bind(old_timestamp)
        .execute(&pool)
        .await
        .unwrap();

        // Record 3: both NULL
        sqlx::query(
            "INSERT INTO audit_logs (
                id, event_type, user_id, action, resource_type, resource_id,
                details, ip_address, user_agent, success, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(Uuid::new_v4())
        .bind(&AuditEventType::AuthenticationSuccess)
        .bind(None::<String>) // NULL user_id
        .bind("test_null_purge")
        .bind("test")
        .bind(None::<String>)
        .bind(None::<serde_json::Value>)
        .bind(None::<String>) // NULL ip_address
        .bind(Some("TestAgent"))
        .bind(true)
        .bind(old_timestamp)
        .execute(&pool)
        .await
        .unwrap();

        // Execute purge
        let affected = logger.purge_old_pii(30).await.unwrap();

        // Should affect all 3 records (NULL values should be treated as needing anonymization)
        assert_eq!(affected, 3, "All records with NULL values should be anonymized");

        // Verify all records are now anonymized
        let anonymized_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs
             WHERE action = 'test_null_purge'
             AND ip_address = 'anonymized'
             AND user_id = 'anonymized'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(anonymized_count, 3, "All test records should have anonymized values");

        // Clean up
        sqlx::query("DELETE FROM audit_logs WHERE action = 'test_null_purge'")
            .execute(&pool)
            .await
            .unwrap();
    }
}
