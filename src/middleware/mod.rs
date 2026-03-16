use actix_limitation::Limiter;
use actix_web::dev::ServiceRequest;
use std::time::Duration;

pub mod cache;
pub mod image;
pub mod limiter;
pub mod panic_handler;

pub fn configure_rate_limiter(redis_url: &str) -> Result<Limiter, String> {
    let limiter = Limiter::builder(redis_url)
        .key_by(|req: &ServiceRequest| {
            // Key by IP address and path
            let ip = req
                .peer_addr()
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let path = req.uri().path().to_string();
            Some(format!("{}:{}", ip, path))
        })
        .limit(100) // 100 requests
        .period(Duration::from_secs(60)) // per minute
        .build()
        .map_err(|e| format!("Failed to build rate limiter: {}", e))?;

    Ok(limiter)
}
