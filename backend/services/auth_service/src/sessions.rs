use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared_errors::AppError;
use shared_models::entities::User;

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
    pub last_active: String,
    pub is_current: bool,
}

/// Get all active sessions for the authenticated user
#[actix_web::get("/auth/sessions")]
async fn get_sessions(
    req: HttpRequest,
    _repo: web::Data<crate::repository::AuthRepository>,
) -> impl Responder {
    HttpResponse::Ok()
        .json(json!({
            "message": "Sessions retrieved successfully",
            "phase": "RED",
            "todo": [
                "Extract user from JWT token",
                "Query sessions from database",
                "Mark current session",
                "Return session list"
            ],
            "sessions": []
        }))
}

/// Delete a specific session
#[actix_web::delete("/auth/sessions/{session_id}")]
async fn delete_session(
    path: web::Path<String>,
    _repo: web::Data<crate::repository::AuthRepository>,
) -> impl Responder {
    let session_id = path.into_inner();

    if session_id.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({ "error": "VALIDATION_ERROR", "message": "Session ID is required" }));
    }

    HttpResponse::Ok()
        .json(json!({
            "message": "Session deleted successfully",
            "phase": "RED",
            "todo": [
                "Validate session exists and belongs to user",
                "Delete session from database",
                "Return success response"
            ]
        }))
}

/// Delete all sessions except the current one
#[actix_web::delete("/auth/sessions")]
async fn delete_all_sessions(
    req: HttpRequest,
    _repo: web::Data<crate::repository::AuthRepository>,
) -> impl Responder {
    HttpResponse::Ok()
        .json(json!({
            "message": "All sessions deleted successfully",
            "phase": "RED",
            "todo": [
                "Extract user from JWT token",
                "Get current session ID from token",
                "Delete all sessions except current",
                "Return count of deleted sessions"
            ],
            "deleted_count": 0
        }))
}
