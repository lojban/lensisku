//! JWT signing secrets, read from the environment once and validated at startup.

use std::sync::OnceLock;

use crate::error::{AppError, AppResult};

struct AuthSecrets {
    jwt: String,
    refresh_token: String,
}

static SECRETS: OnceLock<Result<AuthSecrets, String>> = OnceLock::new();

fn load_from_env() -> Result<AuthSecrets, String> {
    let read = |name: &str| std::env::var(name).map_err(|e| format!("{name}: {e}"));
    Ok(AuthSecrets {
        jwt: read("JWT_SECRET")?,
        refresh_token: read("REFRESH_TOKEN_SECRET")?,
    })
}

fn secrets() -> Result<&'static AuthSecrets, &'static str> {
    SECRETS
        .get_or_init(load_from_env)
        .as_ref()
        .map_err(String::as_str)
}

/// Reads both secrets; call from `main` so a missing secret stops startup
/// instead of failing individual requests.
pub fn init() -> AppResult<()> {
    secrets()
        .map(|_| ())
        .map_err(|e| AppError::Config(vec![e.to_string()]))
}

/// Secret for access tokens and OAuth state.
pub fn jwt_secret() -> Result<&'static str, &'static str> {
    secrets().map(|s| s.jwt.as_str())
}

/// Secret for refresh tokens.
pub fn refresh_token_secret() -> Result<&'static str, &'static str> {
    secrets().map(|s| s.refresh_token.as_str())
}
