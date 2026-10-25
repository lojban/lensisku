//! Shared helpers for injecting page-specific SEO tags into the SPA `index.html`.
//!
//! Crawlers and Open Graph clients do not run the Vue app, so nginx proxies selected
//! page routes to Actix handlers that rewrite `<title>` / meta tags from DB content.

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use actix_web::{http::header, HttpResponse};

pub const SITE_ORIGIN: &str = "https://lensisku.lojban.org";
pub const LOCALES: [&str; 5] = ["en", "jbo", "ru", "ja", "zh"];

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn compact(value: &str, limit: usize) -> String {
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

pub fn site_title(locale: &str) -> &'static str {
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

/// Replace default SPA head tags with page-specific title, description, and hreflang links.
pub fn render_spa_page(
    template: &str,
    locale: &str,
    page_title: &str,
    description: &str,
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

    let page_title = compact(page_title, 180);
    let title = format!("{page_title} | {}", site_title(locale));
    let description = compact(description, 155);
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

fn index_html_path() -> String {
    std::env::var("FRONTEND_DIST_DIR")
        .map(|dir| format!("{dir}/index.html"))
        .unwrap_or_else(|_| {
            if std::path::Path::new("/src/frontend/dist/index.html").exists() {
                "/src/frontend/dist/index.html".to_string()
            } else {
                "/var/www/html/index.html".to_string()
            }
        })
}

/// `(path, mtime, len)` identifies a template version; any change triggers a reload.
type TemplateKey = (String, Option<SystemTime>, u64);

static TEMPLATE_CACHE: Mutex<Option<(TemplateKey, Arc<str>)>> = Mutex::new(None);

/// Read `path`, reusing the previous contents while its mtime and size are unchanged.
/// A `stat` per request replaces a full read, and a rebuilt frontend is picked up
/// without restarting the server.
async fn cached_template(path: &str) -> Option<Arc<str>> {
    let meta = tokio::fs::metadata(path).await.ok()?;
    let key: TemplateKey = (path.to_owned(), meta.modified().ok(), meta.len());
    // A poisoned lock only means another request panicked mid-update; the cache
    // is rebuilt from disk below, so recovering the guard is safe.
    {
        let cache = TEMPLATE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((cached_key, contents)) = cache.as_ref() {
            if *cached_key == key {
                return Some(Arc::clone(contents));
            }
        }
    }
    // Read without holding the (std) lock across the await.
    let contents: Arc<str> = tokio::fs::read_to_string(path).await.ok()?.into();
    let mut cache = TEMPLATE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    *cache = Some((key, Arc::clone(&contents)));
    Some(contents)
}

pub async fn html_response(
    locale: &str,
    page_title: &str,
    description: &str,
    path: &str,
) -> HttpResponse {
    let Some(template) = cached_template(&index_html_path()).await else {
        return HttpResponse::InternalServerError().finish();
    };
    let Some(page) = render_spa_page(&template, locale, page_title, description, path) else {
        return HttpResponse::InternalServerError().finish();
    };
    HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "public, max-age=300"))
        .content_type("text/html; charset=utf-8")
        .body(page)
}

/// Strip markup enough for a meta description snippet.
pub fn plain_preview(text: &str, limit: usize) -> String {
    let without_tags = text
        .replace('<', " <")
        .split('<')
        .enumerate()
        .map(|(i, chunk)| {
            if i == 0 {
                chunk.to_string()
            } else if let Some((_, after)) = chunk.split_once('>') {
                after.to_string()
            } else {
                chunk.to_string()
            }
        })
        .collect::<String>();
    let without_md = without_tags
        .replace("](", " ")
        .chars()
        .filter(|c| !matches!(c, '#' | '*' | '`' | '~' | '[' | ']'))
        .collect::<String>();
    compact(&without_md, limit)
}

#[cfg(test)]
mod tests {
    use super::render_spa_page;

    #[tokio::test]
    async fn cached_template_reloads_when_file_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.html");
        let path_str = path.to_str().unwrap();
        std::fs::write(&path, "<html>one</html>").unwrap();
        assert_eq!(
            &*super::cached_template(path_str).await.unwrap(),
            "<html>one</html>"
        );
        assert_eq!(
            &*super::cached_template(path_str).await.unwrap(),
            "<html>one</html>"
        );
        // Different length guarantees a new key even on coarse-mtime filesystems.
        std::fs::write(&path, "<html>two!</html>").unwrap();
        assert_eq!(
            &*super::cached_template(path_str).await.unwrap(),
            "<html>two!</html>"
        );
        std::fs::remove_file(&path).unwrap();
        assert!(super::cached_template(path_str).await.is_none());
    }

    #[test]
    fn replaces_default_tags_and_escapes_title() {
        let template = r#"<html lang="en"><head><meta name="description" content="Default"><title>Default</title><meta property="og:title" content="Default"><meta property="og:url" content="https://example.com/en"><link rel="canonical" href="https://example.com/en"><link rel="alternate" hreflang="en" href="https://example.com/en"></head><body><div id="app"></div></body></html>"#;
        let page = render_spa_page(
            template,
            "en",
            "A <test> & \"quote\"",
            "A useful reply",
            "/en/wiki/A%20test",
        )
        .unwrap();
        assert!(
            page.contains("<title>A &lt;test&gt; &amp; &quot;quote&quot; | Lojban Portal</title>")
        );
        assert!(page.contains("content=\"A useful reply\""));
        assert!(!page.contains("content=\"Default\""));
        assert_eq!(page.matches("rel=\"canonical\"").count(), 1);
        assert_eq!(page.matches("rel=\"alternate\"").count(), 6);
    }
}
