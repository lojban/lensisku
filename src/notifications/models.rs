use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailNotification {
    pub to_email: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
    pub initiated_by_user_id: Option<i32>,
    pub params: serde_json::Value,
}
