use actix_web::{dev::Payload, Error as ActixError, FromRequest, HttpRequest};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{env, error::Error, str::FromStr};
use utoipa::ToSchema;

use crate::AppError;

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, ToSchema)]
pub enum UserRole {
    Admin,
    Moderator,
    Editor,
    User,
    Unconfirmed,
    Blocked,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Moderator => write!(f, "moderator"),
            UserRole::Editor => write!(f, "editor"),
            UserRole::User => write!(f, "user"),
            UserRole::Unconfirmed => write!(f, "unconfirmed"),
            UserRole::Blocked => write!(f, "blocked"),
        }
    }
}

impl FromStr for UserRole {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "moderator" => Ok(UserRole::Moderator),
            "editor" => Ok(UserRole::Editor),
            "user" => Ok(UserRole::User),
            "unconfirmed" => Ok(UserRole::Unconfirmed),
            "blocked" => Ok(UserRole::Blocked),
            _ => Err("Invalid role".into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct User {
    pub userid: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub followers: i32,
    pub role: String,
    pub email_confirmed: bool,
    pub email_confirmation_token: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub email_confirmation_sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Permission {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: i64,
    pub username: String,
    pub email: String,
    pub created_at: i64,
    pub role: String,
    pub email_confirmed: bool,
    pub authorities: Vec<String>,
    pub sid: Option<uuid::Uuid>,
}

pub fn decode_token(token: &str) -> Result<Claims, AppError> {
    let secret =
        env::var("JWT_SECRET").map_err(|e| AppError::Auth(format!("JWT_SECRET not set: {}", e)))?;
    let key = DecodingKey::from_secret(secret.as_bytes());

    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(token, &key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|_| AppError::Auth("Invalid token".to_string()))
}

/// Validate the subject-bound session and load current account state on every request.
pub async fn validate_claims(
    pool: &deadpool_postgres::Pool,
    mut claims: Claims,
) -> Result<Claims, AppError> {
    let sid = claims
        .sid
        .ok_or_else(|| AppError::Auth("Missing session".into()))?;
    let client = pool.get().await?;
    let row = client
        .query_opt(
            "SELECT u.username, u.email, u.role, u.email_confirmed FROM users u
         JOIN user_sessions s ON s.user_id = u.userid
         WHERE u.userid = $1 AND s.session_uuid = $2 AND s.ended_at IS NULL
         AND NOT u.disabled AND LOWER(u.role::text) <> 'blocked'",
            &[&claims.sub, &sid],
        )
        .await?
        .ok_or_else(|| AppError::Auth("Session is no longer active".into()))?;
    claims.username = row.get("username");
    claims.email = row.get("email");
    claims.role = row.get("role");
    claims.email_confirmed = row.get("email_confirmed");
    claims.authorities = client
        .query(
            "SELECT p.name FROM role_permissions rp JOIN permissions p ON rp.permission_id = p.id
         WHERE LOWER(rp.role::text) = LOWER($1::text)",
            &[&claims.role],
        )
        .await?
        .iter()
        .map(|r| r.get("name"))
        .collect();
    Ok(claims)
}

pub async fn authenticate_token(
    pool: &deadpool_postgres::Pool,
    token: &str,
) -> Result<Claims, AppError> {
    validate_claims(pool, decode_token(token)?).await
}

impl FromRequest for Claims {
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(str::to_owned);
        let pool = req
            .app_data::<actix_web::web::Data<deadpool_postgres::Pool>>()
            .cloned();
        Box::pin(async move {
            let (Some(token), Some(pool)) = (token, pool) else {
                return Err(actix_web::error::ErrorUnauthorized(
                    "No authorization token found",
                ));
            };
            authenticate_token(&pool, &token)
                .await
                .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid or revoked token"))
        })
    }
}

impl From<tokio_postgres::Row> for User {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            userid: row.get("userid"),
            username: row.get("username"),
            email: row.get("email"),
            password: row.get("password"),
            created_at: row.get("created_at"),
            followers: row.get("followers"),
            role: row.get("role"),
            email_confirmed: row.get("email_confirmed"),
            email_confirmation_token: row.try_get("email_confirmation_token").unwrap_or(None),
            email_confirmation_sent_at: row.try_get("email_confirmation_sent_at").unwrap_or(None),
        }
    }
}
