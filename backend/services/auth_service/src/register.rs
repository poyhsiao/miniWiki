use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use thiserror::Error;
use lazy_static::lazy_static;
use shared_errors::AppError;
use shared_models::entities::User;
use crate::models::{RegisterRequest, RegisterResponse, UserResponse};
use crate::repository::AuthRepository;
use crate::jwt::JwtService;
use crate::password::{hash_password, validate_password_strength, verify_password};

#[actix_web::post("/register")]
async fn register(
    req: web::Json<crate::models::RegisterRequest>,
    repo: web::Data<AuthRepository>,
    jwt_service: web::Data<JwtService>,
) -> impl Responder {
    // Validate email format
    if !req.email.contains('@') || !req.email.contains('.') || req.email.len() < 5 || req.email.len() > 255 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "VALIDATION_ERROR", "message": "Invalid email format" }));
    }

    // Validate password strength
    if let Err(e) = validate_password_strength(&req.password) {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Check if user already exists
    let existing_user = repo.find_by_email(&req.email).await;
    if let Ok(Some(_)) = existing_user {
        return HttpResponse::Conflict()
            .json(serde_json::json!({ "error": "AUTH_EMAIL_EXISTS", "message": "Email is already registered" }));
    } else if let Err(e) = existing_user {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
    }

    // Validate password strength
    if let Err(e) = validate_password_strength(&req.password) {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Hash password
    let password_hash = match hash_password(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "INTERNAL_ERROR", "message": e.to_string() }));
        }
    };

    // Create user in database
    let user = match repo.create(&req.email, &password_hash, &req.display_name).await {
        Ok(user) => user,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
        }
    };

    HttpResponse::Created()
        .json(crate::models::RegisterResponse {
            user: crate::models::UserResponse {
                id: user.id.to_string(),
                email: user.email.clone(),
                display_name: user.display_name.clone(),
                avatar_url: user.avatar_url.clone(),
                is_email_verified: user.is_email_verified,
            },
            message: "Registration successful. Please check your email to verify your account.".to_string(),
        })
}
