use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Message {
    pub id: i32,
    pub message_id: Option<String>,
    pub date: Option<String>,
    pub subject: Option<String>,
    pub cleaned_subject: Option<String>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    pub spam_vote_count: i64,
    pub current_user_voted_spam: Option<bool>,
}

impl From<Row> for Message {
    fn from(row: Row) -> Self {
        let file_path = row.try_get("file_path").unwrap_or_default();

        Message {
            id: row.get("id"),
            message_id: row.get("message_id"),
            date: row.get("date"),
            subject: row.get("subject"),
            cleaned_subject: row.try_get("cleaned_subject").unwrap_or_default(),
            from_address: row.get("from_address"),
            to_address: row.get("to_address"),
            parts_json: row.try_get::<_, Option<serde_json::Value>>("parts_json").ok().flatten(),
            file_path,
            spam_vote_count: row.try_get("spam_vote_count").unwrap_or(0),
            current_user_voted_spam: row.try_get("current_user_voted_spam").ok(),
        }
    }
}
