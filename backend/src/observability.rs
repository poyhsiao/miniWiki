//! Observability utilities for structured logging, metrics, and tracing
//!
//! This module provides:
//! - Structured logging with JSON support
//! - Request latency metrics
//! - Error rate tracking
//! - Distributed tracing for sync operations

use actix_web::web;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{Level, span};

/// Request metrics aggregated across all endpoints
#[derive(Debug, Default)]
pub struct RequestMetrics {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_latency_ms: AtomicU64,
}

impl RequestMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
        }
    }

    /// Record a completed request with latency
    pub fn record_request(&self, latency_ms: u64, success: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);

        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            total_latency_ms: self.total_latency_ms.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of current metrics state
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_latency_ms: u64,
}

impl MetricsSnapshot {
    /// Calculate average latency in milliseconds
    pub fn avg_latency_ms(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.total_requests as f64
        }
    }

    /// Calculate error rate as a percentage
    pub fn error_rate_percent(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.failed_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}

/// Latency histogram buckets for request timing (in milliseconds)
const LATENCY_BUCKETS: &[u64] = &[10, 50, 100, 200, 500, 1000, 2000, 5000];

/// Request timing middleware for metrics collection
#[derive(Clone)]
pub struct MetricsMiddleware {
    metrics: Arc<RequestMetrics>,
}

impl MetricsMiddleware {
    /// Create new metrics middleware
    pub fn new(metrics: Arc<RequestMetrics>) -> Self {
        Self { metrics }
    }

    /// Record a request with timing
    pub fn record(&self, latency: std::time::Duration, success: bool) {
        self.metrics.record_request(latency.as_millis() as u64, success);
    }
}

/// Helper function to create a tracing span for sync operations
pub fn create_sync_span(document_id: &str, operation: &str) -> tracing::Span {
    span!(
        Level::INFO,
        "sync_operation",
        document_id = document_id,
        operation = operation,
    )
}

/// Log a sync operation event
pub fn log_sync_event(
    document_id: &str,
    operation: &str,
    level: Level,
    message: &str,
) {
    match level {
        Level::TRACE => {
            tracing::trace!(
                document_id = document_id,
                operation = operation,
                message = message
            );
        }
        Level::DEBUG => {
            tracing::debug!(
                document_id = document_id,
                operation = operation,
                message = message
            );
        }
        Level::INFO => {
            tracing::info!(
                document_id = document_id,
                operation = operation,
                message = message
            );
        }
        Level::WARN => {
            tracing::warn!(
                document_id = document_id,
                operation = operation,
                message = message
            );
        }
        Level::ERROR => {
            tracing::error!(
                document_id = document_id,
                operation = operation,
                message = message
            );
        }
    }
}

/// Extension trait for adding metrics to request data
pub trait RequestMetricsExt {
    fn metrics_snapshot(&self) -> Option<MetricsSnapshot>;
}

impl RequestMetricsExt for web::Data<RequestMetrics> {
    fn metrics_snapshot(&self) -> Option<MetricsSnapshot> {
        Some(self.get_ref().snapshot())
    }
}
