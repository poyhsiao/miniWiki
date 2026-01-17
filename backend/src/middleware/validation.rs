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
    dev::ServiceRequest,
    http::StatusCode,
    HttpResponse,
};
use thiserror::Error;

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Request body exceeds maximum size of {0} bytes")]
    PayloadTooLarge(usize),

    #[error("Content-Type must be application/json or multipart/form-data")]
    UnsupportedMediaType,

    #[error("Missing or invalid Content-Length header")]
    InvalidContentLength,
}

impl actix_web::ResponseError for ValidationError {
    fn status_code(&self) -> StatusCode {
        match self {
            ValidationError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ValidationError::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ValidationError::InvalidContentLength => StatusCode::BAD_REQUEST,
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
pub fn validate_request_size(
    req: &ServiceRequest,
    max_size: usize,
) -> Result<(), ValidationError> {
    let content_length = match req.headers().get("content-length") {
        Some(header) => header,
        None => return Err(ValidationError::InvalidContentLength),
    };

    let size_str = match content_length.to_str() {
        Ok(s) => s,
        Err(_) => return Err(ValidationError::InvalidContentLength),
    };

    let size = match size_str.parse::<usize>() {
        Ok(n) => n,
        Err(_) => return Err(ValidationError::InvalidContentLength),
    };

    if size > max_size {
        return Err(ValidationError::PayloadTooLarge(max_size));
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
        let content_type = match req.headers().get("content-type") {
            Some(header) => header,
            None => return Err(ValidationError::UnsupportedMediaType),
        };

        let ct_str = match content_type.to_str() {
            Ok(s) => s,
            Err(_) => return Err(ValidationError::UnsupportedMediaType),
        };

        if !ct_str.starts_with("application/json")
            && !ct_str.starts_with("multipart/form-data")
        {
            return Err(ValidationError::UnsupportedMediaType);
        }
    }
    Ok(())
}

/// Result type for validation operations
pub type ValidationResult = Result<(), ValidationError>;

/// Helper function to create a request size validator
pub fn validate_request_size_fn(max_bytes: usize) -> impl Fn(&ServiceRequest) -> ValidationResult {
    move |req: &ServiceRequest| validate_request_size(req, max_bytes)
}

/// Helper function to create a content type validator
pub fn validate_content_type_fn() -> impl Fn(&ServiceRequest) -> ValidationResult {
    |req: &ServiceRequest| validate_content_type(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::ServiceRequest;
    use actix_web::http::{header, Method};
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
        let result = validate_request_size(&req, 1024);
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_validate_request_size_fail() {
        let req = create_test_request(Method::POST, Some("application/json"), Some("2048"));
        let result = validate_request_size(&req, 1024);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Request body exceeds maximum size of 1024 bytes"
        );
    }

    #[actix_web::test]
    async fn test_validate_request_size_missing_header() {
        let req = create_test_request(Method::POST, Some("application/json"), None);
        let result = validate_request_size(&req, 1024);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Missing or invalid Content-Length header"
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
    async fn test_validate_content_type_missing_header() {
        let req = create_test_request(Method::POST, None, Some("100"));
        let result = validate_content_type(&req);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Content-Type must be application/json or multipart/form-data"
        );
    }

    #[actix_web::test]
    async fn test_validate_content_type_get_request() {
        let req = create_test_request(Method::GET, None, None);
        let result = validate_content_type(&req);
        assert!(result.is_ok());
    }
}
