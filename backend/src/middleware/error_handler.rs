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

fn create_error_response(
    error: &str,
    message: &str,
    status_code: i32,
    path: Option<&str>,
) -> HttpResponse {
    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status_code as u16)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    )
    .json(ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
        status_code,
        timestamp: chrono::Utc::now().to_rfc3339(),
        path: path.map(|p| p.to_string()),
    })
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
