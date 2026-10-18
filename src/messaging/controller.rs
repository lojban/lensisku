use actix_web::{delete, get, post, put, web, HttpResponse, Result};
use validator::Validate;

use super::{dto::*, service::MessagingService};
use crate::{auth::Claims, AppError};

#[utoipa::path(
    get,
    path = "/messaging/threads",
    tag = "messaging",
    summary = "Get user's message threads",
    params(
        ("page" = i64, Query, description = "Page number"),
        ("per_page" = i64, Query, description = "Items per page"),
        ("thread_type" = Option<String>, Query, description = "Filter by thread type"),
        ("unread_only" = bool, Query, description = "Only show threads with unread messages")
    ),
    responses(
        (status = 200, description = "Thread list retrieved successfully", body = ThreadListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/threads")]
pub async fn get_threads(
    claims: Claims,
    query: web::Query<GetThreadsQuery>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    query
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result = service
        .get_user_threads(claims.sub, query.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    path = "/messaging/threads",
    tag = "messaging",
    summary = "Create a new message thread",
    request_body = CreateThreadRequest,
    responses(
        (status = 201, description = "Thread created successfully", body = ThreadResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - user is blocked"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/threads")]
pub async fn create_thread(
    claims: Claims,
    request: web::Json<CreateThreadRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result = service
        .create_thread(claims.sub, request.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(result))
}

#[utoipa::path(
    get,
    path = "/messaging/threads/{thread_id}",
    tag = "messaging",
    summary = "Get a specific thread",
    params(
        ("thread_id" = i64, Path, description = "Thread ID")
    ),
    responses(
        (status = 200, description = "Thread retrieved successfully", body = ThreadResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Thread not found or access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/threads/{thread_id}")]
pub async fn get_thread(
    claims: Claims,
    path: web::Path<i64>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let thread_id = path.into_inner();
    let result = service.get_thread(thread_id, claims.sub).await?;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    put,
    path = "/messaging/threads/{thread_id}",
    tag = "messaging",
    summary = "Update a thread",
    params(
        ("thread_id" = i64, Path, description = "Thread ID")
    ),
    request_body = UpdateThreadRequest,
    responses(
        (status = 200, description = "Thread updated successfully", body = ThreadResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not an admin"),
        (status = 404, description = "Thread not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[put("/threads/{thread_id}")]
pub async fn update_thread(
    claims: Claims,
    path: web::Path<i64>,
    request: web::Json<UpdateThreadRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let thread_id = path.into_inner();
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result = service
        .update_thread(thread_id, claims.sub, request.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    delete,
    path = "/messaging/threads/{thread_id}",
    tag = "messaging",
    summary = "Delete a thread",
    params(
        ("thread_id" = i64, Path, description = "Thread ID")
    ),
    responses(
        (status = 204, description = "Thread deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not an admin or creator"),
        (status = 404, description = "Thread not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[delete("/threads/{thread_id}")]
pub async fn delete_thread(
    claims: Claims,
    path: web::Path<i64>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let thread_id = path.into_inner();
    service.delete_thread(thread_id, claims.sub).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/messaging/threads/{thread_id}/messages",
    tag = "messaging",
    summary = "Get messages in a thread",
    params(
        ("thread_id" = i64, Path, description = "Thread ID"),
        ("page" = i64, Query, description = "Page number"),
        ("per_page" = i64, Query, description = "Items per page"),
        ("before_message_id" = Option<i64>, Query, description = "Get messages before this ID"),
        ("after_message_id" = Option<i64>, Query, description = "Get messages after this ID"),
        ("message_type" = Option<String>, Query, description = "Filter by message type")
    ),
    responses(
        (status = 200, description = "Messages retrieved successfully", body = MessageListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Thread not found or access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/threads/{thread_id}/messages")]
pub async fn get_thread_messages(
    claims: Claims,
    path: web::Path<i64>,
    query: web::Query<GetMessagesQuery>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let thread_id = path.into_inner();
    query
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let query_data = query.into_inner();

    let result = service
        .get_thread_messages(thread_id, claims.sub, query_data)
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    path = "/messaging/messages",
    tag = "messaging",
    summary = "Send a message to a thread",
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Message sent successfully", body = MessageResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Thread not found or access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/messages")]
pub async fn send_message(
    claims: Claims,
    request: web::Json<SendMessageRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result = service
        .send_message(claims.sub, request.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(result))
}

#[utoipa::path(
    get,
    path = "/messaging/messages/{message_id}",
    tag = "messaging",
    summary = "Get a specific message",
    params(
        ("message_id" = i64, Path, description = "Message ID")
    ),
    responses(
        (status = 200, description = "Message retrieved successfully", body = MessageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Message not found or access denied"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/messages/{message_id}")]
pub async fn get_message(
    claims: Claims,
    path: web::Path<i64>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let message_id = path.into_inner();
    let result = service.get_message(message_id, claims.sub).await?;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    put,
    path = "/messaging/messages/{message_id}",
    tag = "messaging",
    summary = "Update a message",
    params(
        ("message_id" = i64, Path, description = "Message ID")
    ),
    request_body = UpdateMessageRequest,
    responses(
        (status = 200, description = "Message updated successfully", body = MessageResponse),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not the sender"),
        (status = 404, description = "Message not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[put("/messages/{message_id}")]
pub async fn update_message(
    claims: Claims,
    path: web::Path<i64>,
    request: web::Json<UpdateMessageRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let message = service
        .update_message(claims.sub, path.into_inner(), request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(message))
}

#[utoipa::path(
    delete,
    path = "/messaging/messages/{message_id}",
    tag = "messaging",
    summary = "Delete a message",
    params(
        ("message_id" = i64, Path, description = "Message ID")
    ),
    responses(
        (status = 204, description = "Message deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not the sender or admin"),
        (status = 404, description = "Message not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[delete("/messages/{message_id}")]
pub async fn delete_message(
    claims: Claims,
    path: web::Path<i64>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    service
        .delete_message(claims.sub, path.into_inner())
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    path = "/messaging/threads/{thread_id}/participants",
    tag = "messaging",
    summary = "Add a participant to a thread",
    params(
        ("thread_id" = i64, Path, description = "Thread ID")
    ),
    request_body = AddParticipantRequest,
    responses(
        (status = 201, description = "Participant added successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not an admin"),
        (status = 404, description = "Thread not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/threads/{thread_id}/participants")]
pub async fn add_participant(
    claims: Claims,
    path: web::Path<i64>,
    request: web::Json<AddParticipantRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let participant = service
        .add_participant(claims.sub, path.into_inner(), request.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(participant))
}

#[utoipa::path(
    delete,
    path = "/messaging/threads/{thread_id}/participants/{user_id}",
    tag = "messaging",
    summary = "Remove a participant from a thread",
    params(
        ("thread_id" = i64, Path, description = "Thread ID"),
        ("user_id" = i32, Path, description = "User ID to remove")
    ),
    responses(
        (status = 204, description = "Participant removed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not an admin"),
        (status = 404, description = "Thread or participant not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[delete("/threads/{thread_id}/participants/{user_id}")]
pub async fn remove_participant(
    claims: Claims,
    path: web::Path<(i64, i32)>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let (thread_id, user_id) = path.into_inner();
    service
        .remove_participant(claims.sub, thread_id, user_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    put,
    path = "/messaging/threads/{thread_id}/participants/{user_id}/role",
    tag = "messaging",
    summary = "Update participant role",
    params(
        ("thread_id" = i64, Path, description = "Thread ID"),
        ("user_id" = i32, Path, description = "User ID")
    ),
    request_body = UpdateParticipantRoleRequest,
    responses(
        (status = 200, description = "Participant role updated successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not an admin"),
        (status = 404, description = "Thread or participant not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[put("/threads/{thread_id}/participants/{user_id}/role")]
pub async fn update_participant_role(
    claims: Claims,
    path: web::Path<(i64, i32)>,
    request: web::Json<UpdateParticipantRoleRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let (thread_id, user_id) = path.into_inner();
    service
        .update_participant_role(claims.sub, thread_id, user_id, request.role)
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    post,
    path = "/messaging/blocks",
    tag = "messaging",
    summary = "Block a user",
    request_body = BlockUserRequest,
    responses(
        (status = 201, description = "User blocked successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/blocks")]
pub async fn block_user(
    claims: Claims,
    request: web::Json<BlockUserRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let blocked = service.block_user(claims.sub, request.into_inner()).await?;
    Ok(HttpResponse::Created().json(blocked))
}

#[utoipa::path(
    delete,
    path = "/messaging/blocks/{user_id}",
    tag = "messaging",
    summary = "Unblock a user",
    params(
        ("user_id" = i32, Path, description = "User ID to unblock")
    ),
    responses(
        (status = 204, description = "User unblocked successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Block not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[delete("/blocks/{user_id}")]
pub async fn unblock_user(
    claims: Claims,
    path: web::Path<i32>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    service.unblock_user(claims.sub, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/messaging/blocks",
    tag = "messaging",
    summary = "Get blocked users",
    responses(
        (status = 200, description = "Blocked users retrieved successfully", body = Vec<BlockedUserResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/blocks")]
pub async fn get_blocked_users(
    claims: Claims,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let blocked = service.get_blocked_users(claims.sub).await?;
    Ok(HttpResponse::Ok().json(blocked))
}

#[utoipa::path(
    get,
    path = "/messaging/notifications",
    tag = "messaging",
    summary = "Get user notifications",
    params(
        ("page" = i64, Query, description = "Page number"),
        ("per_page" = i64, Query, description = "Items per page"),
        ("unread_only" = bool, Query, description = "Only show unread notifications"),
        ("notification_type" = Option<String>, Query, description = "Filter by notification type")
    ),
    responses(
        (status = 200, description = "Notifications retrieved successfully", body = NotificationListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/notifications")]
pub async fn get_notifications(
    claims: Claims,
    query: web::Query<GetNotificationsQuery>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let notifications = service
        .get_notifications(claims.sub, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(notifications))
}

#[utoipa::path(
    put,
    path = "/messaging/notifications/{notification_id}/read",
    tag = "messaging",
    summary = "Mark notification as read",
    params(
        ("notification_id" = i64, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification marked as read"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notification not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[put("/notifications/{notification_id}/read")]
pub async fn mark_notification_read(
    claims: Claims,
    path: web::Path<i64>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    service
        .mark_notification_read(claims.sub, path.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    post,
    path = "/messaging/webrtc/signal",
    tag = "messaging",
    summary = "Send WebRTC signaling message",
    request_body = SendSignalRequest,
    responses(
        (status = 201, description = "Signal sent successfully"),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn webrtc_signal(
    claims: Claims,
    request: web::Json<super::dto::SendSignalRequest>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    super::webrtc_handler::send_signal(claims, request, service).await
}

#[utoipa::path(
    post,
    path = "/messaging/threads/{thread_id}/read",
    tag = "messaging",
    summary = "Mark thread messages as read",
    params(
        ("thread_id" = i64, Path, description = "Thread ID")
    ),
    responses(
        (status = 200, description = "Thread marked as read"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Thread not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/threads/{thread_id}/read")]
pub async fn mark_thread_read(
    claims: Claims,
    path: web::Path<i64>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    let thread_id = path.into_inner();
    service.mark_thread_read(claims.sub, thread_id).await?;
    Ok(HttpResponse::Ok().finish())
}

// WebRTC signaling handlers
pub async fn get_webrtc_signals(
    claims: Claims,
    path: web::Path<i32>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    super::webrtc_handler::get_pending_signals(claims, path, service).await
}

pub async fn mark_signal_processed(
    claims: Claims,
    path: web::Path<i64>,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    super::webrtc_handler::mark_signal_processed(claims, path, service).await
}

// Health check endpoint for WebSocket/WebRTC services
#[utoipa::path(
    get,
    path = "/messaging/health",
    tag = "messaging",
    summary = "Health check for messaging services",
    responses(
        (status = 200, description = "All services healthy", body = serde_json::Value),
        (status = 503, description = "Service unavailable")
    )
)]
pub async fn health_check() -> Result<HttpResponse, AppError> {
    let health_status = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "services": {
            "websocket": {
                "status": "enabled",
                "endpoints": [
                    "/messaging/ws",
                    "/messaging/ws/{thread_id}"
                ]
            },
            "webrtc": {
                "status": "configured",
                "endpoints": [
                    "/messaging/webrtc/signal",
                    "/messaging/webrtc/signals/{user_id}",
                    "/messaging/webrtc/signal/{signal_id}/processed"
                ],
                "stun_server": std::env::var("STUN_SERVER_URL").unwrap_or_else(|_| "stun:lensisku.lojban.org:3478".to_string()),
                "turn_server": std::env::var("TURN_SERVER_URL").unwrap_or_else(|_| "turn:lensisku.lojban.org:3478".to_string())
            }
        }
    });

    Ok(HttpResponse::Ok().json(health_status))
}

// Additional WebRTC endpoints
pub async fn get_webrtc_config(claims: Claims) -> Result<HttpResponse, AppError> {
    super::webrtc_handler::get_webrtc_config(claims).await
}

pub async fn get_active_calls(
    claims: Claims,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    super::webrtc_handler::get_active_calls(claims, service).await
}

pub async fn cleanup_expired_signals(
    claims: Claims,
    service: web::Data<MessagingService>,
) -> Result<HttpResponse, AppError> {
    super::webrtc_handler::cleanup_expired_signals(claims, service).await
}
