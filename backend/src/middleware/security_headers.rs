use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{self, HeaderValue},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::Ready;
use std::sync::Arc;

use crate::config::SecurityHeadersConfig;

// Logging for security header validation failures
fn log_invalid_header(header_name: &str, value: &str) {
    tracing::warn!(
        "Invalid security header value for '{}': '{}'. Header will not be set.",
        header_name,
        value
    );
}

/// Security headers middleware with configurable policies
///
/// This middleware adds security-related HTTP headers to all responses.
/// The headers can be configured via the `SecurityHeadersConfig` structure.
///
/// # Example
///
/// ```ignore
/// let app = App::new()
///     .wrap(SecurityHeaders::with_config(Arc::new(config)))
///     .route("/", web::get().to(index));
/// ```
pub struct SecurityHeaders {
    config: Arc<SecurityHeadersConfig>,
}

impl SecurityHeaders {
    /// Create a new SecurityHeaders middleware with default configuration
    pub fn new() -> Self {
        Self {
            config: Arc::new(SecurityHeadersConfig::default()),
        }
    }

    /// Create a new SecurityHeaders middleware with custom configuration
    pub fn with_config(config: Arc<SecurityHeadersConfig>) -> Self {
        Self { config }
    }
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self::new()
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
        let fut = self.service.call(req);
        let config = self.config.clone();

        Box::pin(async move {
            let mut res = fut.await?;

            // Apply HSTS header
            match HeaderValue::from_str(&config.strict_transport_security) {
                Ok(hsts_value) => {
                    res.headers_mut()
                        .insert(header::STRICT_TRANSPORT_SECURITY, hsts_value);
                }
                Err(_) => {
                    log_invalid_header("Strict-Transport-Security", &config.strict_transport_security);
                }
            }

            // Apply X-Frame-Options header
            match HeaderValue::from_str(&config.x_frame_options) {
                Ok(frame_value) => {
                    res.headers_mut()
                        .insert(header::X_FRAME_OPTIONS, frame_value);
                }
                Err(_) => {
                    log_invalid_header("X-Frame-Options", &config.x_frame_options);
                }
            }

            // Apply X-Content-Type-Options header
            match HeaderValue::from_str(&config.x_content_type_options) {
                Ok(ct_value) => {
                    res.headers_mut()
                        .insert(header::X_CONTENT_TYPE_OPTIONS, ct_value);
                }
                Err(_) => {
                    log_invalid_header("X-Content-Type-Options", &config.x_content_type_options);
                }
            }

            // Apply Referrer-Policy header
            match HeaderValue::from_str(&config.referrer_policy) {
                Ok(referrer_value) => {
                    res.headers_mut()
                        .insert(header::REFERRER_POLICY, referrer_value);
                }
                Err(_) => {
                    log_invalid_header("Referrer-Policy", &config.referrer_policy);
                }
            }

            // Apply Permissions-Policy header
            match HeaderValue::from_str(&config.permissions_policy) {
                Ok(perm_value) => {
                    res.headers_mut()
                        .insert(header::PERMISSIONS_POLICY, perm_value);
                }
                Err(_) => {
                    log_invalid_header("Permissions-Policy", &config.permissions_policy);
                }
            }

            // Apply CSP header only if not already set
            if !res.headers().contains_key(header::CONTENT_SECURITY_POLICY) {
                match HeaderValue::from_str(&config.content_security_policy) {
                    Ok(csp_value) => {
                        res.headers_mut()
                            .insert(header::CONTENT_SECURITY_POLICY, csp_value);
                    }
                    Err(_) => {
                        log_invalid_header("Content-Security-Policy", &config.content_security_policy);
                    }
                }
            }

            // Apply Cache-Control header
            match HeaderValue::from_str(&config.cache_control) {
                Ok(cache_value) => {
                    res.headers_mut()
                        .insert(header::CACHE_CONTROL, cache_value);
                }
                Err(_) => {
                    log_invalid_header("Cache-Control", &config.cache_control);
                }
            }

            // Apply Pragma header
            match HeaderValue::from_str(&config.pragma) {
                Ok(pragma_value) => {
                    res.headers_mut()
                        .insert(header::PRAGMA, pragma_value);
                }
                Err(_) => {
                    log_invalid_header("Pragma", &config.pragma);
                }
            }

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn index() -> HttpResponse {
        HttpResponse::Ok().body("test")
    }

    #[actix_web::test]
    async fn test_security_headers_are_added() {
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeaders::new())
                .route("/", web::get().to(index)),
        )
        .await;

        let req = test::TestRequest::get().to_request();
        let resp = test::call_service(&app, req).await;

        // Check HSTS header
        assert_eq!(
            resp.headers().get(header::STRICT_TRANSPORT_SECURITY),
            Some(&HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"))
        );

        // Check X-Frame-Options
        assert_eq!(
            resp.headers().get(header::X_FRAME_OPTIONS),
            Some(&HeaderValue::from_static("DENY"))
        );

        // Check X-Content-Type-Options
        assert_eq!(
            resp.headers().get(header::X_CONTENT_TYPE_OPTIONS),
            Some(&HeaderValue::from_static("nosniff"))
        );

        // Check Referrer-Policy
        assert_eq!(
            resp.headers().get(header::REFERRER_POLICY),
            Some(&HeaderValue::from_static("strict-origin-when-cross-origin"))
        );

        // Check Permissions-Policy
        assert!(resp
            .headers()
            .get(header::PERMISSIONS_POLICY)
            .is_some());

        // Check Cache-Control
        assert_eq!(
            resp.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("no-store, no-cache, must-revalidate, private"))
        );

        // Check Pragma
        assert_eq!(
            resp.headers().get(header::PRAGMA),
            Some(&HeaderValue::from_static("no-cache"))
        );
    }

    #[actix_web::test]
    async fn test_csp_header_added_when_missing() {
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeaders::new())
                .route("/", web::get().to(index)),
        )
        .await;

        let req = test::TestRequest::get().to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp
            .headers()
            .get(header::CONTENT_SECURITY_POLICY)
            .is_some());
    }

    #[actix_web::test]
    async fn test_custom_security_headers_config() {
        let custom_config = SecurityHeadersConfig {
            api_origin: None,
            content_security_policy: "default-src 'none'".to_string(),
            strict_transport_security: "max-age=3600".to_string(),
            x_frame_options: "SAMEORIGIN".to_string(),
            x_content_type_options: "nosniff".to_string(),
            referrer_policy: "no-referrer".to_string(),
            permissions_policy: "geolocation=(self)".to_string(),
            cache_control: "public, max-age=3600".to_string(),
            pragma: "no-cache".to_string(),
        };

        let app = test::init_service(
            App::new()
                .wrap(SecurityHeaders::with_config(Arc::new(custom_config)))
                .route("/", web::get().to(index)),
        )
        .await;

        let req = test::TestRequest::get().to_request();
        let resp = test::call_service(&app, req).await;

        // Check custom CSP
        assert_eq!(
            resp.headers().get(header::CONTENT_SECURITY_POLICY),
            Some(&HeaderValue::from_static("default-src 'none'"))
        );

        // Check custom HSTS
        assert_eq!(
            resp.headers().get(header::STRICT_TRANSPORT_SECURITY),
            Some(&HeaderValue::from_static("max-age=3600"))
        );

        // Check custom X-Frame-Options
        assert_eq!(
            resp.headers().get(header::X_FRAME_OPTIONS),
            Some(&HeaderValue::from_static("SAMEORIGIN"))
        );

        // Check custom Referrer-Policy
        assert_eq!(
            resp.headers().get(header::REFERRER_POLICY),
            Some(&HeaderValue::from_static("no-referrer"))
        );
    }

    #[actix_web::test]
    async fn test_csp_header_not_overridden_when_set() {
        async fn index_with_csp() -> HttpResponse {
            HttpResponse::Ok()
                .insert_header((
                    header::CONTENT_SECURITY_POLICY,
                    "custom-csp-directive",
                ))
                .body("test")
        }

        let app = test::init_service(
            App::new()
                .wrap(SecurityHeaders::new())
                .route("/", web::get().to(index_with_csp)),
        )
        .await;

        let req = test::TestRequest::get().to_request();
        let resp = test::call_service(&app, req).await;

        // CSP should remain as set by handler
        assert_eq!(
            resp.headers().get(header::CONTENT_SECURITY_POLICY),
            Some(&HeaderValue::from_static("custom-csp-directive"))
        );
    }

    #[actix_web::test]
    async fn test_security_headers_config_default() {
        let config = SecurityHeadersConfig::default();

        assert_eq!(
            config.content_security_policy,
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'"
        );
        assert_eq!(
            config.strict_transport_security,
            "max-age=31536000; includeSubDomains; preload"
        );
        assert_eq!(config.x_frame_options, "DENY");
        assert_eq!(config.x_content_type_options, "nosniff");
        assert_eq!(config.referrer_policy, "strict-origin-when-cross-origin");
        assert!(config.permissions_policy.contains("accelerometer=()"));
        assert_eq!(config.cache_control, "no-store, no-cache, must-revalidate, private");
        assert_eq!(config.pragma, "no-cache");
    }

    #[actix_web::test]
    async fn test_security_headers_with_invalid_header_config() {
        let custom_config = SecurityHeadersConfig {
            api_origin: None,
            content_security_policy: "invalid\x00header".to_string(),
            strict_transport_security: "invalid\x00header".to_string(),
            x_frame_options: String::new(),
            x_content_type_options: String::new(),
            referrer_policy: String::new(),
            permissions_policy: String::new(),
            cache_control: String::new(),
            pragma: String::new(),
        };

        let app = test::init_service(
            App::new()
                .wrap(SecurityHeaders::with_config(Arc::new(custom_config)))
                .route("/", web::get().to(index)),
        )
        .await;

        let req = test::TestRequest::get().to_request();
        let resp = test::call_service(&app, req).await;

        // Invalid header values should be silently ignored (not set)
        assert!(resp
            .headers()
            .get(header::STRICT_TRANSPORT_SECURITY)
            .is_none());
    }

    #[actix_web::test]
    async fn test_security_headers_default_impl() {
        let headers = SecurityHeaders::default();
        let app = test::init_service(
            App::new()
                .wrap(headers)
                .route("/", web::get().to(index)),
        )
        .await;

        let req = test::TestRequest::get().to_request();
        let resp = test::call_service(&app, req).await;

        // Verify default headers are applied
        assert!(resp
            .headers()
            .get(header::X_CONTENT_TYPE_OPTIONS)
            .is_some());
    }
}
