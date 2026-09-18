use actix_web::{get, post, web, HttpResponse, Responder};
use camxes_rs::camxes::peg::grammar::Peg;
use deadpool_postgres::Pool;
use std::collections::HashMap;
use std::sync::Arc;

use super::{dto::*, models::Language, service};

#[utoipa::path(
    post,
    path = "/language/validate_and_tts",
    tag = "language",
    operation_id = "validate_and_synthesize_lojban",
    summary = "Validate Lojban text and synthesize valid text",
    description = "Checks Lojban morphology and returns valid plus base64-encoded Ogg Opus audio for valid text. Uses the Martin voice at speed 0.8. Invalid Lojban returns HTTP 200 with valid=false and null audio. Text is limited to 2000 characters. All accepted requests share the per-user Kokoro TTS quota with /collections/kokoro-tts. Infrastructure failures return an error without a validity verdict.",
    request_body = LojbanParseRequest,
    responses(
        (status = 200, description = "Validation result and audio when valid", body = LojbanTtsResponse),
        (status = 400, description = "Malformed request or text exceeds 2000 characters"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 429, description = "TTS rate limit exceeded; see Retry-After"),
        (status = 500, description = "Rate limit check, validation task, or synthesis failed")
    ),
    security(("bearer_auth" = []))
)]
#[post("/validate_and_tts")]
pub async fn validate_and_tts(
    claims: crate::auth::Claims,
    parsers: web::Data<Arc<HashMap<i32, Peg>>>,
    limiter: web::Data<crate::middleware::limiter::KokoroTtsLimiter>,
    request: web::Json<LojbanParseRequest>,
) -> HttpResponse {
    let text = request.text.trim().to_string();
    if text.chars().count() > 2000 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "text exceeds 2000 characters" }));
    }
    match limiter.check_and_record(claims.sub).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::TooManyRequests()
                .insert_header(("Retry-After", limiter.retry_after_secs().to_string()))
                .json(serde_json::json!({ "error": "Rate limit exceeded" }));
        }
        Err(e) => {
            log::error!("Lojban TTS rate limit check failed: {e}");
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Rate limit check failed" }));
        }
    }

    let Some(parser) = parsers.get(&1).cloned() else {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "Lojban parser not available" }));
    };
    match tokio::task::spawn_blocking(move || {
        service::validate_and_synthesize_lojban(&parser, &text, |text| {
            crate::utils::kokoro_tts_singleton::synthesize_lojban_to_ogg_opus(text, "Martin", 0.8)
        })
    })
    .await
    {
        Ok(Ok(response)) => HttpResponse::Ok().json(response),
        Ok(Err(e)) => {
            log::error!("Lojban TTS synthesis failed: {e}");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Speech synthesis failed" }))
        }
        Err(e) => {
            log::error!("Lojban validation/TTS task failed: {e}");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Validation and synthesis task failed" }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/language/languages",
    tag = "language",
    operation_id = "get_supported_languages",
    summary = "Get supported languages",
    description = "Retrieves a list of all languages supported by the Lojban dictionary system. \
                  Returns language details including ID, tag, names in English and Lojban, \
                  native name, and optional URL.",
    responses(
        (status = 200, description = "List of supported languages", body = Vec<Language>),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/languages")]
pub async fn get_languages(pool: web::Data<Pool>) -> impl Responder {
    match service::get_languages(&pool).await {
        Ok(languages) => HttpResponse::Ok().json(languages),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/language/parse_lojban",
    tag = "language",
    operation_id = "parse_lojban_text",
    summary = "Parse Lojban text",
    description = "Parses provided Lojban text and returns a structured representation \
                  of its grammatical components. The response includes tokens with their \
                  types, positions, and hierarchical relationships.",
    request_body = LojbanParseRequest,
    responses(
        (status = 200, description = "Successfully parsed Lojban text", body = LojbanParseResponse),
        (status = 400, description = "Invalid Lojban text", body = LojbanParseResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/parse_lojban")]
pub async fn parse_lojban(
    request: web::Json<LojbanParseRequest>,
    // ELI5: This is a special shared container (Arc) that holds our language parsers.
    // It's like a library of dictionaries that many people can use at the same time
    // to understand different languages, without needing separate copies for each person.
    parsers: web::Data<Arc<HashMap<i32, Peg>>>,
) -> impl Responder {
    // Pass the map to the service function
    let response = service::parse_lojban(&parsers, &request.text);
    if response.success {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::BadRequest().json(response)
    }
}

#[utoipa::path(
    post,
    path = "/language/analyze_word",
    tag = "language",
    operation_id = "analyze_lojban_word",
    summary = "Analyze Lojban word",
    description = "Analyzes a single Lojban word to determine its grammatical type \
                  (gismu, lujvo, cmavo, etc.). Returns the word type and analysis success status.",
    request_body = AnalyzeWordRequest,
    responses(
        (status = 200, description = "Successfully analyzed word", body = AnalyzeWordResponse),
        (status = 400, description = "Invalid word", body = AnalyzeWordResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/analyze_word")]
pub async fn analyze_word(
    // ELI5: Just like in parse_lojban, this is our shared language parser library (Arc)
    // that helps us understand words in different languages. Many people can use it
    // at the same time without getting in each other's way.
    parsers: web::Data<Arc<HashMap<i32, Peg>>>,
    pool: web::Data<deadpool_postgres::Pool>,
    request: web::Json<AnalyzeWordRequest>,
) -> impl Responder {
    // Default to Lojban (1) if source_langid is not provided
    let source_langid = request.source_langid.unwrap_or(1);

    // Pass the map and source_langid to the service function
    match service::analyze_word_in_pool(
        parsers.get_ref().clone(),
        &request.word,
        source_langid,
        &pool,
    )
    .await
    {
        Ok(response) => {
            if response.success {
                HttpResponse::Ok().json(response)
            } else {
                HttpResponse::BadRequest().json(response)
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Analysis error: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/language/validate_mathjax",
    tag = "language",
    operation_id = "validate_mathjax_syntax",
    summary = "Validate MathJax syntax",
    description = "Validates MathJax/LaTeX mathematical notation for correct syntax, \
                  balanced delimiters, and proper command usage. Useful for ensuring \
                  mathematical expressions will render correctly.",
    request_body = MathJaxValidationRequest,
    responses(
        (status = 200, description = "MathJax validation successful", body = MathJaxValidationResponse),
        (status = 400, description = "Invalid MathJax", body = MathJaxValidationResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/validate_mathjax")]
pub async fn validate_mathjax(request: web::Json<MathJaxValidationRequest>) -> impl Responder {
    let response = service::validate_mathjax_handler(&request.text).await;
    if response.valid {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::BadRequest().json(response)
    }
}

#[cfg(test)]
mod validation_tts_tests {
    #[actix_web::test]
    async fn endpoint_requires_bearer_authentication() {
        let app = actix_web::test::init_service(
            actix_web::App::new().configure(crate::language::configure),
        )
        .await;
        let request = actix_web::test::TestRequest::post()
            .uri("/language/validate_and_tts")
            .set_json(serde_json::json!({"text": "mi klama"}))
            .to_request();
        let response = actix_web::test::call_service(&app, request).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }
}
