use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Version {
    pub version_id: i32,
    pub definition_id: i32,
    pub user_id: i32,
    pub username: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub content: VersionContent,
    pub commit_message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct VersionContent {
    pub definition: String,
    pub notes: Option<String>,
    pub selmaho: Option<String>,
    pub jargon: Option<String>,
    pub rafsi: Option<String>,
    pub gloss_keywords: Option<Vec<crate::jbovlaste::KeywordMapping>>,
    pub place_keywords: Option<Vec<crate::jbovlaste::KeywordMapping>>,
    /// Whether this version had an image attached to the definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_image: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VersionDiff {
    pub old_content: VersionContent,
    pub new_content: VersionContent,
    pub changes: Vec<Change>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Change {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_type: ChangeType,
    /// For image-type changes: URL to fetch the image (old or new). Present only when `field == "image"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}
