use actix_web::{post, web, HttpResponse};

use crate::config::AppConfig;
use crate::error::AppError;

use super::dto::{ChatRequest, ChatResponse};

#[post("/chat")]
pub async fn chat(
    app_config: web::Data<AppConfig>,
    pool: web::Data<deadpool_postgres::Pool>,
    payload: web::Json<ChatRequest>,
) -> Result<HttpResponse, AppError> {
    let reply = super::handle_chat(&app_config, &pool, payload.into_inner()).await?;

    Ok(HttpResponse::Ok().json(ChatResponse { reply }))
}
