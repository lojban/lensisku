use actix_web::{post, web, HttpResponse};

use crate::error::AppError;

use super::dto::{ChatRequest, ChatResponse};

#[post("/chat")]
pub async fn chat(
    pool: web::Data<deadpool_postgres::Pool>,
    payload: web::Json<ChatRequest>,
) -> Result<HttpResponse, AppError> {
    let reply = super::handle_chat(&pool, payload.into_inner()).await?;

    Ok(HttpResponse::Ok().json(ChatResponse { reply }))
}
