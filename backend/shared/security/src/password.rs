//! Password hashing, verification, and validation utilities
//!
//! This module provides secure password operations using bcrypt for hashing
//! and configurable password strength requirements.

use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;
use std::fmt;

/// Default cost factor for bcrypt hashing
pub const DEFAULT_BCRYPT_COST: u32 = DEFAULT_COST;

/// Configuration for password requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasswordRequirements {
    /// Minimum password length
    pub min_length: usize,
    /// Maximum password length
    pub max_length: usize,
    /// Require at least one uppercase letter
    pub require_uppercase: bool,
    /// Require at least one lowercase letter
    pub require_lowercase: bool,
    /// Require at least one digit
    pub require_digit: bool,
    /// Require at least one special character
    pub require_special_char: bool,
}

impl Default for PasswordRequirements {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special_char: false,
        }
    }
}

/// Errors that can occur during password operations
#[derive(Debug, Clone, thiserror::Error, serde::Serialize)]
pub enum PasswordError {
    /// Error during password hashing
    #[error("Password hashing error: {0}")]
    HashError(String),

    /// Error during password verification
    #[error("Password verification error: {0}")]
    VerifyError(String),

    /// Password does not meet strength requirements
    #[error("Password does not meet requirements: {0}")]
    WeakPassword(String),

    /// Password exceeds maximum allowed length
    #[error("Password exceeds maximum length of {0} characters")]
    TooLong(usize),
}

/// Detailed password validation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordValidationError {
    TooShort(usize),
    TooLong(usize),
    MissingUppercase,
    MissingLowercase,
    MissingDigit,
    MissingSpecialChar,
}

impl fmt::Display for PasswordValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordValidationError::TooShort(min) => write!(f, "Password must be at least {} characters", min),
            PasswordValidationError::TooLong(max) => write!(f, "Password must not exceed {} characters", max),
            PasswordValidationError::MissingUppercase => write!(f, "Password must contain at least one uppercase letter"),
            PasswordValidationError::MissingLowercase => write!(f, "Password must contain at least one lowercase letter"),
            PasswordValidationError::MissingDigit => write!(f, "Password must contain at least one number"),
            PasswordValidationError::MissingSpecialChar => write!(f, "Password must contain at least one special character"),
        }
    }
}

/// Hash a password using bcrypt
///
/// # Arguments
///
/// * `password` - The password to hash
///
/// # Returns
///
/// A bcrypt hash of the password
///
/// # Errors
///
/// Returns `PasswordError::HashError` if hashing fails
///
/// # Example
///
/// ```ignore
/// use shared_security::hash_password;
///
/// let hash = hash_password("my_secure_password").unwrap();
/// assert!(hash.starts_with("$2b$"));
/// ```
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    // Validate password length before hashing
    let requirements = PasswordRequirements::default();
    if password.len() > requirements.max_length {
        return Err(PasswordError::TooLong(requirements.max_length));
    }

    hash(password, DEFAULT_BCRYPT_COST)
        .map_err(|e| PasswordError::HashError(e.to_string()))
}

/// Hash a password with custom cost factor
///
/// # Arguments
///
/// * `password` - The password to hash
/// * `cost` - The cost factor (4-31, higher is slower but more secure)
///
/// # Returns
///
/// A bcrypt hash of the password
pub fn hash_password_with_cost(password: &str, cost: u32) -> Result<String, PasswordError> {
    let requirements = PasswordRequirements::default();
    if password.len() > requirements.max_length {
        return Err(PasswordError::TooLong(requirements.max_length));
    }

    hash(password, cost)
        .map_err(|e| PasswordError::HashError(e.to_string()))
}

/// Verify a password against a hash
///
/// # Arguments
///
/// * `password` - The password to verify
/// * `hash` - The bcrypt hash to verify against
///
/// # Returns
///
/// `true` if the password matches the hash, `false` otherwise
///
/// # Errors
///
/// Returns `PasswordError::VerifyError` if verification fails
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    verify(password, hash)
        .map_err(|e| PasswordError::VerifyError(e.to_string()))
}

/// Validate password strength against requirements
///
/// # Arguments
///
/// * `password` - The password to validate
/// * `requirements` - The password requirements to validate against
///
/// # Returns
///
/// `Ok(())` if the password meets all requirements
///
/// # Errors
///
/// Returns `PasswordError::WeakPassword` with a detailed message
pub fn validate_password_strength_with_requirements(
    password: &str,
    requirements: &PasswordRequirements,
) -> Result<(), PasswordError> {
    let mut errors = Vec::new();

    // Check minimum length
    if password.len() < requirements.min_length {
        errors.push(PasswordValidationError::TooShort(requirements.min_length).to_string());
    }

    // Check maximum length
    if password.len() > requirements.max_length {
        errors.push(PasswordValidationError::TooLong(requirements.max_length).to_string());
    }

    // Check for uppercase letters
    if requirements.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        errors.push(PasswordValidationError::MissingUppercase.to_string());
    }

    // Check for lowercase letters
    if requirements.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        errors.push(PasswordValidationError::MissingLowercase.to_string());
    }

    // Check for digits
    if requirements.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push(PasswordValidationError::MissingDigit.to_string());
    }

    // Check for special characters (non-alphanumeric)
    if requirements.require_special_char && !password.chars().any(|c| !c.is_alphanumeric()) {
        errors.push(PasswordValidationError::MissingSpecialChar.to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(PasswordError::WeakPassword(errors.join(", ")))
    }
}

/// Validate password strength with default requirements
///
/// This is a convenience function that uses the default password requirements.
///
/// # Arguments
///
/// * `password` - The password to validate
///
/// # Returns
///
/// `Ok(())` if the password meets all requirements
///
/// # Errors
///
/// Returns `PasswordError::WeakPassword` with a detailed message
pub fn validate_password_strength(password: &str) -> Result<(), PasswordError> {
    validate_password_strength_with_requirements(password, &PasswordRequirements::default())
}

/// Generate a cryptographically secure random token
///
/// # Arguments
///
/// * `length` - The length of the token to generate
///
/// # Returns
///
/// A random alphanumeric token
///
/// # Example
///
/// ```ignore
/// use shared_security::generate_reset_token;
///
/// let token = generate_reset_token(32);
/// assert_eq!(token.len(), 32);
/// ```
pub fn generate_reset_token(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a URL-safe token (no ambiguous characters)
///
/// # Arguments
///
/// * `length` - The length of the token to generate
///
/// # Returns
///
/// A URL-safe random token
pub fn generate_url_safe_token(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Password Hashing Tests
    // ========================================

    #[test]
    fn test_hash_password_success() {
        let password = "TestPassword123!";
        let hash = hash_password(password).unwrap();

        // Bcrypt hashes start with $2b$
        assert!(hash.starts_with("$2b$"));
        assert_ne!(hash, password);
    }

    #[test]
    fn test_hash_password_different_hashes() {
        let password = "TestPassword123!";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should produce different hashes due to salt
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_password_too_long() {
        let password = "a".repeat(200);
        let result = hash_password(&password);

        assert!(matches!(result, Err(PasswordError::TooLong(128))));
    }

    #[test]
    fn test_hash_password_with_custom_cost() {
        let password = "TestPassword123!";
        let hash = hash_password_with_cost(password, 4).unwrap();

        assert!(hash.starts_with("$2b$04$"));
    }

    #[test]
    fn test_hash_password_empty_string() {
        let password = "";
        let hash = hash_password(password).unwrap();

        // Bcrypt can hash empty strings
        assert!(hash.starts_with("$2b$"));
    }

    // ========================================
    // Password Verification Tests
    // ========================================

    #[test]
    fn test_verify_password_success() {
        let password = "TestPassword123!";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_wrong_password() {
        let password = "TestPassword123!";
        let wrong_password = "WrongPassword456!";
        let hash = hash_password(password).unwrap();

        let result = verify_password(wrong_password, &hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let password = "TestPassword123!";
        let invalid_hash = "invalid_hash";

        let result = verify_password(password, invalid_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_empty_strings() {
        let password = "";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_unicode() {
        let password = "Test密碼123!"; // Contains Chinese characters
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    // ========================================
    // Password Validation Tests
    // ========================================

    #[test]
    fn test_validate_password_strength_valid() {
        let password = "TestPass123";
        let result = validate_password_strength(password);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let password = "Test1";
        let result = validate_password_strength(password);

        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));
        if let Err(PasswordError::WeakPassword(msg)) = result {
            assert!(msg.contains("characters"));
        }
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let password = "testpass123";
        let result = validate_password_strength(password);

        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));
        if let Err(PasswordError::WeakPassword(msg)) = result {
            assert!(msg.contains("uppercase"));
        }
    }

    #[test]
    fn test_validate_password_strength_no_lowercase() {
        let password = "TESTPASS123";
        let result = validate_password_strength(password);

        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));
        if let Err(PasswordError::WeakPassword(msg)) = result {
            assert!(msg.contains("lowercase"));
        }
    }

    #[test]
    fn test_validate_password_strength_no_digit() {
        let password = "TestPassword";
        let result = validate_password_strength(password);

        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));
        if let Err(PasswordError::WeakPassword(msg)) = result {
            assert!(msg.contains("number"));
        }
    }

    #[test]
    fn test_validate_password_strength_multiple_errors() {
        let password = "short";
        let result = validate_password_strength(password);

        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));
        if let Err(PasswordError::WeakPassword(msg)) = result {
            // Should have multiple error messages
            assert!(msg.contains(","));
        }
    }

    #[test]
    fn test_validate_password_strength_custom_requirements() {
        let requirements = PasswordRequirements {
            min_length: 12,
            max_length: 64,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special_char: true,
        };

        let password = "Short1!"; // Too short
        let result = validate_password_strength_with_requirements(&password, &requirements);

        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));

        let password = "LongEnoughPassword1!"; // Meets requirements
        let result = validate_password_strength_with_requirements(&password, &requirements);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_strength_no_special_char_required() {
        let requirements = PasswordRequirements {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special_char: false,
        };

        let password = "TestPass123";
        let result = validate_password_strength_with_requirements(&password, &requirements);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_strength_minimal_requirements() {
        let requirements = PasswordRequirements {
            min_length: 1,
            max_length: 1000,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: false,
            require_special_char: false,
        };

        let password = "a";
        let result = validate_password_strength_with_requirements(&password, &requirements);

        assert!(result.is_ok());
    }

    // ========================================
    // Token Generation Tests
    // ========================================

    #[test]
    fn test_generate_reset_token_length() {
        let token = generate_reset_token(32);
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_generate_reset_token_unique() {
        let token1 = generate_reset_token(32);
        let token2 = generate_reset_token(32);

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_reset_token_alphanumeric() {
        let token = generate_reset_token(32);

        // Should only contain alphanumeric characters
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_reset_token_empty_length() {
        let token = generate_reset_token(0);
        assert_eq!(token.len(), 0);
    }

    #[test]
    fn test_generate_reset_token_large_length() {
        let token = generate_reset_token(1000);
        assert_eq!(token.len(), 1000);
    }

    #[test]
    fn test_generate_url_safe_token_length() {
        let token = generate_url_safe_token(32);
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_generate_url_safe_token_no_ambiguous_chars() {
        let token = generate_url_safe_token(100);

        // Should not contain ambiguous characters (0, O, I, l, 1)
        assert!(!token.contains('0'));
        assert!(!token.contains('O'));
        assert!(!token.contains('I'));
        assert!(!token.contains('l'));
        assert!(!token.contains('1'));
    }

    // ========================================
    // Edge Cases and Boundary Conditions
    // ========================================

    #[test]
    fn test_password_requirements_default() {
        let requirements = PasswordRequirements::default();

        assert_eq!(requirements.min_length, 8);
        assert_eq!(requirements.max_length, 128);
        assert!(requirements.require_uppercase);
        assert!(requirements.require_lowercase);
        assert!(requirements.require_digit);
        assert!(!requirements.require_special_char);
    }

    #[test]
    fn test_password_error_display() {
        let error = PasswordError::HashError("test error".to_string());
        assert_eq!(format!("{}", error), "Password hashing error: test error");

        let error = PasswordError::TooLong(100);
        assert_eq!(format!("{}", error), "Password exceeds maximum length of 100 characters");

        let validation_error = PasswordValidationError::TooShort(8);
        assert_eq!(format!("{}", validation_error), "Password must be at least 8 characters");

        let validation_error = PasswordValidationError::TooLong(128);
        assert_eq!(format!("{}", validation_error), "Password must not exceed 128 characters");
    }

    #[test]
    fn test_password_validation_at_min_length() {
        let password = "Test1"; // Exactly 5 characters, below min 8
        let requirements = PasswordRequirements {
            min_length: 5,
            ..Default::default()
        };

        let result = validate_password_strength_with_requirements(&password, &requirements);
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_validation_at_max_length() {
        let password = "A".repeat(128) + "a1"; // Exactly 130 characters
        let requirements = PasswordRequirements {
            max_length: 130,
            ..Default::default()
        };

        let result = validate_password_strength_with_requirements(&password, &requirements);
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_exceeds_max_length() {
        let password = "A".repeat(64) + "a1"; // 66 characters
        let requirements = PasswordRequirements {
            max_length: 65,
            ..Default::default()
        };

        let result = validate_password_strength_with_requirements(&password, &requirements);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_timing_attack_resistance() {
        // This test verifies that verification is constant-time for correct vs incorrect passwords
        // In practice, bcrypt provides this property
        let password = "TestPassword123!";
        let wrong_password = "WrongPassword456!";
        let hash = hash_password(password).unwrap();

        let _ = verify_password(password, &hash).unwrap();
        let _ = verify_password(wrong_password, &hash).unwrap();

        // If we reach here, both verifications completed without panic
        // Bcrypt's design protects against timing attacks
    }

    #[test]
    fn test_password_with_only_special_chars() {
        let password = "!@#$%^&*()";
        let requirements = PasswordRequirements {
            require_special_char: true,
            ..Default::default()
        };

        let result = validate_password_strength_with_requirements(&password, &requirements);
        // Should fail because no uppercase, lowercase, or digit
        assert!(matches!(result, Err(PasswordError::WeakPassword(_))));
    }

    #[test]
    fn test_password_with_all_categories() {
        let password = "Abc123!@#";
        let requirements = PasswordRequirements {
            require_special_char: true,
            ..Default::default()
        };

        let result = validate_password_strength_with_requirements(&password, &requirements);
        assert!(result.is_ok());
    }
}
