use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared_errors::AppError;

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Refresh access token using refresh token
#[actix_web::post("/auth/refresh")]
async fn refresh_token(
    req: web::Json<RefreshTokenRequest>,
    _jwt_service: web::Data<crate::jwt::JwtService>,
    _repo: web::Data<crate::repository::AuthRepository>,
) -> impl Responder {
    if req.refresh_token.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({ "error": "VALIDATION_ERROR", "message": "Refresh token is required" }));
    }

    // ... existing code ...
    HttpResponse::Ok()
        .json(json!({
            "message": "Token refreshed successfully",
            "phase": "RED",
            "todo": [
                "Validate refresh token from database",
                "Check if token is revoked",
                "Generate new access token",
                "Optionally rotate refresh token",
                "Update token timestamps in database"
            ]
        }))
}

/// Revoke refresh token
#[actix_web::post("/auth/revoke")]
async fn revoke_token(
    req: web::Json<serde_json::Value>,
    _jwt_service: web::Data<crate::jwt::JwtService>,
    _repo: web::Data<crate::repository::AuthRepository>,
) -> impl Responder {
    let token = match req.get("refresh_token") {
        Some(t) => t.as_str().unwrap_or_default(),
        None => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "VALIDATION_ERROR", "message": "Refresh token is required" }));
        }
    };

    if token.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({ "error": "VALIDATION_ERROR", "message": "Refresh token is required" }));
    }

    // ... existing code ...
    HttpResponse::Ok()
        .json(json!({
            "message": "Token revoked successfully",
            "phase": "RED",
            "todo": [
                "Validate refresh token exists in database",
                "Mark token as revoked",
                "Optionally revoke all tokens for the user"
            ]
        }))
}
