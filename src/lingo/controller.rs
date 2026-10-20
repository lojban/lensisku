use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde_json::json;

use super::{
    dto::{LingoLeaderboardQuery, LingoLeaderboardResponse},
    service,
};
use crate::auth::Claims;

#[utoipa::path(
    get,
    path = "/lingo/leaderboard",
    tag = "lingo",
    params(LingoLeaderboardQuery),
    responses(
        (status = 200, description = "Top Lingo learners by study XP", body = LingoLeaderboardResponse),
        (status = 500, description = "Internal server error")
    ),
    security(),
    summary = "Lingo leaderboard",
    description = "Public ranking of learners by flashcard review activity. \
                  Points = sum of `user_flashcard_progress.review_count` × 10. \
                  When a Bearer token is present, `me` includes the caller's standing \
                  even if they are outside the top `limit`."
)]
#[get("/leaderboard")]
pub async fn get_leaderboard(
    pool: web::Data<Pool>,
    query: web::Query<LingoLeaderboardQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let viewer = claims.map(|c| c.sub);
    match service::list_leaderboard(&pool, &query, viewer).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to load Lingo leaderboard: {e}")
        })),
    }
}
