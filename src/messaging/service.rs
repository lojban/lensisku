use crate::{AppError, AppResult};
use actix::Addr;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use deadpool_postgres::Pool;

use super::dto::WebSocketMessage;
use super::dto::*;
use super::encryption::EncryptionService;
use super::models::*;
use super::websocket::{BroadcastToUsers, ChatServer};

// Helper function to build dynamic message query
fn build_message_query(
    query: &GetMessagesQuery,
    thread_id: i64,
    offset: i64,
) -> (String, Vec<Box<dyn postgres_types::ToSql + Sync>>) {
    let mut where_conditions = vec!["m.thread_id = $1 AND m.is_deleted = FALSE".to_string()];
    let mut params: Vec<Box<dyn postgres_types::ToSql + Sync>> = vec![Box::new(thread_id)];
    let mut param_index = 2;

    if let Some(before_id) = query.before_message_id {
        where_conditions.push(format!("m.message_id < ${}", param_index));
        params.push(Box::new(before_id));
        param_index += 1;
    }

    if let Some(after_id) = query.after_message_id {
        where_conditions.push(format!("m.message_id > ${}", param_index));
        params.push(Box::new(after_id));
        param_index += 1;
    }

    if let Some(message_type) = query.message_type {
        where_conditions.push(format!("m.message_type = ${}", param_index));
        params.push(Box::new(message_type));
        param_index += 1;
    }

    let where_clause = where_conditions.join(" AND ");
    let query_str = format!(
        "SELECT m.*, u.username 
         FROM private_messages m
         JOIN users u ON m.sender_id = u.userid
         WHERE {}
         ORDER BY m.created_at DESC
         LIMIT ${} OFFSET ${}",
        where_clause,
        param_index,
        param_index + 1
    );

    params.push(Box::new(query.per_page));
    params.push(Box::new(offset));

    (query_str, params)
}

pub struct MessagingService {
    pool: Pool,
    chat_server: Option<Addr<ChatServer>>,
}

impl MessagingService {
    pub fn new(pool: Pool, chat_server: Option<Addr<ChatServer>>) -> Self {
        Self { pool, chat_server }
    }

    // Thread operations
    pub async fn get_user_threads(
        &self,
        user_id: i32,
        query: GetThreadsQuery,
    ) -> AppResult<ThreadListResponse> {
        let client = self.pool.get().await?;
        let offset = (query.page - 1) * query.per_page;

        let rows = client
            .query(
                "SELECT mt.*,
                        COALESCE(tp.unread_count, 0) as unread_count,
                        (SELECT COUNT(*) FROM thread_participants tp2 WHERE tp2.thread_id = mt.thread_id AND tp2.is_active = TRUE) as participant_count,
                        (tp.role = 'admin') as is_admin
                 FROM message_threads mt
                 JOIN thread_participants tp ON mt.thread_id = tp.thread_id
                 WHERE tp.user_id = $1
                   AND tp.is_active = TRUE
                   AND mt.is_active = TRUE
                 ORDER BY mt.last_message_at DESC NULLS LAST, mt.created_at DESC
                 LIMIT $2 OFFSET $3",
                &[&user_id, &query.per_page, &offset],
            )
            .await?;

        let mut threads = Vec::new();
        for row in rows {
            let thread_id: i64 = row.get("thread_id");
            let participants = self.get_thread_participants(thread_id, None).await?;

            threads.push(ThreadResponse {
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
                unread_count: row.get("unread_count"),
                participant_count: row.get("participant_count"),
                is_admin: row.get("is_admin"),
                participants,
            });
        }

        // Get total count
        let total_row = client
            .query_one(
                "SELECT COUNT(*) FROM thread_participants 
                 WHERE user_id = $1 AND is_active = TRUE",
                &[&user_id],
            )
            .await?;
        let total_count: i64 = total_row.get(0);

        let has_more = (offset + threads.len() as i64) < total_count;

        Ok(ThreadListResponse {
            threads,
            total_count,
            page: query.page,
            per_page: query.per_page,
            has_more,
        })
    }

    pub async fn create_thread(
        &self,
        user_id: i32,
        request: CreateThreadRequest,
    ) -> AppResult<ThreadResponse> {
        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;

        // Check if user is blocked by any participants (for direct threads)
        if let Some(ref participant_ids) = request.participant_ids {
            if self.is_blocked_by_any(participant_ids, user_id).await? {
                return Err(AppError::Auth("User is blocked by participant".to_string()));
            }
        }

        // Create thread
        let max_participants = request.max_participants.unwrap_or(100);
        let thread_row = transaction
            .query_one(
                "INSERT INTO message_threads (thread_name, thread_type, created_by, max_participants)
                 VALUES ($1, $2, $3, $4)
                 RETURNING *",
                &[
                    &request.thread_name,
                    &request.thread_type,
                    &user_id,
                    &max_participants,
                ],
            )
            .await?;

        let thread_id: i64 = thread_row.get("thread_id");

        // Add creator as admin
        transaction
            .execute(
                "INSERT INTO thread_participants (thread_id, user_id, role)
                 VALUES ($1, $2, 'admin')",
                &[&thread_id, &user_id],
            )
            .await?;

        // Add other participants
        if let Some(participant_ids) = request.participant_ids {
            // One round trip for all participants instead of one INSERT each.
            transaction
                .execute(
                    "INSERT INTO thread_participants (thread_id, user_id, role)
                     SELECT $1, p.user_id, 'member' FROM UNNEST($2::int4[]) AS p(user_id)
                     ON CONFLICT (thread_id, user_id) DO NOTHING",
                    &[&thread_id, &participant_ids],
                )
                .await?;
        }

        transaction.commit().await?;

        let crypto = EncryptionService::new(self.pool.clone());
        let (aes_key, hmac_key) = crypto.generate_thread_key_pair(thread_id).await?;
        let stored_key = format!("{aes_key}:{hmac_key}");
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO message_encryption_keys (thread_id, user_id, encrypted_key, key_algorithm)
                 VALUES ($1, $2, $3, 'aes-256-gcm+hmac-sha256')",
                &[&thread_id, &user_id, &stored_key],
            )
            .await?;

        self.get_thread(thread_id, user_id).await
    }

    fn crypto(&self) -> EncryptionService {
        EncryptionService::new(self.pool.clone())
    }

    async fn thread_keys(&self, thread_id: i64) -> AppResult<Option<(String, String)>> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT encrypted_key FROM message_encryption_keys
                 WHERE thread_id = $1 AND is_active = TRUE
                   AND key_algorithm = 'aes-256-gcm+hmac-sha256'
                 ORDER BY key_version DESC
                 LIMIT 1",
                &[&thread_id],
            )
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let stored: String = row.get("encrypted_key");
        let Some((aes, hmac)) = stored.split_once(':') else {
            return Ok(None);
        };
        Ok(Some((aes.to_string(), hmac.to_string())))
    }

    pub async fn get_thread(&self, thread_id: i64, user_id: i32) -> AppResult<ThreadResponse> {
        let client = self.pool.get().await?;

        // Check if user is a participant
        let participant_row = client
            .query_opt(
                "SELECT * FROM thread_participants 
                 WHERE thread_id = $1 AND user_id = $2 AND is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?;

        if participant_row.is_none() {
            return Err(AppError::NotFound(
                "Thread not found or access denied".to_string(),
            ));
        }

        // Get thread details
        let thread_row = client
            .query_one(
                "SELECT mt.*, tp.unread_count, tp.role as user_role,
                        (SELECT COUNT(*) FROM thread_participants tp2 WHERE tp2.thread_id = mt.thread_id AND tp2.is_active = TRUE) as participant_count
                 FROM message_threads mt
                 JOIN thread_participants tp ON mt.thread_id = tp.thread_id
                 WHERE mt.thread_id = $1 AND tp.user_id = $2",
                &[&thread_id, &user_id],
            )
            .await?;

        let participants = self.get_thread_participants(thread_id, None).await?;

        Ok(ThreadResponse {
            thread_id: thread_row.get("thread_id"),
            thread_name: thread_row.get("thread_name"),
            thread_type: thread_row.get("thread_type"),
            created_by: thread_row.get("created_by"),
            created_at: thread_row.get("created_at"),
            updated_at: thread_row.get("updated_at"),
            is_active: thread_row.get("is_active"),
            max_participants: thread_row.get("max_participants"),
            last_message_at: thread_row.get("last_message_at"),
            last_message_preview: thread_row.get("last_message_preview"),
            message_count: thread_row.get("message_count"),
            unread_count: thread_row.get("unread_count"),
            participant_count: thread_row.get("participant_count"),
            is_admin: thread_row.get::<_, ParticipantRole>("user_role") == ParticipantRole::Admin,
            participants,
        })
    }

    pub async fn update_thread(
        &self,
        thread_id: i64,
        user_id: i32,
        request: UpdateThreadRequest,
    ) -> AppResult<ThreadResponse> {
        let client = self.pool.get().await?;

        // Check if user is admin
        let is_admin = client
            .query_opt(
                "SELECT 1 FROM thread_participants 
                 WHERE thread_id = $1 AND user_id = $2 AND role = 'admin' AND is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?
            .is_some();

        if !is_admin {
            return Err(AppError::Auth("Only admins can update threads".to_string()));
        }

        // Build update query
        let mut updates = Vec::new();
        let mut param_index = 1;

        if let Some(ref _name) = request.thread_name {
            updates.push(format!("thread_name = ${}", param_index));
            param_index += 1;
        }

        if let Some(_max_participants) = request.max_participants {
            updates.push(format!("max_participants = ${}", param_index));
            param_index += 1;
        }

        if let Some(_is_active) = request.is_active {
            updates.push(format!("is_active = ${}", param_index));
            param_index += 1;
        }

        if updates.is_empty() {
            return self.get_thread(thread_id, user_id).await;
        }

        let query = format!(
            "UPDATE message_threads SET {} WHERE thread_id = ${}",
            updates.join(", "),
            param_index
        );

        // Use individual parameters to avoid lifetime issues
        let _result = if let Some(ref name) = request.thread_name {
            if let Some(max_participants) = request.max_participants {
                if let Some(is_active) = request.is_active {
                    client
                        .execute(
                            &query,
                            &[
                                &name as &(dyn postgres_types::ToSql + Sync),
                                &max_participants,
                                &is_active,
                                &thread_id,
                            ],
                        )
                        .await?
                } else {
                    client
                        .execute(
                            &query,
                            &[
                                &name as &(dyn postgres_types::ToSql + Sync),
                                &max_participants,
                                &thread_id,
                            ],
                        )
                        .await?
                }
            } else if let Some(is_active) = request.is_active {
                client
                    .execute(
                        &query,
                        &[
                            &name as &(dyn postgres_types::ToSql + Sync),
                            &is_active,
                            &thread_id,
                        ],
                    )
                    .await?
            } else {
                client
                    .execute(
                        &query,
                        &[&name as &(dyn postgres_types::ToSql + Sync), &thread_id],
                    )
                    .await?
            }
        } else if let Some(max_participants) = request.max_participants {
            if let Some(is_active) = request.is_active {
                client
                    .execute(&query, &[&max_participants, &is_active, &thread_id])
                    .await?
            } else {
                client
                    .execute(&query, &[&max_participants, &thread_id])
                    .await?
            }
        } else if let Some(is_active) = request.is_active {
            client.execute(&query, &[&is_active, &thread_id]).await?
        } else {
            0 // This should not happen due to the empty check above
        };

        self.get_thread(thread_id, user_id).await
    }

    pub async fn delete_thread(&self, thread_id: i64, user_id: i32) -> AppResult<()> {
        let client = self.pool.get().await?;

        // Check if user is admin or thread creator
        let can_delete = client
            .query_opt(
                "SELECT 1 FROM thread_participants tp
                 JOIN message_threads mt ON tp.thread_id = mt.thread_id
                 WHERE tp.thread_id = $1 AND tp.user_id = $2 
                 AND (tp.role = 'admin' OR mt.created_by = $2) AND tp.is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?
            .is_some();

        if !can_delete {
            return Err(AppError::Auth(
                "Only admins or creators can delete threads".to_string(),
            ));
        }

        // Soft delete the thread
        client
            .execute(
                "UPDATE message_threads SET is_active = FALSE WHERE thread_id = $1",
                &[&thread_id],
            )
            .await?;

        Ok(())
    }

    // Message operations
    pub async fn get_thread_messages(
        &self,
        thread_id: i64,
        user_id: i32,
        query: GetMessagesQuery,
    ) -> AppResult<MessageListResponse> {
        let client = self.pool.get().await?;

        // Check if user is a participant
        let is_participant = client
            .query_opt(
                "SELECT 1 FROM thread_participants 
                 WHERE thread_id = $1 AND user_id = $2 AND is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?
            .is_some();

        if !is_participant {
            return Err(AppError::NotFound(
                "Thread not found or access denied".to_string(),
            ));
        }

        let offset = (query.page - 1) * query.per_page;

        // Build dynamic query based on parameters
        let (query_str, params) = build_message_query(&query, thread_id, offset);

        // Convert Box<dyn ToSql> to references for the query
        let param_refs: Vec<&(dyn postgres_types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref()).collect();

        let rows = client.query(&query_str, &param_refs).await?;

        let mut messages = Vec::new();
        for row in rows {
            let content_nonce: Vec<u8> = row.get("content_nonce");
            let sender_key_signature: Option<Vec<u8>> = row.get("sender_key_signature");

            messages.push(MessageResponse {
                message_id: row.get("message_id"),
                thread_id: row.get("thread_id"),
                sender_id: row.get("sender_id"),
                username: row.get("username"),
                message_type: row.get("message_type"),
                encrypted_content: row.get("encrypted_content"),
                content_nonce: general_purpose::STANDARD.encode(&content_nonce),
                sender_key_signature: sender_key_signature
                    .map(|sig| general_purpose::STANDARD.encode(&sig)),
                reply_to_message_id: row.get("reply_to_message_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_deleted: row.get("is_deleted"),
                edit_count: row.get("edit_count"),
                last_edited_at: row.get("last_edited_at"),
                is_from_sender: row.get::<_, i32>("sender_id") == user_id,
            });
        }

        // Get total count
        let total_row = client
            .query_one(
                "SELECT COUNT(*) FROM private_messages m 
                 WHERE m.thread_id = $1 AND m.is_deleted = FALSE",
                &[&thread_id],
            )
            .await?;
        let total_count: i64 = total_row.get(0);

        let has_more = (offset + messages.len() as i64) < total_count;

        Ok(MessageListResponse {
            messages,
            total_count,
            page: query.page,
            per_page: query.per_page,
            has_more,
        })
    }

    pub async fn send_message(
        &self,
        user_id: i32,
        request: SendMessageRequest,
    ) -> AppResult<MessageResponse> {
        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;

        // Check if user is a participant and thread is active
        let participant_check = transaction
            .query_opt(
                "SELECT tp.thread_id FROM thread_participants tp
                 JOIN message_threads mt ON tp.thread_id = mt.thread_id
                 WHERE tp.thread_id = $1 AND tp.user_id = $2 
                 AND tp.is_active = TRUE AND mt.is_active = TRUE",
                &[&request.thread_id, &user_id],
            )
            .await?;

        if participant_check.is_none() {
            return Err(AppError::NotFound(
                "Thread not found or access denied".to_string(),
            ));
        }

        // Decode base64 nonce
        let content_nonce = general_purpose::STANDARD
            .decode(&request.content_nonce)
            .map_err(|_| AppError::BadRequest("Invalid content nonce format".to_string()))?;

        let sender_key_signature = request
            .sender_key_signature
            .as_ref()
            .and_then(|sig| general_purpose::STANDARD.decode(sig).ok());

        if let (Some(signature), Some((_, hmac_key))) = (
            request.sender_key_signature.as_deref(),
            self.thread_keys(request.thread_id).await?,
        ) {
            let valid = self
                .crypto()
                .verify_signature(&request.encrypted_content, signature, &hmac_key)
                .await?;
            if !valid {
                return Err(AppError::BadRequest("Invalid message signature".into()));
            }
        }

        // Insert message
        let message_row = transaction
            .query_one(
                "INSERT INTO private_messages (thread_id, sender_id, message_type, encrypted_content, content_nonce, sender_key_signature, reply_to_message_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 RETURNING *",
                &[
                    &request.thread_id,
                    &user_id,
                    &request.message_type,
                    &request.encrypted_content,
                    &content_nonce,
                    &sender_key_signature,
                    &request.reply_to_message_id,
                ],
            )
            .await?;

        let message_id: i64 = message_row.get("message_id");

        // Create notifications for other participants
        transaction
            .execute(
                "INSERT INTO message_notifications (user_id, thread_id, message_id, sender_id, notification_type)
                 SELECT tp.user_id, $1, $2, $3, 'new_message'
                 FROM thread_participants tp
                 WHERE tp.thread_id = $1 AND tp.user_id != $3 AND tp.is_active = TRUE
                   AND user_accepts_notifications(tp.user_id)",
                &[&request.thread_id, &message_id, &user_id],
            )
            .await?;

        transaction.commit().await?;

        // Get the complete message with username
        let message = self.get_message(message_id, user_id).await?;

        // Broadcast to thread participants over WebSocket
        if let Some(chat_server) = &self.chat_server {
            if let Ok(user_ids) = self
                .get_thread_participant_user_ids(request.thread_id)
                .await
            {
                let mut broadcast_message = message.clone();
                broadcast_message.is_from_sender = false;
                let ws_message = WebSocketMessage::MessageSent {
                    message: broadcast_message,
                    thread_id: request.thread_id,
                };
                if let Ok(json) = serde_json::to_string(&ws_message) {
                    chat_server.do_send(BroadcastToUsers {
                        user_ids,
                        message_json: json,
                        exclude_user: Some(user_id),
                    });
                }
            }
        }

        Ok(message)
    }

    pub async fn get_message(&self, message_id: i64, user_id: i32) -> AppResult<MessageResponse> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "SELECT m.*, u.username 
                 FROM private_messages m
                 JOIN users u ON m.sender_id = u.userid
                 WHERE m.message_id = $1",
                &[&message_id],
            )
            .await?;

        if let Some(row) = row {
            let thread_id: i64 = row.get("thread_id");

            // Check if user is a participant
            let is_participant = client
                .query_opt(
                    "SELECT 1 FROM thread_participants 
                     WHERE thread_id = $1 AND user_id = $2 AND is_active = TRUE",
                    &[&thread_id, &user_id],
                )
                .await?
                .is_some();

            if !is_participant {
                return Err(AppError::NotFound("Access denied".to_string()));
            }

            let content_nonce: Vec<u8> = row.get("content_nonce");
            let sender_key_signature: Option<Vec<u8>> = row.get("sender_key_signature");
            let message_type: MessageType = row.get("message_type");
            let mut encrypted_content: String = row.get("encrypted_content");
            let nonce_b64 = general_purpose::STANDARD.encode(&content_nonce);
            if message_type == MessageType::System {
                if let Some((aes_key, _)) = self.thread_keys(thread_id).await? {
                    if let Ok(plain) = self
                        .crypto()
                        .decrypt_message(&encrypted_content, &nonce_b64, &aes_key)
                        .await
                    {
                        encrypted_content = plain;
                    }
                }
            }

            Ok(MessageResponse {
                message_id: row.get("message_id"),
                thread_id: row.get("thread_id"),
                sender_id: row.get("sender_id"),
                username: row.get("username"),
                message_type,
                encrypted_content,
                content_nonce: nonce_b64,
                sender_key_signature: sender_key_signature
                    .map(|sig| general_purpose::STANDARD.encode(&sig)),
                reply_to_message_id: row.get("reply_to_message_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_deleted: row.get("is_deleted"),
                edit_count: row.get("edit_count"),
                last_edited_at: row.get("last_edited_at"),
                is_from_sender: row.get::<_, i32>("sender_id") == user_id,
            })
        } else {
            Err(AppError::NotFound("Message not found".to_string()))
        }
    }

    // Helper methods
    async fn get_thread_participants(
        &self,
        thread_id: i64,
        exclude_user_id: Option<i32>,
    ) -> AppResult<Vec<ThreadParticipantResponse>> {
        let client = self.pool.get().await?;

        let query = if let Some(exclude_id) = exclude_user_id {
            client
                .query(
                    "SELECT tp.*, u.username 
                     FROM thread_participants tp
                     JOIN users u ON tp.user_id = u.userid
                     WHERE tp.thread_id = $1 AND tp.user_id != $2 AND tp.is_active = TRUE
                     ORDER BY tp.joined_at",
                    &[&thread_id, &exclude_id],
                )
                .await?
        } else {
            client
                .query(
                    "SELECT tp.*, u.username 
                     FROM thread_participants tp
                     JOIN users u ON tp.user_id = u.userid
                     WHERE tp.thread_id = $1 AND tp.is_active = TRUE
                     ORDER BY tp.joined_at",
                    &[&thread_id],
                )
                .await?
        };

        let mut participants = Vec::new();
        for row in query {
            participants.push(ThreadParticipantResponse {
                participant_id: row.get("participant_id"),
                user_id: row.get("user_id"),
                username: row.get("username"),
                role: row.get("role"),
                joined_at: row.get("joined_at"),
                is_active: row.get("is_active"),
                unread_count: row.get("unread_count"),
            });
        }

        Ok(participants)
    }

    pub async fn get_thread_participant_user_ids(&self, thread_id: i64) -> AppResult<Vec<i32>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT user_id FROM thread_participants
                 WHERE thread_id = $1 AND is_active = TRUE",
                &[&thread_id],
            )
            .await?;

        Ok(rows.iter().map(|row| row.get("user_id")).collect())
    }

    pub async fn mark_thread_read(&self, user_id: i32, thread_id: i64) -> AppResult<()> {
        let client = self.pool.get().await?;

        let is_participant = client
            .query_opt(
                "SELECT 1 FROM thread_participants
                 WHERE thread_id = $1 AND user_id = $2 AND is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?
            .is_some();

        if !is_participant {
            return Err(AppError::NotFound(
                "Thread not found or access denied".to_string(),
            ));
        }

        client
            .execute(
                "UPDATE thread_participants
                 SET last_read_at = CURRENT_TIMESTAMP, unread_count = 0
                 WHERE thread_id = $1 AND user_id = $2",
                &[&thread_id, &user_id],
            )
            .await?;

        Ok(())
    }

    pub async fn is_user_blocked(&self, blocker_id: i32, blocked_id: i32) -> AppResult<bool> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "SELECT is_user_blocked($1, $2)",
                &[&blocker_id, &blocked_id],
            )
            .await?;

        Ok(row.get(0))
    }

    /// Whether any of `blocker_ids` has blocked `blocked_id`, in one query.
    async fn is_blocked_by_any(&self, blocker_ids: &[i32], blocked_id: i32) -> AppResult<bool> {
        if blocker_ids.is_empty() {
            return Ok(false);
        }
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT COALESCE(bool_or(is_user_blocked(b, $2)), false)
                 FROM UNNEST($1::int4[]) AS b",
                &[&blocker_ids, &blocked_id],
            )
            .await?;
        Ok(row.get(0))
    }

    // WebRTC signaling methods
    pub async fn send_webrtc_signal(
        &self,
        from_user_id: i32,
        to_user_id: i32,
        signal_type: SignalType,
        signal_data: String,
    ) -> AppResult<WebRTCSignaling> {
        let client = self.pool.get().await?;

        // Check if users are blocking each other
        if self.is_user_blocked(to_user_id, from_user_id).await? {
            return Err(AppError::Auth("User is blocked".to_string()));
        }

        let expires_at = Utc::now() + chrono::Duration::minutes(5);

        let row = client
            .query_one(
                "INSERT INTO webrtc_signaling (from_user_id, to_user_id, signal_type, signal_data, expires_at)
                 VALUES ($1, $2, $3, $4, $5)
                 RETURNING *",
                &[
                    &from_user_id,
                    &to_user_id,
                    &signal_type,
                    &signal_data,
                    &expires_at,
                ],
            )
            .await?;

        Ok(WebRTCSignaling::from(row))
    }

    pub async fn get_pending_webrtc_signals(
        &self,
        user_id: i32,
    ) -> AppResult<Vec<WebRTCSignaling>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT ws.*, u.username as from_username
                 FROM webrtc_signaling ws
                 JOIN users u ON ws.from_user_id = u.userid
                 WHERE ws.to_user_id = $1 AND ws.is_processed = FALSE AND ws.expires_at > CURRENT_TIMESTAMP
                 ORDER BY ws.created_at",
                &[&user_id],
            )
            .await?;

        let mut signals = Vec::new();
        for row in rows {
            signals.push(WebRTCSignaling::from(row));
        }

        Ok(signals)
    }

    pub async fn mark_webrtc_signal_processed(
        &self,
        signal_id: i64,
        user_id: i32,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "UPDATE webrtc_signaling 
                 SET is_processed = TRUE 
                 WHERE signal_id = $1 AND to_user_id = $2",
                &[&signal_id, &user_id],
            )
            .await?;

        Ok(())
    }

    pub async fn cleanup_expired_webrtc_signals(&self) -> AppResult<i32> {
        let client = self.pool.get().await?;

        let row = client
            .query_one("SELECT cleanup_expired_webrtc_signals()", &[])
            .await?;

        Ok(row.get(0))
    }

    pub async fn get_active_webrtc_calls(&self, user_id: i32) -> AppResult<Vec<serde_json::Value>> {
        let client = self.pool.get().await?;

        // Get recent signals that indicate active calls
        let rows = client
            .query(
                "SELECT DISTINCT 
                    CASE 
                        WHEN from_user_id = $1 THEN to_user_id 
                        ELSE from_user_id 
                    END as other_user_id,
                    u.username as other_username,
                    MAX(created_at) as last_signal_at
                 FROM webrtc_signaling ws
                 JOIN users u ON (CASE WHEN from_user_id = $1 THEN to_user_id ELSE from_user_id END) = u.userid
                 WHERE (from_user_id = $1 OR to_user_id = $1)
                 AND created_at > CURRENT_TIMESTAMP - INTERVAL '1 hour'
                 AND signal_type IN ('offer', 'answer')
                 GROUP BY other_user_id, u.username
                 ORDER BY last_signal_at DESC",
                &[&user_id],
            )
            .await?;

        let mut active_calls = Vec::new();
        for row in rows {
            active_calls.push(serde_json::json!({
                "user_id": row.get::<_, i32>("other_user_id"),
                "username": row.get::<_, String>("other_username"),
                "last_activity": row.get::<_, chrono::DateTime<Utc>>("last_signal_at")
            }));
        }

        Ok(active_calls)
    }

    async fn require_thread_admin(&self, thread_id: i64, user_id: i32) -> AppResult<()> {
        let client = self.pool.get().await?;
        let is_admin = client
            .query_opt(
                "SELECT 1 FROM thread_participants
                 WHERE thread_id = $1 AND user_id = $2 AND role = 'admin' AND is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?
            .is_some();
        if is_admin {
            Ok(())
        } else {
            Err(AppError::Auth("Only thread admins can do that".to_string()))
        }
    }

    fn broadcast(&self, user_ids: Vec<i32>, message: &WebSocketMessage, exclude_user: Option<i32>) {
        let Some(chat_server) = &self.chat_server else {
            return;
        };
        let Ok(json) = serde_json::to_string(message) else {
            return;
        };
        chat_server.do_send(BroadcastToUsers {
            user_ids,
            message_json: json,
            exclude_user,
        });
    }

    pub async fn update_message(
        &self,
        user_id: i32,
        message_id: i64,
        request: UpdateMessageRequest,
    ) -> AppResult<MessageResponse> {
        let content_nonce = general_purpose::STANDARD
            .decode(&request.content_nonce)
            .map_err(|_| AppError::BadRequest("Invalid content nonce format".to_string()))?;
        let sender_key_signature = request
            .sender_key_signature
            .as_ref()
            .and_then(|sig| general_purpose::STANDARD.decode(sig).ok());
        let client = self.pool.get().await?;
        let updated = client
            .execute(
                "UPDATE private_messages
                 SET encrypted_content = $1,
                     content_nonce = $2,
                     sender_key_signature = $3,
                     edit_count = edit_count + 1,
                     last_edited_at = CURRENT_TIMESTAMP
                 WHERE message_id = $4 AND sender_id = $5 AND is_deleted = FALSE",
                &[
                    &request.encrypted_content,
                    &content_nonce,
                    &sender_key_signature,
                    &message_id,
                    &user_id,
                ],
            )
            .await?;
        if updated == 0 {
            let exists = client
                .query_opt(
                    "SELECT sender_id, is_deleted FROM private_messages WHERE message_id = $1",
                    &[&message_id],
                )
                .await?;
            return match exists {
                None => Err(AppError::NotFound("Message not found".into())),
                Some(row) if row.get::<_, bool>("is_deleted") => {
                    Err(AppError::NotFound("Message not found".into()))
                }
                Some(_) => Err(AppError::Auth(
                    "Only the sender can edit this message".into(),
                )),
            };
        }
        let message = self.get_message(message_id, user_id).await?;
        if let Ok(user_ids) = self
            .get_thread_participant_user_ids(message.thread_id)
            .await
        {
            self.broadcast(
                user_ids,
                &WebSocketMessage::MessageSent {
                    message: message.clone(),
                    thread_id: message.thread_id,
                },
                Some(user_id),
            );
        }
        Ok(message)
    }

    pub async fn delete_message(&self, user_id: i32, message_id: i64) -> AppResult<()> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT thread_id, sender_id, is_deleted FROM private_messages WHERE message_id = $1",
                &[&message_id],
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Message not found".into()))?;
        if row.get::<_, bool>("is_deleted") {
            return Err(AppError::NotFound("Message not found".into()));
        }
        let thread_id: i64 = row.get("thread_id");
        let sender_id: i32 = row.get("sender_id");
        if sender_id != user_id {
            self.require_thread_admin(thread_id, user_id).await?;
        }
        client
            .execute(
                "UPDATE private_messages
                 SET is_deleted = TRUE, deleted_at = CURRENT_TIMESTAMP, deleted_by = $1
                 WHERE message_id = $2",
                &[&user_id, &message_id],
            )
            .await?;
        if let Ok(user_ids) = self.get_thread_participant_user_ids(thread_id).await {
            self.broadcast(
                user_ids,
                &WebSocketMessage::MessageDeleted {
                    message_id,
                    thread_id,
                    deleted_by: user_id,
                },
                None,
            );
        }
        Ok(())
    }

    pub async fn add_participant(
        &self,
        actor_id: i32,
        thread_id: i64,
        request: AddParticipantRequest,
    ) -> AppResult<ThreadParticipantResponse> {
        if request.user_id == actor_id {
            return Err(AppError::BadRequest(
                "You are already in this thread".into(),
            ));
        }
        self.require_thread_admin(thread_id, actor_id).await?;
        if self.is_user_blocked(actor_id, request.user_id).await?
            || self.is_user_blocked(request.user_id, actor_id).await?
        {
            return Err(AppError::Auth(
                "Cannot add a user in a block relationship".into(),
            ));
        }
        let client = self.pool.get().await?;
        let thread = client
            .query_opt(
                "SELECT thread_type, max_participants FROM message_threads
                 WHERE thread_id = $1 AND is_active = TRUE",
                &[&thread_id],
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Thread not found".into()))?;
        let thread_type: ThreadType = thread.get("thread_type");
        if thread_type == ThreadType::Direct {
            return Err(AppError::BadRequest(
                "Direct threads cannot take more participants".into(),
            ));
        }
        let max_participants: i32 = thread.get("max_participants");
        let active_count: i64 = client
            .query_one(
                "SELECT COUNT(*) FROM thread_participants WHERE thread_id = $1 AND is_active = TRUE",
                &[&thread_id],
            )
            .await?
            .get(0);
        let username: String = client
            .query_opt(
                "SELECT username FROM users WHERE userid = $1",
                &[&request.user_id],
            )
            .await?
            .map(|row| row.get("username"))
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;
        let existing = client
            .query_opt(
                "SELECT is_active FROM thread_participants WHERE thread_id = $1 AND user_id = $2",
                &[&thread_id, &request.user_id],
            )
            .await?;
        let role = request.role.unwrap_or(ParticipantRole::Member);
        let role_name = role.to_string();
        match existing {
            Some(row) if row.get::<_, bool>("is_active") => {
                return Err(AppError::BadRequest("User is already a participant".into()));
            }
            Some(_) => {
                client
                    .execute(
                        "UPDATE thread_participants
                         SET is_active = TRUE, left_at = NULL, role = $3, joined_at = CURRENT_TIMESTAMP
                         WHERE thread_id = $1 AND user_id = $2",
                        &[&thread_id, &request.user_id, &role_name],
                    )
                    .await?;
            }
            None => {
                if active_count >= i64::from(max_participants) {
                    return Err(AppError::BadRequest("Thread is full".into()));
                }
                client
                    .execute(
                        "INSERT INTO thread_participants (thread_id, user_id, role)
                         VALUES ($1, $2, $3)",
                        &[&thread_id, &request.user_id, &role_name],
                    )
                    .await?;
            }
        }
        if let Some((aes_key, hmac_key)) = self.thread_keys(thread_id).await? {
            let notice = format!("{username} joined the thread");
            let (ciphertext, nonce) = self.crypto().encrypt_message(&notice, &aes_key).await?;
            let signature = self.crypto().sign_message(&ciphertext, &hmac_key).await?;
            let nonce_bytes = general_purpose::STANDARD
                .decode(nonce)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let signature_bytes = general_purpose::STANDARD
                .decode(signature)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let system = MessageType::System.to_string();
            client
                .execute(
                    "INSERT INTO private_messages (thread_id, sender_id, message_type, encrypted_content, content_nonce, sender_key_signature)
                     VALUES ($1, $2, $3, $4, $5, $6)",
                    &[
                        &thread_id,
                        &actor_id,
                        &system,
                        &ciphertext,
                        &nonce_bytes,
                        &signature_bytes,
                    ],
                )
                .await?;
        }
        client
            .execute(
                "INSERT INTO message_notifications (user_id, thread_id, message_id, sender_id, notification_type)
                 SELECT $1, $2, m.message_id, $3, 'thread_added'
                 FROM private_messages m
                 WHERE m.thread_id = $2
                   AND user_accepts_notifications($1)
                 ORDER BY m.message_id DESC
                 LIMIT 1",
                &[&request.user_id, &thread_id, &actor_id],
            )
            .await?;
        let participant = client
            .query_one(
                "SELECT tp.*, u.username
                 FROM thread_participants tp
                 JOIN users u ON u.userid = tp.user_id
                 WHERE tp.thread_id = $1 AND tp.user_id = $2",
                &[&thread_id, &request.user_id],
            )
            .await?;
        let response = ThreadParticipantResponse {
            participant_id: participant.get("participant_id"),
            user_id: participant.get("user_id"),
            username,
            role: participant.get("role"),
            joined_at: participant.get("joined_at"),
            is_active: participant.get("is_active"),
            unread_count: participant.get("unread_count"),
        };
        if let Ok(user_ids) = self.get_thread_participant_user_ids(thread_id).await {
            self.broadcast(
                user_ids,
                &WebSocketMessage::ParticipantAdded {
                    thread_id,
                    participant: response.clone(),
                },
                None,
            );
        }
        Ok(response)
    }

    pub async fn remove_participant(
        &self,
        actor_id: i32,
        thread_id: i64,
        user_id: i32,
    ) -> AppResult<()> {
        if actor_id != user_id {
            self.require_thread_admin(thread_id, actor_id).await?;
        }
        let client = self.pool.get().await?;
        let role: Option<String> = client
            .query_opt(
                "SELECT role FROM thread_participants
                 WHERE thread_id = $1 AND user_id = $2 AND is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?
            .map(|row| row.get("role"));
        let Some(role) = role else {
            return Err(AppError::NotFound("Participant not found".into()));
        };
        if role == "admin" {
            let admins: i64 = client
                .query_one(
                    "SELECT COUNT(*) FROM thread_participants
                     WHERE thread_id = $1 AND role = 'admin' AND is_active = TRUE",
                    &[&thread_id],
                )
                .await?
                .get(0);
            if admins <= 1 {
                return Err(AppError::BadRequest(
                    "The last admin cannot leave the thread".into(),
                ));
            }
        }
        let updated = client
            .execute(
                "UPDATE thread_participants
                 SET is_active = FALSE, left_at = CURRENT_TIMESTAMP
                 WHERE thread_id = $1 AND user_id = $2 AND is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?;
        if updated == 0 {
            return Err(AppError::NotFound("Participant not found".into()));
        }
        if let Ok(user_ids) = self.get_thread_participant_user_ids(thread_id).await {
            self.broadcast(
                user_ids,
                &WebSocketMessage::ParticipantRemoved {
                    thread_id,
                    user_id,
                    removed_by: actor_id,
                },
                None,
            );
        }
        Ok(())
    }

    pub async fn update_participant_role(
        &self,
        actor_id: i32,
        thread_id: i64,
        user_id: i32,
        role: ParticipantRole,
    ) -> AppResult<()> {
        self.require_thread_admin(thread_id, actor_id).await?;
        let client = self.pool.get().await?;
        let current: Option<String> = client
            .query_opt(
                "SELECT role FROM thread_participants
                 WHERE thread_id = $1 AND user_id = $2 AND is_active = TRUE",
                &[&thread_id, &user_id],
            )
            .await?
            .map(|row| row.get("role"));
        let Some(current) = current else {
            return Err(AppError::NotFound("Participant not found".into()));
        };
        if current == "admin" && role == ParticipantRole::Member {
            let admins: i64 = client
                .query_one(
                    "SELECT COUNT(*) FROM thread_participants
                     WHERE thread_id = $1 AND role = 'admin' AND is_active = TRUE",
                    &[&thread_id],
                )
                .await?
                .get(0);
            if admins <= 1 {
                return Err(AppError::BadRequest(
                    "The last admin cannot be demoted".into(),
                ));
            }
        }
        let role_name = role.to_string();
        client
            .execute(
                "UPDATE thread_participants SET role = $3
                 WHERE thread_id = $1 AND user_id = $2 AND is_active = TRUE",
                &[&thread_id, &user_id, &role_name],
            )
            .await?;
        if let Ok(user_ids) = self.get_thread_participant_user_ids(thread_id).await {
            self.broadcast(
                user_ids,
                &WebSocketMessage::ParticipantRoleUpdated {
                    thread_id,
                    user_id,
                    new_role: role,
                    updated_by: actor_id,
                },
                None,
            );
        }
        Ok(())
    }

    pub async fn block_user(
        &self,
        blocker_id: i32,
        request: BlockUserRequest,
    ) -> AppResult<BlockedUserResponse> {
        if blocker_id == request.user_id {
            return Err(AppError::BadRequest("You cannot block yourself".into()));
        }
        let client = self.pool.get().await?;
        let username: String = client
            .query_opt(
                "SELECT username FROM users WHERE userid = $1",
                &[&request.user_id],
            )
            .await?
            .map(|row| row.get("username"))
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;
        let row = client
            .query_one(
                "INSERT INTO user_message_blocks (blocker_id, blocked_id, reason)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (blocker_id, blocked_id)
                 DO UPDATE SET is_active = TRUE, reason = EXCLUDED.reason, blocked_at = CURRENT_TIMESTAMP
                 RETURNING block_id, blocked_at, reason",
                &[&blocker_id, &request.user_id, &request.reason],
            )
            .await?;
        Ok(BlockedUserResponse {
            block_id: row.get("block_id"),
            blocked_id: request.user_id,
            blocked_username: username,
            blocked_at: row.get("blocked_at"),
            reason: row.get("reason"),
        })
    }

    pub async fn unblock_user(&self, blocker_id: i32, blocked_id: i32) -> AppResult<()> {
        let client = self.pool.get().await?;
        let updated = client
            .execute(
                "UPDATE user_message_blocks
                 SET is_active = FALSE
                 WHERE blocker_id = $1 AND blocked_id = $2 AND is_active = TRUE",
                &[&blocker_id, &blocked_id],
            )
            .await?;
        if updated == 0 {
            return Err(AppError::NotFound("Block not found".into()));
        }
        Ok(())
    }

    pub async fn get_blocked_users(&self, blocker_id: i32) -> AppResult<Vec<BlockedUserResponse>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT b.block_id, b.blocked_id, u.username, b.blocked_at, b.reason
                 FROM user_message_blocks b
                 JOIN users u ON u.userid = b.blocked_id
                 WHERE b.blocker_id = $1 AND b.is_active = TRUE
                 ORDER BY b.blocked_at DESC",
                &[&blocker_id],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| BlockedUserResponse {
                block_id: row.get("block_id"),
                blocked_id: row.get("blocked_id"),
                blocked_username: row.get("username"),
                blocked_at: row.get("blocked_at"),
                reason: row.get("reason"),
            })
            .collect())
    }

    pub async fn get_notifications(
        &self,
        user_id: i32,
        query: GetNotificationsQuery,
    ) -> AppResult<NotificationListResponse> {
        let (page, per_page, offset) =
            crate::pagination::page_window(Some(query.page), Some(query.per_page), 20, 100);
        let type_filter = query.notification_type.map(|kind| kind.to_string());
        let client = self.pool.get().await?;
        let total_count: i64 = client
            .query_one(
                "SELECT COUNT(*) FROM message_notifications
                 WHERE user_id = $1
                   AND ($2 = FALSE OR is_read = FALSE)
                   AND ($3::text IS NULL OR notification_type = $3)",
                &[&user_id, &query.unread_only, &type_filter],
            )
            .await?
            .get(0);
        let unread_count: i64 = client
            .query_one(
                "SELECT COUNT(*) FROM message_notifications
                 WHERE user_id = $1 AND is_read = FALSE",
                &[&user_id],
            )
            .await?
            .get(0);
        let rows = client
            .query(
                "SELECT n.notification_id, n.thread_id, t.thread_name, n.message_id, n.sender_id,
                        u.username AS sender_username, n.notification_type, n.is_read, n.created_at, n.read_at
                 FROM message_notifications n
                 JOIN message_threads t ON t.thread_id = n.thread_id
                 JOIN users u ON u.userid = n.sender_id
                 WHERE n.user_id = $1
                   AND ($2 = FALSE OR n.is_read = FALSE)
                   AND ($3::text IS NULL OR n.notification_type = $3)
                 ORDER BY n.created_at DESC
                 LIMIT $4 OFFSET $5",
                &[&user_id, &query.unread_only, &type_filter, &per_page, &offset],
            )
            .await?;
        let notifications = rows
            .into_iter()
            .map(|row| NotificationResponse {
                notification_id: row.get("notification_id"),
                thread_id: row.get("thread_id"),
                thread_name: row.get("thread_name"),
                message_id: row.get("message_id"),
                sender_id: row.get("sender_id"),
                sender_username: row.get("sender_username"),
                notification_type: row.get("notification_type"),
                is_read: row.get("is_read"),
                created_at: row.get("created_at"),
                read_at: row.get("read_at"),
            })
            .collect();
        Ok(NotificationListResponse {
            notifications,
            unread_count,
            total_count,
            page,
            per_page,
            has_more: offset + per_page < total_count,
        })
    }

    pub async fn mark_notification_read(
        &self,
        user_id: i32,
        notification_id: i64,
    ) -> AppResult<()> {
        let client = self.pool.get().await?;
        let updated = client
            .execute(
                "UPDATE message_notifications
                 SET is_read = TRUE, read_at = CURRENT_TIMESTAMP
                 WHERE notification_id = $1 AND user_id = $2 AND is_read = FALSE",
                &[&notification_id, &user_id],
            )
            .await?;
        if updated == 0 {
            let exists = client
                .query_opt(
                    "SELECT 1 FROM message_notifications WHERE notification_id = $1 AND user_id = $2",
                    &[&notification_id, &user_id],
                )
                .await?;
            if exists.is_none() {
                return Err(AppError::NotFound("Notification not found".into()));
            }
        }
        Ok(())
    }
}
