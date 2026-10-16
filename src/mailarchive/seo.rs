use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;

use crate::utils::seo::{self, LOCALES};

use super::service;

fn content_preview(content: Option<&str>) -> String {
    let Some(content) = content else {
        return String::new();
    };
    let unquoted = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('>'))
        .collect::<Vec<_>>()
        .join(" ");
    seo::compact(&unquoted, 180)
}

fn mail_description(subject: &str, content: Option<&str>, locale: &str) -> String {
    let preview = content_preview(content);
    let description_subject = seo::compact(subject, 80);
    if preview.is_empty() {
        format!("{description_subject} — {}", seo::site_title(locale))
    } else {
        format!("{description_subject}: {preview}")
    }
}

#[get("/{locale}/thread/{subject:.*}")]
pub async fn thread_page(
    pool: web::Data<Pool>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (locale, subject) = path.into_inner();
    if !LOCALES.contains(&locale.as_str()) || subject.len() > 1000 {
        return HttpResponse::NotFound().finish();
    }
    let subject = service::remove_prefixes(subject.trim_end_matches('/'));
    let Ok(client) = pool.get().await else {
        return HttpResponse::InternalServerError().finish();
    };
    let row = client
        .query_opt(
            "SELECT cleaned_subject, LEFT(content, 500) AS content FROM messages WHERE cleaned_subject = $1 ORDER BY date DESC NULLS LAST LIMIT 1",
            &[&subject],
        )
        .await;
    match row {
        Ok(Some(row)) => {
            let title: String = row.get("cleaned_subject");
            let content: Option<String> = row.get("content");
            let encoded = urlencoding::encode(&title);
            let canonical_path = format!("/{locale}/thread/{encoded}");
            let description = mail_description(&title, content.as_deref(), &locale);
            seo::html_response(&locale, &title, &description, &canonical_path)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/{locale}/message/{id}")]
pub async fn message_page(pool: web::Data<Pool>, path: web::Path<(String, i32)>) -> impl Responder {
    let (locale, id) = path.into_inner();
    if !LOCALES.contains(&locale.as_str()) || id <= 0 {
        return HttpResponse::NotFound().finish();
    }
    let Ok(client) = pool.get().await else {
        return HttpResponse::InternalServerError().finish();
    };
    let row = client
        .query_opt(
            "SELECT subject, LEFT(content, 500) AS content FROM messages WHERE id = $1",
            &[&id],
        )
        .await;
    match row {
        Ok(Some(row)) => {
            let title: Option<String> = row.get("subject");
            let content: Option<String> = row.get("content");
            let canonical_path = format!("/{locale}/message/{id}");
            let subject = title
                .as_deref()
                .filter(|subject| !subject.trim().is_empty())
                .unwrap_or("Mail message");
            let description = mail_description(subject, content.as_deref(), &locale);
            seo::html_response(&locale, subject, &description, &canonical_path)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::{message_page, thread_page};
    use actix_web::{http::StatusCode, test as actix_test, web, App, HttpResponse};

    #[actix_web::test]
    async fn mail_api_paths_are_not_captured_by_seo_routes() {
        let app = actix_test::init_service(
            App::new()
                .service(web::scope("mail").route(
                    "/message/{id}",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                ))
                .service(message_page)
                .service(thread_page),
        )
        .await;
        let request = actix_test::TestRequest::get()
            .uri("/mail/message/1500108")
            .to_request();
        let response = actix_test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
