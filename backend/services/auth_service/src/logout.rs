use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use shared_errors::AppError;
use shared_models::entities::User;
use crate::repository::AuthRepository;

#[actix_web::post("/logout")]
async fn logout(
    req: web::HttpRequest,
    repo: web::Data<AuthRepository>,
    jwt_service: web::Data<crate::jwt::JwtService>,
) -> impl Responder {
    // For now, just return success
    // RED phase: Will need to add token revocation when refresh token support is implemented
    
    HttpResponse::Ok()
        .json(serde_json::json!({
            "message": "Logged out successfully"
        }))
}
