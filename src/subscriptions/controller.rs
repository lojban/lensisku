use actix_web::{get, post, put, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde_json::json;

use super::dto::{
    NotificationListResponse, NotificationQuery, SubscriptionRequest, SubscriptionResponse,
    SubscriptionState,
};
use super::models::Subscription;
use crate::auth::Claims;

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
    claims: Claims,
    req: web::Json<SubscriptionRequest>,
) -> impl Responder {
    match super::service::subscribe_to_valsi(
        &pool,
        claims.sub,
        req.valsi_id,
        req.trigger_type.clone(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "message": format!("Failed to subscribe: {}", e)
        })),
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
    claims: Claims,
    path: web::Path<(i32, String)>,
) -> impl Responder {
    let (valsi_id, trigger_type) = path.into_inner();
    if super::models::SubscriptionTrigger::from_label(&trigger_type).is_none() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Unknown trigger type"
        }));
    }

    // The word alert toggle subscribes every trigger, so a valid label clears all of them.
    match super::service::unsubscribe_from_valsi(&pool, claims.sub, valsi_id, None).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "message": format!("Failed to unsubscribe: {}", e)
        })),
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
pub async fn get_subscriptions(pool: web::Data<Pool>, claims: Claims) -> impl Responder {
    match super::service::get_user_subscriptions(&pool, claims.sub).await {
        Ok(subscriptions) => HttpResponse::Ok().json(subscriptions),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get subscriptions: {}", e)
        })),
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
) -> impl Responder {
    let (page, per_page, _) = crate::pagination::page_window(query.page, query.per_page, 20, 100);
    let unread_only = query.unread_only.unwrap_or(false);

    match super::service::get_user_notifications(&pool, claims.sub, page, per_page, unread_only)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get notifications: {}", e)
        })),
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
) -> impl Responder {
    match super::service::mark_notifications_read(
        &pool,
        claims.sub,
        notification_ids.map(|ids| ids.into_inner()),
    )
    .await
    {
        Ok(count) => HttpResponse::Ok().json(json!({
            "marked_count": count
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to mark notifications as read: {}", e)
        })),
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
) -> impl Responder {
    match super::service::get_subscription_state(&pool, claims.sub, valsi_id.into_inner()).await {
        Ok(state) => HttpResponse::Ok().json(state),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get subscription state: {}", e)
        })),
    }
}
