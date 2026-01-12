use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use thiserror::Error;
use lazy_static::lazy_static;
use shared_errors::AppError;
use shared_models::entities::User;
use crate::models::{LoginRequest, LoginResponse, UserResponse};
use crate::repository::AuthRepository;
use crate::jwt::JwtService;
use crate::password::{hash_password, verify_password};

#[actix_web::post("/login")]
async fn login(
    req: web::Json<crate::models::LoginRequest>,
    repo: web::Data<AuthRepository>,
    jwt_service: web::Data<JwtService>,
) -> impl Responder {
    // Validate request
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }

    // Find user by email
    let user_opt = repo.find_by_email(&req.email).await;
    let user = match user_opt {
        Ok(Some(u)) => u,
        Ok(None) => {} Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
        }
    };

    // Verify password
    if !verify_password(&req.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": "Invalid email or password" }));
    }

    // Check email verification status
    if !user.is_email_verified {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": "Please verify your email address before logging in" }));
    }

    // Generate tokens
    let access_token = match jwt_service.generate_access_token(
        &user.id.to_string(),
        &user.email,
        "user"
    ) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "INTERNAL_ERROR", "message": e.to_string() }));
        }
    };

    let refresh_token_hash = uuid::Uuid::new_v4().to_string();
    let refresh_token = format!("{}", refresh_token_hash);

    // Update last login
    match repo.update_last_login(&user.id).await {
        Ok(_) => {},
        Err(e) => {
            // Log error but don't fail login for this
            tracing::error!("Failed to update last login: {}", e);
        }
    }

    HttpResponse::Ok()
        .json(crate::models::LoginResponse {
            user: crate::models::UserResponse {
                id: user.id.to_string(),
                email: user.email.clone(),
                display_name: user.display_name.clone(),
                avatar_url: user.avatar_url.clone(),
                is_email_verified: user.is_email_verified,
            },
            access_token,
            refresh_token,
            expires_in: jwt_service.config.access_expiry,
        })
}
