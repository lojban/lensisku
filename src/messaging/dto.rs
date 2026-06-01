use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::models::{ThreadType, ParticipantRole, MessageType, NotificationType, SignalType};

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateThreadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_name: Option<String>,
    pub thread_type: ThreadType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_ids: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_participants: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateThreadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_participants: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct SendMessageRequest {
    pub thread_id: i64,
    pub message_type: MessageType,
    pub encrypted_content: String,
    pub content_nonce: String, // Base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_key_signature: Option<String>, // Base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateMessageRequest {
    pub encrypted_content: String,
    pub content_nonce: String, // Base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_key_signature: Option<String>, // Base64 encoded
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct AddParticipantRequest {
    pub user_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ParticipantRole>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateParticipantRoleRequest {
    pub role: ParticipantRole,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct BlockUserRequest {
    pub user_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct WebRTCSignalRequest {
    pub to_user_id: i32,
    pub signal_type: SignalType,
    pub signal_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ThreadResponse {
    pub thread_id: i64,
    pub thread_name: Option<String>,
    pub thread_type: ThreadType,
    pub created_by: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub max_participants: i32,
    pub last_message_at: Option<DateTime<Utc>>,
    pub last_message_preview: Option<String>,
    pub message_count: i64,
    pub unread_count: i64,
    pub participant_count: i64,
    pub is_admin: bool,
    pub participants: Vec<ThreadParticipantResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ThreadParticipantResponse {
    pub participant_id: i64,
    pub user_id: i32,
    pub username: String,
    pub role: ParticipantRole,
    pub joined_at: DateTime<Utc>,
    pub is_active: bool,
    pub unread_count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MessageResponse {
    pub message_id: i64,
    pub thread_id: i64,
    pub sender_id: i32,
    pub username: String,
    pub message_type: MessageType,
    pub encrypted_content: String,
    pub content_nonce: String, // Base64 encoded for JSON
    pub sender_key_signature: Option<String>, // Base64 encoded for JSON
    pub reply_to_message_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub edit_count: i32,
    pub last_edited_at: Option<DateTime<Utc>>,
    pub is_from_sender: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub total_count: i64,
    pub page: i64,
    pub per_page: i64,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ThreadListResponse {
    pub threads: Vec<ThreadResponse>,
    pub total_count: i64,
    pub page: i64,
    pub per_page: i64,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct NotificationResponse {
    pub notification_id: i64,
    pub thread_id: i64,
    pub thread_name: Option<String>,
    pub message_id: i64,
    pub sender_id: i32,
    pub sender_username: String,
    pub notification_type: NotificationType,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NotificationListResponse {
    pub notifications: Vec<NotificationResponse>,
    pub unread_count: i64,
    pub total_count: i64,
    pub page: i64,
    pub per_page: i64,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BlockedUserResponse {
    pub block_id: i64,
    pub blocked_id: i32,
    pub blocked_username: String,
    pub blocked_at: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebRTCSignalResponse {
    pub signal_id: i64,
    pub from_user_id: i32,
    pub from_username: String,
    pub signal_type: SignalType,
    pub signal_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EncryptionKeyResponse {
    pub key_id: i64,
    pub thread_id: i64,
    pub encrypted_key: String,
    pub key_algorithm: String,
    pub key_version: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ThreadStatsResponse {
    pub thread_id: i64,
    pub message_count: i64,
    pub participant_count: i64,
    pub active_participant_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserPresenceResponse {
    pub user_id: i32,
    pub username: String,
    pub is_online: bool,
    pub last_seen: Option<DateTime<Utc>>,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "message_sent")]
    MessageSent {
        message: MessageResponse,
        thread_id: i64,
    },
    #[serde(rename = "message_updated")]
    MessageUpdated {
        message: MessageResponse,
        thread_id: i64,
    },
    #[serde(rename = "message_deleted")]
    MessageDeleted {
        message_id: i64,
        thread_id: i64,
        deleted_by: i32,
    },
    #[serde(rename = "thread_updated")]
    ThreadUpdated {
        thread: ThreadResponse,
    },
    #[serde(rename = "participant_added")]
    ParticipantAdded {
        thread_id: i64,
        participant: ThreadParticipantResponse,
    },
    #[serde(rename = "participant_removed")]
    ParticipantRemoved {
        thread_id: i64,
        user_id: i32,
        removed_by: i32,
    },
    #[serde(rename = "participant_role_updated")]
    ParticipantRoleUpdated {
        thread_id: i64,
        user_id: i32,
        new_role: ParticipantRole,
        updated_by: i32,
    },
    #[serde(rename = "typing_started")]
    TypingStarted {
        thread_id: i64,
        user_id: i32,
        username: String,
    },
    #[serde(rename = "typing_stopped")]
    TypingStopped {
        thread_id: i64,
        user_id: i32,
        username: String,
    },
    #[serde(rename = "user_online")]
    UserOnline {
        user_id: i32,
        username: String,
    },
    #[serde(rename = "user_offline")]
    UserOffline {
        user_id: i32,
        username: String,
    },
    #[serde(rename = "new_notification")]
    NewNotification {
        notification: NotificationResponse,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
        code: Option<String>,
    },
}

// Query parameters
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct GetThreadsQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub thread_type: Option<ThreadType>,
    #[serde(default)]
    #[allow(dead_code)]
    pub unread_only: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct GetMessagesQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    #[serde(default)]
    pub before_message_id: Option<i64>,
    #[serde(default)]
    pub after_message_id: Option<i64>,
    #[serde(default)]
    pub message_type: Option<MessageType>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct GetNotificationsQuery {
    #[serde(default = "default_page")]
    #[allow(dead_code)]
    pub page: i64,
    #[serde(default = "default_per_page")]
    #[allow(dead_code)]
    pub per_page: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub unread_only: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub notification_type: Option<NotificationType>,
}

// Helper functions for defaults
fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}
