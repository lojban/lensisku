use crate::{AppError, AppResult};
use deadpool_postgres::Pool;
use chrono::Utc;
use base64::{Engine as _, engine::general_purpose};

use super::models::*;
use super::dto::*;

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
        where_clause, param_index, param_index + 1
    );

    params.push(Box::new(query.per_page));
    params.push(Box::new(offset));

    (query_str, params)
}

pub struct MessagingService {
    pool: Pool,
}

impl MessagingService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
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
                "SELECT * FROM get_user_message_threads($1)
                 ORDER BY last_message_at DESC NULLS LAST, created_at DESC
                 LIMIT $2 OFFSET $3",
                &[&user_id, &query.per_page, &offset],
            )
            .await?;

        let mut threads = Vec::new();
        for row in rows {
            let thread_id: i64 = row.get("thread_id");
            let participants = self.get_thread_participants(thread_id, Some(user_id)).await?;
            
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
            for &participant_id in participant_ids {
                if self.is_user_blocked(participant_id, user_id).await? {
                    return Err(AppError::Auth("User is blocked by participant".to_string()));
                }
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
            for participant_id in participant_ids {
                transaction
                    .execute(
                        "INSERT INTO thread_participants (thread_id, user_id, role)
                         VALUES ($1, $2, 'member')
                         ON CONFLICT (thread_id, user_id) DO NOTHING",
                        &[&thread_id, &participant_id],
                    )
                    .await?;
            }
        }

        transaction.commit().await?;

        // Get the complete thread with participants
        self.get_thread(thread_id, user_id).await
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
            return Err(AppError::NotFound("Thread not found or access denied".to_string()));
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

        let participants = self.get_thread_participants(thread_id, Some(user_id)).await?;

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
                    client.execute(&query, &[&name as &(dyn postgres_types::ToSql + Sync), &max_participants, &is_active, &thread_id]).await?
                } else {
                    client.execute(&query, &[&name as &(dyn postgres_types::ToSql + Sync), &max_participants, &thread_id]).await?
                }
            } else if let Some(is_active) = request.is_active {
                client.execute(&query, &[&name as &(dyn postgres_types::ToSql + Sync), &is_active, &thread_id]).await?
            } else {
                client.execute(&query, &[&name as &(dyn postgres_types::ToSql + Sync), &thread_id]).await?
            }
        } else if let Some(max_participants) = request.max_participants {
            if let Some(is_active) = request.is_active {
                client.execute(&query, &[&max_participants, &is_active, &thread_id]).await?
            } else {
                client.execute(&query, &[&max_participants, &thread_id]).await?
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
            return Err(AppError::Auth("Only admins or creators can delete threads".to_string()));
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
            return Err(AppError::NotFound("Thread not found or access denied".to_string()));
        }

        let offset = (query.page - 1) * query.per_page;
        
        // Build dynamic query based on parameters
        let (query_str, params) = build_message_query(&query, thread_id, offset);
        
        // Convert Box<dyn ToSql> to references for the query
        let param_refs: Vec<&(dyn postgres_types::ToSql + Sync)> = params.iter()
            .map(|p| p.as_ref())
            .collect();
        
        let rows = client
            .query(&query_str, &param_refs)
            .await?;

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
                sender_key_signature: sender_key_signature.map(|sig| general_purpose::STANDARD.encode(&sig)),
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
            return Err(AppError::NotFound("Thread not found or access denied".to_string()));
        }

        // Decode base64 nonce
        let content_nonce = general_purpose::STANDARD.decode(&request.content_nonce)
            .map_err(|_| AppError::BadRequest("Invalid content nonce format".to_string()))?;

        let sender_key_signature = request.sender_key_signature
            .as_ref()
            .and_then(|sig| general_purpose::STANDARD.decode(sig).ok());

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
                 WHERE tp.thread_id = $1 AND tp.user_id != $3 AND tp.is_active = TRUE",
                &[&request.thread_id, &message_id, &user_id],
            )
            .await?;

        transaction.commit().await?;

        // Get the complete message with username
        self.get_message(message_id, user_id).await
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

            Ok(MessageResponse {
                message_id: row.get("message_id"),
                thread_id: row.get("thread_id"),
                sender_id: row.get("sender_id"),
                username: row.get("username"),
                message_type: row.get("message_type"),
                encrypted_content: row.get("encrypted_content"),
                content_nonce: general_purpose::STANDARD.encode(&content_nonce),
                sender_key_signature: sender_key_signature.map(|sig| general_purpose::STANDARD.encode(&sig)),
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
    async fn get_thread_participants(&self, thread_id: i64, exclude_user_id: Option<i32>) -> AppResult<Vec<ThreadParticipantResponse>> {
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

    pub async fn get_pending_webrtc_signals(&self, user_id: i32) -> AppResult<Vec<WebRTCSignaling>> {
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

    pub async fn mark_webrtc_signal_processed(&self, signal_id: i64, user_id: i32) -> AppResult<()> {
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
}
