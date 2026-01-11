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

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    DatabaseError,
    ValidationError,
    AuthenticationError,
    AuthorizationError,
    NotFoundError,
    ConflictError,
    RateLimitError,
    InternalError,
    ConfigurationError,
    ExternalServiceError,
}

impl From<&AppError> for ErrorCode {
    fn from(err: &AppError) -> Self {
        match err {
            AppError::DatabaseError(_) => ErrorCode::DatabaseError,
            AppError::ValidationError(_) => ErrorCode::ValidationError,
            AppError::AuthenticationError(_) => ErrorCode::AuthenticationError,
            AppError::AuthorizationError(_) => ErrorCode::AuthorizationError,
            AppError::NotFoundError(_) => ErrorCode::NotFoundError,
            AppError::ConflictError(_) => ErrorCode::ConflictError,
            AppError::RateLimitError(_) => ErrorCode::RateLimitError,
            AppError::InternalError(_) => ErrorCode::InternalError,
            AppError::ConfigurationError(_) => ErrorCode::ConfigurationError,
            AppError::ExternalServiceError(_) => ErrorCode::ExternalServiceError,
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::DatabaseError => write!(f, "DATABASE_ERROR"),
            ErrorCode::ValidationError => write!(f, "VALIDATION_ERROR"),
            ErrorCode::AuthenticationError => write!(f, "AUTHENTICATION_ERROR"),
            ErrorCode::AuthorizationError => write!(f, "AUTHORIZATION_ERROR"),
            ErrorCode::NotFoundError => write!(f, "NOT_FOUND"),
            ErrorCode::ConflictError => write!(f, "CONFLICT"),
            ErrorCode::RateLimitError => write!(f, "RATE_LIMIT_EXCEEDED"),
            ErrorCode::InternalError => write!(f, "INTERNAL_ERROR"),
            ErrorCode::ConfigurationError => write!(f, "CONFIGURATION_ERROR"),
            ErrorCode::ExternalServiceError => write!(f, "EXTERNAL_SERVICE_ERROR"),
        }
    }
}
