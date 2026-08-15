//! Read paths over `wiki_articles`. Used by `/waves/search`, `/waves/threads`,
//! and `GET /wiki/{title}`.

use deadpool_postgres::Pool;

use super::dto::{WikiArticleDetail, WikiSearchHit, WikiThreadSummary};
use super::markdown::rewrite_wiki_links_for_lensisku;

const PREVIEW_LEN: usize = 400;
/// Exclude soft-redirect stub pages from waves search/list.
const NATIVE_WIKI_NOT_REDIRECT: &str =
    "COALESCE(d.metadata->>'is_redirect', 'false') <> 'true'";

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

/// Search wiki articles and native wiki pages. Returns (hits, total).
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
    use std::cmp::Ordering;

    let client = pool.get().await.map_err(box_err)?;

    let order_dir = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };
    let fetch_limit = (page * per_page).max(1);
    let final_offset = ((page - 1).max(0)) * per_page;
    let is_empty_search = search_term.trim().is_empty();
    let pattern = format!("%{}%", escape_like(search_term));

    // --- Mirrored mw.lojban.org articles ---
    let (mirror_total, mirror_rows) = if is_empty_search {
        let total_row = client
            .query_one(
                "SELECT COUNT(*)::BIGINT AS c FROM wiki_articles WHERE NOT is_redirect",
                &[],
            )
            .await
            .map_err(box_err)?;
        let total: i64 = total_row.get("c");

        let order_clause = format!("last_edited {order_dir} NULLS LAST");
        let sql = format!(
            "SELECT page_id, namespace, title, plain_text, last_edited
             FROM wiki_articles
             WHERE NOT is_redirect
             ORDER BY {order_clause}
             LIMIT $1 OFFSET $2"
        );
        let rows = client
            .query(&sql, &[&fetch_limit, &0i64])
            .await
            .map_err(box_err)?;
        (total, rows)
    } else {
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
            let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
                vec![&pattern, &fetch_limit, &0i64];
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
            let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
                vec![&pattern, &search_term, &fetch_limit, &0i64];
            (sql, params)
        };
        let rows = client.query(&sql, &params).await.map_err(box_err)?;
        (total, rows)
    };

    // --- Native Lensisku wiki pages (stored as valsi type 16) ---
    let (native_total, native_rows) = if is_empty_search {
        let total_row = client
            .query_one(
                "SELECT COUNT(*)::BIGINT AS c
                 FROM definitions d
                 JOIN valsi v ON d.valsiid = v.valsiid
                 WHERE v.typeid = 16
                   AND COALESCE(d.metadata->>'is_redirect', 'false') <> 'true'",
                &[],
            )
            .await
            .map_err(box_err)?;
        let total: i64 = total_row.get("c");

        let sql = format!(
            "SELECT d.definitionid AS page_id, v.word AS title, d.definition, d.created_at AS last_edited
             FROM definitions d
             JOIN valsi v ON d.valsiid = v.valsiid
             WHERE v.typeid = 16
               AND {NATIVE_WIKI_NOT_REDIRECT}
             ORDER BY d.created_at {order_dir} NULLS LAST
             LIMIT $1 OFFSET $2"
        );
        let rows = client
            .query(&sql, &[&fetch_limit, &0i64])
            .await
            .map_err(box_err)?;
        (total, rows)
    } else {
        let total_row = client
            .query_one(
                "SELECT COUNT(*)::BIGINT AS c
                 FROM definitions d
                 JOIN valsi v ON d.valsiid = v.valsiid
                 WHERE v.typeid = 16
                   AND COALESCE(d.metadata->>'is_redirect', 'false') <> 'true'
                   AND (v.word ILIKE $1 ESCAPE '\\' OR d.definition ILIKE $1 ESCAPE '\\')",
                &[&pattern],
            )
            .await
            .map_err(box_err)?;
        let total: i64 = total_row.get("c");

        let (sql, params) = if sort_by == "time" {
            let sql = format!(
                "SELECT d.definitionid AS page_id, v.word AS title, d.definition, d.created_at AS last_edited
                 FROM definitions d
                 JOIN valsi v ON d.valsiid = v.valsiid
                 WHERE v.typeid = 16
                   AND {NATIVE_WIKI_NOT_REDIRECT}
                   AND (v.word ILIKE $1 ESCAPE '\\' OR d.definition ILIKE $1 ESCAPE '\\')
                 ORDER BY d.created_at {order_dir} NULLS LAST
                 LIMIT $2 OFFSET $3"
            );
            let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
                vec![&pattern, &fetch_limit, &0i64];
            (sql, params)
        } else {
            let sql = format!(
                "SELECT d.definitionid AS page_id, v.word AS title, d.definition, d.created_at AS last_edited
                 FROM definitions d
                 JOIN valsi v ON d.valsiid = v.valsiid
                 WHERE v.typeid = 16
                   AND {NATIVE_WIKI_NOT_REDIRECT}
                   AND (v.word ILIKE $1 ESCAPE '\\' OR d.definition ILIKE $1 ESCAPE '\\')
                 ORDER BY (CASE
                     WHEN lower(v.word) = lower($2) THEN 3
                     WHEN v.word ILIKE $1 ESCAPE '\\' THEN 2
                     ELSE 1
                   END) {order_dir},
                   d.created_at DESC NULLS LAST
                 LIMIT $3 OFFSET $4"
            );
            let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
                vec![&pattern, &search_term, &fetch_limit, &0i64];
            (sql, params)
        };
        let rows = client.query(&sql, &params).await.map_err(box_err)?;
        (total, rows)
    };

    let mut hits: Vec<WikiSearchHit> = Vec::with_capacity(mirror_rows.len() + native_rows.len());
    hits.extend(mirror_rows.into_iter().map(row_to_hit));
    hits.extend(native_rows.into_iter().map(row_to_native_hit));

    // Merge-sort the top items from both sources and apply final pagination.
    if sort_by == "time" {
        hits.sort_by(|a, b| match (a.last_edited, b.last_edited) {
            (Some(a), Some(b)) if order_dir == "ASC" => a.cmp(&b),
            (Some(a), Some(b)) => b.cmp(&a),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        });
    } else {
        fn relevance(term: &str, hit: &WikiSearchHit) -> i32 {
            let t = hit.title.to_lowercase();
            let term = term.to_lowercase();
            if t == term {
                3
            } else if t.contains(&term) {
                2
            } else {
                1
            }
        }
        hits.sort_by(|a, b| {
            let score_a = relevance(search_term, a);
            let score_b = relevance(search_term, b);
            score_b.cmp(&score_a).then_with(|| match (b.last_edited, a.last_edited) {
                (Some(tb), Some(ta)) => tb.cmp(&ta),
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (None, None) => Ordering::Equal,
            })
        });
    }

    let total = mirror_total + native_total;
    let paginated: Vec<WikiSearchHit> = hits
        .into_iter()
        .skip(final_offset as usize)
        .take(per_page as usize)
        .collect();
    Ok((paginated, total))
}

/// List wiki articles and native wiki pages for the threads view (paginated, sorted by recency).
pub async fn list_wiki_threads(
    pool: &Pool,
    page: i64,
    per_page: i64,
    sort_order: &str,
) -> Result<(Vec<WikiThreadSummary>, i64), Box<dyn std::error::Error + Send + Sync>> {
    use std::cmp::Ordering;

    let client = pool.get().await.map_err(box_err)?;
    let order_dir = if sort_order.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };
    let fetch_limit = (page * per_page).max(1);
    let final_offset = ((page - 1).max(0)) * per_page;

    // --- Mirrored articles ---
    let mirror_total_row = client
        .query_one(
            "SELECT COUNT(*)::BIGINT AS c FROM wiki_articles WHERE NOT is_redirect",
            &[],
        )
        .await
        .map_err(box_err)?;
    let mirror_total: i64 = mirror_total_row.get("c");

    let mirror_sql = format!(
        "SELECT page_id, namespace, title, plain_text, last_edited
         FROM wiki_articles
         WHERE NOT is_redirect
         ORDER BY last_edited {order_dir} NULLS LAST
         LIMIT $1 OFFSET $2"
    );
    let mirror_rows = client
        .query(&mirror_sql, &[&fetch_limit, &0i64])
        .await
        .map_err(box_err)?;

    // --- Native wiki pages ---
    let native_total_row = client
        .query_one(
            "SELECT COUNT(*)::BIGINT AS c
             FROM definitions d
             JOIN valsi v ON d.valsiid = v.valsiid
             WHERE v.typeid = 16
               AND COALESCE(d.metadata->>'is_redirect', 'false') <> 'true'",
            &[],
        )
        .await
        .map_err(box_err)?;
    let native_total: i64 = native_total_row.get("c");

    let native_sql = format!(
        "SELECT d.definitionid AS page_id, v.word AS title, d.definition, d.created_at AS last_edited
         FROM definitions d
         JOIN valsi v ON d.valsiid = v.valsiid
         WHERE v.typeid = 16
           AND {NATIVE_WIKI_NOT_REDIRECT}
         ORDER BY d.created_at {order_dir} NULLS LAST
         LIMIT $1 OFFSET $2"
    );
    let native_rows = client
        .query(&native_sql, &[&fetch_limit, &0i64])
        .await
        .map_err(box_err)?;

    let mut items: Vec<WikiThreadSummary> = Vec::with_capacity(mirror_rows.len() + native_rows.len());
    items.extend(mirror_rows.into_iter().map(|r| {
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
    }));
    items.extend(native_rows.into_iter().map(|r| {
        let page_id: i32 = r.get("page_id");
        let title: String = r.get("title");
        let definition: String = r.get("definition");
        let last_edited: Option<chrono::DateTime<chrono::Utc>> =
            r.try_get("last_edited").ok().flatten();
        let article_url = format!("/wiki/{}", urlencoding::encode(&title));
        WikiThreadSummary {
            page_id,
            namespace: 0,
            title,
            last_edited,
            content_preview: truncate_preview(&definition),
            article_url,
        }
    }));

    items.sort_by(|a, b| match (a.last_edited, b.last_edited) {
        (Some(a), Some(b)) if order_dir == "ASC" => a.cmp(&b),
        (Some(a), Some(b)) => b.cmp(&a),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    });

    let total = mirror_total + native_total;
    let paginated: Vec<WikiThreadSummary> = items
        .into_iter()
        .skip(final_offset as usize)
        .take(per_page as usize)
        .collect();
    Ok((paginated, total))
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
            "https://mw.lojban.org/papri/{}",
            urlencoding::encode(&target)
        );
        let markdown: String = r.get("markdown");
        WikiArticleDetail {
            page_id: r.get("page_id"),
            namespace: r.get("namespace"),
            title,
            markdown: rewrite_wiki_links_for_lensisku(&markdown),
            last_edited: r.try_get("last_edited").ok().flatten(),
            is_redirect: r.try_get("is_redirect").unwrap_or(false),
            source_url,
        }
    }))
}

fn row_to_hit(r: tokio_postgres::Row) -> WikiSearchHit {
    let title: String = r.get("title");
    let plain: String = r.get("plain_text");
    let last_edited: Option<chrono::DateTime<chrono::Utc>> =
        r.try_get("last_edited").ok().flatten();
    let article_url = format!("/wiki/{}", urlencoding::encode(&title));
    WikiSearchHit {
        page_id: r.get("page_id"),
        namespace: r.get("namespace"),
        title,
        last_edited,
        content_preview: truncate_preview(&plain),
        article_url,
    }
}

fn row_to_native_hit(r: tokio_postgres::Row) -> WikiSearchHit {
    let title: String = r.get("title");
    let definition: String = r.get("definition");
    let last_edited: Option<chrono::DateTime<chrono::Utc>> =
        r.try_get("last_edited").ok().flatten();
    let article_url = format!("/wiki/{}", urlencoding::encode(&title));
    WikiSearchHit {
        page_id: r.get("page_id"),
        namespace: 0,
        title,
        last_edited,
        content_preview: truncate_preview(&definition),
        article_url,
    }
}

fn box_err<E: std::fmt::Display>(e: E) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::other(e.to_string()))
}
