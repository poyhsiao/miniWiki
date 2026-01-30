//! Password utilities for auth_service
//!
//! This module re-exports password utilities from shared_security for backward compatibility.

pub use shared_security::{
    generate_reset_token, generate_url_safe_token, hash_password, hash_password_with_cost, validate_password_strength,
    validate_password_strength_with_requirements, verify_password, PasswordError, PasswordRequirements,
    PasswordValidationError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "TestPassword123!";
        let hash = hash_password(password).unwrap();

        // Verify correct password
        assert!(verify_password(password, &hash).unwrap());

        // Verify wrong password
        assert!(!verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_validate_password_strength_valid() {
        let password = "TestPass123";
        assert!(validate_password_strength(password).is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let password = "Test1";
        assert!(validate_password_strength(password).is_err());
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let password = "testpass123";
        assert!(validate_password_strength(password).is_err());
    }

    #[test]
    fn test_validate_password_strength_no_lowercase() {
        let password = "TESTPASS123";
        assert!(validate_password_strength(password).is_err());
    }

    #[test]
    fn test_validate_password_strength_no_digit() {
        let password = "TestPassword";
        assert!(validate_password_strength(password).is_err());
    }

    #[test]
    fn test_generate_reset_token() {
        let token = generate_reset_token(32);
        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_password_error_messages() {
        // Test error display
        let error = PasswordError::HashError("test error".to_string());
        assert_eq!(format!("{}", error), "Password hashing error: test error");

        let error = PasswordError::WeakPassword("too short".to_string());
        assert_eq!(format!("{}", error), "Password does not meet requirements: too short");
    }
}
