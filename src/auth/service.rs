//TODO: refactor
use crate::utils::remove_html_tags;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::{error, info};
use rand::distr::Alphanumeric;
use std::env;
use tokio::task;

use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, HttpMessage};
use actix_web::{web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};

use deadpool_postgres::Pool;
use rand::{rng, Rng};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::error::EmailError;
use crate::auth::models::UserRole;
use crate::auth::{AuthResponse, RoleWithPermissions, SignupRequest};
use crate::middleware::limiter::{EmailConfirmationLimiter, PasswordResetLimiter};
use crate::notifications::service::EmailNotification;
use crate::notifications::EmailService;
use crate::sessions;
use crate::{AppError, AppResult};

use super::error::PasswordHashError;
use super::models::TokenPair;
use super::{
    Claims, CompletePasswordChangeRequest, CompletePasswordChangeResponse, CreateRoleRequest,
    FollowResponse, InitiatePasswordChangeRequest, InitiatePasswordChangeResponse, LoginRequest,
    PasswordResetRequest, PasswordResetResponse, PasswordRestoreRequest, PermissionInfo,
    ProfileResponse, ResendConfirmationRequest, ResendConfirmationResponse, RoleResponse,
    UpdateProfileRequest, UpdateRoleRequest, User,
};

pub fn hash_password(password: &str) -> Result<String, PasswordHashError> {
    // Always use bcrypt for new passwords
    Ok(hash(password, DEFAULT_COST)?)
}

pub fn sanitize_html(html: &str) -> String {
    remove_html_tags(html)
}

fn rot13(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='M' | 'a'..='m' => ((c as u8) + 13) as char,
            'N'..='Z' | 'n'..='z' => ((c as u8) - 13) as char,
            _ => c,
        })
        .collect()
}

/// Verify a password against a stored hash, supporting both MD5 and bcrypt
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, PasswordHashError> {
    // Check if the stored hash is MD5 (32 hex characters) or bcrypt (starts with $)
    if needs_rehash(stored_hash) {
        // MD5 hash
        let digest = format!("{:x}", md5::compute(rot13(password).as_bytes()));
        Ok(digest == stored_hash)
    } else if stored_hash.starts_with('$') {
        // bcrypt hash
        Ok(verify(password, stored_hash)?)
    } else {
        Err(PasswordHashError::InvalidHashFormat)
    }
}

pub async fn list_permissions(pool: &Pool) -> AppResult<Vec<PermissionInfo>> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = client
        .query(
            "SELECT name, description FROM permissions ORDER BY name",
            &[],
        )
        .await?;

    Ok(rows
        .iter()
        .map(|row| PermissionInfo {
            name: row.get("name"),
            description: row.get("description"),
        })
        .collect())
}

pub async fn get_roles_with_permissions(
    pool: &Pool,
    actor_role: &str,
) -> AppResult<Vec<RoleWithPermissions>> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // First, get all distinct roles from users table to ensure we include roles like "unconfirmed"
    // that might not have any permissions assigned
    // Use LOWER() to avoid duplicates with different casing
    let all_roles_rows = client
        .query(
            "SELECT DISTINCT LOWER(role::text) as role FROM users ORDER BY role",
            &[],
        )
        .await?;

    use std::collections::BTreeMap;
    let mut roles: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // Initialize all roles with empty permissions
    for row in all_roles_rows {
        let role: String = row.get("role");
        roles.insert(role, Vec::new());
    }

    // Then, get roles with permissions (filtered by actor's permissions)
    // Use case-insensitive comparison for role names
    let rows = client
        .query(
            "SELECT LOWER(r.role::text) as role, p.name as permission 
         FROM role_permissions r
         JOIN permissions p ON r.permission_id = p.id
         WHERE NOT EXISTS (
             SELECT 1 
             FROM role_permissions rp 
             WHERE LOWER(rp.role::text) = LOWER(r.role::text)
             AND NOT EXISTS (
                 SELECT 1 
                 FROM role_permissions actor_perms 
                 WHERE LOWER(actor_perms.role::text) = LOWER($1::text)
                 AND actor_perms.permission_id = rp.permission_id
             )
         )
         ORDER BY role, p.name",
            &[&actor_role],
        )
        .await?;

    // Add permissions to roles
    for row in rows {
        let role: String = row.get("role");
        let permission: String = row.get("permission");
        roles.entry(role).or_default().push(permission);
    }

    // Convert to Vec and sort permissions for each role
    Ok(roles
        .into_iter()
        .map(|(name, permissions)| RoleWithPermissions { name, permissions })
        .collect())
}

pub async fn create_role(
    pool: &Pool,
    request: &CreateRoleRequest,
    actor_role: &str,
) -> AppResult<RoleResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check if role already exists (case-insensitive)
    let role_exists: bool = transaction
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM role_permissions WHERE LOWER(role::text) = LOWER($1::text))",
            &[&request.name],
        )
        .await?
        .get(0);

    if role_exists {
        return Err(AppError::BadRequest("Role already exists".to_string()));
    }

    // Get actor's permissions
    let actor_permissions: Vec<String> = transaction
        .query(
            "SELECT p.name 
             FROM role_permissions rp
             JOIN permissions p ON rp.permission_id = p.id
             WHERE rp.role = $1",
            &[&actor_role],
        )
        .await?
        .iter()
        .map(|row| row.get("name"))
        .collect();

    // Check if actor has all requested permissions
    for permission in &request.permissions {
        if !actor_permissions.contains(permission) {
            return Err(AppError::Auth(format!(
                "Cannot assign permission '{}' that you don't possess",
                permission
            )));
        }
    }

    // Add permissions
    for permission in &request.permissions {
        transaction
            .execute(
                "INSERT INTO role_permissions (role, permission_id)
             SELECT $1, id FROM permissions WHERE name = $2
             ON CONFLICT DO NOTHING",
                &[&request.name, &permission],
            )
            .await?;
    }

    transaction.commit().await?;

    Ok(RoleResponse {
        name: request.name.clone(),
        permissions: request.permissions.clone(),
    })
}

pub async fn update_role(
    pool: &Pool,
    role_name: &str,
    request: &UpdateRoleRequest,
    actor_role: &str,
) -> AppResult<RoleResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get actor's permissions (case-insensitive role lookup)
    let actor_permissions: Vec<String> = transaction
        .query(
            "SELECT p.name 
             FROM role_permissions rp
             JOIN permissions p ON rp.permission_id = p.id
             WHERE LOWER(rp.role::text) = LOWER($1::text)",
            &[&actor_role],
        )
        .await?
        .iter()
        .map(|row| row.get("name"))
        .collect();

    // Check if actor has all requested permissions
    for permission in &request.permissions {
        if !actor_permissions.contains(permission) {
            return Err(AppError::Auth(format!(
                "Cannot assign permission '{}' that you don't possess",
                permission
            )));
        }
    }

    // Clear existing permissions (case-insensitive)
    transaction
        .execute(
            "DELETE FROM role_permissions WHERE LOWER(role::text) = LOWER($1::text)",
            &[&role_name],
        )
        .await?;

    // Add new permissions
    for permission in &request.permissions {
        transaction
            .execute(
                "INSERT INTO role_permissions (role, permission_id)
             SELECT $1, id FROM permissions WHERE name = $2",
                &[&role_name, &permission],
            )
            .await?;
    }

    transaction.commit().await?;

    Ok(RoleResponse {
        name: role_name.to_string(),
        permissions: request.permissions.clone(),
    })
}

pub async fn delete_role(pool: &Pool, role_name: &str) -> AppResult<()> {
    let protected_roles = ["admin", "editor", "unconfirmed", "blocked"];
    if protected_roles.contains(&role_name) {
        return Err(AppError::BadRequest(format!(
            "Cannot delete protected role: {}",
            role_name
        )));
    }

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client.transaction().await?;

    // Convert users to default 'user' role (case-insensitive)
    transaction
        .execute(
            "UPDATE users SET role = 'user' WHERE LOWER(role::text) = LOWER($1::text)",
            &[&role_name],
        )
        .await?;

    // Remove role permissions (case-insensitive)
    transaction
        .execute(
            "DELETE FROM role_permissions WHERE LOWER(role::text) = LOWER($1::text)",
            &[&role_name],
        )
        .await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn block_user(
    pool: &Pool,
    actor_id: i32,
    target_user_id: i32,
    block: bool,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client.transaction().await?;

    // Check if actor has block_users permission (case-insensitive role comparison)
    let has_block_users: bool = transaction
        .query_one(
            "SELECT EXISTS (
                SELECT 1 
                FROM role_permissions rp
                JOIN permissions p ON rp.permission_id = p.id
                JOIN users u ON LOWER(rp.role::text) = LOWER(u.role::text)
                WHERE u.userid = $1 AND p.name = 'block_users'
            )",
            &[&actor_id],
        )
        .await?
        .get(0);

    if !has_block_users {
        return Err(AppError::Auth(
            "Insufficient permissions to block/unblock users".to_string(),
        ));
    }

    // Check if target user has manage_users permission (case-insensitive role comparison)
    let target_has_manage_users: bool = transaction
        .query_one(
            "SELECT EXISTS (
                SELECT 1 
                FROM role_permissions rp
                JOIN permissions p ON rp.permission_id = p.id
                JOIN users u ON LOWER(rp.role::text) = LOWER(u.role::text)
                WHERE u.userid = $1 AND p.name = 'manage_users'
            )",
            &[&target_user_id],
        )
        .await?
        .get(0);

    if target_has_manage_users && actor_id != target_user_id {
        return Err(AppError::Auth(
            "Cannot block/unblock users with manage_users permission".to_string(),
        ));
    }

    // Update user's disabled status
    transaction
        .execute(
            "UPDATE users 
             SET disabled = $1,
                 disabled_at = CASE WHEN $1 THEN NOW() ELSE NULL END,
                 disabled_by = CASE WHEN $1 THEN $2 ELSE NULL::integer END
             WHERE userid = $3",
            &[&block, &actor_id, &target_user_id],
        )
        .await?;

    log::debug!("Attempting to commit transaction");
    match transaction.commit().await {
        Ok(_) => log::debug!("Transaction committed successfully"),
        Err(e) => {
            error!("Failed to commit transaction: {}", e); // Keep log
            return Err(AppError::Database(e.to_string())); // Return AppError
        }
    }
    Ok(())
}

pub async fn assign_role(
    pool: &Pool,
    assigner_id: i32,
    target_user_id: i32,
    new_role: String,
) -> AppResult<()> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Check if assigner has manage_roles permission
    let has_manage_roles: bool = transaction
        .query_one(
            "SELECT EXISTS (
                SELECT 1 
                FROM role_permissions rp
                JOIN permissions p ON rp.permission_id = p.id
                JOIN users u ON LOWER(rp.role::text) = LOWER(u.role::text)
                WHERE u.userid = $1 AND p.name = 'manage_roles'
            )",
            &[&assigner_id],
        )
        .await?
        .get(0);

    if !has_manage_roles {
        return Err(AppError::Auth(
            "Insufficient permissions to assign roles".to_string(),
        ));
    }

    // Get assigner's permissions (case-insensitive role comparison)
    let assigner_perms: Vec<String> = transaction
        .query(
            "SELECT p.name 
             FROM role_permissions rp
             JOIN permissions p ON rp.permission_id = p.id
             JOIN users u ON LOWER(rp.role::text) = LOWER(u.role::text)
             WHERE u.userid = $1",
            &[&assigner_id],
        )
        .await?
        .iter()
        .map(|row| row.get("name"))
        .collect();

    // Get target role's permissions (case-insensitive role lookup)
    let target_role_perms: Vec<String> = transaction
        .query(
            "SELECT p.name 
             FROM role_permissions rp
             JOIN permissions p ON rp.permission_id = p.id
             WHERE LOWER(rp.role::text) = LOWER($1::text)",
            &[&new_role],
        )
        .await?
        .iter()
        .map(|row| row.get("name"))
        .collect();

    // Check if assigner has all permissions of the target role
    for perm in &target_role_perms {
        if !assigner_perms.contains(perm) {
            return Err(AppError::Auth(format!(
                "Cannot assign role '{}' as it contains permission '{}' that you don't possess",
                new_role, perm
            )));
        }
    }

    // Update user's role and disable status
    transaction
        .execute(
            "UPDATE users 
             SET role = $1,
                 disabled = CASE WHEN LOWER($1::text) = 'blocked' THEN true ELSE false END,
                 disabled_at = CASE WHEN LOWER($1::text) = 'blocked' THEN NOW() ELSE NULL END,
                 disabled_by = CASE WHEN LOWER($1::text) = 'blocked' THEN $3 ELSE NULL::integer END
             WHERE userid = $2",
            &[&new_role, &target_user_id, &assigner_id],
        )
        .await?;

    transaction.commit().await?;
    Ok(())
}

/// Helper function to check if a password needs to be upgraded from MD5 to bcrypt
pub fn needs_rehash(stored_hash: &str) -> bool {
    // If it's an MD5 hash (32 hex characters) or doesn't start with bcrypt format
    stored_hash.len() == 32 && stored_hash.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn generate_token(user: &User) -> AppResult<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .ok_or_else(|| {
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
        })?
        .timestamp();

    let claims = Claims {
        sub: user.userid,
        exp: expiration,
        username: user.username.clone(),
        email: user.email.clone(),
        created_at: user.created_at.timestamp(),
        role: user.role.clone(),
        email_confirmed: user.email_confirmed,
        authorities: Vec::new(), // Will be populated in create_token_pair
        sid: None,               // Will be populated by specific token generation functions
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        // Map JWT error
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Auth(format!("Token encoding error: {}", e)))
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(_) => Err((ErrorUnauthorized("Invalid token"), req)),
    }
}

pub async fn signup(pool: &Pool, user_data: &SignupRequest) -> AppResult<AuthResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check if user exists
    let sanitized_username = sanitize_html(&user_data.username);
    let existing_check = transaction
        .query(
            "SELECT userid FROM users WHERE username = $1 OR email = $2",
            &[&sanitized_username, &user_data.email],
        )
        .await?;

    if !existing_check.is_empty() {
        return Err(AppError::Auth(
            "Username or email already exists".to_string(),
        ));
    }

    // Hash password
    let password_hash = hash_password(&user_data.password)
        .map_err(|e| AppError::Auth(format!("Password hashing failed: {}", e)))?;
    let created_at = Utc::now();

    // Insert new user with Unconfirmed role
    let row = transaction
        .query_one(
            "INSERT INTO users (
                username, email, password, created_at, 
                role, email_confirmed, votesize
            ) VALUES ($1, $2, $3, $4, $5, false, $6) 
            RETURNING userid",
            &[
                &sanitized_username,
                &user_data.email,
                &password_hash,
                &created_at,
                &UserRole::Unconfirmed.to_string(),
                &1.0_f32,
            ],
        )
        .await?;

    let user = User {
        userid: row.get("userid"),
        username: sanitized_username,
        email: user_data.email.clone(),
        password: password_hash,
        created_at,
        followers: 0,
        role: UserRole::Unconfirmed.to_string(),
        email_confirmed: false,
        email_confirmation_token: None,
        email_confirmation_sent_at: None,
    };

    let token = Uuid::new_v4().to_string();

    // Update user with confirmation token
    transaction
        .execute(
            "UPDATE users 
             SET email_confirmation_token = $1,
                 email_confirmation_sent_at = $2
             WHERE userid = $3",
            &[&token, &Utc::now(), &user.userid],
        )
        .await?;

    transaction.commit().await?;

    let email = user.email.clone();
    let confirmation_url = format!(
        "{}/confirm-email?token={}",
        env::var("FRONTEND_URL")
            .map_err(|e| EmailError::ConfigError(format!("FRONTEND_URL not set: {}", e)))?
            .as_str(),
        token
    );

    task::spawn(async move {
        if let Err(e) = send_confirmation_email(&email, &confirmation_url).await {
            error!("Failed to send confirmation email: {}", e); // Keep log
        }
    });

    let token: String = generate_token(&user)?; // generate_token now returns AppResult
    Ok(AuthResponse { token })
}

async fn send_confirmation_email(to_email: &str, confirmation_url: &str) -> Result<(), EmailError> {
    let email_service = EmailService::new().map_err(|e| EmailError::ConfigError(e.to_string()))?;
    let message_content = &[
        "Welcome to Lojban Dictionary!",
        "Please confirm your email address to activate your account.",
        "This link will expire in 24 hours.",
        "",
        "If you didn't create this account, please ignore this email.",
    ];

    let (text_body, html_body) = email_service.build_email_content(
        message_content,
        Some(("Confirm Email", confirmation_url)),
        "en",
    );

    email_service
        .send_notification(EmailNotification {
            to_email: to_email.to_string(),
            subject: "Confirm your email address".to_string(),
            text_body,
            html_body,
        })
        .map_err(|e| EmailError::SendError(e.to_string()))?;

    Ok(())
}

pub async fn resend_confirmation(
    pool: &Pool,
    email_limiter: web::Data<EmailConfirmationLimiter>,
    req: &ResendConfirmationRequest,
) -> AppResult<ResendConfirmationResponse> {
    if !email_limiter.check_rate_limit(&req.email).await? {
        return Ok(ResendConfirmationResponse {
            success: false,
            message: "Too many confirmation requests. Please wait before trying again.".to_string(),
        });
    }

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check if user exists and needs confirmation
    let user = transaction
        .query_opt(
            "SELECT userid, email_confirmed 
             FROM users 
             WHERE email = $1",
            &[&req.email],
        )
        .await?;

    match user {
        Some(row) if !row.get::<_, bool>("email_confirmed") => {
            let user_id: i32 = row.get("userid");
            let token = Uuid::new_v4().to_string();

            // Update confirmation token
            transaction
                .execute(
                    "UPDATE users 
                     SET email_confirmation_token = $1,
                         email_confirmation_sent_at = $2
                     WHERE userid = $3",
                    &[&token, &Utc::now(), &user_id],
                )
                .await?;

            transaction.commit().await?;

            let confirmation_url = format!(
                "{}/confirm-email?token={}",
                env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),
                token
            );

            // Send email asynchronously
            let email = req.email.clone();
            task::spawn(async move {
                if let Err(e) = send_confirmation_email(&email, &confirmation_url).await {
                    error!("Failed to send confirmation email: {}", e); // Keep log
                }
            });

            Ok(ResendConfirmationResponse {
                success: true,
                message: "Confirmation email has been resent".to_string(),
            })
        }
        Some(_) => Ok(ResendConfirmationResponse {
            success: false,
            message: "Email is already confirmed".to_string(),
        }),
        None => Ok(ResendConfirmationResponse {
            success: false,
            message: "If this email exists in our system and requires confirmation, a new confirmation email will be sent".to_string(),
        }),
    }
}

pub async fn login(
    pool: &Pool,
    user_data: &LoginRequest,
    ip_address: String,
    user_agent: String,
) -> AppResult<TokenPair> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client.transaction().await?;

    let user_query_result = transaction
        .query_opt(
            "SELECT userid, username, email, password, created_at, followers,
                    role, email_confirmed
             FROM users
             WHERE username = $1 OR email = $1",
            &[&user_data.username_or_email],
        )
        .await?;

    match user_query_result {
        Some(row) => {
            // Then create the User struct. If User::from doesn't use 'user_uuid', it's fine.
            let user = User::from(row);

            if user.role.to_lowercase() == UserRole::Blocked.to_string() {
                Err(AppError::Auth("Account is blocked".to_string()))
            } else if verify_password(&user_data.password, &user.password).unwrap_or(false) {
                let mut user_for_token = user.clone();

                if needs_rehash(&user_for_token.password) {
                    let new_hash = hash_password(&user_data.password)
                        .map_err(|e| AppError::Auth(format!("Password hashing failed: {}", e)))?;
                    transaction
                        .execute(
                            "UPDATE users SET password = $1 WHERE userid = $2",
                            &[&new_hash, &user_for_token.userid],
                        )
                        .await?;
                    user_for_token.password = new_hash;
                }

                // Start user session before committing and creating token pair
                let mut jwt_session_id: Option<Uuid> = None;
                match sessions::service::start_session(pool, user.userid, ip_address, user_agent)
                    .await
                {
                    Ok(session) => {
                        log::info!(
                            "User session started: id={}, user_id={}",
                            session.id,
                            session.user_id
                        );
                        jwt_session_id = Some(session.session_uuid);
                    }
                    Err(e) => {
                        // Decide if login should fail if session logging fails.
                        // For now, log error and continue.
                        log::error!(
                            "Failed to start user session for user_id {}: {}",
                            user.userid,
                            e
                        );
                        // Optionally, return an error:
                        // return Err(AppError::Internal(format!("Failed to start session: {}", e)));
                    }
                }

                transaction.commit().await?;
                create_token_pair(pool, &user_for_token, jwt_session_id).await
            } else {
                Err(AppError::Auth("Invalid credentials".to_string()))
            }
        }
        None => Err(AppError::Auth("Invalid credentials".to_string())),
    }
}

pub async fn get_profile(pool: &Pool, user_id: i32) -> AppResult<ProfileResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let row = client
        .query_one(
            "SELECT realname, url, personal FROM users WHERE userid = $1",
            &[&user_id],
        )
        .await?;

    Ok(ProfileResponse {
        realname: row.get("realname"),
        url: row.get("url"),
        personal: row.get("personal"),
    })
}

pub async fn update_profile(
    pool: &Pool,
    user_id: i32,
    profile: &UpdateProfileRequest,
) -> AppResult<Option<String>> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client.transaction().await?;

    let mut updates = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut param_count = 1;
    let mut username_changed = false;

    // Handle username update separately due to uniqueness check
    if let Some(new_username) = &profile.username {
        // Check if the new username is different from the current one
        let current_username: String = transaction
            .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
            .await?
            .get("username");

        if new_username != &current_username {
            // Check if the new username is already taken
            let exists = transaction
                .query_opt("SELECT 1 FROM users WHERE username = $1", &[&new_username])
                .await?
                .is_some();

            if exists {
                return Err(AppError::BadRequest("Username already taken".to_string()));
            }
            updates.push(format!("username = ${}", param_count));
            params.push(new_username);
            param_count += 1;
            username_changed = true;
        }
    }

    // Store sanitized values in outer scope so references live long enough
    let sanitized_realname = profile.realname.as_ref().map(|r| sanitize_html(r));
    if let Some(ref realname) = sanitized_realname {
        updates.push(format!("realname = ${}", param_count));
        params.push(realname);
        param_count += 1;
    }
    let sanitized_url = profile.url.as_ref().map(|u| sanitize_html(u));
    if let Some(ref url) = sanitized_url {
        updates.push(format!("url = ${}", param_count));
        params.push(url);
        param_count += 1;
    }
    let sanitized_personal = profile.personal.as_ref().map(|p| sanitize_html(p));
    if let Some(ref personal) = sanitized_personal {
        updates.push(format!("personal = ${}", param_count));
        params.push(personal);
        param_count += 1;
    }

    if updates.is_empty() {
        // No fields to update
        return Ok(None);
    }

    // Add user_id as the last parameter
    params.push(&user_id);
    let user_id_param_index = param_count;

    let query = format!(
        "UPDATE users SET {} WHERE userid = ${}",
        updates.join(", "),
        user_id_param_index
    );

    transaction.execute(&query, &params).await?;

    // If username was changed, generate a new token
    let new_token = if username_changed {
        // Get updated user data for token generation
        let user_row = transaction
            .query_one(
                "SELECT userid, username, email, password, created_at, followers, role, email_confirmed, email_confirmation_token, email_confirmation_sent_at FROM users WHERE userid = $1",
                &[&user_id],
            )
            .await?;

        let user = User::from(user_row);
        Some(generate_token(&user)?)
    } else {
        None
    };

    transaction.commit().await?;
    Ok(new_token)
}

pub async fn request_password_reset(
    pool: &Pool,
    password_reset_limiter: web::Data<PasswordResetLimiter>,
    req: &PasswordResetRequest,
) -> AppResult<PasswordResetResponse> {
    if !password_reset_limiter.check_rate_limit(&req.email).await? {
        info!("Rate limit exceeded for email: {}", req.email);
        return Ok(PasswordResetResponse {
            success: false,
            message: "Too many password reset requests".to_string(),
            session_id: None,
        });
    }

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check if email exists
    let email_exists: bool = transaction
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
            &[&req.email],
        )
        .await?
        .get(0);

    let session_id = Uuid::new_v4().to_string();

    // Always return success to avoid revealing if email exists
    if !email_exists {
        return Ok(PasswordResetResponse {
            success: true,
            message: "If this email exists in our system, a password reset email will be sent"
                .to_string(),
            session_id: Some(session_id),
        });
    }

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("SystemTime error: {}", e)))?
        .as_secs() as i64;

    // Generate token
    let token: String = rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let token_expiry = current_time
        + env::var("TOKEN_EXPIRY_MINUTES")
            .unwrap_or("30".to_string())
            .parse::<i64>()
            .unwrap_or(30)
            * 60;

    // Store request in database
    let result = transaction
        .execute(
            "INSERT INTO password_reset_requests (
            email, session_id, token, token_expiry, created_at
        ) VALUES ($1, $2, $3, $4, $5)",
            &[
                &req.email,
                &session_id,
                &token,
                &token_expiry,
                &current_time,
            ],
        )
        .await;

    match result {
        Ok(_) => info!("Successfully inserted password reset request"),
        Err(e) => {
            error!("Failed to insert password reset request: {}", e);
            return Err(AppError::Database(e.to_string()));
        }
    }

    let reset_url = format!(
        "{}/reset-password?token={}&session_id={}",
        env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),
        token,
        session_id
    );

    transaction.commit().await?;

    let email_service =
        EmailService::new().map_err(|e| AppError::ExternalService(e.to_string()))?;
    let expiry_minutes = env::var("TOKEN_EXPIRY_MINUTES").unwrap_or("30".to_string());
    let message_content = &[
        "We received a request to reset your password.",
        &format!("This link will expire in {} minutes.", expiry_minutes),
        "",
        "If you didn't request this change, you can safely ignore this email.",
    ];

    let (text_body, html_body) = email_service.build_email_content(
        message_content,
        Some(("Reset Password", reset_url.as_str())),
        "en",
    );

    if let Err(e) = email_service.send_notification(EmailNotification {
        to_email: req.email.clone(),
        subject: "Password Reset Request".to_string(),
        text_body,
        html_body,
    }) {
        error!("Failed to send password reset email: {}", e);
        return Err(AppError::ExternalService(format!(
            "Failed to send email: {}",
            e
        )));
    }

    Ok(PasswordResetResponse {
        success: true,
        message: "If this email exists in our system, a password reset email will be sent"
            .to_string(),
        session_id: Some(session_id),
    })
}

pub async fn restore_password(
    pool: &Pool,
    req: &PasswordRestoreRequest,
) -> AppResult<serde_json::Value> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("SystemTime before UNIX EPOCH: {}", e)))?
        .as_secs() as i64;

    // Verify token and session
    let reset_request = transaction
        .query_opt(
            "SELECT email, token_expiry 
             FROM password_reset_requests 
             WHERE session_id = $1 AND token = $2
             AND used = false",
            &[&req.session_id, &req.token],
        )
        .await?;

    let (email, token_expiry) = match reset_request {
        Some(row) => (row.get::<_, String>(0), row.get::<_, i64>(1)),
        None => return Err(AppError::Auth("Invalid token or session ID".to_string())),
    };

    if current_time > token_expiry {
        return Err(AppError::Auth("Token has expired".to_string()));
    }

    // Update password
    let password_hash = hash_password(&req.new_password).map_err(|e| {
        error!("Password hashing failed during restore: {}", e);
        AppError::Internal("Password hashing failed".to_string())
    })?;

    transaction
        .execute(
            "UPDATE users SET password = $1 WHERE email = $2",
            &[&password_hash, &email],
        )
        .await?;

    // Mark token as used
    transaction
        .execute(
            "UPDATE password_reset_requests 
             SET used = true, used_at = $1 
             WHERE session_id = $2",
            &[&current_time, &req.session_id],
        )
        .await?;

    transaction.commit().await?;

    Ok(json!({
        "success": true,
        "message": "Password has been reset successfully"
    }))
}

pub async fn create_token_pair(
    pool: &Pool,
    user: &User,
    session_id: Option<Uuid>,
) -> AppResult<TokenPair> {
    // Get permissions for the user's role (map potential errors)
    // Use case-insensitive role comparison
    let client = pool.get().await?;
    let permissions = client
        .query(
            "SELECT p.name
         FROM role_permissions rp
         JOIN permissions p ON rp.permission_id = p.id
         WHERE LOWER(rp.role::text) = LOWER($1::text)",
            &[&user.role],
        )
        .await?;

    let authorities: Vec<String> = permissions
        .iter()
        .map(|row| row.get::<_, String>("name"))
        .collect();

    let access_token = generate_access_token(user, &authorities, session_id)?;
    let refresh_token = generate_refresh_token(user, session_id)?;

    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

pub fn generate_access_token(
    user: &User,
    authorities: &[String],
    session_id: Option<Uuid>,
) -> AppResult<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.userid,
        exp: expiration,
        username: user.username.clone(),
        email: user.email.clone(),
        created_at: user.created_at.timestamp(),
        role: user.role.clone(),
        email_confirmed: user.email_confirmed,
        authorities: authorities.to_vec(),
        sid: session_id,
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        // Map JWT error
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Auth(format!("Token encoding error: {}", e)))
}

pub fn generate_refresh_token(user: &User, session_id: Option<Uuid>) -> AppResult<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(30))
        .ok_or_else(|| {
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
        })?
        .timestamp();

    let claims = Claims {
        sub: user.userid,
        exp: expiration,
        username: user.username.clone(),
        email: user.email.clone(),
        created_at: user.created_at.timestamp(),
        role: user.role.clone(),
        email_confirmed: user.email_confirmed,
        authorities: Vec::new(), // Not needed for refresh token but required by struct
        sid: session_id,
    };

    let secret = env::var("REFRESH_TOKEN_SECRET").expect("REFRESH_TOKEN_SECRET must be set");
    encode(
        // Map JWT error
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Auth(format!("Token encoding error: {}", e)))
}

pub async fn refresh_tokens(
    pool: &Pool,
    refresh_token: &str,
    ip_address: String,
    user_agent: String,
) -> AppResult<TokenPair> {
    let secret =
        env::var("REFRESH_TOKEN_SECRET").map_err(|e| AppError::Config(vec![e.to_string()]))?;

    // Validate refresh token
    let token_data = decode::<Claims>(
        refresh_token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    let claims = token_data.claims;

    // Attempt to update session activity
    if let Some(session_uuid) = claims.sid {
        match sessions::service::get_session_id_from_uuid(pool, session_uuid).await {
            Ok(Some(session_id_i64)) => {
                match sessions::service::update_session_activity(
                    pool,
                    claims.sub,
                    session_id_i64,
                    ip_address,
                    user_agent,
                )
                .await
                {
                    Ok(Some(session)) => {
                        log::info!(
                            "User session activity updated: session_id={}, user_id={}",
                            session.id,
                            session.user_id
                        );
                    }
                    Ok(None) => {
                        log::warn!("Attempted to update activity for session_uuid {} (user_id {}), but session was not found or already ended.", session_uuid, claims.sub);
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to update session activity for session_id {}: {}",
                            session_id_i64,
                            e
                        );
                    }
                }
            }
            Ok(None) => {
                log::warn!(
                    "No active session found for session_uuid {} during token refresh.",
                    session_uuid
                );
            }
            Err(e) => {
                log::error!(
                    "Failed to get session_id from session_uuid {} during token refresh: {}",
                    session_uuid,
                    e
                );
            }
        }
    } else {
        log::warn!("No session ID (sid) found in refresh token claims for user_id {}. Cannot update session activity.", claims.sub);
    }

    // Get user details for generating new tokens
    let client = pool.get().await?;
    let row = client
        .query_one(
            // Ensure the query for User struct is correct and doesn't conflict with 'id as user_uuid' if that alias was specific to login
            "SELECT userid, username, email, password, created_at, followers, role, email_confirmed FROM users WHERE userid = $1",
            &[&claims.sub],
        )
        .await?;
    let user = User::from(row);

    // Generate new token pair, passing the original session_id from the refresh token
    create_token_pair(pool, &user, claims.sid).await
}

pub async fn set_following(
    pool: &Pool,
    follower_id: i32,
    followee_id: i32,
    wants_follow: bool,
) -> AppResult<FollowResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client.transaction().await?;

    let is_following = transaction
        .query_opt(
            "SELECT 1 FROM follows 
             WHERE follower_id = $1 AND followee_id = $2",
            &[&follower_id, &followee_id],
        )
        .await?
        .is_some();

    if wants_follow == is_following {
        return Ok(FollowResponse {
            success: true,
            message: if wants_follow {
                "Already following".to_string()
            } else {
                "Already not following".to_string()
            },
        });
    }

    if wants_follow {
        transaction
            .execute(
                "INSERT INTO follows (follower_id, followee_id) VALUES ($1, $2)",
                &[&follower_id, &followee_id],
            )
            .await?;
    } else {
        transaction
            .execute(
                "DELETE FROM follows 
                 WHERE follower_id = $1 AND followee_id = $2",
                &[&follower_id, &followee_id],
            )
            .await?;
    }

    transaction.commit().await?;

    Ok(FollowResponse {
        success: true,
        message: if wants_follow {
            "Now following".to_string()
        } else {
            "Unfollowed".to_string()
        },
    })
}

pub async fn initiate_password_change(
    pool: &Pool,
    user_id: i32,
    request: &InitiatePasswordChangeRequest,
) -> AppResult<InitiatePasswordChangeResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client.transaction().await?;

    // Verify current password
    let stored_password: String = transaction
        .query_one("SELECT password FROM users WHERE userid = $1", &[&user_id])
        .await?
        .get("password");

    if !verify_password(&request.current_password, &stored_password)
        .map_err(|e| AppError::Auth(e.to_string()))?
    {
        return Err(AppError::Auth("Invalid password".to_string()));
    }

    // Generate verification data
    let verification_id = Uuid::new_v4().to_string();
    let verification_code: String = rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    // Store verification data
    transaction
        .execute(
            "INSERT INTO password_change_verifications (
                user_id, verification_id, verification_code, expires_at
            ) VALUES ($1, $2, $3, $4)",
            &[
                &user_id,
                &verification_id,
                &verification_code,
                &(Utc::now() + Duration::minutes(30)),
            ],
        )
        .await?;

    // Get user email
    let email: String = transaction
        .query_one("SELECT email FROM users WHERE userid = $1", &[&user_id])
        .await?
        .get("email");

    // Send verification email
    let email_service =
        EmailService::new().map_err(|e| AppError::ExternalService(e.to_string()))?;
    let message_content = &[
        &format!("Your verification code is: {}", verification_code),
        "This code will expire in 30 minutes.",
        "",
        "If you didn't request this change, please contact us immediately.",
    ];

    let (text_body, html_body) = email_service.build_email_content(message_content, None, "en");

    email_service
        .send_notification(EmailNotification {
            to_email: email,
            subject: "Password Change Verification".to_string(),
            text_body,
            html_body,
        })
        .map_err(|e| AppError::ExternalService(e.to_string()))?;

    transaction.commit().await?;

    Ok(InitiatePasswordChangeResponse {
        message: "Verification code sent to your email".to_string(),
        verification_id,
    })
}

pub async fn complete_password_change(
    pool: &Pool,
    user_id: i32,
    request: &CompletePasswordChangeRequest,
) -> AppResult<CompletePasswordChangeResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client.transaction().await?;

    // Verify the code
    let verification = transaction
        .query_opt(
            "SELECT id FROM password_change_verifications 
             WHERE user_id = $1 
             AND verification_id = $2 
             AND verification_code = $3
             AND completed_at IS NULL
             AND expires_at > NOW()",
            &[
                &user_id,
                &request.verification_id,
                &request.verification_code,
            ],
        )
        .await?;

    let verification_id = match verification {
        Some(row) => row.get::<_, i32>("id"),
        None => {
            return Err(AppError::Auth(
                "Invalid verification code or expired".to_string(),
            ))
        }
    };

    // Hash new password
    let password_hash = hash_password(&request.new_password).map_err(|e| {
        error!("Password hashing failed during change: {}", e);
        AppError::Internal("Password hashing failed".to_string())
    })?;

    // Update password
    transaction
        .execute(
            "UPDATE users SET password = $1 WHERE userid = $2",
            &[&password_hash, &user_id],
        )
        .await?;

    // Mark verification as completed
    transaction
        .execute(
            "UPDATE password_change_verifications 
             SET completed_at = NOW() 
             WHERE id = $1",
            &[&verification_id],
        )
        .await?;

    transaction.commit().await?;

    Ok(CompletePasswordChangeResponse {
        success: true,
        message: "Password changed successfully".to_string(),
    })
}

pub async fn google_oauth_signup(pool: &Pool, code: &str, _state: &str) -> AppResult<AuthResponse> {
    use oauth2::{
        AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
    };
    use serde_json::Value;

    let google_client_id = env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set");
    let google_client_secret =
        env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set");
    let google_redirect_url =
        env::var("GOOGLE_REDIRECT_URL").expect("GOOGLE_REDIRECT_URL must be set");

    let client = oauth2::basic::BasicClient::new(
        ClientId::new(google_client_id),
        Some(ClientSecret::new(google_client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(google_redirect_url).unwrap());

    // 1. Verify the state parameter
    // TODO: Implement CSRF token verification

    // 2. Exchange the code for an access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| AppError::Auth(format!("Failed to exchange code for token: {}", e)))?;

    // 3. Get the user's profile information from Google
    let access_token = token_result.access_token().secret();
    let user_info_url = "https://www.googleapis.com/oauth2/v3/userinfo";
    let client = reqwest::Client::new();
    let user_info_response = client
        .get(user_info_url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .send()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to get user info: {}", e)))?;

    if !user_info_response.status().is_success() {
        return Err(AppError::Auth(format!(
            "Failed to get user info: {}",
            user_info_response.status()
        )));
    }

    let user_info: Value = user_info_response
        .json()
        .await
        .map_err(|e| AppError::Auth(format!("Failed to parse user info: {}", e)))?;

    let google_id = user_info["sub"]
        .as_str()
        .ok_or(AppError::Auth("Missing google id".to_string()))?
        .to_string();
    let email = user_info["email"]
        .as_str()
        .ok_or(AppError::Auth("Missing email".to_string()))?
        .to_string();
    let username = user_info["name"]
        .as_str()
        .ok_or(AppError::Auth("Missing name".to_string()))?
        .to_string();

    let mut transaction = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = transaction
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 4. Check if the user already exists in the database
    let existing_user = transaction
        .query_opt(
            "SELECT u.userid, u.username, u.email, u.password, u.created_at, u.followers, u.role, u.email_confirmed FROM users u INNER JOIN oauth_accounts o ON u.userid = o.user_id WHERE o.provider = 'google' AND o.provider_id = $1",
            &[&google_id],
        )
        .await.map_err(|e| AppError::Database(e.to_string()))?;

    match existing_user {
        Some(row) => {
            // 5. If the user exists, log them in
            let user = User::from(row);
            transaction
                .commit()
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            let token_pair = create_token_pair(pool, &user, None).await?;
            Ok(AuthResponse {
                token: token_pair.access_token,
            })
        }
        None => {
            // 6. If the user doesn't exist, create a new user account
            let password_hash = hash_password(&Uuid::new_v4().to_string())
                .map_err(|e| AppError::Auth(format!("Password hashing failed: {}", e)))?;
            let created_at = Utc::now();

            let row = transaction
                .query_one(
                    "INSERT INTO users (
                        username, email, password, created_at,
                        role, email_confirmed, votesize, oauth_signup
                    ) VALUES ($1, $2, $3, $4, $5, true, $6, $7)
                    RETURNING userid",
                    &[
                        &username,
                        &email,
                        &password_hash,
                        &created_at,
                        &UserRole::User.to_string(),
                        &1.0_f32,
                        &true,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            let user_id: i32 = row.get("userid");

            transaction
                .execute(
                    "INSERT INTO oauth_accounts (user_id, provider, provider_id) VALUES ($1, 'google', $2)",
                    &[&user_id, &google_id],
                )
                .await.map_err(|e| AppError::Database(e.to_string()))?;

            let user = User {
                userid: user_id,
                username: username.clone(),
                email: email.clone(),
                password: password_hash,
                created_at,
                followers: 0,
                role: UserRole::User.to_string(),
                email_confirmed: true,
                email_confirmation_token: None,
                email_confirmation_sent_at: None,
            };

            transaction
                .commit()
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            // 7. Generate a JWT token and return it
            let token_pair = create_token_pair(pool, &user, None).await?;
            Ok(AuthResponse {
                token: token_pair.access_token,
            })
        }
    }
}
