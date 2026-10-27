use actix_web::{get, post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde_json::json;

use super::service::{self, DialogueError};
use crate::auth::Claims;
use crate::middleware::limiter::KokoroTtsLimiter;

fn err(e: DialogueError) -> HttpResponse {
    match e {
        DialogueError::NotFound => HttpResponse::NotFound().json(json!({"error": "not found"})),
        DialogueError::Forbidden => {
            HttpResponse::Forbidden().json(json!({"error": "access denied"}))
        }
        DialogueError::BadRequest(m) => HttpResponse::BadRequest().json(json!({ "error": m })),
        DialogueError::Internal(m) => {
            log::error!("dialogue: {m}");
            HttpResponse::InternalServerError().json(json!({ "error": m }))
        }
    }
}

/// Parsed dialogue, voices, per-line audio status and word-bank chips (guest-readable).
#[get("/items/{item_id}")]
pub async fn get_dialogue(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
    claims: Option<Claims>,
) -> impl Responder {
    match service::get_view(&pool, path.into_inner(), claims.map(|c| c.sub)).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => err(e),
    }
}

#[get("/items/{item_id}/lines/{line}/audio")]
pub async fn get_line_audio(
    pool: web::Data<Pool>,
    path: web::Path<(i32, i32)>,
    claims: Option<Claims>,
) -> impl Responder {
    let (item_id, line) = path.into_inner();
    match service::line_audio(&pool, item_id, line, claims.map(|c| c.sub)).await {
        Ok((bytes, mime)) => HttpResponse::Ok()
            .content_type(mime)
            .insert_header(("X-Content-Type-Options", "nosniff"))
            .insert_header(("Cache-Control", "private, max-age=300"))
            .body(bytes),
        Err(e) => err(e),
    }
}

/// Grade one question without storing anything (guest study).
#[post("/items/{item_id}/check")]
pub async fn check_answer(
    pool: web::Data<Pool>,
    path: web::Path<i32>,
    claims: Option<Claims>,
    req: web::Json<service::DialogueCheckRequest>,
) -> impl Responder {
    match service::check(&pool, path.into_inner(), claims.map(|c| c.sub), &req).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => err(e),
    }
}

/// Live editor preview: parse a script and resolve voices.
#[post("/preview")]
pub async fn preview(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<service::PreviewRequest>,
) -> impl Responder {
    if req.text.chars().count() > 20_000 {
        return HttpResponse::BadRequest().json(json!({"error": "script too long"}));
    }
    match service::preview(&pool, claims.sub, &req).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => err(e),
    }
}

/// Owner: synthesize missing / changed lines (or one line with `force`) via Kokoro TTS.
#[post("/items/{item_id}/audio")]
pub async fn generate_audio(
    pool: web::Data<Pool>,
    claims: Claims,
    limiter: web::Data<KokoroTtsLimiter>,
    path: web::Path<i32>,
    req: web::Json<service::GenerateAudioRequest>,
) -> impl Responder {
    match limiter.check_and_record(claims.sub).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::TooManyRequests()
                .insert_header(("Retry-After", limiter.retry_after_secs().to_string()))
                .json(json!({ "error": "Rate limit exceeded" }))
        }
        Err(e) => {
            log::error!("dialogue TTS rate limit Redis error: {e}");
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Rate limit check failed"}));
        }
    }
    let synth = |text: String, voice: String| {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                crate::utils::kokoro_tts_singleton::synthesize_lojban_to_ogg_opus(
                    &text,
                    &voice,
                    service::TTS_SPEED,
                )
            })
            .await
            .map_err(|e| e.to_string())?
        })
            as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>
    };
    match service::generate_audio(&pool, path.into_inner(), claims.sub, &req, synth).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => err(e),
    }
}

/// Answer one question (own row + score); `finish` commits the card review.
#[post("/flashcards/{flashcard_id}/answer")]
pub async fn answer(
    pool: web::Data<Pool>,
    claims: Claims,
    path: web::Path<i32>,
    req: web::Json<service::DialogueAnswerRequest>,
) -> impl Responder {
    match service::answer(&pool, claims.sub, path.into_inner(), &req).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => err(e),
    }
}
