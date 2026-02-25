use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Collection {
    pub collection_id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CollectionItem {
    pub item_id: i32,
    pub collection_id: i32,
    pub definition_id: i32,
    pub notes: Option<String>,
    pub position: i32,
    #[schema(value_type = String, format = DateTime)]
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageData {
    #[schema(format = "byte")]
    pub data: String, // Base64 encoded image data
    pub mime_type: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SoundData {
    #[schema(format = "byte")]
    pub data: String, // Base64 encoded audio data
    pub mime_type: String,
}
