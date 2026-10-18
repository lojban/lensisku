use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Subscription {
    pub subscription_id: i32,
    pub valsi_id: i32,
    pub user_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub unsubscribed: bool,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub unsubscribed_at: Option<DateTime<Utc>>,
    pub trigger_type: SubscriptionTrigger,
    pub source_definition_id: Option<i32>,
    pub source_comment_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, ToSql, FromSql)]
#[postgres(name = "subscription_trigger")]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTrigger {
    #[postgres(name = "comment")]
    Comment,
    #[postgres(name = "definition")]
    Definition,
    #[postgres(name = "edit")]
    Edit,
}

impl SubscriptionTrigger {
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "comment" => Some(Self::Comment),
            "definition" => Some(Self::Definition),
            "edit" => Some(Self::Edit),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Notification {
    pub notification_id: i32,
    pub user_id: i32,
    pub notification_type: String,
    pub message: String,
    pub link: Option<String>,
    pub valsi_id: Option<i32>,
    pub actor_id: Option<i32>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub read_at: Option<DateTime<Utc>>,
}

impl From<tokio_postgres::Row> for Notification {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            notification_id: row.get("notification_id"),
            user_id: row.get("user_id"),
            notification_type: row.get("notification_type"),
            message: row.get("message"),
            link: row.get("link"),
            valsi_id: row.get("valsi_id"),
            actor_id: row.get("actor_id"),
            created_at: row.get("created_at"),
            read_at: row.get("read_at"),
        }
    }
}

impl From<tokio_postgres::Row> for Subscription {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            subscription_id: row.get("subscription_id"),
            valsi_id: row.get("valsi_id"),
            user_id: row.get("user_id"),
            created_at: row.get("created_at"),
            unsubscribed: row.get("unsubscribed"),
            unsubscribed_at: row.get("unsubscribed_at"),
            trigger_type: row.get("trigger_type"),
            source_definition_id: row.get("source_definition_id"),
            source_comment_id: row.get("source_comment_id"),
        }
    }
}
