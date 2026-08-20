//! Shared OAuth social-login pipeline (GitHub, Google).
//!
//! Adding a provider: extend [`OAuthProvider`], env `{PREFIX}_CLIENT_ID/_SECRET/_REDIRECT_URL`,
//! and [`fetch_profile`]. Account linking and JWT issuance stay in this module.

use chrono::{Duration, Utc};
use deadpool_postgres::Pool;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EndpointNotSet, EndpointSet, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use uuid::Uuid;

use crate::auth::models::UserRole;
use crate::auth::service::{create_token_pair, hash_password, sanitize_html};
use crate::auth::User;
use crate::sessions;
use crate::{AppError, AppResult};

const OAUTH_STATE_PURPOSE: &str = "oauth_state";
const OAUTH_STATE_TTL_MINUTES: i64 = 10;
const MAX_USERNAME_LEN: usize = 64;
const USER_COLUMNS: &str = "userid, username, email, password, created_at, followers, role, \
     email_confirmed, email_confirmation_token, email_confirmation_sent_at";
const USER_COLUMNS_U: &str =
    "u.userid, u.username, u.email, u.password, u.created_at, u.followers, \
     u.role, u.email_confirmed, u.email_confirmation_token, u.email_confirmation_sent_at";

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct OAuthAccount {
    pub id: i32,
    pub user_id: i32,
    pub provider: String,
    pub provider_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProvider {
    Github,
    Google,
}

impl OAuthProvider {
    const ALL: [OAuthProvider; 2] = [OAuthProvider::Github, OAuthProvider::Google];

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "github" => Some(Self::Github),
            "google" => Some(Self::Google),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Google => "google",
        }
    }

    fn env_prefix(self) -> &'static str {
        match self {
            Self::Github => "GITHUB",
            Self::Google => "GOOGLE",
        }
    }

    fn auth_url(self) -> &'static str {
        match self {
            Self::Github => "https://github.com/login/oauth/authorize",
            Self::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        }
    }

    fn token_url(self) -> &'static str {
        match self {
            Self::Github => "https://github.com/login/oauth/access_token",
            Self::Google => "https://oauth2.googleapis.com/token",
        }
    }

    fn scopes(self) -> &'static [&'static str] {
        match self {
            Self::Github => &["read:user", "user:email"],
            Self::Google => &["openid", "email", "profile"],
        }
    }
}

struct ProviderConfig {
    client_id: String,
    client_secret: String,
    redirect_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuthStateClaims {
    purpose: String,
    nonce: String,
    provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_to: Option<String>,
    exp: i64,
}

#[derive(Debug)]
struct OAuthProfile {
    provider_id: String,
    email: Option<String>,
    email_verified: bool,
    suggested_username: String,
}

pub struct OAuthAuthorizeResult {
    pub authorize_url: String,
}

pub struct OAuthCompleteResult {
    pub access_token: String,
    pub refresh_token: String,
    pub username: String,
    pub return_to: Option<String>,
}

fn env_nonempty(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn load_config(provider: OAuthProvider) -> AppResult<ProviderConfig> {
    let prefix = provider.env_prefix();
    let client_id_key = format!("{prefix}_CLIENT_ID");
    let client_secret_key = format!("{prefix}_CLIENT_SECRET");
    let redirect_url_key = format!("{prefix}_REDIRECT_URL");
    let client_id = env_nonempty(&client_id_key);
    let client_secret = env_nonempty(&client_secret_key);
    let redirect_url = env_nonempty(&redirect_url_key);
    match (client_id, client_secret, redirect_url) {
        (Some(client_id), Some(client_secret), Some(redirect_url)) => Ok(ProviderConfig {
            client_id,
            client_secret,
            redirect_url,
        }),
        _ => Err(AppError::Config(vec![format!(
            "{} is not configured",
            provider.as_str()
        )])),
    }
}

fn jwt_secret() -> AppResult<String> {
    env_nonempty("JWT_SECRET").ok_or_else(|| {
        AppError::Config(vec![
            "JWT_SECRET must be set to sign OAuth state".to_string()
        ])
    })
}

/// Relative in-app path only (blocks open redirects).
pub(crate) fn sanitize_return_to(raw: Option<&str>) -> Option<String> {
    let s = raw?.trim();
    if s.is_empty() || s.len() > 512 {
        return None;
    }
    if !s.starts_with('/') || s.starts_with("//") {
        return None;
    }
    if s.contains("://") || s.contains('\\') || s.contains('\n') || s.contains('\r') {
        return None;
    }
    Some(s.to_string())
}

fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

pub(crate) fn sanitize_username(raw: &str) -> String {
    let stripped = sanitize_html(raw);
    let mut out = String::new();
    let mut last_sep = false;
    for c in stripped.trim().chars() {
        let ok = c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.');
        if ok {
            out.push(c);
            last_sep = matches!(c, '-' | '_' | '.');
        } else if !last_sep {
            out.push('-');
            last_sep = true;
        }
    }
    let out = out
        .trim_matches(|c| matches!(c, '-' | '_' | '.'))
        .to_string();
    if out.is_empty() {
        "user".to_string()
    } else {
        truncate_chars(&out, MAX_USERNAME_LEN)
    }
}

fn username_with_suffix(base: &str, n: u32) -> String {
    let suffix = format!("-{n}");
    let max_base = MAX_USERNAME_LEN.saturating_sub(suffix.len());
    format!("{}{}", truncate_chars(base, max_base), suffix)
}

fn parse_provider(name: &str) -> AppResult<OAuthProvider> {
    OAuthProvider::parse(name)
        .ok_or_else(|| AppError::NotFound(format!("Unknown OAuth provider: {name}")))
}

fn oauth_http_client() -> AppResult<reqwest::Client> {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("lensisku-oauth")
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build OAuth HTTP client: {e}")))
}

type ConfiguredOAuthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

fn build_oauth_client(
    provider: OAuthProvider,
    config: &ProviderConfig,
) -> AppResult<ConfiguredOAuthClient> {
    let auth_url = AuthUrl::new(provider.auth_url().to_string())
        .map_err(|e| AppError::Internal(format!("Invalid auth URL: {e}")))?;
    let token_url = TokenUrl::new(provider.token_url().to_string())
        .map_err(|e| AppError::Internal(format!("Invalid token URL: {e}")))?;
    let redirect = RedirectUrl::new(config.redirect_url.clone())
        .map_err(|e| AppError::Internal(format!("Invalid redirect URL: {e}")))?;
    Ok(BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_client_secret(ClientSecret::new(config.client_secret.clone()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect))
}

fn sign_state(provider: OAuthProvider, return_to: Option<String>) -> AppResult<String> {
    let secret = jwt_secret()?;
    let exp = Utc::now()
        .checked_add_signed(Duration::minutes(OAUTH_STATE_TTL_MINUTES))
        .ok_or_else(|| AppError::Internal("OAuth state expiry overflow".to_string()))?
        .timestamp();
    let claims = OAuthStateClaims {
        purpose: OAUTH_STATE_PURPOSE.to_string(),
        nonce: Uuid::new_v4().to_string(),
        provider: provider.as_str().to_string(),
        return_to,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to sign OAuth state: {e}")))
}

fn verify_state(state: &str, provider: OAuthProvider) -> AppResult<OAuthStateClaims> {
    let secret = jwt_secret()?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp"]);
    let data = decode::<OAuthStateClaims>(
        state,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::BadRequest("Invalid or expired OAuth state".to_string()))?;
    let claims = data.claims;
    if claims.purpose != OAUTH_STATE_PURPOSE {
        return Err(AppError::BadRequest("Invalid OAuth state".to_string()));
    }
    if claims.provider != provider.as_str() {
        return Err(AppError::BadRequest(
            "OAuth state provider mismatch".to_string(),
        ));
    }
    Ok(claims)
}

pub fn configured_providers() -> Vec<&'static str> {
    OAuthProvider::ALL
        .iter()
        .filter(|p| load_config(**p).is_ok())
        .map(|p| p.as_str())
        .collect()
}

pub fn authorize_url(
    provider_name: &str,
    return_to: Option<&str>,
) -> AppResult<OAuthAuthorizeResult> {
    let provider = parse_provider(provider_name)?;
    let config = load_config(provider)?;
    let client = build_oauth_client(provider, &config)?;
    let state = sign_state(provider, sanitize_return_to(return_to))?;
    let mut request = client.authorize_url(|| CsrfToken::new(state));
    for scope in provider.scopes() {
        request = request.add_scope(Scope::new((*scope).to_string()));
    }
    if provider == OAuthProvider::Google {
        request = request.add_extra_param("prompt", "select_account");
    }
    let (url, _) = request.url();
    Ok(OAuthAuthorizeResult {
        authorize_url: url.to_string(),
    })
}

fn json_true(value: &Value) -> bool {
    value.as_bool().unwrap_or(false) || value.as_str() == Some("true")
}

async fn fetch_profile(
    provider: OAuthProvider,
    access_token: &str,
    http: &reqwest::Client,
) -> AppResult<OAuthProfile> {
    match provider {
        OAuthProvider::Github => fetch_github_profile(access_token, http).await,
        OAuthProvider::Google => fetch_google_profile(access_token, http).await,
    }
}

async fn fetch_github_profile(
    access_token: &str,
    http: &reqwest::Client,
) -> AppResult<OAuthProfile> {
    let user_resp = http
        .get("https://api.github.com/user")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        )
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to get GitHub user: {e}")))?;
    if !user_resp.status().is_success() {
        return Err(AppError::Auth(format!(
            "Failed to get GitHub user: {}",
            user_resp.status()
        )));
    }
    let user: Value = user_resp
        .json()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to parse GitHub user: {e}")))?;
    let provider_id = user
        .get("id")
        .and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_u64().map(|n| n.to_string()))
                .or_else(|| v.as_str().map(ToOwned::to_owned))
        })
        .ok_or_else(|| AppError::Auth("Missing GitHub user id".to_string()))?;
    let login = user.get("login").and_then(Value::as_str).unwrap_or("user");

    let emails_resp = http
        .get("https://api.github.com/user/emails")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        )
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to get GitHub emails: {e}")))?;

    let mut email = None;
    let mut email_verified = false;
    if emails_resp.status().is_success() {
        let emails: Value = emails_resp
            .json()
            .await
            .map_err(|e| AppError::Auth(format!("Failed to parse GitHub emails: {e}")))?;
        if let Some(list) = emails.as_array() {
            let verified = list.iter().find(|e| json_true(&e["verified"]));
            let primary_verified = list
                .iter()
                .find(|e| json_true(&e["verified"]) && json_true(&e["primary"]));
            let chosen = primary_verified.or(verified).or(list.first());
            if let Some(row) = chosen {
                email = row.get("email").and_then(Value::as_str).map(str::to_string);
                email_verified = json_true(&row["verified"]);
            }
        }
    }
    if email.is_none() {
        email = user
            .get("email")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
    }

    Ok(OAuthProfile {
        provider_id,
        email,
        email_verified,
        suggested_username: sanitize_username(login),
    })
}

async fn fetch_google_profile(
    access_token: &str,
    http: &reqwest::Client,
) -> AppResult<OAuthProfile> {
    let resp = http
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {access_token}"),
        )
        .send()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to get Google user: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::Auth(format!(
            "Failed to get Google user: {}",
            resp.status()
        )));
    }
    let user: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to parse Google user: {e}")))?;
    let provider_id = user
        .get("sub")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Auth("Missing Google subject".to_string()))?
        .to_string();
    let email = user
        .get("email")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let email_verified = json_true(&user["email_verified"]);
    let suggested_username = sanitize_username(
        email
            .as_deref()
            .and_then(|e| e.split('@').next())
            .unwrap_or("user"),
    );
    Ok(OAuthProfile {
        provider_id,
        email,
        email_verified,
        suggested_username,
    })
}

fn fallback_email(provider: OAuthProvider, provider_id: &str) -> String {
    format!(
        "{}+{}@users.noreply.lensisku.invalid",
        provider.as_str(),
        provider_id
    )
}

async fn unique_username(
    transaction: &deadpool_postgres::Transaction<'_>,
    suggested: &str,
) -> AppResult<String> {
    let base = sanitize_username(suggested);
    for n in 0u32..100 {
        let candidate = if n == 0 {
            base.clone()
        } else {
            username_with_suffix(&base, n + 1)
        };
        let exists = transaction
            .query_opt("SELECT 1 FROM users WHERE username = $1", &[&candidate])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .is_some();
        if !exists {
            return Ok(candidate);
        }
    }
    Err(AppError::Internal(
        "Could not allocate a unique username".to_string(),
    ))
}

fn reject_if_blocked(user: &User) -> AppResult<()> {
    if user.role.to_lowercase() == UserRole::Blocked.to_string() {
        Err(AppError::Auth("Account is blocked".to_string()))
    } else {
        Ok(())
    }
}

async fn issue_tokens(
    pool: &Pool,
    user: &User,
    ip_address: String,
    user_agent: String,
) -> AppResult<OAuthCompleteResult> {
    let mut jwt_session_id = None;
    match sessions::service::start_session(pool, user.userid, ip_address, user_agent).await {
        Ok(session) => jwt_session_id = Some(session.session_uuid),
        Err(e) => log::error!(
            "Failed to start user session for oauth user_id {}: {}",
            user.userid,
            e
        ),
    }
    let pair = create_token_pair(pool, user, jwt_session_id).await?;
    Ok(OAuthCompleteResult {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        username: user.username.clone(),
        return_to: None,
    })
}

pub async fn complete_oauth(
    pool: &Pool,
    provider_name: &str,
    code: &str,
    state: &str,
    ip_address: String,
    user_agent: String,
) -> AppResult<OAuthCompleteResult> {
    if code.trim().is_empty() || state.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Missing OAuth code or state".to_string(),
        ));
    }
    let provider = parse_provider(provider_name)?;
    let config = load_config(provider)?;
    let claims = verify_state(state, provider)?;
    let return_to = sanitize_return_to(claims.return_to.as_deref());

    let client = build_oauth_client(provider, &config)?;
    let http = oauth_http_client()?;
    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(&http)
        .await
        .map_err(|e| AppError::Auth(format!("Failed to exchange code for token: {e}")))?;
    let access_token = token_result.access_token().secret().to_string();
    let profile = fetch_profile(provider, &access_token, &http).await?;

    let mut db = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = db
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let existing_oauth = transaction
        .query_opt(
            &format!(
                "SELECT {USER_COLUMNS_U} FROM users u \
                 INNER JOIN oauth_accounts o ON u.userid = o.user_id \
                 WHERE o.provider = $1 AND o.provider_id = $2"
            ),
            &[&provider.as_str(), &profile.provider_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let user = if let Some(row) = existing_oauth {
        let user = User::from(row);
        reject_if_blocked(&user)?;
        transaction
            .commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        user
    } else if profile.email_verified {
        let email = profile
            .email
            .as_ref()
            .ok_or_else(|| AppError::Auth("Verified email missing".to_string()))?;
        let existing_email = transaction
            .query_opt(
                &format!(
                    "SELECT {USER_COLUMNS} FROM users WHERE LOWER(email) = LOWER($1) ORDER BY userid LIMIT 1"
                ),
                &[email],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = existing_email {
            let mut user = User::from(row);
            reject_if_blocked(&user)?;
            transaction
                .execute(
                    "INSERT INTO oauth_accounts (user_id, provider, provider_id) VALUES ($1, $2, $3)",
                    &[&user.userid, &provider.as_str(), &profile.provider_id],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            if !user.email_confirmed
                || user.role.to_lowercase() == UserRole::Unconfirmed.to_string()
            {
                let new_role = if user.role.to_lowercase() == UserRole::Unconfirmed.to_string() {
                    UserRole::User.to_string()
                } else {
                    user.role.clone()
                };
                transaction
                    .execute(
                        "UPDATE users SET email_confirmed = true, role = $2 WHERE userid = $1",
                        &[&user.userid, &new_role],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                user.email_confirmed = true;
                user.role = new_role;
            }
            transaction
                .commit()
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            user
        } else {
            let user = insert_oauth_user(&transaction, provider, &profile).await?;
            transaction
                .commit()
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            user
        }
    } else {
        if let Some(email) = profile.email.as_ref() {
            let taken = transaction
                .query_opt(
                    "SELECT userid FROM users WHERE LOWER(email) = LOWER($1) LIMIT 1",
                    &[email],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
                .is_some();
            if taken {
                return Err(AppError::BadRequest("account_collision".to_string()));
            }
        }
        let user = insert_oauth_user(&transaction, provider, &profile).await?;
        transaction
            .commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        user
    };

    let mut result = issue_tokens(pool, &user, ip_address, user_agent).await?;
    result.return_to = return_to;
    Ok(result)
}

async fn insert_oauth_user(
    transaction: &deadpool_postgres::Transaction<'_>,
    provider: OAuthProvider,
    profile: &OAuthProfile,
) -> AppResult<User> {
    let username = unique_username(transaction, &profile.suggested_username).await?;
    let (email, email_confirmed) = if profile.email_verified {
        (
            profile
                .email
                .clone()
                .unwrap_or_else(|| fallback_email(provider, &profile.provider_id)),
            true,
        )
    } else {
        (fallback_email(provider, &profile.provider_id), true)
    };
    let password_hash = hash_password(&Uuid::new_v4().to_string())
        .map_err(|e| AppError::Auth(format!("Password hashing failed: {e}")))?;
    let created_at = Utc::now();
    let role = UserRole::User.to_string();
    let votesize = 1.0_f32;
    let oauth_signup = true;

    let row = transaction
        .query_one(
            "INSERT INTO users (
                username, email, password, created_at,
                role, email_confirmed, votesize, oauth_signup
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING userid",
            &[
                &username,
                &email,
                &password_hash,
                &created_at,
                &role,
                &email_confirmed,
                &votesize,
                &oauth_signup,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let user_id: i32 = row.get("userid");
    transaction
        .execute(
            "INSERT INTO oauth_accounts (user_id, provider, provider_id) VALUES ($1, $2, $3)",
            &[&user_id, &provider.as_str(), &profile.provider_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(User {
        userid: user_id,
        username,
        email,
        password: password_hash,
        created_at,
        followers: 0,
        role,
        email_confirmed,
        email_confirmation_token: None,
        email_confirmation_sent_at: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_providers() {
        assert_eq!(OAuthProvider::parse("github"), Some(OAuthProvider::Github));
        assert_eq!(OAuthProvider::parse("google"), Some(OAuthProvider::Google));
        assert_eq!(OAuthProvider::parse("discord"), None);
    }

    #[test]
    fn return_to_must_be_relative() {
        assert_eq!(
            sanitize_return_to(Some("/lingo/courses")),
            Some("/lingo/courses".to_string())
        );
        assert_eq!(sanitize_return_to(Some("https://evil.example/")), None);
        assert_eq!(sanitize_return_to(Some("//evil.example")), None);
        assert_eq!(sanitize_return_to(Some("/ok")), Some("/ok".to_string()));
        assert_eq!(sanitize_return_to(Some("")), None);
    }

    #[test]
    fn username_sanitization() {
        assert_eq!(sanitize_username("octocat"), "octocat");
        assert_eq!(sanitize_username("John Doe"), "John-Doe");
        assert_eq!(sanitize_username("!!!"), "user");
        assert_eq!(sanitize_username("foo.bar+baz"), "foo.bar-baz");
        let long = "a".repeat(80);
        assert_eq!(sanitize_username(&long).len(), MAX_USERNAME_LEN);
        assert_eq!(
            username_with_suffix(&"a".repeat(64), 2).len(),
            MAX_USERNAME_LEN
        );
    }
}
