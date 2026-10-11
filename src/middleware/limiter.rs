use redis::{AsyncCommands, Client};
use std::env;
use std::time::Duration;

/// Per-user rate limit for expensive Kokoro Martin TTS (`POST /collections/kokoro-tts`).
/// Keys: `kokoro_tts:user:{sub}`. Configure via `KOKORO_TTS_RATE_LIMIT_MAX` /
/// `KOKORO_TTS_RATE_LIMIT_WINDOW_SECS` (falls back to legacy `KITTEN_TTS_*`).
pub struct KokoroTtsLimiter {
    client: Client,
    window_secs: u64,
    max_requests: u32,
}

impl KokoroTtsLimiter {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let max_requests = env::var("KOKORO_TTS_RATE_LIMIT_MAX")
            .or_else(|_| env::var("KITTEN_TTS_RATE_LIMIT_MAX"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);
        let window_secs = env::var("KOKORO_TTS_RATE_LIMIT_WINDOW_SECS")
            .or_else(|_| env::var("KITTEN_TTS_RATE_LIMIT_WINDOW_SECS"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30 * 60);
        Ok(Self {
            client: Client::open(redis_url)?,
            window_secs,
            max_requests,
        })
    }

    pub fn retry_after_secs(&self) -> u64 {
        self.window_secs
    }

    /// Returns `true` if the request is allowed (counter incremented), `false` if over limit.
    pub async fn check_and_record(&self, user_sub: i32) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("kokoro_tts:user:{user_sub}");
        let count: Option<u32> = conn.get(&key).await?;
        match count {
            Some(c) if c >= self.max_requests => Ok(false),
            Some(_) => {
                let _: () = conn.incr(&key, 1).await?;
                Ok(true)
            }
            None => {
                let _: () = conn.set_ex(&key, 1, self.window_secs).await?;
                Ok(true)
            }
        }
    }
}

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
            attempt_window_secs: 15 * 60, // 15 minutes
            max_attempts_per_ip: 20,
            failure_window_secs: 15 * 60, // 15 minutes
            max_failures_per_identifier: 5,
        })
    }

    /// Reserve IP/account attempts atomically, before password verification.
    /// Redis time and a per-account next-attempt time prevent parallel guesses.
    pub async fn reserve_login(&self, ip: &str, account: &str) -> Result<u64, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        redis::Script::new(
            r#"
            local t = redis.call('TIME')
            local now = tonumber(t[1]) * 1000 + math.floor(tonumber(t[2]) / 1000)
            local ip_count = tonumber(redis.call('GET', KEYS[1]) or '0')
            local count = tonumber(redis.call('HGET', KEYS[2], 'count') or '0')
            local next_at = tonumber(redis.call('HGET', KEYS[2], 'next') or '0')
            if ip_count >= tonumber(ARGV[1]) then return math.max(1, redis.call('TTL', KEYS[1])) end
            if count >= tonumber(ARGV[3]) then return math.max(1, redis.call('TTL', KEYS[2])) end
            if next_at > now then return math.max(1, math.ceil((next_at - now) / 1000)) end
            local n = redis.call('INCR', KEYS[1])
            if n == 1 then redis.call('EXPIRE', KEYS[1], ARGV[2]) end
            count = count + 1
            redis.call('HSET', KEYS[2], 'count', count, 'next', now + 1000 * 2^(count - 1))
            if count == 1 then redis.call('EXPIRE', KEYS[2], ARGV[4]) end
            return 0
        "#,
        )
        .key(format!("login_attempts:{ip}"))
        .key(format!("login_account:{account}"))
        .arg(self.max_attempts_per_ip)
        .arg(self.attempt_window_secs)
        .arg(self.max_failures_per_identifier)
        .arg(self.failure_window_secs)
        .invoke_async(&mut conn)
        .await
    }

    pub async fn clear_success(&self, account: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.del(format!("login_account:{account}")).await?;
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
