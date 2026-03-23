//! PostgreSQL persistence for assistant chat threads.

use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::error::AppError;

pub const MAX_CHATS_PER_USER: i64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantChatRow {
    pub id: Uuid,
    pub user_id: i32,
    pub title: String,
    pub messages: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_model_id: Option<String>,
    pub scroll_top: f64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantChatListItem {
    pub id: Uuid,
    pub title: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantChatPutBody {
    pub title: String,
    pub messages: Value,
    #[serde(default)]
    pub primary_model_id: Option<String>,
    pub scroll_top: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSession {
    #[serde(default)]
    pub title: String,
    pub messages: Value,
    #[serde(default)]
    pub primary_model_id: Option<String>,
    #[serde(default)]
    pub scroll_top: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBody {
    pub sessions: Vec<ImportSession>,
}

pub async fn count_user_chats(pool: &Pool, user_id: i32) -> Result<i64, AppError> {
    let client = pool.get().await.map_err(|e| {
        AppError::Database(format!("assistant_chats count pool: {}", e))
    })?;
    let row = client
        .query_one(
            "SELECT COUNT(*)::bigint FROM assistant_chats WHERE user_id = $1",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::Database(format!("assistant_chats count: {}", e)))?;
    Ok(row.get::<_, i64>(0))
}

pub async fn list_chats(
    pool: &Pool,
    user_id: i32,
) -> Result<Vec<AssistantChatListItem>, AppError> {
    let client = pool.get().await.map_err(|e| {
        AppError::Database(format!("assistant_chats list pool: {}", e))
    })?;
    let rows = client
        .query(
            "SELECT id, title, updated_at FROM assistant_chats WHERE user_id = $1 ORDER BY updated_at DESC LIMIT $2",
            &[&user_id, &MAX_CHATS_PER_USER],
        )
        .await
        .map_err(|e| AppError::Database(format!("assistant_chats list: {}", e)))?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(AssistantChatListItem {
            id: r.get(0),
            title: r.get(1),
            updated_at: r.get(2),
        });
    }
    Ok(out)
}

pub async fn get_chat(
    pool: &Pool,
    user_id: i32,
    chat_id: Uuid,
) -> Result<Option<AssistantChatRow>, AppError> {
    let client = pool.get().await.map_err(|e| {
        AppError::Database(format!("assistant_chats get pool: {}", e))
    })?;
    let row = client
        .query_opt(
            "SELECT id, user_id, title, messages, primary_model_id, scroll_top, updated_at, created_at \
             FROM assistant_chats WHERE id = $1 AND user_id = $2",
            &[&chat_id, &user_id],
        )
        .await
        .map_err(|e| AppError::Database(format!("assistant_chats get: {}", e)))?;
    Ok(row.map(|r| AssistantChatRow {
        id: r.get(0),
        user_id: r.get(1),
        title: r.get(2),
        messages: r.get(3),
        primary_model_id: r.get(4),
        scroll_top: r.get(5),
        updated_at: r.get(6),
        created_at: r.get(7),
    }))
}

pub async fn create_chat(pool: &Pool, user_id: i32) -> Result<ChatCreateResult, AppError> {
    let n = count_user_chats(pool, user_id).await?;
    if n >= MAX_CHATS_PER_USER {
        return Err(AppError::BadRequest(format!(
            "Maximum {} assistant chats per user",
            MAX_CHATS_PER_USER
        )));
    }
    let client = pool.get().await.map_err(|e| {
        AppError::Database(format!("assistant_chats create pool: {}", e))
    })?;
    let row = client
        .query_one(
            "INSERT INTO assistant_chats (user_id, title, messages) VALUES ($1, $2, $3::jsonb) \
             RETURNING id, title, updated_at",
            &[&user_id, &String::new(), &Value::Array(vec![])],
        )
        .await
        .map_err(|e| AppError::Database(format!("assistant_chats create: {}", e)))?;
    Ok(ChatCreateResult {
        id: row.get(0),
        title: row.get(1),
        updated_at: row.get(2),
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCreateResult {
    pub id: Uuid,
    pub title: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn update_chat(
    pool: &Pool,
    user_id: i32,
    chat_id: Uuid,
    body: &AssistantChatPutBody,
) -> Result<(), AppError> {
    let client = pool.get().await.map_err(|e| {
        AppError::Database(format!("assistant_chats update pool: {}", e))
    })?;
    let n = client
        .execute(
            "UPDATE assistant_chats SET title = $1, messages = $2::jsonb, \
             primary_model_id = $3, scroll_top = $4, updated_at = now() \
             WHERE id = $5 AND user_id = $6",
            &[
                &body.title,
                &body.messages,
                &body.primary_model_id,
                &body.scroll_top,
                &chat_id,
                &user_id,
            ],
        )
        .await
        .map_err(|e| AppError::Database(format!("assistant_chats update: {}", e)))?;
    if n == 0 {
        return Err(AppError::NotFound("assistant chat not found".into()));
    }
    Ok(())
}

/// Persists messages JSON and optionally sets `primary_model_id` only if null (first model lock-in).
pub async fn persist_messages_and_maybe_model(
    pool: &Pool,
    user_id: i32,
    chat_id: Uuid,
    messages: &Value,
    primary_model_from_done: Option<&str>,
) -> Result<(), AppError> {
    let client = pool.get().await.map_err(|e| {
        AppError::Database(format!("assistant_chats persist pool: {}", e))
    })?;
    if let Some(mid) = primary_model_from_done {
        let n = client
            .execute(
                "UPDATE assistant_chats SET messages = $1::jsonb, \
                 primary_model_id = COALESCE(primary_model_id, $2), \
                 updated_at = now() WHERE id = $3 AND user_id = $4",
                &[&messages, &mid, &chat_id, &user_id],
            )
            .await
            .map_err(|e| AppError::Database(format!("assistant_chats persist: {}", e)))?;
        if n == 0 {
            return Err(AppError::NotFound("assistant chat not found".into()));
        }
    } else {
        let n = client
            .execute(
                "UPDATE assistant_chats SET messages = $1::jsonb, updated_at = now() \
                 WHERE id = $2 AND user_id = $3",
                &[&messages, &chat_id, &user_id],
            )
            .await
            .map_err(|e| AppError::Database(format!("assistant_chats persist: {}", e)))?;
        if n == 0 {
            return Err(AppError::NotFound("assistant chat not found".into()));
        }
    }
    Ok(())
}

pub async fn delete_chat(pool: &Pool, user_id: i32, chat_id: Uuid) -> Result<(), AppError> {
    let client = pool.get().await.map_err(|e| {
        AppError::Database(format!("assistant_chats delete pool: {}", e))
    })?;
    let n = client
        .execute(
            "DELETE FROM assistant_chats WHERE id = $1 AND user_id = $2",
            &[&chat_id, &user_id],
        )
        .await
        .map_err(|e| AppError::Database(format!("assistant_chats delete: {}", e)))?;
    if n == 0 {
        return Err(AppError::NotFound("assistant chat not found".into()));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResultItem {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub chats: Vec<ImportResultItem>,
}

pub async fn import_sessions(
    pool: &Pool,
    user_id: i32,
    body: &ImportBody,
) -> Result<ImportResponse, AppError> {
    let mut sessions = body.sessions.clone();
    if sessions.len() as i64 > MAX_CHATS_PER_USER {
        sessions.truncate(MAX_CHATS_PER_USER as usize);
    }
    let mut client = pool.get().await.map_err(|e| {
        AppError::Database(format!("assistant_chats import pool: {}", e))
    })?;
    let tx = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(format!("assistant_chats import tx: {}", e)))?;
    let mut out = Vec::new();
    for s in &sessions {
        let row = tx
            .query_one(
                "INSERT INTO assistant_chats (user_id, title, messages, primary_model_id, scroll_top) \
                 VALUES ($1, $2, $3::jsonb, $4, $5) RETURNING id",
                &[
                    &user_id,
                    &s.title,
                    &s.messages,
                    &s.primary_model_id,
                    &s.scroll_top,
                ],
            )
            .await
            .map_err(|e| AppError::Database(format!("assistant_chats import insert: {}", e)))?;
        out.push(ImportResultItem { id: row.get(0) });
    }
    tx.commit()
        .await
        .map_err(|e| AppError::Database(format!("assistant_chats import commit: {}", e)))?;
    Ok(ImportResponse { chats: out })
}
