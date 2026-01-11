use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::{ready, Ready};
use std::task::{Context, Poll};
use std::sync::Arc;

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
        let auth_header = req.headers().get("authorization").cloned();
        let fut = self.service.call(req);

        let auth_user = auth_header.and_then(|header_value| {
            header_value.to_str().ok().and_then(|token_str| {
                if token_str.starts_with("Bearer ") {
                    let token = &token_str[7..];
                    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
                    let validation = Validation::default();
                    
                    decode::<serde_json::Value>(token, &decoding_key, &validation).ok().map(|token_data| {
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
                        
                        AuthUser { user_id, email, role }
                    })
                } else {
                    None
                }
            })
        });

        Box::pin(async move {
            let res = fut.await?;
            
            if let Some(user) = auth_user {
                res.request().extensions_mut().insert(user);
            }
            
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

pub fn get_auth_user(req: &ServiceRequest) -> Option<AuthUser> {
    req.extensions().get::<AuthUser>().cloned()
}
