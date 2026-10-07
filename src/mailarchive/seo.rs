use actix_web::{get, http::header, web, HttpResponse, Responder};
use deadpool_postgres::Pool;

use super::service;

const SITE_ORIGIN: &str = "https://lensisku.lojban.org";
const LOCALES: [&str; 5] = ["en", "jbo", "ru", "ja", "zh"];

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn compact(value: &str, limit: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= limit {
        return normalized;
    }
    let mut shortened = normalized
        .chars()
        .take(limit.saturating_sub(1))
        .collect::<String>();
    shortened.push('…');
    shortened
}

fn content_preview(content: Option<&str>) -> String {
    let Some(content) = content else {
        return String::new();
    };
    let unquoted = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('>'))
        .collect::<Vec<_>>()
        .join(" ");
    compact(&unquoted, 180)
}

fn site_title(locale: &str) -> &'static str {
    match locale {
        "jbo" => "la lensisku",
        "ru" => "Портал Ложбана",
        "ja" => "ロジバン・ポータル",
        "zh" => "逻辑语门户",
        _ => "Lojban Portal",
    }
}

fn remove_tag(mut head: String, prefix: &str, closing: &str) -> String {
    while let Some(start) = head.find(prefix) {
        let Some(end) = head[start..].find(closing) else {
            break;
        };
        head.replace_range(start..start + end + closing.len(), "");
    }
    head
}

fn render_mail_page(
    template: &str,
    locale: &str,
    subject: &str,
    content: Option<&str>,
    path: &str,
) -> Option<String> {
    let head_end = template.find("</head>")?;
    let (original_head, rest) = template.split_at(head_end);
    let mut head = original_head.to_string();

    head = remove_tag(head, "<title", "</title>");
    for prefix in [
        "<meta name=\"description\"",
        "<meta name=\"twitter:",
        "<meta property=\"og:",
        "<link rel=\"canonical\"",
        "<link rel=\"alternate\"",
    ] {
        head = remove_tag(head, prefix, ">");
    }
    head = head.replacen(
        "<html lang=\"en\">",
        &format!("<html lang=\"{locale}\">"),
        1,
    );

    let subject = compact(subject, 180);
    let title = format!("{subject} | {}", site_title(locale));
    let preview = content_preview(content);
    let description_subject = compact(&subject, 80);
    let description = if preview.is_empty() {
        format!("{description_subject} — {}", site_title(locale))
    } else {
        format!("{description_subject}: {preview}")
    };
    let description = compact(&description, 155);
    let canonical = format!("{SITE_ORIGIN}{path}");

    head.push_str(&format!(
        "<title>{}</title><meta name=\"description\" content=\"{}\"><meta property=\"og:title\" content=\"{}\"><meta property=\"og:description\" content=\"{}\"><meta property=\"og:site_name\" content=\"{}\"><meta property=\"og:type\" content=\"website\"><meta property=\"og:locale\" content=\"{}\"><meta property=\"og:url\" content=\"{}\"><meta name=\"twitter:card\" content=\"summary\"><meta name=\"twitter:title\" content=\"{}\"><meta name=\"twitter:description\" content=\"{}\"><link rel=\"canonical\" href=\"{}\">",
        escape_html(&title),
        escape_html(&description),
        escape_html(&title),
        escape_html(&description),
        escape_html(site_title(locale)),
        locale,
        escape_html(&canonical),
        escape_html(&title),
        escape_html(&description),
        escape_html(&canonical),
    ));

    let path_without_locale = path.strip_prefix(&format!("/{locale}")).unwrap_or(path);
    for alternate_locale in LOCALES {
        head.push_str(&format!(
            "<link rel=\"alternate\" hreflang=\"{alternate_locale}\" href=\"{}\">",
            escape_html(&format!(
                "{SITE_ORIGIN}/{alternate_locale}{path_without_locale}"
            )),
        ));
    }
    head.push_str(&format!(
        "<link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\">",
        escape_html(&format!("{SITE_ORIGIN}/en{path_without_locale}")),
    ));
    Some(format!("{head}{rest}"))
}

fn html_response(locale: &str, subject: &str, content: Option<&str>, path: &str) -> HttpResponse {
    let file = std::env::var("FRONTEND_DIST_DIR")
        .map(|dir| format!("{dir}/index.html"))
        .unwrap_or_else(|_| {
            if std::path::Path::new("/src/frontend/dist/index.html").exists() {
                "/src/frontend/dist/index.html".to_string()
            } else {
                "/var/www/html/index.html".to_string()
            }
        });
    let Ok(template) = std::fs::read_to_string(file) else {
        return HttpResponse::InternalServerError().finish();
    };
    let Some(page) = render_mail_page(&template, locale, subject, content, path) else {
        return HttpResponse::InternalServerError().finish();
    };
    HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "public, max-age=300"))
        .content_type("text/html; charset=utf-8")
        .body(page)
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
            html_response(&locale, &title, content.as_deref(), &canonical_path)
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
            html_response(
                &locale,
                title
                    .as_deref()
                    .filter(|subject| !subject.trim().is_empty())
                    .unwrap_or("Mail message"),
                content.as_deref(),
                &canonical_path,
            )
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::render_mail_page;
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

    #[test]
    fn replaces_default_tags_and_escapes_mail_subject() {
        let template = r#"<html lang="en"><head><meta name="description" content="Default"><title>Default</title><meta property="og:title" content="Default"><meta property="og:url" content="https://example.com/en"><link rel="canonical" href="https://example.com/en"><link rel="alternate" hreflang="en" href="https://example.com/en"></head><body><div id="app"></div></body></html>"#;
        let page = render_mail_page(
            template,
            "en",
            "A <test> & \"quote\"",
            Some("A useful reply"),
            "/en/thread/A%20test",
        )
        .unwrap();
        assert!(
            page.contains("<title>A &lt;test&gt; &amp; &quot;quote&quot; | Lojban Portal</title>")
        );
        assert!(page.contains("content=\"A &lt;test&gt; &amp; &quot;quote&quot;: A useful reply\""));
        assert!(!page.contains("content=\"Default\""));
        assert_eq!(page.matches("rel=\"canonical\"").count(), 1);
        assert_eq!(page.matches("rel=\"alternate\"").count(), 6);
    }
}
