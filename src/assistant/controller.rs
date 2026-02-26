use actix_web::{post, web, HttpResponse};

use crate::error::AppError;

use super::dto::{ChatRequest, ChatResponse};

const DEBUG_BODY_LIMIT: usize = 2000;

#[post("/chat")]
pub async fn chat(
    pool: web::Data<deadpool_postgres::Pool>,
    body: web::Bytes,
) -> Result<HttpResponse, AppError> {
    let request: ChatRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
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
            return Err(AppError::Json(e));
        }
    };

    let reply = super::handle_chat(&pool, request).await?;

    Ok(HttpResponse::Ok().json(ChatResponse { reply }))
}
