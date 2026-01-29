//! Shared security utilities for miniWiki backend
//!
//! This crate provides common security-related functionality including:
//! - Password hashing and verification
//! - Password strength validation
//! - Secure token generation
//! - Email validation (ReDoS-safe)

pub mod password;
pub mod email;

pub use password::{
    hash_password,
    hash_password_with_cost,
    verify_password,
    validate_password_strength,
    validate_password_strength_with_requirements,
    generate_reset_token,
    generate_url_safe_token,
    PasswordError,
    PasswordRequirements,
    PasswordValidationError,
    DEFAULT_BCRYPT_COST,
};

pub use email::{
    validate_email,
    EmailValidationError,
    is_valid_email,
};
