use actix_web::{web, Responder, HttpResponse};
use crate::models::{RegisterRequest, LoginRequest, RegisterResponse, LoginResponse, RefreshRequest, RefreshResponse, LogoutRequest};
use crate::jwt::JwtService;
use crate::password::{hash_password, verify_password, validate_password_strength};
use crate::repository::AuthRepository;
use shared_models::entities::RefreshToken;

pub async fn register(
    req: web::Json<RegisterRequest>,
    repo: web::Data<AuthRepository>,
    _jwt_service: web::Data<JwtService>,
) -> impl Responder {
    // Hash password and create user
    let password_hash = match hash_password(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "VALIDATION_ERROR", "message": e }));
        }
    };

    // Check if user already exists
    match repo.find_by_email(&req.email).await {
        Ok(Some(_)) => {
            return HttpResponse::Conflict()
                .json(serde_json::json!({ "error": "CONFLICT", "message": "Email already registered" }));
        }
        Ok(None) => {}
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
        }
    }

    // Validate password strength
    match validate_password_strength(&req.password) {
        Ok(()) => {}
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "VALIDATION_ERROR", "message": e }));
        }
    }

    // Create user
    let user = match repo.create(&req.email, &password_hash, &req.display_name).await {
        Ok(user) => user,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
        }
    };

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
    match verify_password(&req.password, &user.password_hash) {
        Ok(true) => {
            // Password is correct, continue to token generation
        }
        Ok(false) => {
            // Password is incorrect
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": "Invalid email or password" }));
        }
        Err(e) => {
            // Internal error during password verification (e.g., corrupted hash)
            // Mask PII before logging
            let masked_id = {
                let id_str = user.id.to_string();
                if id_str.chars().count() > 8 {
                    format!("{}...", id_str.chars().take(8).collect::<String>())
                } else {
                    "***".to_string()
                }
            };

            let masked_email = {
                let parts: Vec<&str> = user.email.split('@').collect();
                if parts.len() == 2 {
                    let name = parts[0];
                    let domain = parts[1];
                    let name_len = name.chars().count();
                    let visible_len = std::cmp::min(2, name_len);
                    let visible_part: String = name.chars().take(visible_len).collect();
                    format!("{}***@{}", visible_part, domain)
                } else {
                    "***@***.***".to_string()
                }
            };

            tracing::error!(
                "Password verification failed for user {} ({}): verify_password error: {}",
                masked_id,
                masked_email,
                e
            );
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "INTERNAL_ERROR", "message": "Authentication system error" }));
        }
    }

    // Generate tokens
    let access_token = match jwt_service.generate_access_token(
        &user.id.to_string(),
        &user.email,
        "user",
    ) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "INTERNAL_ERROR", "message": e.to_string() }));
        }
    };

    let refresh_token = match jwt_service.generate_refresh_token(&user.id.to_string()) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "INTERNAL_ERROR", "message": e.to_string() }));
        }
    };

    // Store refresh token in database
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(jwt_service.config.refresh_expiry);
    let refresh_token_record = RefreshToken {
        id: uuid::Uuid::new_v4(),
        user_id: user.id,
        token: refresh_token.clone(),
        expires_at: expires_at.naive_utc(),
        ip_address: None,
        user_agent: None,
        is_revoked: false,
        revoked_at: None,
        created_at: chrono::Utc::now().naive_utc(),
    };

    if let Err(e) = repo.create_refresh_token(&refresh_token_record).await {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
    }

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

pub async fn logout(
    req: web::Json<LogoutRequest>,
    repo: web::Data<AuthRepository>,
) -> impl Responder {
    if let Some(refresh_token) = &req.refresh_token {
        if let Err(e) = repo.revoke_refresh_token(refresh_token).await {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
        }
    }

    HttpResponse::Ok()
        .json(serde_json::json!({ "message": "Logged out successfully" }))
}

pub async fn refresh(
    req: web::Json<RefreshRequest>,
    jwt_service: web::Data<JwtService>,
    repo: web::Data<AuthRepository>,
) -> impl Responder {
    let claims = match jwt_service.validate_token(&req.refresh_token) {
        Ok(claims) => claims,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": e.to_string() }));
        }
    };

    // Check if refresh token exists and is not revoked
    match repo.find_refresh_token(&req.refresh_token).await {
        Ok(Some(_)) => {
            // Token is valid and not revoked
        }
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": "Refresh token is invalid or has been revoked" }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "DATABASE_ERROR", "message": e.to_string() }));
        }
    }

    let new_access_token = match jwt_service.generate_access_token(
        &claims.user_id,
        &claims.email,
        &claims.role,
    ) {
        Ok(token) => token,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "INTERNAL_ERROR", "message": e.to_string() }));
        }
    };

    HttpResponse::Ok()
        .json(RefreshResponse {
            access_token: new_access_token,
            expires_in: jwt_service.config.access_expiry,
        })
}

pub async fn me(
    req: actix_web::HttpRequest,
    jwt_service: web::Data<JwtService>,
) -> impl Responder {
    let auth_header = match req.headers().get("authorization") {
        Some(h) if h.to_str().ok().map(|s| s.starts_with("Bearer ")).unwrap_or(false) => {
            &h.to_str().unwrap()[7..]
        }
        _ => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": "Missing or invalid authorization header" }));
        }
    };

    let claims = match jwt_service.validate_token(auth_header) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "AUTHENTICATION_ERROR", "message": e.to_string() }));
        }
    };

    HttpResponse::Ok()
        .json(serde_json::json!({
            "id": claims.user_id,
            "email": claims.email,
            "role": claims.role
        }))
}
