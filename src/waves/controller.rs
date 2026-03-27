use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde_json::json;

use crate::auth::Claims;

use super::dto::{WavesSearchQuery, WavesThreadsQuery};
use super::service;

#[utoipa::path(
    get,
    path = "/waves/search",
    tag = "waves",
    params(
        ("search" = Option<String>, Query, description = "Search term"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("sort_by" = Option<String>, Query, description = "Sort field"),
        ("sort_order" = Option<String>, Query, description = "Sort order: asc, desc"),
        ("source" = Option<String>, Query, description = "Filter: all, jbotcan, comments, mail")
    ),
    responses(
        (status = 200, description = "Combined search results (comments + mail)", body = crate::waves::dto::WavesSearchResponse),
        (status = 500, description = "Internal server error")
    ),
    summary = "Search waves (comments and mail)",
    description = "Unified search across discussion comments and mail archive (comment search is implemented in comments::service::search_comments; this route merges with mail). Use query `source` to restrict to jbotcan, site comments, or mail only."
)]
#[get("/search")]
pub async fn search_waves(
    pool: web::Data<Pool>,
    query: web::Query<WavesSearchQuery>,
    claims: Option<Claims>,
) -> impl Responder {
    let current_user_id = claims.map(|c| c.sub);
    match service::search_waves(&pool, query.into_inner(), current_user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to search waves",
            "details": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/waves/threads",
    tag = "waves",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("sort_by" = Option<String>, Query, description = "Sort field"),
        ("sort_order" = Option<String>, Query, description = "Sort order: asc, desc"),
        ("source" = Option<String>, Query, description = "Filter: all, jbotcan, comments, mail")
    ),
    responses(
        (status = 200, description = "Combined thread list (comments + mail)", body = crate::waves::dto::WavesThreadsResponse),
        (status = 500, description = "Internal server error")
    ),
    summary = "List waves threads",
    description = "Unified list of recent threads from discussion comments and mail archive (replaces the former GET /comments/threads). Use query `source` to filter: all, jbotcan, comments, mail."
)]
#[get("/threads")]
pub async fn list_wave_threads(
    pool: web::Data<Pool>,
    query: web::Query<WavesThreadsQuery>,
) -> impl Responder {
    match service::list_wave_threads(&pool, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to list wave threads",
            "details": e.to_string()
        })),
    }
}
