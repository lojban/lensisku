use crate::error::AppError;
use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;

use crate::utils::seo::{self, LOCALES};

fn is_reserved_valsi_path(word: &str) -> bool {
    word.eq_ignore_ascii_case("add")
}

fn entry_description(
    word: &str,
    type_name: &str,
    definition: Option<&str>,
    locale: &str,
) -> String {
    if let Some(def) = definition.filter(|s| !s.trim().is_empty()) {
        let preview = seo::plain_preview(def, 120);
        if !preview.is_empty() {
            return seo::compact(&format!("{word} ({type_name}): {preview}"), 155);
        }
    }
    seo::compact(
        &format!(
            "{word} — Lojban word ({type_name}). Definitions and usage in {}.",
            seo::site_title(locale)
        ),
        155,
    )
}

async fn fetch_valsi_seo(
    pool: &Pool,
    id_or_word: &str,
    locale: &str,
) -> Result<Option<(String, String)>, AppError> {
    let client = pool.get().await?;
    let decoded = urlencoding::decode(id_or_word)
        .map(|s| s.into_owned())
        .unwrap_or_else(|_| id_or_word.to_string());
    let word = decoded.replace('_', " ");

    let row = client
        .query_opt(
            "SELECT v.word, vt.descriptor AS type_name,
                    (SELECT LEFT(d.definition, 800)
                     FROM definitions d
                     WHERE d.valsiid = v.valsiid
                     ORDER BY d.definitionid
                     LIMIT 1) AS body
             FROM valsi v
             JOIN valsitypes vt ON v.typeid = vt.typeid
             WHERE v.typeid <> 16
               AND v.source_langid = 1
               AND CASE
                     WHEN $1 ~ '^\\d+$' THEN v.valsiid = $1::int
                     ELSE v.word = $2
                   END
             LIMIT 1",
            &[&decoded, &word],
        )
        .await?;

    Ok(row.map(|row| {
        let word: String = row.get("word");
        let type_name: String = row.get("type_name");
        let body: Option<String> = row.get("body");
        (
            word.clone(),
            entry_description(&word, &type_name, body.as_deref(), locale),
        )
    }))
}

#[get("/{locale}/valsi/{word}")]
pub async fn valsi_page(
    pool: web::Data<Pool>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (locale, word_raw) = path.into_inner();
    let word_raw = word_raw.trim_end_matches('/').to_string();
    if !LOCALES.contains(&locale.as_str())
        || word_raw.is_empty()
        || word_raw.len() > 500
        || is_reserved_valsi_path(&word_raw)
    {
        return HttpResponse::NotFound().finish();
    }

    match fetch_valsi_seo(&pool, &word_raw, &locale).await {
        Ok(Some((word, description))) => {
            let slug = word.replace(' ', "_");
            let encoded = urlencoding::encode(&slug);
            let canonical_path = format!("/{locale}/valsi/{encoded}");
            seo::html_response(&locale, &word, &description, &canonical_path).await
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_reserved_valsi_path, valsi_page};
    use actix_web::{http::StatusCode, test as actix_test, web, App, HttpResponse};

    #[test]
    fn reserves_add_path() {
        assert!(is_reserved_valsi_path("add"));
        assert!(!is_reserved_valsi_path("broda"));
    }

    #[actix_web::test]
    async fn jbovlaste_api_paths_are_not_captured_by_seo_routes() {
        let app = actix_test::init_service(
            App::new()
                .service(web::scope("jbovlaste").route(
                    "/valsi/{word}",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                ))
                .service(valsi_page),
        )
        .await;
        let request = actix_test::TestRequest::get()
            .uri("/jbovlaste/valsi/broda")
            .to_request();
        let response = actix_test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
