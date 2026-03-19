use actix_web::{post, web, HttpResponse};
use actix_web_lab::sse;
use tokio::sync::mpsc;

use crate::error::AppError;

use super::dto::{ChatRequest, ChatResponse};

const DEBUG_BODY_LIMIT: usize = 2000;

fn parse_chat_request(body: &web::Bytes) -> Result<ChatRequest, AppError> {
    serde_json::from_slice(body).map_err(|e| {
        let snippet = if body.len() <= DEBUG_BODY_LIMIT {
            body.as_ref()
        } else {
            &body[..DEBUG_BODY_LIMIT]
        };
        let snippet_lossy = String::from_utf8_lossy(snippet);
        log::warn!(
            "Assistant /chat JSON parse error: {}; raw body ({} bytes): {:?}",
            e,
            body.len(),
            snippet_lossy
        );
        AppError::BadRequest(format!(
            "Invalid request body: expected JSON with \"messages\" (array of {{role, content}}) and optional \"locale\". Parse error: {}. Body (first {} bytes): {}",
            e,
            body.len().min(DEBUG_BODY_LIMIT),
            snippet_lossy
        ))
    })
}

#[post("/chat")]
pub async fn chat(
    pool: web::Data<deadpool_postgres::Pool>,
    body: web::Bytes,
) -> Result<HttpResponse, AppError> {
    let request = parse_chat_request(&body)?;

    let (reply, steps) = super::handle_chat(pool.get_ref(), request).await?;

    Ok(HttpResponse::Ok().json(ChatResponse {
        reply,
        steps: if steps.is_empty() {
            None
        } else {
            Some(steps)
        },
    }))
}

/// Streams assistant steps and final reply via SSE. Same pattern as jbovlaste bulk_import
/// (see jbovlaste/broadcast.rs): one channel, `sse::Data::new(json_str).into()` per event,
/// response body = `Sse::from_infallible_receiver(rx)`. Sender is dropped when the agent loop
/// finishes so the client sees the stream end.
#[post("/chat/stream")]
pub async fn chat_stream(
    pool: web::Data<deadpool_postgres::Pool>,
    body: web::Bytes,
) -> Result<impl actix_web::Responder, AppError> {
    let request = parse_chat_request(&body)?;

    let (tx, rx) = mpsc::channel::<sse::Event>(32);
    let pool_clone = pool.get_ref().clone();

    actix_web::rt::spawn(async move {
        let result = super::run_agent_loop(&pool_clone, &request, Some(tx)).await;
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
