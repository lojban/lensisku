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
        log::warn!(
            "Assistant /chat JSON parse error: {}; raw body ({} bytes): {:?}",
            e,
            body.len(),
            String::from_utf8_lossy(snippet)
        );
        AppError::Json(e)
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
        if let Err(e) = result {
            log::error!("Assistant stream agent loop error: {}", e);
        }
    });

    Ok(sse::Sse::from_infallible_receiver(rx))
}
