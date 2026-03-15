use crate::auth::permissions::PermissionCache;
use crate::sessions;
use crate::AppError;
use actix_web::{delete, get, http::Error, post, put, web, HttpResponse, Responder};
use actix_web_grants::protect;
use deadpool_postgres::Pool;
use serde_json::json;

use crate::{
    auth::{
        dto::{AuthResponse, LoginRequest, PasswordResetRequest, PasswordResetResponse},
        email_confirmation, service, Claims, CompletePasswordChangeRequest,
        EmailConfirmationRequest, FollowRequest, RefreshTokenRequest, ResendConfirmationRequest,
    },
    middleware::limiter::{EmailConfirmationLimiter, LoginLimiter, PasswordResetLimiter},
};

use super::dto::*;

#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "auth",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Tokens refreshed successfully", body = inline(serde_json::Value), example = json!({"access_token": "new_access_token", "refresh_token": "new_refresh_token"})),
        (status = 401, description = "Invalid refresh token", body = String, example = "Invalid refresh token")
    ),
    summary = "Refresh authentication tokens",
    description = "Exchange a valid refresh token for a new pair of access and refresh tokens. This endpoint should be used when the access token has expired but the refresh token is still valid. If the refresh token is invalid or expired, the user will need to log in again."
)]
#[post("/refresh")]
pub async fn refresh_token(
    pool: web::Data<Pool>,
    request: web::Json<RefreshTokenRequest>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    let user_agent = req
        .headers()
        .get("User-Agent")
        .map_or("unknown".to_string(), |h| {
            h.to_str().unwrap_or("unknown").to_string()
        });

    match service::refresh_tokens(&pool, &request.refresh_token, ip_address, user_agent).await {
        Ok(token_pair) => Ok(HttpResponse::Ok().json(json!({
            "access_token": token_pair.access_token,
            "refresh_token": token_pair.refresh_token
        }))),
        Err(e) => Ok(HttpResponse::Unauthorized().json(json!({
            "error": "Invalid refresh token",
            "details": e.to_string()
        }))),
    }
}

#[utoipa::path(
    post,
    path = "/auth/roles",
    tag = "auth",
    request_body(content = CreateRoleRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Role created successfully", body = RoleResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = ["manage_roles"])),
    summary = "Create new role",
    description = "Creates a new role with specified permissions. Requires manage_roles permission."
)]
#[post("/roles")]
#[protect(any("manage_roles"))]
pub async fn create_role(
    pool: web::Data<Pool>,
    perm_cache: web::Data<PermissionCache>,
    request: web::Json<CreateRoleRequest>,
    claims: Claims,
) -> impl Responder {
    match service::create_role(&pool, &request, &claims.role.to_string()).await {
        Ok(role) => {
            if let Err(e) = perm_cache.load_permissions().await {
                return HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to reload permissions: {}", e)
                }));
            }
            HttpResponse::Created().json(role)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to create role: {}", e)
        })),
    }
}

#[utoipa::path(
    put,
    path = "/auth/roles/{role_name}",
    tag = "auth",
    request_body(content = UpdateRoleRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Role updated successfully", body = RoleResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = ["manage_roles"])),
    summary = "Update existing role",
    description = "Updates permissions for an existing role. Requires manage_roles permission."
)]
#[put("/roles/{role_name}")]
#[protect("manage_roles")]
pub async fn update_role(
    pool: web::Data<Pool>,
    perm_cache: web::Data<PermissionCache>,
    role_name: web::Path<String>,
    request: web::Json<UpdateRoleRequest>,
    claims: Claims,
) -> impl Responder {
    match service::update_role(&pool, &role_name, &request, &claims.role.to_string()).await {
        Ok(role) => {
            if let Err(e) = perm_cache.load_permissions().await {
                return HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to reload permissions: {}", e)
                }));
            }
            HttpResponse::Ok().json(role)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to update role: {}", e)
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/auth/roles/{role_name}",
    tag = "auth",
    responses(
        (status = 204, description = "Role deleted successfully"),
        (status = 400, description = "Cannot delete protected role"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = ["manage_roles"])),
    summary = "Delete role",
    description = "Deletes a role and converts existing users to 'user' role. Requires manage_roles permission."
)]
#[delete("/roles/{role_name}")]
#[protect(any("manage_roles"))]
pub async fn delete_role(
    pool: web::Data<Pool>,
    perm_cache: web::Data<PermissionCache>,
    role_name: web::Path<String>,
) -> impl Responder {
    match service::delete_role(&pool, &role_name).await {
        Ok(_) => {
            if let Err(e) = perm_cache.load_permissions().await {
                return HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("Failed to reload permissions: {}", e)
                }));
            }
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": format!("Role '{}' deleted successfully", role_name)
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": format!("Failed to delete role: {}", e)
        })),
    }
}

#[utoipa::path(
    post,
    path = "/auth/block-user",
    tag = "auth",
    request_body = BlockUserRequest,
    responses(
        (status = 200, description = "User blocked/unblocked successfully", body = BlockUserResponse),
        (status = 403, description = "Insufficient permissions", body = BlockUserResponse),
        (status = 404, description = "User not found", body = BlockUserResponse),
        (status = 500, description = "Internal server error", body = BlockUserResponse)
    ),
    security(("bearer_auth" = [])),
    summary = "Block or unblock a user",
    description = "Block or unblock a user's account. Requires appropriate permissions."
)]
#[post("/block-user")]
pub async fn block_user(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<BlockUserRequest>,
) -> impl Responder {
    match service::block_user(&pool, claims.sub, request.user_id, request.block).await {
        Ok(_) => HttpResponse::Ok().json(BlockUserResponse {
            success: true,
            message: if request.block {
                "User blocked successfully"
            } else {
                "User unblocked successfully"
            }
            .to_string(),
        }),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Insufficient permissions") {
                HttpResponse::Forbidden().json(BlockUserResponse {
                    success: false,
                    message: error_msg,
                })
            } else if error_msg.contains("User not found") {
                HttpResponse::NotFound().json(BlockUserResponse {
                    success: false,
                    message: error_msg,
                })
            } else {
                HttpResponse::InternalServerError().json(BlockUserResponse {
                    success: false,
                    message: format!("Failed to update user status: {}", error_msg),
                })
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/auth/assign-role",
    tag = "auth",
    request_body = AssignRoleRequest,
    responses(
        (status = 200, description = "Role assigned successfully", body = AssignRoleResponse),
        (status = 403, description = "Insufficient permissions", body = AssignRoleResponse),
        (status = 404, description = "User not found", body = AssignRoleResponse),
        (status = 500, description = "Internal server error", body = AssignRoleResponse)
    ),
    security(("bearer_auth" = [])),
    summary = "Assign role to user",
    description = "Assign a new role to a user. Requires appropriate permissions and prevents privilege escalation."
)]
#[post("/assign-role")]
pub async fn assign_role(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<AssignRoleRequest>,
) -> impl Responder {
    match service::assign_role(&pool, claims.sub, request.user_id, request.role.clone()).await {
        Ok(_) => HttpResponse::Ok().json(AssignRoleResponse {
            success: true,
            message: "Role assigned successfully".to_string(),
        }),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Insufficient permissions") {
                HttpResponse::Forbidden().json(AssignRoleResponse {
                    success: false,
                    message: error_msg,
                })
            } else {
                HttpResponse::InternalServerError().json(AssignRoleResponse {
                    success: false,
                    message: format!("Failed to assign role: {}", error_msg),
                })
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 200, description = "User successfully created", body = AuthResponse),
        (status = 409, description = "Username or email already exists", body = String),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    summary = "Create new user account",
    description = "Register a new user account with the system. Requires unique username and email. Upon successful registration, returns authentication tokens that can be used to access protected endpoints. The password must meet the system's security requirements."
)]
#[post("/signup")]
pub async fn signup(pool: web::Data<Pool>, user_data: web::Json<SignupRequest>) -> impl Responder {
    match service::signup(&pool, &user_data).await {
        Ok(response) => HttpResponse::Ok().json(json!({
            "message": "Account created successfully. Please check your email to confirm your address.",
            "token": response.token
        })),
        Err(e) => match e {
            AppError::Auth(msg) if msg.contains("already exists") => {
                HttpResponse::Conflict().json(json!({
                    "error": "user_exists",
                    "error_description": "Username or email already exists"
                }))
            },
            _ => HttpResponse::InternalServerError().json(json!({"error": format!("Signup failed: {}", e)})),
        },
    }
}

#[utoipa::path(
    post,
    path = "/auth/confirm-email",
    tag = "auth",
    request_body = EmailConfirmationRequest,
    responses(
        (status = 200, description = "Email confirmed successfully"),
        (status = 400, description = "Invalid or expired token"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Confirm user email address",
    description = "Confirms a user's email address using the token sent to their email"
)]
#[post("/confirm-email")]
pub async fn confirm_email(
    pool: web::Data<Pool>,
    req: web::Json<EmailConfirmationRequest>,
    email_limiter: web::Data<EmailConfirmationLimiter>,
) -> impl Responder {
    match email_confirmation::confirm_email(&pool, &req.token, &email_limiter).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Email confirmed successfully"
        })),
        Err(e) => {
            if e.to_string().contains("expired") {
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "error": "Confirmation token has expired"
                }))
            } else if e.to_string().contains("Invalid") {
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "error": "Invalid confirmation token"
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("Failed to confirm email: {}", e)
                }))
            }
        }
    }
}

#[utoipa::path(
    post,
    tag = "auth",
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = String),
        (status = 429, description = "Too many login attempts", body = String),
        (status = 500, description = "Internal server error")
    ),
    summary = "Authenticate user",
    description = "Authenticate a user with their credentials and receive access and refresh tokens. Rate limited by IP and by failed attempts per identifier."
)]
#[post("/login")]
pub async fn login(
    pool: web::Data<Pool>,
    login_limiter: web::Data<LoginLimiter>,
    user_data: web::Json<LoginRequest>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    let user_agent = req
        .headers()
        .get("User-Agent")
        .map_or("unknown".to_string(), |h| {
            h.to_str().unwrap_or("unknown").to_string()
        });

    if !login_limiter
        .check_and_record_attempt(&ip_address)
        .await
        .unwrap_or(false)
    {
        return Ok(HttpResponse::TooManyRequests().json(
            "Too many login attempts from this IP. Please try again later.",
        ));
    }

    if login_limiter
        .is_identifier_over_failure_limit(&user_data.username_or_email)
        .await
        .unwrap_or(false)
    {
        return Ok(HttpResponse::TooManyRequests().json(
            "Too many failed attempts for this account. Please try again later.",
        ));
    }

    match service::login(&pool, &user_data, ip_address, user_agent).await {
        Ok(token_pair) => Ok(HttpResponse::Ok().json(json!({
            "access_token": token_pair.access_token,
            "refresh_token": token_pair.refresh_token
        }))),
        Err(_) => {
            if login_limiter
                .record_failed_login(&user_data.username_or_email)
                .await
                .is_err()
            {
                log::warn!("Failed to record login failure for rate limiting");
            }
            Ok(HttpResponse::Unauthorized().json("Invalid credentials"))
        }
    }
}

#[utoipa::path(
    get,
    tag = "auth",
    path = "/auth/profile",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = ProfileResponse),
        (status = 404, description = "Profile not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get user profile",
    description = "Retrieve the profile information for the currently authenticated user. Requires a valid access token in the Authorization header. Returns user profile data including personal information and preferences."
)]
#[get("/profile")]
pub async fn get_profile(pool: web::Data<Pool>, claims: Claims) -> impl Responder {
    match service::get_profile(&pool, claims.sub).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[utoipa::path(
    put,
    tag = "auth",
    path = "/auth/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Update user profile",
    description = "Update the profile information for the currently authenticated user. Requires a valid access token in the Authorization header. Allows modification of user profile data such as personal information and preferences. All fields in the request are optional and only provided fields will be updated."
)]
#[put("/profile")]
pub async fn update_profile(
    pool: web::Data<Pool>,
    claims: Claims,
    profile: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    match service::update_profile(&pool, claims.sub, &profile).await {
        Ok(new_token) => {
            if let Some(token) = new_token {
                // Username was changed, return new token
                HttpResponse::Ok().json(json!({
                    "message": "Profile updated successfully",
                    "new_token": token
                }))
            } else {
                // No username change, return success message only
                HttpResponse::Ok().json("Profile updated successfully")
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error updating profile: {}", e))
        }
    }
}

#[utoipa::path(
    post,
    tag = "auth",
    path = "/auth/request_password_reset",
    request_body = PasswordResetRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = PasswordResetResponse),
        (status = 429, description = "Too many requests", body = PasswordResetResponse),
        (status = 500, description = "Internal server error", body = PasswordResetResponse)
    ),
    summary = "Request password reset",
    description = "Initiate the password reset process by requesting a reset email. Requires the user's email address. A reset token and session ID will be generated and sent to the provided email address. Rate limiting is applied to prevent abuse. Even if the email doesn't exist in the system, a success response will be returned for security reasons."
)]
#[post("/request_password_reset")]
pub async fn request_password_reset(
    pool: web::Data<Pool>,
    password_limiter: web::Data<PasswordResetLimiter>,
    req: web::Json<PasswordResetRequest>,
) -> impl Responder {
    match service::request_password_reset(&pool, password_limiter, &req).await {
        Ok(response) => {
            if response.success {
                HttpResponse::Ok().json(response)
            } else {
                HttpResponse::TooManyRequests().json(response)
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(PasswordResetResponse {
            success: false,
            message: format!("Failed to process request: {}", e),
            session_id: None,
        }),
    }
}

#[utoipa::path(
    post,
    tag = "auth",
    path = "/auth/restore_password",
    request_body = PasswordRestoreRequest,
    responses(
        (status = 200, description = "Password successfully reset", body = serde_json::Value),
        (status = 400, description = "Invalid token or session ID", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    summary = "Reset user password",
    description = "Complete the password reset process by setting a new password. Requires the reset token and session ID received from the password reset email, along with the new password. The new password must meet the system's security requirements. The reset token is single-use and expires after a set time period."
)]
#[post("/restore_password")]
pub async fn restore_password(
    pool: web::Data<Pool>,
    req: web::Json<PasswordRestoreRequest>,
) -> impl Responder {
    match service::restore_password(&pool, &req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            if e.to_string().contains("Invalid token") {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": e.to_string()
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to restore password: {}", e)
                }))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/users/follow",
    tag = "users",
    request_body = FollowRequest,
    responses(
        (status = 200, description = "Follow action completed successfully", body = FollowResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Follow or unfollow a user",
    description = "Sets the follow status for the current user in relation to another user"
)]
#[post("/follow")]
pub async fn set_following(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<FollowRequest>,
) -> impl Responder {
    match service::set_following(&pool, claims.sub, request.followee_id, request.follow).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to update follow status: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/auth/resend-confirmation",
    tag = "auth",
    request_body = ResendConfirmationRequest,
    responses(
        (status = 200, description = "Confirmation email resent successfully", body = ResendConfirmationResponse),
        (status = 429, description = "Too many requests", body = ResendConfirmationResponse),
        (status = 500, description = "Internal server error", body = ResendConfirmationResponse)
    ),
    summary = "Resend email confirmation token",
    description = "Resends the email confirmation token for unconfirmed accounts. Implements exponential backoff rate limiting."
)]
#[post("/resend-confirmation")]
pub async fn resend_confirmation(
    pool: web::Data<Pool>,
    email_limiter: web::Data<EmailConfirmationLimiter>,
    req: web::Json<ResendConfirmationRequest>,
) -> impl Responder {
    match service::resend_confirmation(&pool, email_limiter, &req).await {
        Ok(response) => {
            if response.success {
                HttpResponse::Ok().json(response)
            } else {
                HttpResponse::TooManyRequests().json(response)
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ResendConfirmationResponse {
            success: false,
            message: format!("Failed to process request: {}", e),
        }),
    }
}

#[utoipa::path(
    post,
    path = "/auth/change-password/initiate",
    tag = "auth",
    request_body = InitiatePasswordChangeRequest,
    responses(
        (status = 200, description = "Password change initiated", body = InitiatePasswordChangeResponse),
        (status = 401, description = "Current password is incorrect"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Initiate password change process",
    description = "Start the password change process by verifying current password and sending verification code via email"
)]
#[post("/change-password/initiate")]
pub async fn initiate_password_change(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<InitiatePasswordChangeRequest>,
) -> impl Responder {
    match service::initiate_password_change(&pool, claims.sub, &request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            if e.to_string().contains("Invalid password") {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Current password is incorrect"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to initiate password change: {}", e)
                }))
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/auth/permissions",
    responses(
        (status = 200, description = "List of all system permissions", body = PermissionsListResponse),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    summary = "Get available permissions",
    description = "Returns a list of all permissions available in the system",
    security(("bearer_auth" = ["manage_roles"]))
)]
#[get("/permissions")]
pub async fn get_permissions(pool: web::Data<Pool>) -> impl Responder {
    match service::list_permissions(&pool).await {
        Ok(permissions) => HttpResponse::Ok().json(PermissionsListResponse { permissions }),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to fetch permissions: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/auth/roles",
    responses(
        (status = 200, description = "List of roles with permissions", body = UserRoleResponse),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    summary = "Get roles with permissions",
    description = "Returns list of roles that the current user can assign, showing only permissions they possess",
    security(("bearer_auth" = ["manage_roles"]))
)]
#[get("/roles")]
pub async fn get_roles(pool: web::Data<Pool>, claims: Claims) -> impl Responder {
    match service::get_roles_with_permissions(&pool, &claims.role.to_string()).await {
        Ok(roles) => HttpResponse::Ok().json(UserRoleResponse { roles }),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get roles: {}", e)
        })),
    }
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logout successful", body = inline(serde_json::Value), example = json!({"message": "Logout successful"})),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found or no active session", body = inline(serde_json::Value), example = json!({"error": "User not found or no active session to end"})),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Log out user",
    description = "Logs out the currently authenticated user by ending their active session."
)]
#[post("/logout")]
pub async fn logout(pool: web::Data<Pool>, claims: Claims) -> Result<HttpResponse, Error> {
    match sessions::service::end_session(&pool, claims.sub).await {
        Ok(Some(_session)) => Ok(HttpResponse::Ok().json(json!({"message": "Logout successful"}))),
        Ok(None) => {
            // This case means the user was found, but they had no active session to end.
            // This can be treated as a successful logout from the user's perspective.
            log::info!(
                "User {} logged out, but no active session was found to end.",
                claims.sub
            );
            Ok(HttpResponse::Ok()
                .json(json!({"message": "Logout successful, no active session to end."})))
        }
        Err(e) => {
            log::error!("Error ending session for user_id {}: {}", claims.sub, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to end session"
            })))
        }
    }
}

#[utoipa::path(
    post,
    path = "/auth/change-password/complete",
    tag = "auth",
    request_body = CompletePasswordChangeRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = CompletePasswordChangeResponse),
        (status = 400, description = "Invalid verification code"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    summary = "Complete password change",
    description = "Complete the password change process by verifying the code sent via email and setting the new password"
)]
#[post("/change-password/complete")]
pub async fn complete_password_change(
    pool: web::Data<Pool>,
    claims: Claims,
    request: web::Json<CompletePasswordChangeRequest>,
) -> impl Responder {
    match service::complete_password_change(&pool, claims.sub, &request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            if e.to_string().contains("Invalid verification") {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid verification code"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to complete password change: {}", e)
                }))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/auth/google_oauth_signup",
    tag = "auth",
    request_body = GoogleOAuthSignupRequest,
    responses(
        (status = 200, description = "User successfully created or logged in with Google OAuth", body = AuthResponse),
        (status = 500, description = "Internal server error")
    ),
    summary = "Sign up or log in with Google OAuth2",
    description = "Handles user signup or login via Google OAuth2.  It expects a code and state parameters from the frontend."
)]
#[post("/google_oauth_signup")]
pub async fn google_oauth_signup(
    pool: web::Data<Pool>,
    req: web::Json<GoogleOAuthSignupRequest>,
    request: actix_web::HttpRequest,
) -> impl Responder {
    // Verify CSRF token from state parameter
    if req.state.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "invalid_request",
            "error_description": "Missing state parameter"
        }));
    }

    // Get session cookie
    let session_cookie = match request.cookie("session") {
        Some(cookie) => cookie,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "invalid_request",
                "error_description": "Session cookie not found"
            }))
        }
    };

    // Verify state matches expected CSRF token from session
    if req.state != session_cookie.value() {
        return HttpResponse::BadRequest().json(json!({
            "error": "invalid_request",
            "error_description": "Invalid state parameter"
        }));
    }

    match service::google_oauth_signup(&pool, &req.code, &req.state).await {
        Ok(response) => HttpResponse::Ok().json(json!({
            "message": "Google OAuth signup/login successful",
            "token": response.token
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Google OAuth signup/login failed: {}", e)
        })),
    }
}
