use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    #[schema(example = "user")]
    pub role: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserListResponse {
    pub users: Vec<UserInfo>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub user_id: i32,
    pub username: String,
    pub realname: Option<String>,
    pub email: Option<String>,
    pub personal: Option<String>,
    pub url: Option<String>,
    pub is_enabled: bool,
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PublicUserProfile {
    pub username: String,
    pub role: String,
    pub realname: Option<String>,
    pub url: Option<String>,
    pub user_id: i32,
    pub personal: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub join_date: DateTime<Utc>,
    pub definition_count: i64,
    pub comment_count: i64,
    pub vote_count: i64,
    pub has_profile_image: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ContributionsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Definition {
    pub word: String,
    pub version_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub definitionid: i32,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct ContributionsResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Vote {
    pub definition_id: i32,
    pub valsi_word: String,
    pub definition: String,
    pub language: String,
    pub langid: i32,
    pub vote_value: i32,
    pub score: i32,
    #[schema(value_type = String, format = DateTime)]
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProfileImageResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProfileImageRequest {
    #[schema(format = "binary")]
    pub data: String, // Base64 encoded image data
    pub mime_type: String,
}
