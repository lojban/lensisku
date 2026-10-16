//! Shared helpers for injecting page-specific SEO tags into the SPA `index.html`.
//!
//! Crawlers and Open Graph clients do not run the Vue app, so nginx proxies selected
//! page routes to Actix handlers that rewrite `<title>` / meta tags from DB content.

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

pub fn html_response(locale: &str, page_title: &str, description: &str, path: &str) -> HttpResponse {
    let Ok(template) = std::fs::read_to_string(index_html_path()) else {
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
