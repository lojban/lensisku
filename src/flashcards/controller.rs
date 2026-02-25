use crate::flashcards::dto::{QuizAnswerResultDto, QuizFlashcardQuestionDto, SubmitQuizAnswerDto};
use std::collections::HashMap;

use actix_web::{delete, get, patch, post, put, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde_json::json;

use super::{
    dto::{
        self, AddCardsRequest, CreateFlashcardRequest, CreateLevelRequest, DirectAnswerResponse,
        FillInAnswerRequest, FlashcardListResponse, FlashcardResponse, ImportFromCollectionRequest,
        ImportFromCollectionResponse, LevelCardListResponse, LevelCardResponse, LevelListResponse,
        LevelResponse, MergeProgressRequest, MergeProgressResponse, ReviewRequest, ReviewResponse,
        StreakResponse, UpdateLevelRequest,
    },
    models::*,
    service,
};
use crate::{
    auth::Claims,
    flashcards::{
        dto::{DirectAnswerRequest, FlashcardListQuery, UpdateFlashcardPositionRequest},
        models::FlashcardQuizOptions,
    },
};

#[utoipa::path(
    post,
    path = "/flashcards/{collection_id}",
    tag = "flashcards",
    params(
        ("collection_id" = i32, Path, description = "Collection ID")
    ),
    request_body = CreateFlashcardRequest,
    responses(
        (status = 200, description = "Flashcard created successfully", body = FlashcardResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User doesn't have access to collection"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Create a new flashcard",
    description = "Creates a new flashcard in the specified collection. The definition will be used as the card content."
)]
#[post("/{collection_id}")]
pub async fn create_flashcard(
    pool: web::Data<Pool>,
    claims: Claims,
    collection_id: web::Path<i32>,
    req: web::Json<CreateFlashcardRequest>,
) -> impl Responder {
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().body(e.to_string());
    }

    match service::create_flashcard(&pool, collection_id.into_inner(), claims.sub, &req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") || message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError()
                    .body(format!("Failed to create flashcard: {}", e))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/{flashcard_id}/snooze",
    tag = "flashcards",
    params(
        ("flashcard_id" = i32, Path, description = "Flashcard ID")
    ),
    responses(
        (status = 200, description = "Flashcard snoozed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User doesn't have access to flashcard"),
        (status = 404, description = "Flashcard progress not found for this user"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Snooze flashcard review",
    description = "Postpones the next review time for a flashcard by 6 hours for the current user."
)]
#[post("/{flashcard_id}/snooze")]
pub async fn snooze_flashcard(
    pool: web::Data<Pool>,
    claims: Claims,
    flashcard_id: web::Path<i32>,
) -> impl Responder {
    match service::snooze_flashcard(&pool, claims.sub, flashcard_id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Flashcard snoozed successfully"
        })),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") {
                HttpResponse::NotFound().body(message)
            } else if message.contains("access denied") || message.contains("locked") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError()
                    .body(format!("Failed to snooze flashcard: {}", e))
            }
        }
    }
}

#[utoipa::path(
    delete,
    path = "/flashcards/{flashcard_id}",
    tag = "flashcards",
    params(
        ("flashcard_id" = i32, Path, description = "Flashcard ID")
    ),
    responses(
        (status = 200, description = "Flashcard deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User doesn't have access to flashcard"),
        (status = 404, description = "Flashcard not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[delete("/{flashcard_id}")]
pub async fn delete_flashcard(
    pool: web::Data<Pool>,
    claims: Claims,
    flashcard_id: web::Path<i32>,
) -> impl Responder {
    match service::delete_flashcard(&pool, claims.sub, flashcard_id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json("Flashcard deleted successfully"),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") || message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError()
                    .body(format!("Failed to delete flashcard: {}", e))
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/flashcards",
    tag = "flashcards",
    params(
        ("collection_id" = i32, Query, description = "Collection ID"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("status" = Option<FlashcardStatus>, Query, description = "Filter by status"),
        ("due" = Option<bool>, Query, description = "Show only cards due for review")
    ),
    responses(
        (status = 200, description = "List of flashcards", body = FlashcardListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("")]
pub async fn list_flashcards(
    pool: web::Data<Pool>,
    claims: Claims,
    query: web::Query<FlashcardListQuery>,
) -> impl Responder {
    match service::list_flashcards(&pool, claims.sub, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to list flashcards: {}", e))
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/{flashcard_id}/review",
    tag = "flashcards",
    params(
        ("flashcard_id" = i32, Path, description = "Flashcard ID")
    ),
    request_body = ReviewRequest,
    responses(
        (status = 200, description = "Review recorded successfully", body = ReviewResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User doesn't have access to flashcard"),
        (status = 404, description = "Flashcard not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/{flashcard_id}/review")]
pub async fn review_flashcard(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<ReviewRequest>,
) -> impl Responder {
    match service::review_flashcard(&pool, claims.sub, &req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") || message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError().body(format!("Failed to record review: {}", e))
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/flashcards/due",
    tag = "flashcards",
    params(
        ("collection_id" = i32, Query, description = "Collection ID"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of due flashcards", body = FlashcardListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get due cards",
    description = "Returns a list of flashcards that are due for review"
)]
#[get("/due")]
pub async fn get_due_cards(
    pool: web::Data<Pool>,
    claims: Claims,
    query: web::Query<FlashcardListQuery>,
) -> impl Responder {
    let mut query_params = query.into_inner();

    // If a specific card_id is requested, fetch only that card, ignoring due status
    if query_params.flashcard_id.is_some() {
        query_params.due = None; // Ignore due filter when fetching a specific card
        query_params.status = None; // Ignore status filter as well
    } else {
        // Otherwise, fetch only due cards
        query_params.due = Some(true);
    }

    match service::list_flashcards(&pool, claims.sub, query_params).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to get cards: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/{flashcard_id}/reset",
    tag = "flashcards",
    params(
        ("flashcard_id" = i32, Path, description = "Flashcard ID")
    ),
    responses(
        (status = 200, description = "Progress reset successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User doesn't have access to flashcard"),
        (status = 404, description = "Flashcard not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/{flashcard_id}/reset")]
pub async fn reset_progress(
    pool: web::Data<Pool>,
    claims: Claims,
    flashcard_id: web::Path<i32>,
) -> impl Responder {
    match service::reset_progress(&pool, claims.sub, flashcard_id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json("Progress reset successfully"),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") || message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError().body(format!("Failed to reset progress: {}", e))
            }
        }
    }
}

#[utoipa::path(
    patch,
    path = "/flashcards/{flashcard_id}/position",
    tag = "flashcards",
    params(
        ("flashcard_id" = i32, Path, description = "Flashcard ID")
    ),
    request_body = UpdateFlashcardPositionRequest,
    responses(
        (status = 200, description = "Position updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User doesn't have access to flashcard"),
        (status = 404, description = "Flashcard not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Update flashcard position",
    description = "Updates the position of a flashcard within its collection. This affects the order in which new cards are presented for study."
)]
#[patch("/{flashcard_id}/position")]
pub async fn update_flashcard_position(
    pool: web::Data<Pool>,
    claims: Claims,
    flashcard_id: web::Path<i32>,
    req: web::Json<UpdateFlashcardPositionRequest>,
) -> impl Responder {
    match service::update_flashcard_position(
        &pool,
        claims.sub,
        flashcard_id.into_inner(),
        req.position,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Flashcard position updated successfully"
        })),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") || message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError()
                    .body(format!("Failed to update position: {}", e))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/collection/import",
    tag = "flashcards",
    request_body = ImportFromCollectionRequest,
    responses(
        (status = 200, description = "Successfully imported collection items", body = ImportFromCollectionResponse),
        (status = 400, description = "Invalid collection ID"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/collection/import")]
pub async fn import_from_collection(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<ImportFromCollectionRequest>,
) -> impl Responder {
    match service::import_from_collection(&pool, claims.sub, req.collection_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") || message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError().body(format!("Failed to import: {}", e))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/{flashcard_id}/answer",
    tag = "flashcards",
    params(
        ("flashcard_id" = i32, Path, description = "Flashcard ID")
    ),
    request_body = DirectAnswerRequest,
    responses(
        (status = 200, description = "Answer checked successfully", body = DirectAnswerResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Forbidden - User doesn't have access to flashcard"),
        (status = 404, description = "Flashcard not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Submit direct answer for flashcard",
    description = "Checks a direct answer for a flashcard and automatically records a review if correct."
)]
#[post("/{flashcard_id}/answer")]
pub async fn submit_answer(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<DirectAnswerRequest>,
) -> impl Responder {
    match service::check_answer(&pool, claims.sub, &req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") || message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError().body(format!("Failed to check answer: {}", e))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/{flashcard_id}/fillin",
    tag = "flashcards",
    params(
        ("flashcard_id" = i32, Path, description = "Flashcard ID")
    ),
    request_body = FillInAnswerRequest,
    responses(
        (status = 200, description = "Fill-in answer processed successfully", body = ReviewResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Forbidden - User doesn't have access to flashcard"),
        (status = 404, description = "Flashcard not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Submit fill-in answer for flashcard",
    description = "Processes a fill-in answer with server-side validation and automatically records a review with appropriate rating based on answer similarity."
)]
#[post("/{flashcard_id}/fillin")]
pub async fn submit_fillin_answer(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<FillInAnswerRequest>,
) -> impl Responder {
    match service::review_flashcard_serverside(&pool, claims.sub, &req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") || message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError()
                    .body(format!("Failed to process fill-in answer: {}", e))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/quiz/submit",
    tag = "flashcards",
    request_body = SubmitQuizAnswerDto,
    responses(
        (status = 200, description = "Quiz answer processed successfully", body = QuizAnswerResultDto),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Forbidden - User doesn't have access to flashcard"),
        (status = 404, description = "Flashcard not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Submit an answer for a quiz flashcard",
    description = "Submits the user's selected answer for a quiz-type flashcard. The backend will check correctness, update FSRS progress, and log the attempt with all presented options."
)]
#[post("/quiz/submit")]
pub async fn submit_quiz_answer(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<dto::SubmitQuizAnswerDto>,
) -> impl Responder {
    match service::submit_quiz_answer(&pool, claims.sub, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => crate::utils::handle_error(e, "Failed to submit quiz answer"),
    }
}
#[utoipa::path(
    get,
    path = "/flashcards/streak",
    tag = "flashcards",
    params(
        ("days" = Option<i32>, Query, description = "Number of days to return progress for (default: 30)")
    ),
    responses(
        (status = 200, description = "User's streak information", body = StreakResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/streak")]
pub async fn get_streak(
    pool: web::Data<Pool>,
    claims: Claims,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let days = query
        .get("days")
        .and_then(|d| d.parse::<i32>().ok())
        .map(|d| d.min(3650))
        .unwrap_or(30);

    match service::get_streak(&pool, claims.sub, days).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get streak info: {}", e)
        })),
    }
}

#[utoipa::path(
    post,
    path = "flashcards/levels/{collection_id}",
    tag = "flashcards",
    request_body = CreateLevelRequest,
    responses(
        (status = 200, description = "Level created successfully", body = LevelResponse),
        (status = 403, description = "Forbidden - User doesn't have access"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[post("/levels/{collection_id}")]
pub async fn create_level(
    pool: web::Data<Pool>,
    claims: Claims,
    collection_id: web::Path<i32>,
    req: web::Json<CreateLevelRequest>,
) -> impl Responder {
    match service::create_level(&pool, collection_id.into_inner(), claims.sub, &req).await {
        Ok(level) => HttpResponse::Ok().json(level),
        Err(e) => match e.to_string().as_str() {
            "access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/flashcards/levels/{level_id}",
    tag = "flashcards",
    params(
        ("level_id" = i32, Path, description = "Level ID")
    ),
    responses(
        (status = 200, description = "Level deleted successfully"),
        (status = 400, description = "Cannot delete level if it's a prerequisite"),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Level not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[delete("/levels/{level_id}")]
pub async fn delete_level(
    pool: web::Data<Pool>,
    claims: Claims,
    level_id: web::Path<i32>,
) -> impl Responder {
    match service::delete_level(&pool, level_id.into_inner(), claims.sub).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "message": "Level deleted successfully" })),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Level not found") {
                HttpResponse::NotFound().finish()
            } else if msg.contains("Access denied") {
                HttpResponse::Forbidden().finish()
            } else if msg.contains("prerequisite for other levels") {
                HttpResponse::BadRequest().json(json!({ "error": msg }))
            } else {
                HttpResponse::InternalServerError()
                    .json(json!({ "error": format!("Failed to delete level: {}", e) }))
            }
        }
    }
}

#[utoipa::path(
    put,
    path = "/flashcards/levels/{level_id}",
    tag = "flashcards",
    request_body = UpdateLevelRequest,
    responses(
        (status = 200, description = "Level updated successfully", body = LevelResponse),
        (status = 404, description = "Level not found"),
        (status = 403, description = "Forbidden - User doesn't have access"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[put("/levels/{level_id}")]
pub async fn update_level(
    pool: web::Data<Pool>,
    claims: Claims,
    level_id: web::Path<i32>,
    req: web::Json<UpdateLevelRequest>,
) -> impl Responder {
    match service::update_level(&pool, level_id.into_inner(), claims.sub, &req).await {
        Ok(level) => HttpResponse::Ok().json(level),
        Err(e) => match e.to_string().as_str() {
            "Level not found" => HttpResponse::NotFound().finish(),
            "access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
        },
    }
}

#[utoipa::path(
    get,
    path = "/flashcards/levels/{collection_id}",
    tag = "flashcards",
    responses(
        (status = 200, description = "List of levels", body = LevelListResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[get("/levels/{collection_id}")]
pub async fn list_levels(
    pool: web::Data<Pool>,
    claims: Option<Claims>,
    collection_id: web::Path<i32>,
) -> impl Responder {
    match service::get_collection_levels(&pool, collection_id.into_inner(), claims.map(|c| c.sub))
        .await
    {
        Ok(levels) => HttpResponse::Ok().json(levels),
        Err(e) => {
            let msg = e.to_string();
            if msg.to_lowercase().contains("access denied") {
                HttpResponse::Forbidden().json(json!({ "error": msg }))
            } else {
                HttpResponse::InternalServerError().json(json!({ "error": msg }))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/progress/merge",
    tag = "flashcards",
    request_body = MergeProgressRequest,
    responses(
        (status = 200, description = "Progress merged", body = MergeProgressResponse),
        (status = 403, description = "Access denied to collection"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[post("/progress/merge")]
pub async fn merge_progress(
    pool: web::Data<Pool>,
    claims: Claims,
    req: web::Json<MergeProgressRequest>,
) -> impl Responder {
    match service::merge_anonymous_progress(&pool, claims.sub, &req).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let msg = e.to_string();
            if msg.to_lowercase().contains("access denied") {
                HttpResponse::Forbidden().json(json!({ "error": msg }))
            } else {
                HttpResponse::InternalServerError().json(json!({ "error": msg }))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/cards/{level_id}",
    tag = "flashcards",
    request_body = AddCardsRequest,
    responses(
        (status = 200, description = "Cards added successfully", body = Vec<LevelCardResponse>),
        (status = 404, description = "Level not found"),
        (status = 403, description = "Forbidden - User doesn't have access"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[post("/cards/{level_id}")]
pub async fn add_cards(
    pool: web::Data<Pool>,
    claims: Claims,
    level_id: web::Path<i32>,
    req: web::Json<AddCardsRequest>,
) -> impl Responder {
    match service::add_cards_to_level(&pool, level_id.into_inner(), claims.sub, &req).await {
        Ok(cards) => HttpResponse::Ok().json(cards),
        Err(e) => match e.to_string().as_str() {
            "Level not found" => HttpResponse::NotFound().finish(),
            "access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
        },
    }
}

#[utoipa::path(
    get,
    path = "/flashcards/levels/{level_id}/cards",
    tag = "flashcards",
    params(
        ("level_id" = i32, Path, description = "Level ID"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of level cards", body = LevelCardListResponse),
        (status = 404, description = "Level not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[get("/levels/{level_id}/cards")]
pub async fn list_level_cards(
    pool: web::Data<Pool>,
    claims: Option<Claims>,
    level_id: web::Path<i32>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let page = query.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page = query
        .get("per_page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(20);

    match service::get_level_cards_paginated(
        &pool,
        level_id.into_inner(),
        claims.map(|c| c.sub),
        page,
        per_page,
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let msg = e.to_string();
            if msg.to_lowercase().contains("access denied") {
                HttpResponse::Forbidden().json(json!({ "error": msg }))
            } else if msg == "Level not found" {
                HttpResponse::NotFound().finish()
            } else {
                HttpResponse::InternalServerError().json(json!({ "error": msg }))
            }
        }
    }
}

#[utoipa::path(
    delete,
    path = "/flashcards/levels/{level_id}/cards/{flashcard_id}",
    tag = "flashcards",
    params(
        ("level_id" = i32, Path, description = "Level ID"),
        ("flashcard_id" = i32, Path, description = "Flashcard ID")
    ),
    responses(
        (status = 200, description = "Card removed from level"),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Level or card not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[delete("/levels/{level_id}/cards/{flashcard_id}")]
pub async fn remove_card_from_level(
    pool: web::Data<Pool>,
    claims: Claims,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (level_id, flashcard_id) = path.into_inner();

    match service::remove_card_from_level(&pool, level_id, flashcard_id, claims.sub).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Card removed successfully"
        })),
        Err(e) => match e.to_string().as_str() {
            "Level not found" | "Card not found" => HttpResponse::NotFound().finish(),
            "Access denied" => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to remove card: {}", e)
            })),
        },
    }
}

#[utoipa::path(
    get,
    path = "/flashcards/quiz/next",
    tag = "flashcards",
    responses(
        (status = 200, description = "Next quiz flashcard question", body = QuizFlashcardQuestionDto),
        (status = 404, description = "No quiz flashcards available"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Get next quiz flashcard",
    description = "Returns the next quiz flashcard question with answer options for the authenticated user. The answer options are shuffled."
)]
#[get("/quiz/next")]
pub async fn get_next_quiz(pool: web::Data<Pool>, claims: Claims) -> impl Responder {
    match service::get_next_quiz_for_user(&pool, claims.sub).await {
        Ok(Some(quiz)) => HttpResponse::Ok().json(quiz),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "message": "No quiz flashcards available"
        })),
        Err(e) => {
            let message = e.to_string();
            if message.contains("access denied") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError().body(format!("Failed to get next quiz: {}", e))
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/flashcards/{flashcard_id}/quiz-options",
    tag = "flashcards",
    params(
        ("flashcard_id" = i32, Path, description = "Flashcard ID for which to generate and set quiz options")
    ),
    responses(
        (status = 200, description = "Quiz options generated and set successfully", body = FlashcardQuizOptions),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - User doesn't have access to the flashcard or it's not a quiz type"),
        (status = 404, description = "Flashcard not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Generate and set quiz options for a flashcard",
    description = "Generates and stores the correct answer text for a quiz-type flashcard. This is typically done when a card is first designated as a quiz, if its content changes, or if the quiz options need to be re-generated."
)]
#[post("/{flashcard_id}/quiz-options")]
pub async fn generate_quiz_options(
    pool: web::Data<Pool>,
    claims: Claims,
    flashcard_id: web::Path<i32>,
) -> impl Responder {
    match service::generate_and_set_quiz_options(&pool, claims.sub, flashcard_id.into_inner()).await
    {
        Ok(quiz_options) => HttpResponse::Ok().json(quiz_options),
        Err(e) => {
            let message = e.to_string();
            if message.contains("not found") {
                HttpResponse::NotFound().body(message)
            } else if message.contains("access denied") || message.contains("not a quiz type") {
                HttpResponse::Forbidden().body(message)
            } else {
                HttpResponse::InternalServerError()
                    .body(format!("Failed to generate quiz options: {}", e))
            }
        }
    }
}
