use serde::{Deserialize, Serialize};
use shared_security::PasswordRequirements;
use validator::Validate;

// Helper function to validate password using shared security module
// This delegates to shared_security for centralized password validation
fn validate_password(password: &str) -> Result<(), validator::ValidationError> {
    let requirements = PasswordRequirements {
        min_length: 8,
        max_length: 100,
        require_uppercase: true,
        require_lowercase: true,
        require_digit: true,
        require_special_char: false,
    };

    match shared_security::validate_password_strength_with_requirements(password, &requirements) {
        Ok(()) => Ok(()),
        Err(e) => {
            Err(validator::ValidationError::new("invalid_password")
                .with_message(std::borrow::Cow::Owned(e.to_string())))
        },
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 100, code = "Password must be 8-100 characters"))]
    #[validate(custom(function = "validate_password"))]
    pub password: String,

    #[validate(length(min = 1, max = 100))]
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: UserResponse,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_email_verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_valid() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "Password123".to_string(),
            display_name: "Test User".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_register_request_invalid_email() {
        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            password: "Password123".to_string(),
            display_name: "Test User".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_request_short_password() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "short".to_string(),
            display_name: "Test User".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_request_empty_display_name() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "Password123".to_string(),
            display_name: "".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_login_request_valid() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "Password123".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_login_request_invalid_email() {
        let request = LoginRequest {
            email: "not-an-email".to_string(),
            password: "Password123".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_user_response_creation() {
        let user = UserResponse {
            id: "user-123".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            is_email_verified: true,
        };
        assert_eq!(user.id, "user-123");
        assert_eq!(user.email, "test@example.com");
        assert!(user.avatar_url.is_some());
        assert!(user.is_email_verified);
    }

    #[test]
    fn test_user_response_no_avatar() {
        let user = UserResponse {
            id: "user-456".to_string(),
            email: "test2@example.com".to_string(),
            display_name: "Another User".to_string(),
            avatar_url: None,
            is_email_verified: false,
        };
        assert!(user.avatar_url.is_none());
        assert!(!user.is_email_verified);
    }

    #[test]
    fn test_register_response_creation() {
        let user = UserResponse {
            id: "user-789".to_string(),
            email: "register@example.com".to_string(),
            display_name: "Register User".to_string(),
            avatar_url: None,
            is_email_verified: false,
        };
        let response = RegisterResponse {
            user,
            message: "Registration successful".to_string(),
        };
        assert_eq!(response.message, "Registration successful");
    }

    #[test]
    fn test_login_response_creation() {
        let user = UserResponse {
            id: "user-login".to_string(),
            email: "login@example.com".to_string(),
            display_name: "Login User".to_string(),
            avatar_url: None,
            is_email_verified: true,
        };
        let response = LoginResponse {
            user,
            access_token: "access-token-123".to_string(),
            refresh_token: "refresh-token-456".to_string(),
            expires_in: 3600,
        };
        assert_eq!(response.access_token, "access-token-123");
        assert_eq!(response.refresh_token, "refresh-token-456");
        assert_eq!(response.expires_in, 3600);
    }

    #[test]
    fn test_refresh_request_creation() {
        let request = RefreshRequest {
            refresh_token: "refresh-token-abc".to_string(),
        };
        assert_eq!(request.refresh_token, "refresh-token-abc");
    }

    #[test]
    fn test_refresh_response_creation() {
        let response = RefreshResponse {
            access_token: "new-access-token".to_string(),
            expires_in: 7200,
        };
        assert_eq!(response.access_token, "new-access-token");
        assert_eq!(response.expires_in, 7200);
    }

    #[test]
    fn test_logout_request_with_token() {
        let request = LogoutRequest {
            refresh_token: Some("token-to-revoke".to_string()),
        };
        assert!(request.refresh_token.is_some());
        assert_eq!(request.refresh_token.unwrap(), "token-to-revoke");
    }

    #[test]
    fn test_logout_request_without_token() {
        let request = LogoutRequest { refresh_token: None };
        assert!(request.refresh_token.is_none());
    }

    #[test]
    fn test_register_request_display_name_max_length() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "Password123".to_string(),
            display_name: "a".repeat(100),
        };
        assert_eq!(request.display_name.len(), 100);
        assert!(request.validate().is_ok()); // Max is 100, so 100 is valid
    }

    #[test]
    fn test_password_regex_requirement() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "weak".to_string(),
            display_name: "Test".to_string(),
        };
        assert!(request.validate().is_err());
    }
}
