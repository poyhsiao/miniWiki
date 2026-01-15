use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{self, HeaderValue},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::Ready;

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            res.headers_mut().insert(
                header::STRICT_TRANSPORT_SECURITY,
                HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            );

            res.headers_mut().insert(
                header::X_FRAME_OPTIONS,
                HeaderValue::from_static("DENY"),
            );

            res.headers_mut().insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );

            res.headers_mut().insert(
                header::REFERRER_POLICY,
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            res.headers_mut().insert(
                header::PERMISSIONS_POLICY,
                HeaderValue::from_static("accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"),
            );

            if !res.headers().contains_key(header::CONTENT_SECURITY_POLICY) {
                res.headers_mut().insert(
                    header::CONTENT_SECURITY_POLICY,
                    HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self' https://api.miniwiki.local; frame-ancestors 'none'"),
                );
            }

            res.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
            );

            res.headers_mut().insert(
                header::PRAGMA,
                HeaderValue::from_static("no-cache"),
            );

            Ok(res)
        })
    }
}
