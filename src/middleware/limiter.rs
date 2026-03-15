use redis::{AsyncCommands, Client};
use std::time::Duration;

/// Rate limiter for login attempts by IP (any attempt counts).
/// 20 attempts per 15 minutes per IP.
pub struct LoginLimiter {
    client: Client,
    attempt_window_secs: u64,
    max_attempts_per_ip: u32,
    failure_window_secs: u64,
    max_failures_per_identifier: u32,
}

impl LoginLimiter {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: Client::open(redis_url)?,
            attempt_window_secs: 15 * 60,      // 15 minutes
            max_attempts_per_ip: 20,
            failure_window_secs: 15 * 60,      // 15 minutes
            max_failures_per_identifier: 5,
        })
    }

    /// Returns true if the IP is under the attempt limit and increments the counter.
    pub async fn check_and_record_attempt(&self, ip: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("login_attempts:{}", ip);
        let count: Option<u32> = conn.get(&key).await?;
        match count {
            Some(count) if count >= self.max_attempts_per_ip => Ok(false),
            Some(_) => {
                let _: () = conn.incr(&key, 1).await?;
                Ok(true)
            }
            None => {
                let _: () = conn.set_ex(&key, 1, self.attempt_window_secs).await?;
                Ok(true)
            }
        }
    }

    /// Returns true if the identifier (username/email) is over the failure limit (should block).
    pub async fn is_identifier_over_failure_limit(
        &self,
        identifier: &str,
    ) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("login_failures:{}", identifier);
        let count: Option<u32> = conn.get(&key).await?;
        Ok(count.is_some_and(|c| c >= self.max_failures_per_identifier))
    }

    /// Records a failed login for the given identifier.
    pub async fn record_failed_login(&self, identifier: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("login_failures:{}", identifier);
        let count: Option<u32> = conn.get(&key).await?;
        match count {
            Some(_) => {
                let _: () = conn.incr(&key, 1).await?;
            }
            None => {
                let _: () = conn.set_ex(&key, 1, self.failure_window_secs).await?;
            }
        }
        Ok(())
    }
}

pub struct RateLimiter {
    client: Client,
    window: Duration,
    max_requests: u32,
}

impl RateLimiter {
    pub fn new(
        redis_url: &str,
        window: Duration,
        max_requests: u32,
    ) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: Client::open(redis_url)?,
            window,
            max_requests,
        })
    }

    pub async fn check_rate_limit(&self, email: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("password_reset:{}", email);

        let count: Option<u32> = conn.get(&key).await?;
        match count {
            Some(count) if count >= self.max_requests => Ok(false),
            Some(_) => {
                let _: () = conn.incr(&key, 1).await?;
                Ok(true)
            }
            None => {
                let _: () = conn.set_ex(&key, 1, self.window.as_secs()).await?;
                Ok(true)
            }
        }
    }
}

pub struct PasswordResetLimiter {
    limiter: RateLimiter,
}

impl PasswordResetLimiter {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            limiter: RateLimiter::new(redis_url, std::time::Duration::from_secs(4 * 60 * 60), 1)?,
        })
    }

    pub async fn check_rate_limit(&self, email: &str) -> Result<bool, redis::RedisError> {
        self.limiter.check_rate_limit(email).await
    }
}

pub struct EmailConfirmationLimiter {
    client: Client,
    base_window: Duration,
    max_attempts: u32,
}

impl EmailConfirmationLimiter {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: Client::open(redis_url)?,
            base_window: Duration::from_secs(30 * 60), // 30 minutes base window
            max_attempts: 5,                           // Maximum 5 attempts
        })
    }

    pub async fn check_rate_limit(&self, email: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("email_confirmation_resend:{}", email);

        let attempts: Option<u32> = conn.get(&key).await?;
        match attempts {
            Some(count) => {
                if count >= self.max_attempts {
                    return Ok(false);
                }

                // Calculate exponential backoff window
                let window = self.base_window.as_secs() * 2u64.pow(count);
                let _: () = conn.set_ex(&key, count + 1, window).await?;
                Ok(true)
            }
            None => {
                let _: () = conn.set_ex(&key, 1, self.base_window.as_secs()).await?;
                Ok(true)
            }
        }
    }
}
