use super::models::CachedExport;
use super::service;
use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;

use crate::{
    auth::Claims,
    export::models::{ExportFormat, ExportOptions},
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
        ("format" = String, Path, description = "Export format (pdf, latex, xml, json)")
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
) -> impl Responder {
    let (language_tag, format) = path.into_inner();

    match service::get_cached_export(&pool, &language_tag, &format).await {
        Ok((content, content_type, filename)) => HttpResponse::Ok()
            .content_type(content_type)
            .append_header((
                "Content-Disposition",
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
        ("positive_scores_only" = Option<bool>, Query, description = "Include only entries with positive scores"),
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
