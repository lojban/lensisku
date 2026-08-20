use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EmailConfirmationRequest {
    pub token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileResponse {
    pub realname: Option<String>,
    pub url: Option<String>,
    pub personal: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub realname: Option<String>,
    pub url: Option<String>,
    pub personal: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PasswordResetResponse {
    pub success: bool,
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasswordRestoreRequest {
    pub token: String,
    pub session_id: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FollowRequest {
    pub followee_id: i32,
    pub follow: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FollowResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResendConfirmationRequest {
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResendConfirmationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InitiatePasswordChangeRequest {
    pub current_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompletePasswordChangeRequest {
    pub verification_id: String,
    pub verification_code: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CompletePasswordChangeResponse {
    pub message: String,
    pub success: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionsListResponse {
    pub permissions: Vec<PermissionInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionInfo {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserRoleResponse {
    pub roles: Vec<RoleWithPermissions>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleWithPermissions {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRoleRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleResponse {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignRoleRequest {
    pub user_id: i32,
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AssignRoleResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BlockUserRequest {
    pub user_id: i32,
    pub block: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BlockUserResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InitiatePasswordChangeResponse {
    pub message: String,
    pub verification_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthProvidersResponse {
    pub providers: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OAuthAuthorizeQuery {
    pub return_to: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthAuthorizeResponse {
    pub authorize_url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OAuthCompleteRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthCompleteResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub username: String,
    pub return_to: Option<String>,
}
