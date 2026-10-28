use crate::error::AppError;
use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;

use crate::utils::seo::{self, LOCALES};
use crate::wiki::MW_WIKI_SOURCE;

fn is_reserved_wiki_path(title: &str) -> bool {
    let trimmed = title.trim_matches('/');
    trimmed.eq_ignore_ascii_case("add")
        || trimmed.eq_ignore_ascii_case("id")
        || trimmed.to_ascii_lowercase().ends_with("/edit")
        || trimmed.to_ascii_lowercase().starts_with("id/")
}

fn fallback_description(title: &str, locale: &str) -> String {
    format!("{} — {}", seo::compact(title, 80), seo::site_title(locale))
}

fn description_from_body(title: &str, body: Option<&str>, locale: &str) -> String {
    body.filter(|s| !s.trim().is_empty())
        .map(|s| seo::plain_preview(s, 155))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback_description(title, locale))
}

/// Native (non–mw.lojban.org) wiki definition, then mirrored `wiki_articles` row.
async fn fetch_wiki_seo(
    pool: &Pool,
    title_raw: &str,
    locale: &str,
) -> Result<Option<(String, String)>, AppError> {
    let client = pool.get().await?;
    let decoded = urlencoding::decode(title_raw)
        .map(|s| s.into_owned())
        .unwrap_or_else(|_| title_raw.to_string());
    let title = decoded.replace('_', " ");
    let underscored = title.replace(' ', "_");

    // Prefer native wiki pages that are not MediaWiki imports (matches WikiArticle.vue).
    if let Some(row) = client
        .query_opt(
            "SELECT v.word, LEFT(d.definition, 800) AS body
             FROM valsi v
             JOIN definitions d ON d.valsiid = v.valsiid
             WHERE v.typeid = 16
               AND v.word = $1
               AND COALESCE(d.metadata->>'source', '') <> $2
               AND COALESCE((d.metadata->>'imported')::boolean, false) = false
               AND d.metadata->>'mw_page_id' IS NULL
             LIMIT 1",
            &[&title, &MW_WIKI_SOURCE],
        )
        .await?
    {
        let word: String = row.get("word");
        let body: Option<String> = row.get("body");
        return Ok(Some((
            word.clone(),
            description_from_body(&word, body.as_deref(), locale),
        )));
    }

    if let Some(row) = client
        .query_opt(
            "SELECT title,
                    COALESCE(NULLIF(LEFT(plain_text, 800), ''), LEFT(markdown, 800)) AS body
             FROM wiki_articles
             WHERE title = $1 OR title = $2 OR title = $3
             ORDER BY (CASE WHEN title = $1 THEN 0 WHEN title = $2 THEN 1 ELSE 2 END)
             LIMIT 1",
            &[&decoded, &title, &underscored],
        )
        .await?
    {
        let word: String = row.get("title");
        let body: Option<String> = row.get("body");
        return Ok(Some((
            word.clone(),
            description_from_body(&word, body.as_deref(), locale),
        )));
    }

    Ok(None)
}

async fn fetch_wiki_seo_by_definition_id(
    pool: &Pool,
    definition_id: i32,
    locale: &str,
) -> Result<Option<(String, String)>, AppError> {
    let client = pool.get().await?;
    let row = client
        .query_opt(
            "SELECT v.word, LEFT(d.definition, 800) AS body
             FROM definitions d
             JOIN valsi v ON v.valsiid = d.valsiid
             WHERE d.definitionid = $1 AND v.typeid = 16",
            &[&definition_id],
        )
        .await?;
    Ok(row.map(|row| {
        let word: String = row.get("word");
        let body: Option<String> = row.get("body");
        (
            word.clone(),
            description_from_body(&word, body.as_deref(), locale),
        )
    }))
}

#[get("/{locale}/wiki/id/{id}")]
pub async fn wiki_by_id_page(
    pool: web::Data<Pool>,
    path: web::Path<(String, i32)>,
) -> impl Responder {
    let (locale, id) = path.into_inner();
    if !LOCALES.contains(&locale.as_str()) || id <= 0 {
        return HttpResponse::NotFound().finish();
    }
    match fetch_wiki_seo_by_definition_id(&pool, id, &locale).await {
        Ok(Some((title, description))) => {
            let slug = title.replace(' ', "_");
            let encoded = urlencoding::encode(&slug);
            // Canonicalize to the title URL; id URLs are stable bookmarks.
            let canonical_path = format!("/{locale}/wiki/{encoded}");
            seo::html_response(&locale, &title, &description, &canonical_path).await
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/{locale}/wiki/{title:.*}")]
pub async fn wiki_page(pool: web::Data<Pool>, path: web::Path<(String, String)>) -> impl Responder {
    let (locale, title_raw) = path.into_inner();
    let title_raw = title_raw.trim_end_matches('/').to_string();
    if !LOCALES.contains(&locale.as_str())
        || title_raw.is_empty()
        || title_raw.len() > 1000
        || is_reserved_wiki_path(&title_raw)
    {
        return HttpResponse::NotFound().finish();
    }

    match fetch_wiki_seo(&pool, &title_raw, &locale).await {
        Ok(Some((title, description))) => {
            let slug = title.replace(' ', "_");
            let encoded = urlencoding::encode(&slug);
            let canonical_path = format!("/{locale}/wiki/{encoded}");
            seo::html_response(&locale, &title, &description, &canonical_path).await
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_reserved_wiki_path, wiki_page};
    use actix_web::{http::StatusCode, test as actix_test, web, App, HttpResponse};

    #[test]
    fn reserves_editor_and_add_paths() {
        assert!(is_reserved_wiki_path("add"));
        assert!(is_reserved_wiki_path("Lojban/edit"));
        assert!(is_reserved_wiki_path("id/12"));
        assert!(!is_reserved_wiki_path("Lojban"));
        assert!(!is_reserved_wiki_path("A/B page"));
    }

    #[actix_web::test]
    async fn wiki_api_paths_are_not_captured_by_seo_routes() {
        let app = actix_test::init_service(
            App::new()
                .service(web::scope("wiki").route(
                    "/{title:.*}",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                ))
                .service(wiki_page),
        )
        .await;
        let request = actix_test::TestRequest::get()
            .uri("/wiki/Lojban")
            .to_request();
        let response = actix_test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
