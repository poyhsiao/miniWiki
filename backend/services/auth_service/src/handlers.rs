use actix_web::{web, Responder, HttpResponse};
use serde::Deserialize;
use crate::models::{RegisterRequest, LoginRequest, RegisterResponse, LoginResponse, RefreshRequest, RefreshResponse};
use crate::jwt::JwtService;
use crate::password::{hash_password, verify_password, validate_password_strength};
use shared_errors::AppError;
use shared_models::entities::User;

#[derive(Deserialize)]
pub struct AuthRepository {
    // This would be injected from the app data
}

impl AuthRepository {
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        // Implementation would query the database
        Ok(None)
    }
    
    pub async fn create(&self, email: &str, password_hash: &str, display_name: &str) -> Result<User, sqlx::Error> {
        // Implementation would insert into database
        Ok(User {
            id: uuid::Uuid::new_v4(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            display_name: display_name.to_string(),
            avatar_url: None,
            timezone: "UTC".to_string(),
            language: "en".to_string(),
            is_active: true,
            is_email_verified: false,
            email_verified_at: None,
            last_login_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    
    pub async fn update_last_login(&self, user_id: &uuid::Uuid) -> Result<(), sqlx::Error> {
        Ok(())
    }
}

pub async fn register(
    req: web::Json<RegisterRequest>,
    repo: web::Data<AuthRepository>,
    jwt_service: web::Data<JwtService>,
) -> impl Responder {
    // Validate request
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }
    
    // Check if user already exists
    if let Ok(Some(_)) = repo.find_by_email(&req.email).await {
        return HttpResponse::Conflict()
            .json(serde_json::json!({ "error": "CONFLICT", "message": "Email already registered" }));
    }
    
    // Validate password strength
    if let Err(e) = validate_password_strength(&req.password) {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "VALIDATION_ERROR", "message": e.to_string() }));
    }
    
    // Hash password and create user
    let password_hash = hash_password(&req.password)
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    let user = repo.create(&req.email, &password_hash, &req.display_name).await
        .map_err(|e| AppError::DatabaseError(e))?;
    
    HttpResponse::Created()
        .json(RegisterResponse {
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

pub async fn login(
    req: web::Json<LoginRequest>,
    repo: web::Data<AuthRepository>,
    jwt_service: web::Data<JwtService>,
) -> impl Responder {
    // Find user by email
    let user = match repo.find_by_email(&req.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": "Invalid email or password" }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
        }
    };
    
    // Verify password
    if !verify_password(&req.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": "Invalid email or password" }));
    }
    
    // Generate tokens
    let access_token = jwt_service.generate_access_token(
        &user.id.to_string(),
        &user.email,
        "user",
    ).map_err(|e| AppError::InternalError(e.to_string()))?;
    
    let refresh_token = jwt_service.generate_refresh_token(&user.id.to_string())
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    
    // Update last login
    repo.update_last_login(&user.id).await.ok();
    
    HttpResponse::Ok()
        .json(LoginResponse {
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

pub async fn logout() -> impl Responder {
    HttpResponse::Ok()
        .json(serde_json::json!({ "message": "Logged out successfully" }))
}

pub async fn refresh(
    req: web::Json<RefreshRequest>,
    jwt_service: web::Data<JwtService>,
) -> impl Responder {
    let claims = jwt_service.validate_token(&req.refresh_token)
        .map_err(|e| AppError::AuthenticationError(e.to_string()))?;
    
    let new_access_token = jwt_service.generate_access_token(
        &claims.user_id,
        &claims.email,
        &claims.role,
    ).map_err(|e| AppError::InternalError(e.to_string()))?;
    
    HttpResponse::Ok()
        .json(RefreshResponse {
            access_token: new_access_token,
            expires_in: jwt_service.config.access_expiry,
        })
}

pub async fn me() -> impl Responder {
    HttpResponse::Ok()
        .json(serde_json::json!({ "message": "Current user endpoint" }))
}
