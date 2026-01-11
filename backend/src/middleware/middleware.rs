use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation, TokenData};
use std::future::{ready, Ready};
use std::task::{Context, Poll};
use std::sync::Arc;

use shared_errors::error_types::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

pub struct JwtMiddleware<S> {
    service: S,
    jwt_secret: Arc<String>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt_secret = self.jwt_secret.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            // Extract token from Authorization header
            let auth_header = req.headers().get("authorization");
            
            if let Some(header_value) = auth_header {
                if let Ok(token_str) = header_value.to_str() {
                    if token_str.starts_with("Bearer ") {
                        let token = &token_str[7..];
                        
                        // Decode and validate JWT
                        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
                        let validation = Validation::default();
                        
                        match decode::<serde_json::Value>(token, &decoding_key, &validation) {
                            Ok(token_data) => {
                                // Extract user info from token claims
                                let claims = token_data.claims;
                                let user_id = claims.get("sub")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string();
                                let email = claims.get("email")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string();
                                let role = claims.get("role")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("user")
                                    .to_string();
                                
                                // Store user in request extensions
                                let auth_user = AuthUser {
                                    user_id,
                                    email,
                                    role,
                                };
                                req.extensions_mut().insert(auth_user);
                            }
                            Err(_) => {
                                // Invalid token, continue without auth user
                                // Could return 401 here if required
                            }
                        }
                    }
                }
            }

            let res = fut.await?;
            Ok(res)
        })
    }
}

pub struct JwtAuth {
    jwt_secret: Arc<String>,
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddleware {
            service,
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub fn require_auth(jwt_secret: &str) -> JwtAuth {
    JwtAuth {
        jwt_secret: Arc::new(jwt_secret.to_string()),
    }
}

/// Helper function to get auth user from request
pub fn get_auth_user(req: &ServiceRequest) -> Option<AuthUser> {
    req.extensions().get::<AuthUser>().cloned()
}
