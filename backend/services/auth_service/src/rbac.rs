use actix_web::{HttpMessage, HttpRequest, HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use jsonwebtoken::TokenData;
use thiserror::Error;
use lazy_static::lazy_static;

use crate::jwt::Claims;
use crate::permissions::{Role, Permission, ActionType};

lazy_static! {
    static ref JWT_SECRET: String = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    static ref JWT_DECODING_KEY: jsonwebtoken::DecodingKey =
        jsonwebtoken::DecodingKey::from_secret(JWT_SECRET.as_bytes());
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Unauthorized(_) => {
                HttpResponse::build(StatusCode::UNAUTHORIZED).body("Unauthorized")
            }
            Error::Forbidden(_) => {
                HttpResponse::build(StatusCode::FORBIDDEN).body("Forbidden")
            }
            Error::InternalServerError(_) => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("Internal server error")
            }
        }
    }
}



/// RBAC middleware for checking user permissions
///
/// This middleware validates JWT tokens, extracts user roles,
/// and enforces permission-based access control.
pub struct RbacMiddleware;

impl RbacMiddleware {
    pub fn new() -> Self {
        Self
    }

    /// Extracts and validates JWT token from request
    fn extract_claims(req: &HttpRequest) -> Result<Claims, Error> {
        // Get Authorization header
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        if let Some(token_str) = auth_header {
            let secret = std::env::var("JWT_SECRET")
                .map_err(|_| Error::InternalServerError("JWT_SECRET not configured".to_string()))?;

            let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
            let token_data: jsonwebtoken::TokenData<Claims> = jsonwebtoken::decode(
                token_str,
                &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
                &validation,
            )
            .map_err(|e| Error::Unauthorized(format!("Invalid token: {}", e)))?;

            Ok(token_data.claims)
        } else {
            Err(Error::Unauthorized("Missing authorization header".to_string()))
        }
    }

    /// Checks if a user has a specific permission
    pub fn has_permission(role: &str, permission: &Permission) -> bool {
        if let Some(parsed_role) = Role::from_str(role) {
            parsed_role.has_permission(permission)
        } else {
            false
        }
    }

    /// Checks if a user can perform a specific action
    pub fn can_perform_action(role: &str, action: &ActionType) -> bool {
        if let Some(parsed_role) = Role::from_str(role) {
            parsed_role.can_perform_action(action)
        } else {
            false
        }
    }

    /// Extract user role from claims
    pub fn extract_role(claims: &Claims) -> Option<String> {
        Some(claims.role.clone())
    }

    /// Verify user is a member of a specific space
    ///
    /// This would typically check space_memberships table
    /// For now, we'll implement a simple check
    /// 
    /// IMPORTANT: This is a stub implementation that always returns false
    /// to enforce authorization until actual DB-backed membership checking is implemented.
    /// Once space_memberships table is available, this should query it
    /// to verify that Claims::user_id is actually a member of the space.
    pub fn is_space_member(claims: &Claims, space_id: &str) -> bool {
        // TODO: Implement actual space membership check
        // This should query space_memberships table
        // SECURITY: Return false by default to prevent auth bypass
        // until real implementation is added
        false
    }
}

 /// Actix-web guard for permission checking
 ///
 /// Usage: 
 /// ```rust,ignore
 /// use actix_web::{get, web};
 /// use crate::rbac::{RbacMiddleware, check_permission};
 /// use crate::permissions::ActionType;
 ///
 /// #[get("/documents/{id}")]
 /// async fn get_document(
 ///     req: web::HttpRequest,
 ///     data: web::Path<(String,)>,
 /// ) -> Result<HttpResponse, Error> {
 ///     check_permission(&req, ActionType::ViewDocument)?;
 ///     // ... rest of handler
 /// }
 /// ```
 pub fn check_permission(
     req: &HttpRequest,
     action: ActionType,
 ) -> Result<(), Error> {
    // Extract and validate claims
    let claims = RbacMiddleware::extract_claims(req)?;
    
    // Check if user can perform the action
    let role = RbacMiddleware::extract_role(&claims)
            .ok_or_else(|| Error::InternalServerError("Role not found in claims".to_string()))?;

    if !RbacMiddleware::can_perform_action(&role, &action) {
        return Err(Error::Forbidden(format!(
            "Insufficient permissions to perform {:?}",
            action
        )));
    }

    Ok(())
}

/// Actix-web guard for role-based access control
    ///
    /// Usage:
    /// ```rust,ignore
    /// use actix_web::{get, web, HttpResponse};
    /// use crate::rbac::check_role;
    /// use crate::permissions::Role;
    ///
    /// #[get("/admin")]
    /// async fn admin_only(
    ///     req: web::HttpRequest,
    /// ) -> Result<HttpResponse, Error> {
    ///     check_role(&req, Role::Owner)?;
    ///     // ... rest of handler
    /// }
    /// ```
pub fn check_role(
    req: &HttpRequest,
    required_role: Role,
) -> Result<(), Error> {
    // Extract and validate claims
    let claims = RbacMiddleware::extract_claims(req)?;
    
    // Check user's role
    let role_str = RbacMiddleware::extract_role(&claims)
            .ok_or_else(|| Error::InternalServerError("Role not found in claims".to_string()))?;

    if let Some(user_role) = Role::from_str(&role_str) {
        if user_role.level() < required_role.level() {
            return Err(Error::Forbidden(format!(
                "Insufficient privileges. Required role: {:?}",
                required_role
            )));
        }
    } else {
        return Err(Error::InternalServerError(
            "Invalid role format in token".to_string(),
        ));
    }

    Ok(())
}

/// Extracts user claims from request
pub fn get_claims(req: &HttpRequest) -> Result<Claims, Error> {
    RbacMiddleware::extract_claims(req)
}

/// Gets user ID from request
pub fn get_user_id(req: &HttpRequest) -> Result<String, Error> {
    let claims = get_claims(req)?;
    Ok(claims.user_id)
}

/// Gets user role from request
pub fn get_user_role(req: &HttpRequest) -> Result<Role, Error> {
    let claims = get_claims(req)?;
    let role_str = RbacMiddleware::extract_role(&claims)
            .ok_or_else(|| Error::InternalServerError("Role not found in claims".to_string()))?;

            Role::from_str(&role_str)
                .ok_or_else(|| Error::InternalServerError(format!("Invalid role: {}", role_str)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_permission() {
        // Owner has all permissions
        assert!(RbacMiddleware::has_permission("owner", &Permission::DeleteDocuments));
        assert!(RbacMiddleware::has_permission("editor", &Permission::ViewDocuments));
        assert!(!RbacMiddleware::has_permission("viewer", &Permission::EditDocuments));
    }

    #[test]
    fn test_can_perform_action() {
        // Owner can delete
        assert!(RbacMiddleware::can_perform_action(
            "owner",
            &ActionType::DeleteDocument,
        ));
        
        // Viewer cannot delete
        assert!(!RbacMiddleware::can_perform_action(
            "viewer",
            &ActionType::DeleteDocument,
        ));
    }

    #[test]
    fn test_extract_role() {
        assert_eq!(extract_role_from_string("owner"), Role::Owner);
        assert_eq!(extract_role_from_string("editor"), Role::Editor);
        assert_eq!(extract_role_from_string("viewer"), Role::Viewer);
    }

    // Helper function for tests (not exposed in main API)
    fn extract_role_from_string(role: &str) -> Role {
        Role::from_str(role).expect("Failed to parse role")
    }
}
