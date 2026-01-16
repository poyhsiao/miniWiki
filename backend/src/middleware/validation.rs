//! Request Validation Utilities
//!
//! Provides request validation utilities for security.
//! Validates incoming request size and content-type before reaching handlers.
//!
//! Usage: Add validation logic in your handlers using the helper functions.
//! Example:
//! ```ignore
//! async fn handler(req: ServiceRequest) -> Result<ServiceResponse, Error> {
//!     validate_content_type(&req)?;
//!     validate_request_size(&req, 1024 * 1024)?;
//!     // ... handle request
//! }
//! ```

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::StatusCode,
    Error, HttpResponse,
};
use thiserror::Error;

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Request body exceeds maximum size of {0} bytes")]
    PayloadTooLarge(usize),

    #[error("Content-Type must be application/json or multipart/form-data")]
    UnsupportedMediaType,
}

impl actix_web::ResponseError for ValidationError {
    fn status_code(&self) -> StatusCode {
        match self {
            ValidationError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ValidationError::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let message = self.to_string();
        HttpResponse::build(self.status_code())
            .json(serde_json::json!({
                "error": "VALIDATION_ERROR",
                "message": message,
            }))
    }
}

/// Validate request content-length header against maximum size
pub fn validate_request_size<B>(
    req: &ServiceRequest,
    max_size: usize,
) -> Result<(), ValidationError> {
    if let Some(content_length) = req.headers().get("content-length") {
        if let Ok(size_str) = content_length.to_str() {
            if let Ok(size) = size_str.parse::<usize>() {
                if size > max_size {
                    return Err(ValidationError::PayloadTooLarge(max_size));
                }
            }
        }
    }
    Ok(())
}

/// Validate Content-Type header for requests with bodies
pub fn validate_content_type(req: &ServiceRequest) -> Result<(), ValidationError> {
    let method = req.method();
    let has_body = *method == actix_web::http::Method::POST
        || *method == actix_web::http::Method::PUT
        || *method == actix_web::http::Method::PATCH;

    if has_body {
        if let Some(content_type) = req.headers().get("content-type") {
            if let Ok(ct_str) = content_type.to_str() {
                // Allow application/json and multipart/form-data
                if !ct_str.starts_with("application/json")
                    && !ct_str.starts_with("multipart/form-data")
                {
                    return Err(ValidationError::UnsupportedMediaType);
                }
            }
        }
    }
    Ok(())
}

/// Result type for validation operations
pub type ValidationResult = Result<(), ValidationError>;

/// Helper function to create a request size validator
pub fn validate_request_size_fn<B: MessageBody + 'static>(max_bytes: usize) -> impl Fn(&ServiceRequest) -> ValidationResult {
    move |req: &ServiceRequest| validate_request_size::<B>(req, max_bytes)
}

/// Helper function to create a content type validator
pub fn validate_content_type_fn<B: MessageBody + 'static>() -> impl Fn(&ServiceRequest) -> ValidationResult {
    |req: &ServiceRequest| validate_content_type(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::{ServiceRequest, ServiceResponse};
    use actix_web::http::{header, Method, StatusCode};
    use actix_web::HttpResponse;
    use actix_web::test::TestRequest;

    /// Helper to create a ServiceRequest for testing
    fn create_test_request(method: Method, content_type: Option<&str>, content_length: Option<&str>) -> ServiceRequest {
        let mut test_req = TestRequest::default();
        test_req = test_req.method(method);
        
        if let Some(ct) = content_type {
            test_req = test_req.insert_header((header::CONTENT_TYPE, ct));
        }
        
        if let Some(cl) = content_length {
            test_req = test_req.insert_header((header::CONTENT_LENGTH, cl));
        }
        
        test_req.to_srv_request()
    }

    #[actix_web::test]
    async fn test_validate_request_size_pass() {
        let req = create_test_request(Method::POST, Some("application/json"), Some("100"));
        let result = validate_request_size::<actix_web::body::BoxBody>(&req, 1024);
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_validate_request_size_fail() {
        let req = create_test_request(Method::POST, Some("application/json"), Some("2048"));
        let result = validate_request_size::<actix_web::body::BoxBody>(&req, 1024);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Request body exceeds maximum size of 1024 bytes"
        );
    }

    #[actix_web::test]
    async fn test_validate_content_type_pass() {
        let req = create_test_request(Method::POST, Some("application/json"), Some("100"));
        let result = validate_content_type(&req);
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_validate_content_type_fail() {
        let req = create_test_request(Method::POST, Some("text/plain"), Some("100"));
        let result = validate_content_type(&req);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Content-Type must be application/json or multipart/form-data"
        );
    }

    #[actix_web::test]
    async fn test_validate_content_type_get_request() {
        // GET requests should pass validation (no body expected)
        let req = create_test_request(Method::GET, None, None);
        let result = validate_content_type(&req);
        assert!(result.is_ok());
    }
}
