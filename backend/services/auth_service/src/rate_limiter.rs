use actix_web::{
    dev::Payload, Error, FromRequest, HttpRequest, HttpResponse,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration,
        }
    }

    pub fn check_rate_limit(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        let entry = requests.entry(key.to_string()).or_insert_with(Vec::new);

        entry.retain(|&timestamp| now.duration_since(timestamp) < self.window_duration);

        if entry.len() < self.max_requests {
            entry.push(now);
            true
        } else {
            false
        }
    }
}

impl FromRequest for RateLimiter {
    type Error = Error;
    type Future = futures::future::Ready<Result<Self, Error>>;

    fn from_request(_req: &HttpRequest, _: &mut Payload) -> Self::Future {
        futures::future::ok(Self::new(100, Duration::from_secs(60)))
    }
}

pub fn rate_limit_middleware(
    rate_limiter: web::Data<RateLimiter>,
) -> impl actix_web::dev::Transform<
    actix_web::service::ServiceResponse,
    actix_web::service::ServiceRequest,
    actix_web::dev::Body,
> + Clone {
    actix_web::middleware::from_fn(move |req: ServiceRequest, next: Next| {
        let rate_limiter = rate_limiter.clone();
        async move {
            let ip = req
                .peer_addr()
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            if !rate_limiter.check_rate_limit(&ip) {
                return Ok(req.into_response(
                    HttpResponse::TooManyRequests()
                        .json(serde_json::json!({
                            "error": "RATE_LIMIT_EXCEEDED",
                            "message": "Too many requests. Please try again later.",
                            "phase": "RED",
                            "todo": [
                                "Implement distributed rate limiting using Redis",
                                "Add per-user rate limiting",
                                "Add configurable limits per endpoint"
                            ]
                        })),
                ));
            }

            next.call(req).await
        }
    })
}

#[actix_web::rt::test]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));

        assert!(limiter.check_rate_limit("test_ip"));
        assert!(limiter.check_rate_limit("test_ip"));
        assert!(limiter.check_rate_limit("test_ip"));
        assert!(!limiter.check_rate_limit("test_ip"));
    }

    #[test]
    fn test_rate_limiter_different_keys() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        assert!(limiter.check_rate_limit("ip1"));
        assert!(limiter.check_rate_limit("ip1"));
        assert!(!limiter.check_rate_limit("ip1"));

        assert!(limiter.check_rate_limit("ip2"));
        assert!(limiter.check_rate_limit("ip2"));
        assert!(!limiter.check_rate_limit("ip2"));
    }
}
