use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("Conflict: {0}")]
    ConflictError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

impl AppError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFoundError(msg.into())
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::DatabaseError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::AuthenticationError(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError(_) => actix_web::http::StatusCode::FORBIDDEN,
            AppError::NotFoundError(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::ConflictError(_) => actix_web::http::StatusCode::CONFLICT,
            AppError::RateLimitError(_) => actix_web::http::StatusCode::TOO_MANY_REQUESTS,
            AppError::InternalError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ConfigurationError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalServiceError(_) => actix_web::http::StatusCode::BAD_GATEWAY,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code())
            .json(serde_json::json!({
                "error": ErrorCode::from(self).to_string(),
                "message": self.to_string()
            }))
    }
}

use super::error_codes::ErrorCode;
