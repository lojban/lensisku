//! Read paths over `wiki_articles`. Used by `/waves/search`, `/waves/threads`,
//! and `GET /wiki/{title}`.

use deadpool_postgres::Pool;

use super::dto::{WikiArticleDetail, WikiSearchHit, WikiThreadSummary};
use super::markdown::wiki_target_url;

const PREVIEW_LEN: usize = 400;

fn truncate_preview(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().count() <= PREVIEW_LEN {
        return Some(trimmed.to_string());
    }
    let mut out = String::new();
    for (i, c) in trimmed.chars().enumerate() {
        if i >= PREVIEW_LEN {
            break;
        }
        out.push(c);
    }
    out.push('…');
    Some(out)
}

/// LIKE-escape: `%`, `_`, `\` become escaped under `ESCAPE '\'` semantics.
fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Search wiki articles. Returns (hits, total).
///
/// `sort_by` accepts `"time"` (most recently edited first) or anything else
/// (treated as "relevance" — exact title match > title hit > body hit).
pub async fn search_wiki(
    pool: &Pool,
    search_term: &str,
    sort_by: &str,
    sort_order: &str,
    page: i64,
    per_page: i64,
) -> Result<(Vec<WikiSearchHit>, i64), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool
        .get()
        .await
        .map_err(box_err)?;

    let offset = ((page - 1).max(0)) * per_page;
    let order_dir = if sort_order.eq_ignore_ascii_case("asc") { "ASC" } else { "DESC" };

    if search_term.trim().is_empty() {
        // No query → most-recently-edited (or oldest with asc).
        let total_row = client
            .query_one(
                "SELECT COUNT(*)::BIGINT AS c FROM wiki_articles WHERE NOT is_redirect",
                &[],
            )
            .await
            .map_err(box_err)?;
        let total: i64 = total_row.get("c");

        let order_clause = match sort_by {
            "time" => format!("last_edited {order_dir} NULLS LAST"),
            _ => format!("last_edited {order_dir} NULLS LAST"),
        };
        let sql = format!(
            "SELECT page_id, namespace, title, plain_text, last_edited
             FROM wiki_articles
             WHERE NOT is_redirect
             ORDER BY {order_clause}
             LIMIT $1 OFFSET $2"
        );
        let rows = client
            .query(&sql, &[&per_page, &offset])
            .await
            .map_err(box_err)?;
        let hits = rows.into_iter().map(row_to_hit).collect();
        return Ok((hits, total));
    }

    let pattern = format!("%{}%", escape_like(search_term));

    let total_row = client
        .query_one(
            "SELECT COUNT(*)::BIGINT AS c
             FROM wiki_articles
             WHERE NOT is_redirect
               AND (title ILIKE $1 ESCAPE '\\' OR plain_text ILIKE $1 ESCAPE '\\')",
            &[&pattern],
        )
        .await
        .map_err(box_err)?;
    let total: i64 = total_row.get("c");

    let (sql, params) = if sort_by == "time" {
        let sql = format!(
            "SELECT page_id, namespace, title, plain_text, last_edited
             FROM wiki_articles
             WHERE NOT is_redirect
               AND (title ILIKE $1 ESCAPE '\\' OR plain_text ILIKE $1 ESCAPE '\\')
             ORDER BY last_edited {order_dir} NULLS LAST
             LIMIT $2 OFFSET $3"
        );
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&pattern, &per_page, &offset];
        (sql, params)
    } else {
        let sql = format!(
            "SELECT page_id, namespace, title, plain_text, last_edited
             FROM wiki_articles
             WHERE NOT is_redirect
               AND (title ILIKE $1 ESCAPE '\\' OR plain_text ILIKE $1 ESCAPE '\\')
             ORDER BY (CASE
                 WHEN lower(title) = lower($2) THEN 3
                 WHEN title ILIKE $1 ESCAPE '\\' THEN 2
                 ELSE 1
               END) {order_dir},
               last_edited DESC NULLS LAST
             LIMIT $3 OFFSET $4"
        );
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&pattern, &search_term, &per_page, &offset];
        (sql, params)
    };
    let rows = client
        .query(&sql, &params)
        .await
        .map_err(box_err)?;
    let hits = rows.into_iter().map(row_to_hit).collect();
    Ok((hits, total))
}

/// List wiki articles for the threads view (paginated, sorted by recency).
pub async fn list_wiki_threads(
    pool: &Pool,
    page: i64,
    per_page: i64,
    sort_order: &str,
) -> Result<(Vec<WikiThreadSummary>, i64), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await.map_err(box_err)?;
    let offset = ((page - 1).max(0)) * per_page;
    let order_dir = if sort_order.eq_ignore_ascii_case("asc") { "ASC" } else { "DESC" };

    let total_row = client
        .query_one(
            "SELECT COUNT(*)::BIGINT AS c FROM wiki_articles WHERE NOT is_redirect",
            &[],
        )
        .await
        .map_err(box_err)?;
    let total: i64 = total_row.get("c");

    let sql = format!(
        "SELECT page_id, namespace, title, plain_text, last_edited
         FROM wiki_articles
         WHERE NOT is_redirect
         ORDER BY last_edited {order_dir} NULLS LAST
         LIMIT $1 OFFSET $2"
    );
    let rows = client
        .query(&sql, &[&per_page, &offset])
        .await
        .map_err(box_err)?;
    let items = rows
        .into_iter()
        .map(|r| {
            let page_id: i32 = r.get("page_id");
            let namespace: i32 = r.get("namespace");
            let title: String = r.get("title");
            let plain: String = r.get("plain_text");
            let last_edited: Option<chrono::DateTime<chrono::Utc>> =
                r.try_get("last_edited").ok().flatten();
            let article_url = format!("/wiki/{}", urlencoding::encode(&title));
            WikiThreadSummary {
                page_id,
                namespace,
                title,
                last_edited,
                content_preview: truncate_preview(&plain),
                article_url,
            }
        })
        .collect();
    Ok((items, total))
}

/// Fetch a single article by `title` (URL-decoded) for the detail page.
pub async fn get_article_by_title(
    pool: &Pool,
    title: &str,
) -> Result<Option<WikiArticleDetail>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await.map_err(box_err)?;
    let normalized = title.replace('_', " ");
    let row = client
        .query_opt(
            "SELECT page_id, namespace, title, markdown, last_edited, is_redirect
             FROM wiki_articles
             WHERE title = $1 OR title = $2
             ORDER BY (CASE WHEN title = $1 THEN 0 ELSE 1 END)
             LIMIT 1",
            &[&title, &normalized],
        )
        .await
        .map_err(box_err)?;
    Ok(row.map(|r| {
        let title: String = r.get("title");
        let target = title.replace(' ', "_");
        let source_url = format!(
            "https://mw.lojban.org/index.php?title={}",
            urlencoding::encode(&target)
        );
        WikiArticleDetail {
            page_id: r.get("page_id"),
            namespace: r.get("namespace"),
            title,
            markdown: r.get("markdown"),
            last_edited: r.try_get("last_edited").ok().flatten(),
            is_redirect: r.try_get("is_redirect").unwrap_or(false),
            source_url,
        }
    }))
}

fn row_to_hit(r: tokio_postgres::Row) -> WikiSearchHit {
    let title: String = r.get("title");
    let plain: String = r.get("plain_text");
    let last_edited: Option<chrono::DateTime<chrono::Utc>> = r.try_get("last_edited").ok().flatten();
    let article_url = format!("/wiki/{}", urlencoding::encode(&title));
    let _ = wiki_target_url(&title); // keep import used to avoid dead_code if not referenced
    WikiSearchHit {
        page_id: r.get("page_id"),
        namespace: r.get("namespace"),
        title,
        last_edited,
        content_preview: truncate_preview(&plain),
        article_url,
    }
}

fn box_err<E: std::fmt::Display>(e: E) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::other(e.to_string()))
}
