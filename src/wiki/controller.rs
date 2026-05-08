use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde_json::json;

use super::service;

#[utoipa::path(
    get,
    path = "/wiki/{title}",
    tag = "wiki",
    params(("title" = String, Path, description = "Article title (URL-encoded)")),
    responses(
        (status = 200, description = "Wiki article detail", body = crate::wiki::dto::WikiArticleDetail),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    summary = "Fetch a mirrored mw.lojban.org article rendered as Markdown",
)]
#[get("/{title}")]
pub async fn get_wiki_article(
    pool: web::Data<Pool>,
    path: web::Path<String>,
) -> impl Responder {
    let title_raw = path.into_inner();
    let title = match urlencoding::decode(&title_raw) {
        Ok(s) => s.into_owned(),
        Err(_) => title_raw,
    };
    match service::get_article_by_title(&pool, &title).await {
        Ok(Some(article)) => HttpResponse::Ok().json(article),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Article not found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to fetch wiki article",
            "details": e.to_string()
        })),
    }
}
