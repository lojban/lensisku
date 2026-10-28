use actix_web::{get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use serde_json::json;

use super::dto::{
    NotificationListResponse, NotificationQuery, SubscriptionRequest, SubscriptionResponse,
    SubscriptionState,
};
use super::models::Subscription;
use crate::auth::Claims;
use crate::middleware::cache::RedisCache;
use crate::AppError;

async fn invalidate_news(redis_cache: &RedisCache, user_id: i32) {
    if let Err(e) = redis_cache.invalidate_user_recent_changes(user_id).await {
        log::error!("Failed to invalidate recent changes cache for user {user_id}: {e}");
    }
}

#[utoipa::path(
    post,
    path = "/subscriptions/subscribe",
    tag = "subscriptions",
    request_body = SubscriptionRequest,
    responses(
        (status = 200, description = "Successfully subscribed to valsi", body = SubscriptionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Subscribe to valsi",
    description = "Subscribe to notifications for a specific valsi with specified trigger type"
)]
#[post("/subscribe")]
pub async fn subscribe(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
    claims: Claims,
    req: web::Json<SubscriptionRequest>,
) -> Result<HttpResponse, AppError> {
    match super::service::subscribe_to_valsi(
        &pool,
        claims.sub,
        req.valsi_id,
        req.trigger_type.clone(),
    )
    .await
    {
        Ok(response) if response.success => {
            invalidate_news(&redis_cache, claims.sub).await;
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(response) => Err(AppError::NotFound(response.message)),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to subscribe: {}",
            e.detail()
        ))),
    }
}

#[utoipa::path(
    post,
    path = "/subscriptions/{valsi_id}/unsubscribe/{trigger_type}",
    tag = "subscriptions",
    params(
        ("valsi_id" = i32, Path, description = "Valsi ID"),
        ("trigger_type" = Option<String>, Path, description = "Trigger type (comment, definition, or edit)")
    ),
    responses(
        (status = 200, description = "Successfully unsubscribed from valsi", body = SubscriptionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Unsubscribe from valsi notifications",
    description = "Unsubscribe from notifications for a specific valsi and trigger type. If trigger_type is not specified, \
                  unsubscribes from all notification types for the valsi."
)]
#[post("/{valsi_id}/unsubscribe/{trigger_type}")]
pub async fn unsubscribe(
    pool: web::Data<Pool>,
    redis_cache: web::Data<RedisCache>,
    claims: Claims,
    path: web::Path<(i32, String)>,
) -> Result<HttpResponse, AppError> {
    let (valsi_id, trigger_type) = path.into_inner();
    let Some(trigger) = super::models::SubscriptionTrigger::from_label(&trigger_type) else {
        return Err(AppError::BadRequest("Unknown trigger type".into()));
    };

    match super::service::unsubscribe_from_valsi(&pool, claims.sub, valsi_id, trigger).await {
        Ok(response) if response.success => {
            invalidate_news(&redis_cache, claims.sub).await;
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(response) => Err(AppError::NotFound(response.message)),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to unsubscribe: {}",
            e.detail()
        ))),
    }
}

#[utoipa::path(
    get,
    path = "/subscriptions",
    tag = "subscriptions",
    responses(
        (status = 200, description = "List of user's subscriptions", body = Vec<Subscription>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "List user subscriptions",
    description = "Retrieves a list of all active subscriptions for the authenticated user. Each subscription includes \
                  the valsi details and trigger types the user is subscribed to."
)]
#[get("")]
pub async fn get_subscriptions(
    pool: web::Data<Pool>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    match super::service::get_user_subscriptions(&pool, claims.sub).await {
        Ok(subscriptions) => Ok(HttpResponse::Ok().json(subscriptions)),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to get subscriptions: {}",
            e.detail()
        ))),
    }
}

#[utoipa::path(
    get,
    path = "/subscriptions/notifications",
    tag = "subscriptions",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("unread_only" = Option<bool>, Query, description = "Show only unread notifications")
    ),
    responses(
        (status = 200, description = "List of notifications", body = NotificationListResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get user notifications",
    description = "Retrieves a paginated list of notifications for the authenticated user. \
                  Includes total count and unread count in the response. \
                  Notifications are ordered by creation date descending."
)]
#[get("/notifications")]
pub async fn get_notifications(
    pool: web::Data<Pool>,
    claims: Claims,
    query: web::Query<NotificationQuery>,
) -> Result<HttpResponse, AppError> {
    let (page, per_page, _) = crate::pagination::page_window(query.page, query.per_page, 20, 100);
    let unread_only = query.unread_only.unwrap_or(false);

    match super::service::get_user_notifications(&pool, claims.sub, page, per_page, unread_only)
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to get notifications: {}",
            e.detail()
        ))),
    }
}

#[utoipa::path(
    put,
    path = "/subscriptions/notifications/mark-read",
    tag = "subscriptions",
    request_body = Option<Vec<i32>>,
    responses(
        (status = 200, description = "Number of notifications marked as read", body = i64),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Mark notifications as read",
    description = "Marks specified notifications as read for the authenticated user. \
                  If no notification IDs are provided, marks all unread notifications as read. \
                  Returns the number of notifications that were marked as read."
)]
#[put("/notifications/mark-read")]
pub async fn mark_notifications_read(
    pool: web::Data<Pool>,
    claims: Claims,
    notification_ids: Option<web::Json<Vec<i32>>>,
) -> Result<HttpResponse, AppError> {
    match super::service::mark_notifications_read(
        &pool,
        claims.sub,
        notification_ids.map(|ids| ids.into_inner()),
    )
    .await
    {
        Ok(count) => Ok(HttpResponse::Ok().json(json!({
            "marked_count": count
        }))),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to mark notifications as read: {}",
            e.detail()
        ))),
    }
}

#[utoipa::path(
    get,
    path = "/subscriptions/{valsi_id}/state",
    tag = "subscriptions",
    params(
        ("valsi_id" = i32, Path, description = "Valsi ID")
    ),
    responses(
        (status = 200, description = "Subscription state", body = SubscriptionState),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get subscription state",
    description = "Retrieves the current subscription state for a specific valsi. \
                  Returns whether the user is subscribed and which trigger types are active."
)]
#[get("/{valsi_id}/state")]
pub async fn get_subscription_state(
    pool: web::Data<Pool>,
    claims: Claims,
    valsi_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    match super::service::get_subscription_state(&pool, claims.sub, valsi_id.into_inner()).await {
        Ok(state) => Ok(HttpResponse::Ok().json(state)),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to get subscription state: {}",
            e.detail()
        ))),
    }
}
