use serde::{Deserialize, Serialize};
use validator::Validate;

lazy_static::lazy_static! {
    static ref PASSWORD_REGEX: regex::Regex = regex::Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$").unwrap();
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, max = 100, code = "Password must be 8-100 characters"))]
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
