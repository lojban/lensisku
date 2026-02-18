use actix_web::{dev::Payload, Error as ActixError, FromRequest, HttpRequest};
use chrono::{DateTime, Utc};
use futures::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{env, error::Error, str::FromStr};
use utoipa::ToSchema;

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

impl FromRequest for Claims {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Extract token from Authorization header
        let auth_header = req.headers().get("Authorization");
        let token = auth_header
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));

        if let Some(token) = token {
            let secret = match env::var("JWT_SECRET") {
                Ok(s) => s,
                Err(e) => {
                    return ready(Err(actix_web::error::ErrorInternalServerError(format!(
                        "JWT_SECRET not set: {}",
                        e
                    ))))
                }
            };
            let key = DecodingKey::from_secret(secret.as_bytes());

            match decode::<Claims>(token, &key, &Validation::default()) {
                Ok(token_data) => ready(Ok(token_data.claims)),
                Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
            }
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized(
                "No authorization token found",
            )))
        }
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
