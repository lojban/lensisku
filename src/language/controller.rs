use actix_web::{get, post, web, HttpResponse, Responder};
use camxes_rs::camxes::peg::grammar::Peg;
use deadpool_postgres::Pool;
use std::collections::HashMap;
use std::sync::Arc;

use super::{dto::*, models::Language, service};

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
