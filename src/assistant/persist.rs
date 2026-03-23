//! Persist streamed assistant UI state to PostgreSQL (before emitting each SSE event).

use std::sync::Arc;

use deadpool_postgres::Pool;
use serde_json::Value;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::assistant::chat_store;
use crate::assistant::sse_ui_sync::apply_sse_event_to_messages;
use crate::error::AppError;

pub struct ChatPersistState {
    pub pool: Pool,
    pub user_id: i32,
    pub chat_id: Uuid,
    /// Full `messages` array (same shape as frontend).
    pub messages: Arc<Mutex<Value>>,
    pub assistant_index: usize,
}

impl ChatPersistState {
    /// Applies one SSE payload to stored messages and writes to DB before the client receives the event.
    pub async fn apply_and_save(&self, payload: &Value) -> Result<(), AppError> {
        let ty = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if ty == "stream_debug" {
            return Ok(());
        }

        let primary_from_done = if ty == "done" {
            payload.get("model").and_then(|m| m.as_str())
        } else {
            None
        };

        let mut guard = self.messages.lock().await;
        apply_sse_event_to_messages(&mut guard, self.assistant_index, payload)?;
        let to_save = guard.clone();
        drop(guard);

        chat_store::persist_messages_and_maybe_model(
            &self.pool,
            self.user_id,
            self.chat_id,
            &to_save,
            primary_from_done,
        )
        .await
    }
}
