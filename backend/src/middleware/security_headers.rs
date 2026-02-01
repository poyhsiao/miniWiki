use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, HeaderValue, Method},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::Ready;
use std::sync::Arc;

/// Security headers middleware configuration
#[derive(Clone)]
pub struct SecurityHeadersConfig {
    /// Optional CSP connect-src origin. If None, only 'self' will be used.
    pub csp_connect_src: Option<String>,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self { csp_connect_src: None }
    }
}

pub struct SecurityHeaders {
    pub config: Arc<SecurityHeadersConfig>,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityHeaders {
    pub fn new() -> Self {
        Self {
            config: Arc::new(SecurityHeadersConfig::default()),
        }
    }

    pub fn with_config(config: SecurityHeadersConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(SecurityHeadersMiddleware {
            service,
            config: self.config.clone(),
        }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
    config: Arc<SecurityHeadersConfig>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Capture path and method before moving req
        let request_path = req.path().to_string();
        let config = self.config.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // HSTS
            res.headers_mut().insert(
                header::STRICT_TRANSPORT_SECURITY,
                HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            );

            // Frame options
            res.headers_mut()
                .insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

            // Content type options
            res.headers_mut()
                .insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

            // Referrer policy
            res.headers_mut().insert(
                header::REFERRER_POLICY,
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Permissions policy
            res.headers_mut().insert(
                header::PERMISSIONS_POLICY,
                HeaderValue::from_static("accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"),
            );

            // CSP with configurable connect-src
            if !res.headers().contains_key(header::CONTENT_SECURITY_POLICY) {
                let connect_src = normalize_csp_connect_src(config.csp_connect_src.as_deref());

                let csp_value = format!(
                    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src {}; frame-ancestors 'none'",
                    connect_src
                );

                if let Ok(value) = HeaderValue::from_str(&csp_value) {
                    res.headers_mut().insert(header::CONTENT_SECURITY_POLICY, value);
                }
            }

            // Apply cache-control only for dynamic/API responses
            // Skip for static assets (images, fonts, etc.)
            let should_apply_cache_headers =
                is_dynamic_response(&request_path, &res) && !res.headers().contains_key(header::CACHE_CONTROL);

            if should_apply_cache_headers {
                res.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
                );

                res.headers_mut().insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
            }

            Ok(res)
        })
    }
}

/// Normalizes CSP connect-src origins to prevent injection attacks.
///
/// Parses each configured origin with strict URL parsing, ensuring:
/// - Scheme is https, http, ws, or wss
/// - Extracts only scheme://host[:port]
/// - Rejects values containing semicolons, newlines, or additional directives
///
/// Returns a space-separated list of normalized origins, or "'self'" if validation fails.
fn normalize_csp_connect_src(origins: Option<&str>) -> String {
    const SELF_ORIGIN: &str = "'self'";

    let Some(origins_str) = origins else {
        return SELF_ORIGIN.to_string();
    };

    if origins_str.trim().is_empty() {
        return SELF_ORIGIN.to_string();
    }

    let mut normalized_origins = vec![SELF_ORIGIN];

    for origin in origins_str
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
    {
        // Reject values that could inject additional directives
        if origin.contains(';') || origin.contains('\n') || origin.contains('\r') {
            continue;
        }

        // Try to parse as URL and validate scheme
        match normalize_single_origin(origin) {
            Some(normalized) => normalized_origins.push(normalized),
            None => continue, // Skip invalid origins
        }
    }

    // If only 'self' remains after validation, return just 'self'
    if normalized_origins.len() == 1 {
        SELF_ORIGIN.to_string()
    } else {
        normalized_origins.join(" ")
    }
}

/// Normalizes a single origin string for CSP use.
///
/// Ensures origin has a valid http/https/ws/wss scheme and extracts
/// only scheme://host[:port] portion.
///
/// WebSocket schemes (ws://, wss://) are preserved as-is for CSP connect-src.
fn normalize_single_origin(origin: &str) -> Option<String> {
    // Check for valid scheme first (http://, https://, ws://, or wss://)
    let lower_origin = origin.to_ascii_lowercase();

    let scheme_end = lower_origin.find("://")?;
    let scheme = &lower_origin[..scheme_end];

    if !matches!(scheme, "http" | "https" | "ws" | "wss") {
        return None;
    }

    // Find the end of the host:port portion (stop at path, query, or fragment)
    let after_scheme = &origin[scheme_end + 3..];
    let host_end = after_scheme
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(after_scheme.len());

    let host_port = &after_scheme[..host_end];

    // Ensure host_port is not empty and doesn't contain invalid characters
    if host_port.is_empty() || host_port.contains([';', '\n', '\r', '"', '\'']) {
        return None;
    }

    Some(format!("{}://{}", scheme, host_port))
}

/// Determines if a response should have cache-control headers applied.
///
/// Returns `true` for dynamic API responses, `false` for static assets.
/// Static assets include: images, fonts, stylesheets, scripts, and assets under /static/ or /public/ paths.
fn is_dynamic_response(path: &str, res: &ServiceResponse<impl MessageBody>) -> bool {
    // Check if path indicates static asset
    let is_static_path = path.starts_with("/static/")
        || path.starts_with("/public/")
        || path.starts_with("/assets/")
        || path.starts_with("/media/");

    if is_static_path {
        return false;
    }

    // Check content-type for static asset types
    if let Some(content_type) = res.headers().get(header::CONTENT_TYPE) {
        if let Ok(ct_str) = content_type.to_str() {
            let is_static_content = ct_str.starts_with("image/")
                || ct_str.starts_with("font/")
                || ct_str.contains("javascript")
                || ct_str.contains("css");

            if is_static_content {
                return false;
            }
        }
    }

    // Default to dynamic for API endpoints
    true
}
