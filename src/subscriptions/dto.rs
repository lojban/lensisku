use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::models::{Notification, SubscriptionTrigger};

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SubscriptionRequest {
    pub valsi_id: i32,
    pub trigger_type: SubscriptionTrigger,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubscriptionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NotificationListResponse {
    pub notifications: Vec<Notification>,
    pub total: i64,
    pub unread_count: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NotificationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub unread_only: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubscriptionState {
    pub is_subscribed: bool,
    pub subscriptions: Vec<SubscriptionTrigger>,
}
