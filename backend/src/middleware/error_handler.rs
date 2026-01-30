use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, ResponseError,
};
use shared_errors::error_types::AppError;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: i32,
    pub timestamp: String,
    pub path: Option<String>,
}


pub struct ErrorHandler;

impl<T, B> Transform<T, ServiceRequest> for ErrorHandler
where
    T: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    T::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ErrorHandlerMiddleware<T>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: T) -> Self::Future {
        std::future::ready(Ok(ErrorHandlerMiddleware { service }))
    }
}

pub struct ErrorHandlerMiddleware<T> {
    service: T,
}

impl<T, B> Service<ServiceRequest> for ErrorHandlerMiddleware<T>
where
    T: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    T::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

pub trait AppErrorResponse {
    fn to_error_response(&self) -> HttpResponse;
}

impl AppErrorResponse for AppError {
    fn to_error_response(&self) -> HttpResponse {
        let status_code = self.status_code().as_u16();
        let error_code = match self {
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::AuthenticationError(_) => "AUTHENTICATION_ERROR",
            AppError::AuthorizationError(_) => "AUTHORIZATION_ERROR",
            AppError::NotFoundError(_) => "NOT_FOUND",
            AppError::ConflictError(_) => "CONFLICT",
            AppError::RateLimitError(_) => "RATE_LIMIT_EXCEEDED",
            AppError::InternalError(_) => "INTERNAL_ERROR",
            AppError::ConfigurationError(_) => "CONFIGURATION_ERROR",
            AppError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
        };

        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: error_code.to_string(),
            message: self.to_string(),
            status_code: status_code as i32,
            timestamp: chrono::Utc::now().to_rfc3339(),
            path: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_errors::error_types::AppError;

    #[test]
    fn test_error_response_structure() {
        let error = ErrorResponse {
            error: "TEST_ERROR".to_string(),
            message: "Test message".to_string(),
            status_code: 400,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            path: Some("/test/path".to_string()),
        };

        assert_eq!(error.error, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
        assert_eq!(error.status_code, 400);
        assert_eq!(error.timestamp, "2024-01-01T00:00:00Z");
        assert_eq!(error.path, Some("/test/path".to_string()));
    }

    #[test]
    fn test_internal_error_to_response() {
        let error = AppError::InternalError("Connection failed".to_string());
        let response = error.to_error_response();

        assert_eq!(response.status(), 500);
    }

    #[test]
    fn test_validation_error_to_response() {
        let error = AppError::ValidationError("Invalid input".to_string());
        let response = error.to_error_response();

        assert_eq!(response.status(), 400);
    }

    #[test]
    fn test_authentication_error_to_response() {
        let error = AppError::AuthenticationError("Invalid credentials".to_string());
        let response = error.to_error_response();

        assert_eq!(response.status(), 401);
    }

    #[test]
    fn test_authorization_error_to_response() {
        let error = AppError::AuthorizationError("Access denied".to_string());
        let response = error.to_error_response();

        assert_eq!(response.status(), 403);
    }

    #[test]
    fn test_not_found_error_to_response() {
        let error = AppError::NotFoundError("Resource not found".to_string());
        let response = error.to_error_response();

        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_conflict_error_to_response() {
        let error = AppError::ConflictError("Resource already exists".to_string());
        let response = error.to_error_response();

        assert_eq!(response.status(), 409);
    }

    #[test]
    fn test_rate_limit_error_to_response() {
        let error = AppError::RateLimitError("Too many requests".to_string());
        let response = error.to_error_response();

        assert_eq!(response.status(), 429);
    }
}
