use actix_web::{delete, get, post, put, web, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_lab::sse;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use std::sync::Arc;

use crate::auth::models::Claims;
use crate::error::AppError;
use crate::middleware::cache::RedisCache;

use super::chat_store::{
    self, AssistantChatPutBody, ChatCreateResult, ImportBody, ImportResponse,
};
use super::dto::ChatRequest;
use super::persist::ChatPersistState;
use super::stored_messages::{build_chat_messages_from_stored, strip_trailing_empty_assistant_stub};

const DEBUG_BODY_LIMIT: usize = 2000;

fn parse_chat_request_bytes(body: &actix_web::web::Bytes) -> Result<ChatRequest, AppError> {
    serde_json::from_slice(body).map_err(|e| {
        let snippet = if body.len() <= DEBUG_BODY_LIMIT {
            body.as_ref()
        } else {
            &body[..DEBUG_BODY_LIMIT]
        };
        let snippet_lossy = String::from_utf8_lossy(snippet);
        log::warn!(
            "Assistant /chat/stream JSON parse error: {}; raw body ({} bytes): {:?}",
            e,
            body.len(),
            snippet_lossy
        );
        AppError::BadRequest(format!(
            "Invalid request body: expected JSON with \"messages\". Parse error: {}. Body (first {} bytes): {}",
            e,
            body.len().min(DEBUG_BODY_LIMIT),
            snippet_lossy
        ))
    })
}

/// Anonymous / legacy stream: client sends full `messages` + `locale`; no DB persistence.
#[post("/chat/stream")]
pub async fn chat_stream_public(
    pool: web::Data<deadpool_postgres::Pool>,
    redis_cache: web::Data<RedisCache>,
    body: web::Bytes,
) -> Result<impl actix_web::Responder, AppError> {
    let request = parse_chat_request_bytes(&body)?;

    let (tx, rx) = mpsc::channel::<sse::Event>(32);
    let pool_clone = pool.get_ref().clone();
    let redis_cache = redis_cache.clone();

    actix_web::rt::spawn(async move {
        let result = super::run_agent_loop(
            &pool_clone,
            &request,
            Some(tx),
            Some(redis_cache.get_ref()),
            None,
        )
        .await;
        if let Err(ref e) = result {
            log::error!("Assistant stream agent loop error: {}", e);
            if let crate::error::AppError::ExternalServiceWithRaw { raw_response, .. }
            | crate::error::AppError::ExternalServiceRetryable { raw_response, .. } = e
            {
                const LOG_PREVIEW_LEN: usize = 2000;
                if raw_response.len() <= LOG_PREVIEW_LEN {
                    log::error!("Assistant raw_response: {}", raw_response);
                } else {
                    log::error!(
                        "Assistant raw_response (first {} of {} bytes): {}...",
                        LOG_PREVIEW_LEN,
                        raw_response.len(),
                        &raw_response[..LOG_PREVIEW_LEN]
                    );
                }
            }
        }
    });

    Ok(sse::Sse::from_infallible_receiver(rx))
}

#[derive(Debug, Deserialize)]
pub struct LocaleBody {
    pub locale: Option<String>,
}

#[get("/chats")]
pub async fn list_chats(
    pool: web::Data<deadpool_postgres::Pool>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let list = chat_store::list_chats(pool.get_ref(), claims.sub).await?;
    Ok(HttpResponse::Ok().json(list))
}

#[post("/chats")]
pub async fn create_chat(
    pool: web::Data<deadpool_postgres::Pool>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let row: ChatCreateResult = chat_store::create_chat(pool.get_ref(), claims.sub).await?;
    Ok(HttpResponse::Ok().json(row))
}

#[post("/chats/import")]
pub async fn import_chats(
    pool: web::Data<deadpool_postgres::Pool>,
    claims: Claims,
    body: web::Json<ImportBody>,
) -> Result<HttpResponse, AppError> {
    let res: ImportResponse = chat_store::import_sessions(pool.get_ref(), claims.sub, &body).await?;
    Ok(HttpResponse::Ok().json(res))
}

#[get("/chats/{chat_id}")]
pub async fn get_chat(
    pool: web::Data<deadpool_postgres::Pool>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let chat_id = path.into_inner();
    let row = chat_store::get_chat(pool.get_ref(), claims.sub, chat_id)
        .await?
        .ok_or_else(|| AppError::NotFound("assistant chat not found".into()))?;
    Ok(HttpResponse::Ok().json(row))
}

#[put("/chats/{chat_id}")]
pub async fn put_chat(
    pool: web::Data<deadpool_postgres::Pool>,
    claims: Claims,
    path: web::Path<Uuid>,
    body: web::Json<AssistantChatPutBody>,
) -> Result<HttpResponse, AppError> {
    let chat_id = path.into_inner();
    chat_store::update_chat(pool.get_ref(), claims.sub, chat_id, &body).await?;
    Ok(HttpResponse::Ok().finish())
}

#[delete("/chats/{chat_id}")]
pub async fn delete_chat(
    pool: web::Data<deadpool_postgres::Pool>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let chat_id = path.into_inner();
    chat_store::delete_chat(pool.get_ref(), claims.sub, chat_id).await?;
    Ok(HttpResponse::Ok().finish())
}

/// Streams assistant output; loads conversation from PostgreSQL and persists each SSE payload before sending.
#[post("/chats/{chat_id}/stream")]
pub async fn chat_stream_by_id(
    pool: web::Data<deadpool_postgres::Pool>,
    redis_cache: web::Data<RedisCache>,
    claims: Claims,
    path: web::Path<Uuid>,
    body: web::Json<LocaleBody>,
) -> Result<impl actix_web::Responder, AppError> {
    let chat_id = path.into_inner();
    let chat = chat_store::get_chat(pool.get_ref(), claims.sub, chat_id)
        .await?
        .ok_or_else(|| AppError::NotFound("assistant chat not found".into()))?;

    let primary = chat.primary_model_id.clone();
    let arr: Vec<Value> = chat
        .messages
        .as_array()
        .cloned()
        .unwrap_or_default();
    if arr.is_empty() {
        return Err(AppError::BadRequest("no messages in chat".into()));
    }
    let last_role = arr
        .last()
        .and_then(|m| m.get("role"))
        .and_then(|r| r.as_str());
    if last_role == Some("user") {
        return Err(AppError::BadRequest(
            "last message must be assistant (save placeholder before streaming)".into(),
        ));
    }
    if last_role != Some("assistant") {
        return Err(AppError::BadRequest("invalid message thread".into()));
    }

    let assistant_index = arr.len().saturating_sub(1);

    let mut api_arr = arr.clone();
    strip_trailing_empty_assistant_stub(&mut api_arr);
    let msgs = build_chat_messages_from_stored(&api_arr, primary.as_deref())?;
    let request = ChatRequest {
        messages: msgs,
        locale: body.locale.clone(),
    };

    let messages_arc = Arc::new(Mutex::new(Value::Array(arr)));
    let persist = Arc::new(ChatPersistState {
        pool: pool.get_ref().clone(),
        user_id: claims.sub,
        chat_id,
        messages: messages_arc,
        assistant_index,
    });

    let (tx, rx) = mpsc::channel::<sse::Event>(32);
    let pool_clone = pool.get_ref().clone();
    let redis_cache = redis_cache.clone();
    let request = request.clone();

    actix_web::rt::spawn(async move {
        let result = super::run_agent_loop(
            &pool_clone,
            &request,
            Some(tx),
            Some(redis_cache.get_ref()),
            Some(persist),
        )
        .await;
        if let Err(ref e) = result {
            log::error!("Assistant stream agent loop error: {}", e);
            if let crate::error::AppError::ExternalServiceWithRaw { raw_response, .. }
            | crate::error::AppError::ExternalServiceRetryable { raw_response, .. } = e
            {
                const LOG_PREVIEW_LEN: usize = 2000;
                if raw_response.len() <= LOG_PREVIEW_LEN {
                    log::error!("Assistant raw_response: {}", raw_response);
                } else {
                    log::error!(
                        "Assistant raw_response (first {} of {} bytes): {}...",
                        LOG_PREVIEW_LEN,
                        raw_response.len(),
                        &raw_response[..LOG_PREVIEW_LEN]
                    );
                }
            }
        }
    });

    Ok(sse::Sse::from_infallible_receiver(rx))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("assistant")
            .service(chat_stream_public)
            .service(
                web::scope("")
                    .wrap(HttpAuthentication::bearer(crate::auth::validator))
                    .service(list_chats)
                    .service(create_chat)
                    .service(import_chats)
                    .service(get_chat)
                    .service(put_chat)
                    .service(delete_chat)
                    .service(chat_stream_by_id),
            ),
    );
}
