use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;
use std::collections::HashSet;

pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    hash(password, DEFAULT_COST).map_err(|e| PasswordError::HashError(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    verify(password, hash).map_err(|e| PasswordError::VerifyError(e.to_string()))
}

pub fn validate_password_strength(password: &str) -> Result<(), PasswordError> {
    let mut errors = Vec::new();
    
    if password.len() < 8 {
        errors.push("Password must be at least 8 characters");
    }
    
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain at least one uppercase letter");
    }
    
    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("Password must contain at least one lowercase letter");
    }
    
    if !password.chars().any(|c| c.is_digit(10)) {
        errors.push("Password must contain at least one number");
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(PasswordError::WeakPassword(errors.join(", ")))
    }
}

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

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password hashing error: {0}")]
    HashError(String),
    #[error("Password verification error: {0}")]
    VerifyError(String),
    #[error("Password does not meet requirements: {0}")]
    WeakPassword(String),
}
