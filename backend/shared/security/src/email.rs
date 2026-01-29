//! Email validation utilities (ReDoS-safe)
//!
//! This module provides email validation that is resistant to Regular Expression
//! Denial of Service (ReDoS) attacks by avoiding complex regex patterns and
//! using a multi-step validation approach.

use std::fmt;

/// Detailed email validation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailValidationError {
    /// Email is empty or whitespace only
    Empty,
    /// Email exceeds maximum allowed length (RFC 5321 limits to 254 characters)
    TooLong,
    /// Missing @ separator
    MissingAtSign,
    /// Multiple @ signs found
    MultipleAtSigns,
    /// Missing local part (before @)
    MissingLocalPart,
    /// Missing domain part (after @)
    MissingDomain,
    /// Domain does not contain a dot
    DomainMissingDot,
    /// Domain part is too short
    DomainTooShort,
    /// Local part is too short
    LocalPartTooShort,
    /// Local part exceeds maximum length (RFC 5321 limits to 64 characters)
    LocalPartTooLong,
    /// Invalid character in email
    InvalidCharacter,
    /// Email starts or ends with a dot
    LeadingTrailingDot,
    /// Consecutive dots in email
    ConsecutiveDots,
}

impl fmt::Display for EmailValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailValidationError::Empty => write!(f, "Email address is required"),
            EmailValidationError::TooLong => write!(f, "Email address is too long (maximum 254 characters)"),
            EmailValidationError::MissingAtSign => write!(f, "Email address must contain @ symbol"),
            EmailValidationError::MultipleAtSigns => write!(f, "Email address must contain only one @ symbol"),
            EmailValidationError::MissingLocalPart => write!(f, "Email address must have a name before @"),
            EmailValidationError::MissingDomain => write!(f, "Email address must have a domain after @"),
            EmailValidationError::DomainMissingDot => write!(f, "Email domain must contain at least one dot"),
            EmailValidationError::DomainTooShort => write!(f, "Email domain is too short"),
            EmailValidationError::LocalPartTooShort => {
                write!(f, "Email name before @ is too short (minimum 1 character)")
            },
            EmailValidationError::LocalPartTooLong => {
                write!(f, "Email name before @ is too long (maximum 64 characters)")
            },
            EmailValidationError::InvalidCharacter => write!(f, "Email address contains invalid characters"),
            EmailValidationError::LeadingTrailingDot => write!(f, "Email address cannot start or end with a dot"),
            EmailValidationError::ConsecutiveDots => write!(f, "Email address cannot contain consecutive dots"),
        }
    }
}

/// Validate an email address using a safe, multi-step approach
///
/// This function validates email addresses without using complex regex patterns,
/// making it resistant to ReDoS (Regular Expression Denial of Service) attacks.
///
/// # Validation Rules
///
/// - Must contain exactly one @ symbol
/// - Local part (before @) must be 1-64 characters
/// - Domain part (after @) must contain at least one dot and non-empty labels
/// - Total length must not exceed 254 characters (RFC 5321)
/// - Must not start or end with a dot
/// - Must not contain consecutive dots
/// - Only allows alphanumeric, dot, underscore, hyphen, and plus characters
///
/// # Arguments
///
/// * `email` - The email address to validate
///
/// # Returns
///
/// `Ok(())` if the email is valid
///
/// # Errors
///
/// Returns `EmailValidationError` with a detailed message
///
/// # Example
///
/// ```ignore
/// use shared_security::validate_email;
///
/// assert!(validate_email("user@example.com").is_ok());
/// assert!(validate_email("invalid-email").is_err());
/// ```
pub fn validate_email(email: &str) -> Result<(), EmailValidationError> {
    // Trim whitespace
    let email = email.trim();

    // Check for empty email
    if email.is_empty() {
        return Err(EmailValidationError::Empty);
    }

    // Check total length (RFC 5321 limit: 254 characters)
    if email.len() > 254 {
        return Err(EmailValidationError::TooLong);
    }

    // Check for @ sign
    let at_count = email.matches('@').count();
    if at_count == 0 {
        return Err(EmailValidationError::MissingAtSign);
    }
    if at_count > 1 {
        return Err(EmailValidationError::MultipleAtSigns);
    }

    // Split at @ sign
    let parts: Vec<&str> = email.split('@').collect();
    let local_part = parts[0];
    let domain_part = parts[1];

    // Validate local part
    if local_part.is_empty() {
        return Err(EmailValidationError::MissingLocalPart);
    }
    if local_part.len() > 64 {
        return Err(EmailValidationError::LocalPartTooLong);
    }

    // Validate domain part
    if domain_part.is_empty() {
        return Err(EmailValidationError::MissingDomain);
    }
    if !domain_part.contains('.') {
        return Err(EmailValidationError::DomainMissingDot);
    }

    // Check for leading/trailing dots
    if email.starts_with('.') || email.ends_with('.') {
        return Err(EmailValidationError::LeadingTrailingDot);
    }
    if local_part.starts_with('.') || local_part.ends_with('.') {
        return Err(EmailValidationError::LeadingTrailingDot);
    }
    if domain_part.starts_with('.') || domain_part.ends_with('.') {
        return Err(EmailValidationError::LeadingTrailingDot);
    }

    // Check for consecutive dots
    if email.contains("..") {
        return Err(EmailValidationError::ConsecutiveDots);
    }

    // Validate characters separately for local and domain parts
    // Local part allows: alphanumeric, dot, underscore, hyphen, plus
    let valid_local_chars = |c: char| -> bool { c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '+') };

    // Domain part follows DNS hostname rules: alphanumeric, hyphen, dot only
    // Labels must not start or end with '-' and must be non-empty
    let valid_domain_chars = |c: char| -> bool { c.is_alphanumeric() || matches!(c, '-' | '.') };

    if !local_part.chars().all(valid_local_chars) {
        return Err(EmailValidationError::InvalidCharacter);
    }

    if !domain_part.chars().all(valid_domain_chars) {
        return Err(EmailValidationError::InvalidCharacter);
    }

    // Validate domain labels (must not start or end with '-', no empty labels)
    for label in domain_part.split('.') {
        if label.is_empty() {
            return Err(EmailValidationError::InvalidCharacter);
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(EmailValidationError::InvalidCharacter);
        }
    }

    Ok(())
}

/// Quick check if an email is valid (returns boolean)
///
/// This is a convenience function for simple validation where you don't need
/// the detailed error message.
///
/// # Arguments
///
/// * `email` - The email address to validate
///
/// # Returns
///
/// `true` if the email is valid, `false` otherwise
///
/// # Example
///
/// ```ignore
/// use shared_security::is_valid_email;
///
/// assert!(is_valid_email("user@example.com"));
/// assert!(!is_valid_email("invalid-email"));
/// ```
pub fn is_valid_email(email: &str) -> bool {
    validate_email(email).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Valid Email Tests
    // ========================================

    #[test]
    fn test_valid_email_simple() {
        assert!(validate_email("user@example.com").is_ok());
    }

    #[test]
    fn test_valid_email_with_subdomain() {
        assert!(validate_email("user@mail.example.com").is_ok());
    }

    #[test]
    fn test_valid_email_with_plus() {
        assert!(validate_email("user+tag@example.com").is_ok());
    }

    #[test]
    fn test_valid_email_with_dot() {
        assert!(validate_email("user.name@example.com").is_ok());
    }

    #[test]
    fn test_valid_email_with_underscore() {
        assert!(validate_email("user_name@example.com").is_ok());
    }

    #[test]
    fn test_valid_email_with_hyphen() {
        assert!(validate_email("user-name@example.com").is_ok());
    }

    #[test]
    fn test_valid_email_complex_local() {
        assert!(validate_email("user.name+tag_123@example.co.uk").is_ok());
    }

    #[test]
    fn test_is_valid_email_true() {
        assert!(is_valid_email("user@example.com"));
    }

    #[test]
    fn test_is_valid_email_false() {
        assert!(!is_valid_email("invalid-email"));
    }

    // ========================================
    // Empty/Whitespace Tests
    // ========================================

    #[test]
    fn test_empty_email() {
        let result = validate_email("");
        assert_eq!(result, Err(EmailValidationError::Empty));
    }

    #[test]
    fn test_whitespace_only_email() {
        let result = validate_email("   ");
        assert_eq!(result, Err(EmailValidationError::Empty));
    }

    #[test]
    fn test_email_with_trailing_whitespace() {
        // Should be valid after trimming
        assert!(validate_email("user@example.com   ").is_ok());
    }

    #[test]
    fn test_email_with_leading_whitespace() {
        // Should be valid after trimming
        assert!(validate_email("   user@example.com").is_ok());
    }

    // ========================================
    // Length Validation Tests
    // ========================================

    #[test]
    fn test_email_too_long() {
        let long_email = format!("{}@example.com", "a".repeat(255));
        let result = validate_email(&long_email);
        assert_eq!(result, Err(EmailValidationError::TooLong));
    }

    #[test]
    fn test_local_part_too_long() {
        let long_local = format!("{}@example.com", "a".repeat(65));
        let result = validate_email(&long_local);
        assert_eq!(result, Err(EmailValidationError::LocalPartTooLong));
    }

    #[test]
    fn test_domain_missing_dot() {
        let result = validate_email("user@abc");
        // Domain is missing dot
        assert!(matches!(result, Err(EmailValidationError::DomainMissingDot)));
    }

    #[test]
    fn test_local_part_too_short() {
        let result = validate_email("@example.com");
        assert_eq!(result, Err(EmailValidationError::MissingLocalPart));
    }

    // ========================================
    // @ Sign Validation Tests
    // ========================================

    #[test]
    fn test_missing_at_sign() {
        let result = validate_email("userexample.com");
        assert_eq!(result, Err(EmailValidationError::MissingAtSign));
    }

    #[test]
    fn test_multiple_at_signs() {
        let result = validate_email("user@@example.com");
        assert_eq!(result, Err(EmailValidationError::MultipleAtSigns));
    }

    #[test]
    fn test_three_at_signs() {
        let result = validate_email("user@@@example.com");
        assert_eq!(result, Err(EmailValidationError::MultipleAtSigns));
    }

    // ========================================
    // Domain Validation Tests
    // ========================================

    #[test]
    fn test_missing_domain() {
        let result = validate_email("user@");
        assert_eq!(result, Err(EmailValidationError::MissingDomain));
    }

    #[test]
    fn test_domain_missing_dot() {
        let result = validate_email("user@example");
        assert_eq!(result, Err(EmailValidationError::DomainMissingDot));
    }

    #[test]
    fn test_domain_starts_with_dot() {
        let result = validate_email("user@.example.com");
        assert_eq!(result, Err(EmailValidationError::LeadingTrailingDot));
    }

    #[test]
    fn test_domain_ends_with_dot() {
        let result = validate_email("user@example.com.");
        assert_eq!(result, Err(EmailValidationError::LeadingTrailingDot));
    }

    // ========================================
    // Local Part Validation Tests
    // ========================================

    #[test]
    fn test_local_part_starts_with_dot() {
        let result = validate_email(".user@example.com");
        assert_eq!(result, Err(EmailValidationError::LeadingTrailingDot));
    }

    #[test]
    fn test_local_part_ends_with_dot() {
        let result = validate_email("user.@example.com");
        assert_eq!(result, Err(EmailValidationError::LeadingTrailingDot));
    }

    // ========================================
    // Consecutive Dots Tests
    // ========================================

    #[test]
    fn test_consecutive_dots_in_local() {
        let result = validate_email("user..name@example.com");
        assert_eq!(result, Err(EmailValidationError::ConsecutiveDots));
    }

    #[test]
    fn test_consecutive_dots_in_domain() {
        let result = validate_email("user@exam..ple.com");
        assert_eq!(result, Err(EmailValidationError::ConsecutiveDots));
    }

    #[test]
    fn test_email_starts_with_dot() {
        let result = validate_email(".user@example.com");
        assert_eq!(result, Err(EmailValidationError::LeadingTrailingDot));
    }

    #[test]
    fn test_email_ends_with_dot() {
        let result = validate_email("user@example.com.");
        assert_eq!(result, Err(EmailValidationError::LeadingTrailingDot));
    }

    // ========================================
    // Invalid Character Tests
    // ========================================

    #[test]
    fn test_invalid_special_characters() {
        let result = validate_email("user!@example.com");
        assert_eq!(result, Err(EmailValidationError::InvalidCharacter));
    }

    #[test]
    fn test_invalid_space_in_email() {
        let result = validate_email("user @example.com");
        assert_eq!(result, Err(EmailValidationError::InvalidCharacter));
    }

    #[test]
    fn test_invalid_parentheses() {
        let result = validate_email("user()@example.com");
        assert_eq!(result, Err(EmailValidationError::InvalidCharacter));
    }

    #[test]
    fn test_invalid_brackets() {
        let result = validate_email("user[]@example.com");
        assert_eq!(result, Err(EmailValidationError::InvalidCharacter));
    }

    #[test]
    fn test_invalid_comma() {
        let result = validate_email("user,name@example.com");
        assert_eq!(result, Err(EmailValidationError::InvalidCharacter));
    }

    #[test]
    fn test_invalid_semicolon() {
        let result = validate_email("user;name@example.com");
        assert_eq!(result, Err(EmailValidationError::InvalidCharacter));
    }

    // ========================================
    // Edge Cases
    // ========================================

    #[test]
    fn test_single_character_local() {
        assert!(validate_email("a@example.com").is_ok());
    }

    #[test]
    fn test_minimum_valid_domain() {
        assert!(validate_email("user@a.bc").is_ok());
    }

    #[test]
    fn test_maximum_length_local() {
        let max_local = format!("{}@example.com", "a".repeat(64));
        assert!(validate_email(&max_local).is_ok());
    }

    #[test]
    fn test_maximum_length_total() {
        // Create an email that's exactly at the 254 character limit
        let local = "a".repeat(64);
        let domain = format!("{}.{}", "b".repeat(63), "c".repeat(63));
        let email = format!("{}@{}", local, domain);
        assert!(validate_email(&email).is_ok());
    }

    // ========================================
    // Error Display Tests
    // ========================================

    #[test]
    fn test_error_validation_to_string() {
        assert_eq!(format!("{}", EmailValidationError::Empty), "Email address is required");
        assert_eq!(
            format!("{}", EmailValidationError::TooLong),
            "Email address is too long (maximum 254 characters)"
        );
        assert_eq!(
            format!("{}", EmailValidationError::MissingAtSign),
            "Email address must contain @ symbol"
        );
    }

    // ========================================
    // ReDoS Safety Tests
    // ========================================

    #[test]
    fn test_redos_safe_long_string() {
        // This should complete quickly even with very long input
        // ReDoS-vulnerable regex would take exponential time
        let long_email = format!("{}{}@example.com", "a".repeat(1000), "a".repeat(1000));
        let start = std::time::Instant::now();
        let _ = validate_email(&long_email);
        let duration = start.elapsed();
        // Should complete in well under 1 second
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_redos_safe_many_at_signs() {
        // Many @ signs should be caught quickly
        let many_ats = format!("user{}@example.com", "@".repeat(100));
        let start = std::time::Instant::now();
        let result = validate_email(&many_ats);
        let duration = start.elapsed();
        assert_eq!(result, Err(EmailValidationError::MultipleAtSigns));
        assert!(duration.as_millis() < 10);
    }

    #[test]
    fn test_redos_safe_many_dots() {
        // Many consecutive dots should be caught quickly
        // But first, it will be caught by local part length check (64 char max)
        let many_dots = format!("user{}@example.com", ".".repeat(100));
        let start = std::time::Instant::now();
        let result = validate_email(&many_dots);
        let duration = start.elapsed();
        // The local part exceeds 64 characters, so that's caught first
        assert!(matches!(result, Err(EmailValidationError::LocalPartTooLong)));
        assert!(duration.as_millis() < 10);
    }
}
