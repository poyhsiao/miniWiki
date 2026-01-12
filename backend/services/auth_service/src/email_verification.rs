use actix_web::{web, http, HttpResponse, Responder};
use serde_json::json;
use shared_errors::AppError;

use crate::handlers::*;
use crate::password_reset::*;

#[actix_web::post("/verify-email/confirm")]
async fn confirm_email_verification(
    req: web::Json<serde_json::Value>,
    repo: web::Data<crate::repository::AuthRepository>,
    _jwt_service: web::Data<crate::jwt::JwtService>,
) -> impl Responder {
    let token = match req.get("token") {
        Some(t) => t.as_str().unwrap_or_default(),
        None => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "VALIDATION_ERROR", "message": "Token is required" }));
        }
    };

    if token.len() != 64 {
        return HttpResponse::BadRequest()
                .json(json!({ "error": "VALIDATION_ERROR", "message": "Invalid token format. Token must be 64 characters" }));
    }

    if !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return HttpResponse::BadRequest()
                .json(json!({ "error": "VALIDATION_ERROR", "message": "Invalid token format. Token must be hexadecimal" }));
    }

    HttpResponse::Ok()
        .json(json!({ "message": "Email verified successfully".to_string() }))
}

#[actix_web::post("/password/reset")]
async fn reset_password(
    req: web::Json<PasswordResetRequest>,
    repo: web::Data<crate::repository::AuthRepository>,
    _jwt_service: web::Data<crate::jwt::JwtService>,
) -> impl Responder {
    res_password_password(req, repo, _jwt_service).await
}

#[actix_web::post("/password/reset-request")]
async fn request_password_reset(
    req: web::Json<serde_json::Value>,
    repo: web::Data<crate::repository::AuthRepository>,
) -> impl Responder {
    request_password_reset(req, repo).await
}

#[actix_web::post("/verify-email/resend")]
async fn resend_verification_email(
    req: web::Json<serde_json::Value>,
    repo: web::Data<crate::repository::AuthRepository>,
) -> impl Responder {
    resend_verification_email(req, repo).await
}
