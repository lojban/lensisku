use actix_web::{get, post, web, HttpResponse, Responder};
// use actix_web_grants::protect;
use deadpool_postgres::Pool;
use serde_json::json;

use super::{service, Message, SpamVoteResponse, ThreadQuery, ThreadResponse};
use crate::auth::Claims;

#[utoipa::path(
    post,
    tag = "mail",
    path = "/mail/messages/{id}/spam-vote",
    params(
        ("id" = i32, Path, description = "Message ID to vote for")
    ),
    responses(
        (status = 200, description = "Vote registered successfully", body = SpamVoteResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Message not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Vote for a message as spam",
    description = "Allows an authenticated user to mark a message as spam. Each user can vote once per message."
)]
#[post("/messages/{id}/spam-vote")]
// #[protect("vote_spam_message")]
pub async fn vote_spam_message(
    pool: web::Data<Pool>,
    id: web::Path<i32>,
    claims: Claims,
) -> impl Responder {
    let message_id = id.into_inner();
    match service::vote_spam(&pool, message_id, claims.sub).await {
        Ok((spam_vote_count, user_voted)) => HttpResponse::Ok().json(SpamVoteResponse {
            message_id,
            spam_vote_count,
            success: true,
            user_voted,
        }),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({"error": "Failed to record spam vote", "details": e.to_string()})),
    }
}

#[utoipa::path(
    get,
    tag = "mail",
    path = "/mail/message/{id}",
    params(
        ("id" = i32, Path, description = "Message ID")
    ),
    responses(
        (status = 200, description = "Message details", body = Message),
        (status = 404, description = "Message not found")
    ),
    summary = "Get single message",
    description = "Retrieve a specific message by its ID. Returns the complete message including headers, \
                  content, and metadata. If the message is not found, returns a 404 status.",
)]
#[get("/message/{id}")]
pub async fn get_message(
    pool: web::Data<Pool>,
    id: web::Path<i32>,
    claims: Option<Claims>,
) -> impl Responder {
    let user_id = claims.map(|c| c.sub);
    match service::get_message(&pool, id.into_inner(), user_id).await {
        Ok(Some(message)) => HttpResponse::Ok().json(message),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

#[utoipa::path(
    get,
    tag = "mail",
    path = "/mail/thread",
    params(
        ("subject" = String, Query, description = "Thread subject"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (date)"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc or desc)"),
        ("include_content" = Option<bool>, Query, description = "Include message content")
    ),
    responses(
        (status = 200, description = "Thread messages", body = ThreadResponse),
        (status = 500, description = "Internal server error")
    ),
    summary = "Show message thread",
    description = "Retrieve all messages in a thread based on the subject. The subject is normalized by \
                  removing common prefixes (Re:, [tags], etc) to group related messages. Results are \
                  paginated and can be sorted chronologically. Message content can be optionally included.",
)]
#[get("/thread")]
pub async fn show_thread(pool: web::Data<Pool>, query: web::Query<ThreadQuery>) -> impl Responder {
    match service::show_thread(&pool, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}
