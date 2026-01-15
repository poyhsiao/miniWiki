use super::error_types::AppError;

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
