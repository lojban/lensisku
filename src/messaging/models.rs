use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThreadType {
    Direct,
    Group,
}

impl fmt::Display for ThreadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadType::Direct => write!(f, "direct"),
            ThreadType::Group => write!(f, "group"),
        }
    }
}

impl FromSql<'_> for ThreadType {
    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "thread_type" || ty.name() == "text" || ty.name() == "varchar"
    }

    fn from_sql(
        _ty: &postgres_types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = String::from_utf8(raw.to_vec())?;
        match value.as_str() {
            "direct" => Ok(ThreadType::Direct),
            "group" => Ok(ThreadType::Group),
            _ => Err(format!("Unknown thread type: {}", value).into()),
        }
    }
}

impl ToSql for ThreadType {
    fn to_sql(
        &self,
        _ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        out.extend_from_slice(self.to_string().as_bytes());
        Ok(postgres_types::IsNull::No)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar"
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantRole {
    Admin,
    Member,
}

impl fmt::Display for ParticipantRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParticipantRole::Admin => write!(f, "admin"),
            ParticipantRole::Member => write!(f, "member"),
        }
    }
}

impl FromSql<'_> for ParticipantRole {
    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "participant_role" || ty.name() == "text" || ty.name() == "varchar"
    }

    fn from_sql(
        _ty: &postgres_types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = String::from_utf8(raw.to_vec())?;
        match value.as_str() {
            "admin" => Ok(ParticipantRole::Admin),
            "member" => Ok(ParticipantRole::Member),
            _ => Err(format!("Unknown participant role: {}", value).into()),
        }
    }
}

impl ToSql for ParticipantRole {
    fn to_sql(
        &self,
        _ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        out.extend_from_slice(self.to_string().as_bytes());
        Ok(postgres_types::IsNull::No)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar"
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Image,
    File,
    System,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Text => write!(f, "text"),
            MessageType::Image => write!(f, "image"),
            MessageType::File => write!(f, "file"),
            MessageType::System => write!(f, "system"),
        }
    }
}

impl FromSql<'_> for MessageType {
    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "message_type" || ty.name() == "text" || ty.name() == "varchar"
    }

    fn from_sql(
        _ty: &postgres_types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = String::from_utf8(raw.to_vec())?;
        match value.as_str() {
            "text" => Ok(MessageType::Text),
            "image" => Ok(MessageType::Image),
            "file" => Ok(MessageType::File),
            "system" => Ok(MessageType::System),
            _ => Err(format!("Unknown message type: {}", value).into()),
        }
    }
}

impl ToSql for MessageType {
    fn to_sql(
        &self,
        _ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        out.extend_from_slice(self.to_string().as_bytes());
        Ok(postgres_types::IsNull::No)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar"
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    NewMessage,
    ThreadAdded,
    ThreadRemoved,
}

impl fmt::Display for NotificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationType::NewMessage => write!(f, "new_message"),
            NotificationType::ThreadAdded => write!(f, "thread_added"),
            NotificationType::ThreadRemoved => write!(f, "thread_removed"),
        }
    }
}

impl FromSql<'_> for NotificationType {
    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "notification_type" || ty.name() == "text" || ty.name() == "varchar"
    }

    fn from_sql(
        _ty: &postgres_types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = String::from_utf8(raw.to_vec())?;
        match value.as_str() {
            "new_message" => Ok(NotificationType::NewMessage),
            "thread_added" => Ok(NotificationType::ThreadAdded),
            "thread_removed" => Ok(NotificationType::ThreadRemoved),
            _ => Err(format!("Unknown notification type: {}", value).into()),
        }
    }
}

impl ToSql for NotificationType {
    fn to_sql(
        &self,
        _ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        out.extend_from_slice(self.to_string().as_bytes());
        Ok(postgres_types::IsNull::No)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar"
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SignalType {
    Offer,
    Answer,
    IceCandidate,
}

impl fmt::Display for SignalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalType::Offer => write!(f, "offer"),
            SignalType::Answer => write!(f, "answer"),
            SignalType::IceCandidate => write!(f, "ice-candidate"),
        }
    }
}

impl FromSql<'_> for SignalType {
    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "signal_type" || ty.name() == "text" || ty.name() == "varchar"
    }

    fn from_sql(
        _ty: &postgres_types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = String::from_utf8(raw.to_vec())?;
        match value.as_str() {
            "offer" => Ok(SignalType::Offer),
            "answer" => Ok(SignalType::Answer),
            "ice-candidate" => Ok(SignalType::IceCandidate),
            _ => Err(format!("Unknown signal type: {}", value).into()),
        }
    }
}

impl ToSql for SignalType {
    fn to_sql(
        &self,
        _ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        out.extend_from_slice(self.to_string().as_bytes());
        Ok(postgres_types::IsNull::No)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar"
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MessageThread {
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
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ThreadParticipant {
    pub participant_id: i64,
    pub thread_id: i64,
    pub user_id: i32,
    pub role: ParticipantRole,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub last_read_at: Option<DateTime<Utc>>,
    pub unread_count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Message {
    pub message_id: i64,
    pub thread_id: i64,
    pub sender_id: i32,
    pub message_type: MessageType,
    pub encrypted_content: String,
    pub content_nonce: Vec<u8>,
    pub sender_key_signature: Option<Vec<u8>>,
    pub reply_to_message_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<i32>,
    pub edit_count: i32,
    pub last_edited_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MessageEncryptionKey {
    pub key_id: i64,
    pub thread_id: i64,
    pub user_id: i32,
    pub encrypted_key: String,
    pub key_algorithm: String,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
    pub key_version: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserMessageBlock {
    pub block_id: i64,
    pub blocker_id: i32,
    pub blocked_id: i32,
    pub blocked_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MessageNotification {
    pub notification_id: i64,
    pub user_id: i32,
    pub thread_id: i64,
    pub message_id: i64,
    pub sender_id: i32,
    pub notification_type: NotificationType,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct WebRTCSignaling {
    pub signal_id: i64,
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub signal_type: SignalType,
    pub signal_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_processed: bool,
}

// Database row implementations
impl From<tokio_postgres::Row> for MessageThread {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            thread_id: row.get("thread_id"),
            thread_name: row.get("thread_name"),
            thread_type: row.get("thread_type"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
            max_participants: row.get("max_participants"),
            last_message_at: row.get("last_message_at"),
            last_message_preview: row.get("last_message_preview"),
            message_count: row.get("message_count"),
        }
    }
}

impl From<tokio_postgres::Row> for ThreadParticipant {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            participant_id: row.get("participant_id"),
            thread_id: row.get("thread_id"),
            user_id: row.get("user_id"),
            role: row.get("role"),
            joined_at: row.get("joined_at"),
            left_at: row.get("left_at"),
            is_active: row.get("is_active"),
            last_read_at: row.get("last_read_at"),
            unread_count: row.get("unread_count"),
        }
    }
}

impl From<tokio_postgres::Row> for Message {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            message_id: row.get("message_id"),
            thread_id: row.get("thread_id"),
            sender_id: row.get("sender_id"),
            message_type: row.get("message_type"),
            encrypted_content: row.get("encrypted_content"),
            content_nonce: row.get("content_nonce"),
            sender_key_signature: row.get("sender_key_signature"),
            reply_to_message_id: row.get("reply_to_message_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_deleted: row.get("is_deleted"),
            deleted_at: row.get("deleted_at"),
            deleted_by: row.get("deleted_by"),
            edit_count: row.get("edit_count"),
            last_edited_at: row.get("last_edited_at"),
        }
    }
}

impl From<tokio_postgres::Row> for MessageEncryptionKey {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            key_id: row.get("key_id"),
            thread_id: row.get("thread_id"),
            user_id: row.get("user_id"),
            encrypted_key: row.get("encrypted_key"),
            key_algorithm: row.get("key_algorithm"),
            created_at: row.get("created_at"),
            is_active: row.get("is_active"),
            key_version: row.get("key_version"),
        }
    }
}

impl From<tokio_postgres::Row> for UserMessageBlock {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            block_id: row.get("block_id"),
            blocker_id: row.get("blocker_id"),
            blocked_id: row.get("blocked_id"),
            blocked_at: row.get("blocked_at"),
            reason: row.get("reason"),
            is_active: row.get("is_active"),
        }
    }
}

impl From<tokio_postgres::Row> for MessageNotification {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            notification_id: row.get("notification_id"),
            user_id: row.get("user_id"),
            thread_id: row.get("thread_id"),
            message_id: row.get("message_id"),
            sender_id: row.get("sender_id"),
            notification_type: row.get("notification_type"),
            is_read: row.get("is_read"),
            created_at: row.get("created_at"),
            read_at: row.get("read_at"),
        }
    }
}

impl From<tokio_postgres::Row> for WebRTCSignaling {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            signal_id: row.get("signal_id"),
            from_user_id: row.get("from_user_id"),
            to_user_id: row.get("to_user_id"),
            signal_type: row.get("signal_type"),
            signal_data: row.get("signal_data"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            is_processed: row.get("is_processed"),
        }
    }
}
