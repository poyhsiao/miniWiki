use super::error_types::AppError;

#[derive(Debug, PartialEq, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_types::AppError;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::DatabaseError.to_string(), "DATABASE_ERROR");
        assert_eq!(ErrorCode::ValidationError.to_string(), "VALIDATION_ERROR");
        assert_eq!(ErrorCode::AuthenticationError.to_string(), "AUTHENTICATION_ERROR");
        assert_eq!(ErrorCode::AuthorizationError.to_string(), "AUTHORIZATION_ERROR");
        assert_eq!(ErrorCode::NotFoundError.to_string(), "NOT_FOUND");
        assert_eq!(ErrorCode::ConflictError.to_string(), "CONFLICT");
        assert_eq!(ErrorCode::RateLimitError.to_string(), "RATE_LIMIT_EXCEEDED");
        assert_eq!(ErrorCode::InternalError.to_string(), "INTERNAL_ERROR");
        assert_eq!(ErrorCode::ConfigurationError.to_string(), "CONFIGURATION_ERROR");
        assert_eq!(ErrorCode::ExternalServiceError.to_string(), "EXTERNAL_SERVICE_ERROR");
    }

    #[test]
    fn test_error_code_from_app_error() {
        let error = AppError::ValidationError("test".to_string());
        let code = ErrorCode::from(&error);
        assert_eq!(code, ErrorCode::ValidationError);

        let error = AppError::NotFoundError("test".to_string());
        let code = ErrorCode::from(&error);
        assert_eq!(code, ErrorCode::NotFoundError);

        let error = AppError::AuthenticationError("test".to_string());
        let code = ErrorCode::from(&error);
        assert_eq!(code, ErrorCode::AuthenticationError);
    }
}
