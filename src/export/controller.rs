use super::models::CachedExport;
use super::service;
use actix_web::http::header;
use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;

use crate::{
    auth::Claims,
    export::models::{ExportFormat, ExportOptions, SearchExportQuery},
};

#[utoipa::path(
    get,
    path = "/export/cached",
    tag = "export",
    responses(
        (status = 200, description = "List of cached exports", body = Vec<CachedExport>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "List all cached dictionary exports"
)]
#[get("/cached")]
pub async fn list_cached_exports(pool: web::Data<Pool>) -> impl Responder {
    match service::list_cached_exports(&pool).await {
        Ok(exports) => HttpResponse::Ok().json(exports),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    get,
    path = "/export/cached/{language_tag}/{format}",
    tag = "export",
    params(
        ("language_tag" = String, Path, description = "Language tag"),
        ("format" = String, Path, description = "Export format (pdf, latex, xml, json, tsv)"),
        ("source_lang" = Option<String>, Query, description = "Source language tag (defaults to Lojban / jbo)"),
        ("positive_scores_only" = Option<bool>, Query, description = "Only include positive-scored entries (defaults to true)")
    ),
    responses(
        (status = 200, description = "Cached export file"),
        (status = 404, description = "Export not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Download a cached dictionary export"
)]
#[get("/cached/{language_tag}/{format}")]
pub async fn download_cached_export(
    pool: web::Data<Pool>,
    path: web::Path<(String, String)>,
    query: web::Query<ExportOptions>,
) -> impl Responder {
    let (language_tag, format) = path.into_inner();
    let source_language_tag = query
        .source_lang
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("jbo");
    let positive_scores_only = query.positive_scores_only.unwrap_or(true);

    match service::get_cached_export(
        &pool,
        &language_tag,
        source_language_tag,
        &format,
        positive_scores_only,
    )
    .await
    {
        Ok((content, content_type, filename)) => HttpResponse::Ok()
            .insert_header((header::CACHE_CONTROL, "no-store"))
            .content_type(content_type)
            .append_header((
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ))
            .body(content),
        Err(e) if e.to_string() == "Export not found" => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[utoipa::path(
    get,
    path = "/export/dictionary/{lang}",
    tag = "export",
    params(
        ("lang" = String, Path, description = "Language tag"),
        ("format" = Option<String>, Query, description = "Export format (pdf, latex, xml, json)"),
        ("positive_scores_only" = Option<bool>, Query, description = "When true (default), export one best positive-scored definition per word. When false, export every definition including zero/negative scores."),
        ("collection_id" = Option<i32>, Query, description = "Export only definitions from specific collection"),
        ("source_lang" = Option<String>, Query, description = "Language tag of the source/word language (defaults to Lojban)")
    ),
    responses(
        (status = 200, description = "Dictionary exported successfully"),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Export dictionary for specified language"
)]
#[get("/dictionary/{lang}")]
pub async fn export_dictionary(
    pool: web::Data<Pool>,
    lang: web::Path<String>,
    query: web::Query<ExportOptions>,
    claims: Option<Claims>,
) -> impl Responder {
    let format = match query.format.as_deref().unwrap_or("pdf") {
        "pdf" => ExportFormat::Pdf,
        "latex" | "tex" => ExportFormat::LaTeX,
        "xml" => ExportFormat::Xml,
        "json" => ExportFormat::Json,
        "tsv" => ExportFormat::Tsv,
        _ => {
            return HttpResponse::BadRequest()
                .body("Invalid format. Supported formats: pdf, latex, xml, json, tsv");
        }
    };

    match service::export_with_access_check(&pool, &lang, format, &query, claims.map(|c| c.sub))
        .await
    {
        Ok((content, content_type, filename)) => HttpResponse::Ok()
            .insert_header((header::CACHE_CONTROL, "no-store"))
            .content_type(content_type)
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            ))
            .body(content),
        Err(e) => match e.to_string().as_str() {
            "Access denied" => HttpResponse::Forbidden().finish(),
            "Invalid language tag" => HttpResponse::BadRequest().body(e.to_string()),
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
    }
}

#[utoipa::path(
    get,
    path = "/export/search",
    tag = "export",
    params(
        ("format" = Option<String>, Query, description = "Export format (pdf, latex, xml, json, tsv)"),
        ("search" = Option<String>, Query, description = "Search term"),
        ("languages" = Option<String>, Query, description = "Comma-separated definition language ids"),
        ("selmaho" = Option<String>, Query, description = "Filter by selma'o"),
        ("word_type" = Option<i16>, Query, description = "Filter by valsi type id"),
        ("username" = Option<String>, Query, description = "Comma-separated author usernames to include"),
        ("exclude_usernames" = Option<String>, Query, description = "Comma-separated author usernames to exclude"),
        ("source_langid" = Option<i32>, Query, description = "Source language id of the valsi (default Lojban)"),
        ("search_in_phrases" = Option<bool>, Query, description = "When false, exclude phrase (type 15) entries"),
        ("semantic" = Option<bool>, Query, description = "Use semantic ranking when search text is present"),
        ("collection_ids" = Option<String>, Query, description = "Comma-separated public collection ids; unioned with include-authors unless collection_only is set. Without authors, only collection matches are exported (no unscoped dictionary fallback)"),
        ("collection_only" = Option<bool>, Query, description = "When true with collection_ids, export only filtered collection items (authors filter items; no dictionary union)")
    ),
    responses(
        (status = 200, description = "Filtered search exported successfully"),
        (status = 400, description = "Invalid parameters, empty result, or too many rows"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    summary = "Export dictionary and/or collection search results"
)]
#[get("/search")]
pub async fn export_search(
    pool: web::Data<Pool>,
    query: web::Query<SearchExportQuery>,
) -> impl Responder {
    match ExportFormat::from_query(query.format.as_deref()) {
        Ok(_) => {}
        Err(msg) => return HttpResponse::BadRequest().body(msg),
    }

    match service::export_search_results(&pool, &query).await {
        Ok((content, content_type, filename)) => HttpResponse::Ok()
            .insert_header((header::CACHE_CONTROL, "no-store"))
            .content_type(content_type)
            .append_header((
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ))
            .body(content),
        Err(e) => {
            let msg = e.to_string();
            if msg.starts_with("Add a search")
                || msg.starts_with("Too many matching")
                || msg.starts_with("No matching")
                || msg.starts_with("Invalid format")
                || msg.starts_with("Semantic search is disabled")
            {
                HttpResponse::BadRequest().body(msg)
            } else {
                HttpResponse::InternalServerError().body(msg)
            }
        }
    }
}
