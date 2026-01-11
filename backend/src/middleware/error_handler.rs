use actix_web::{
    body::MessageBody,
    dev::{ServiceResponse, Transform},
    Error, HttpResponse,
};
use shared_errors::error_types::AppError;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Standard error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: i32,
    pub timestamp: String,
    pub path: Option<String>,
}

/// Create a standardized error response
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

/// Error handler middleware that transforms AppError into consistent HTTP responses
pub struct ErrorHandler;

impl<T, B> Transform<T, ServiceResponse<B>> for ErrorHandler
where
    T: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = ServiceResponse<B>,
        Error = Error,
    >,
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

impl<T, B> actix_web::dev::Service<actix_web::dev::ServiceRequest> for ErrorHandlerMiddleware<T>
where
    T: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = ServiceResponse<B>,
        Error = Error,
    >,
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

    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            // If response is an error, transform it
            if let Some(err) = res.error() {
                let status_code = err.as_response_error().status_code();
                let error_response = match err.as_response_error().as_ref() {
                    AppError::Unauthorized(msg) => {
                        create_error_response("UNAUTHORIZED", msg, 401, req.uri().path())
                    }
                    AppError::Forbidden(msg) => {
                        create_error_response("FORBIDDEN", msg, 403, req.uri().path())
                    }
            AppError::NotFound(msg) => {
                create_error_response("NOT_FOUND", msg, 404, None)
            }
                    AppError::Validation(msg) => {
                        create_error_response("VALIDATION_ERROR", msg, 400, req.uri().path())
                    }
                    AppError::Conflict(msg) => {
                        create_error_response("CONFLICT", msg, 409, req.uri().path())
                    }
                    AppError::Internal(msg) => {
                        create_error_response("INTERNAL_ERROR", msg, 500, req.uri().path())
                    }
                    _ => create_error_response(
                        "ERROR",
                        "An unexpected error occurred",
                        500,
                        req.uri().path(),
                    ),
                };

                // Create a new response with the error body
                let (req, _res) = res.into_parts();
                let res = error_response;
                let (head, body) = res.into_parts();

                Ok(ServiceResponse::new(req, actix_web::web::BytesMut::from(body.to_string().as_bytes()).freeze().into()))
            } else {
                Ok(res)
            }
        })
    }
}

/// Extension trait for easier error handling
pub trait AppErrorResponse {
    fn to_error_response(&self) -> HttpResponse;
}

impl AppErrorResponse for AppError {
    fn to_error_response(&self) -> HttpResponse {
        match self {
            AppError::Unauthorized(msg) => {
                create_error_response("UNAUTHORIZED", msg, 401, None)
            }
            AppError::Forbidden(msg) => create_error_response("FORBIDDEN", msg, 403, None),
            AppError::NotFound(msg) => create_error_error_response("NOT_FOUND", msg, 404, None),
            AppError::Validation(msg) => {
                create_error_response("VALIDATION_ERROR", msg, 400, None)
            }
            AppError::Conflict(msg) => create_error_response("CONFLICT", msg, 409, None),
            AppError::Internal(msg) => create_error_response("INTERNAL_ERROR", msg, 500, None),
        }
    }
}
