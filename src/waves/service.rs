use std::io;

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;

use crate::comments::{
    dto::SearchCommentsParams,
    service as comments_service,
};
use crate::mailarchive::{
    self as mailarchive,
    service as mailarchive_service,
};
use crate::waves::dto::{WaveSearchHit, WaveThreadSummary, WavesSearchQuery, WavesThreadsQuery};

use super::dto::{WavesSearchResponse, WavesThreadsResponse};

async fn fetch_comment_import_sources(
    pool: &Pool,
    comment_ids: &[i32],
) -> Result<std::collections::HashMap<i32, Option<String>>, Box<dyn std::error::Error + Send + Sync>>
{
    use std::collections::HashMap;
    if comment_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let client = pool
        .get()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(io::Error::other(e.to_string()))
        })?;
    let rows = client
        .query(
            "SELECT commentid, import_source FROM comments WHERE commentid = ANY($1)",
            &[&comment_ids],
        )
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(io::Error::other(e.to_string()))
        })?;
    let mut out = HashMap::new();
    for row in rows {
        let comment_id: i32 = row.get("commentid");
        let import_source: Option<String> = row.get("import_source");
        out.insert(comment_id, import_source);
    }
    Ok(out)
}

/// Parse message date string to unix timestamp for sorting. Returns 0 on parse failure.
fn message_timestamp(date_str: Option<&String>) -> i64 {
    let s = match date_str {
        Some(s) if !s.is_empty() => s,
        _ => return 0,
    };
    // Try RFC 2822 first (e.g. "Mon, 1 Jan 2024 12:00:00 +0000")
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
        return dt.with_timezone(&Utc).timestamp();
    }
    // Try ISO 8601
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return dt.with_timezone(&Utc).timestamp();
    }
    // Try a simple format
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc).timestamp();
    }
    0
}

pub async fn search_waves(
    pool: &Pool,
    query: WavesSearchQuery,
    current_user_id: Option<i32>,
) -> Result<WavesSearchResponse, Box<dyn std::error::Error + Send + Sync>> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let search_term = query.search.as_deref().unwrap_or("").to_string();
    let sort_order = query.sort_order.as_deref().unwrap_or("desc");

    // Fetch enough from each source to merge and then paginate.
    let fetch_per_source = (page * per_page).min(100);

    let comments_params = SearchCommentsParams {
        page: 1,
        per_page: fetch_per_source,
        search_term: search_term.clone(),
        sort_by: query.sort_by.as_deref().unwrap_or("time").to_string(),
        sort_order: sort_order.to_string(),
        username: None,
        valsi_id: None,
        definition_id: None,
        definition_link_id: None,
        target_user_id: None,
    };

    let mail_query = mailarchive::SearchQuery {
        query: search_term.clone(),
        page: Some(1),
        per_page: Some(fetch_per_source),
        sort_by: Some("date".to_string()),
        sort_order: Some(sort_order.to_string()),
        include_content: Some(true),
        group_by_thread: Some(false),
    };

    let (comments_res, mail_res) = tokio::try_join!(
        comments_service::search_comments(pool, comments_params, current_user_id),
        mailarchive_service::search_messages(pool, mail_query),
    ).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(io::Error::other(e.to_string()))
    })?;

    let comment_ids: Vec<i32> = comments_res.comments.iter().map(|c| c.comment_id).collect();
    let import_sources = fetch_comment_import_sources(pool, &comment_ids).await?;

    let mut merged: Vec<(i64, WaveSearchHit)> = Vec::new();
    for c in comments_res.comments {
        let ts = c.time as i64;
        let import_source = import_sources.get(&c.comment_id).cloned().flatten();
        merged.push((ts, WaveSearchHit::Comment { comment: c, import_source }));
    }
    for m in mail_res.messages {
        let ts = message_timestamp(m.date.as_ref());
        merged.push((ts, WaveSearchHit::Mail { message: m }));
    }
    merged.sort_by(|a, b| b.0.cmp(&a.0));

    let total = comments_res.total + mail_res.total;
    let start = ((page - 1) * per_page) as usize;
    let items: Vec<WaveSearchHit> = merged
        .into_iter()
        .skip(start)
        .take(per_page as usize)
        .map(|(_, hit)| hit)
        .collect();

    Ok(WavesSearchResponse {
        items,
        total,
        page,
        per_page,
    })
}

pub async fn list_wave_threads(
    pool: &Pool,
    query: WavesThreadsQuery,
) -> Result<WavesThreadsResponse, Box<dyn std::error::Error + Send + Sync>> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let sort_by = query.sort_by.as_deref().unwrap_or("time");
    let sort_order = query.sort_order.as_deref().unwrap_or("desc");

    // Fetch enough from each source to merge and paginate.
    let fetch_per_source = (page * per_page).min(100);

    let (comment_res, (mail_threads, mail_total)) = tokio::try_join!(
        comments_service::list_threads(pool, 1, fetch_per_source, sort_by, sort_order),
        mailarchive_service::list_mail_threads(pool, 1, fetch_per_source, sort_order),
    ).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(io::Error::other(e.to_string()))
    })?;

    let mut items: Vec<WaveThreadSummary> = Vec::new();
    let comment_ids: Vec<i32> = comment_res.comments.iter().map(|c| c.comment_id).collect();
    let import_sources = fetch_comment_import_sources(pool, &comment_ids).await?;

    for c in comment_res.comments {
        let content = &c.content;
        let simple_content: Option<String> = if content.is_empty() {
            None
        } else {
            Some(
                content
                    .iter()
                    .filter(|p| p.r#type == "text")
                    .map(|p| p.data.as_str())
                    .collect::<String>(),
            )
        };
        items.push(WaveThreadSummary::Comment {
            thread_id: c.thread_id,
            comment_id: c.comment_id,
            import_source: import_sources.get(&c.comment_id).cloned().flatten(),
            first_comment_subject: c.first_comment_subject.clone(),
            first_comment_content: c.first_comment_content.as_ref().map(|v| {
                serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
            }),
            username: c.username.clone(),
            last_comment_username: c.last_comment_username.clone(),
            last_activity_time: c.time,
            total_replies: c.total_replies,
            simple_content,
            valsi_id: c.valsi_id,
            definition_id: c.definition_id,
        });
    }

    for m in mail_threads {
        let last_ts = m
            .last_sent_at
            .map(|t: chrono::DateTime<chrono::Utc>| t.timestamp())
            .unwrap_or(0);
        items.push(WaveThreadSummary::Mail {
            cleaned_subject: m.cleaned_subject,
            subject: m.subject,
            from_address: m.from_address,
            last_activity_time: last_ts,
            message_count: m.message_count,
            content_preview: m.content_preview,
        });
    }

    // Sort by last activity (comment time is i32 unix, mail last_activity_time is i64)
    items.sort_by(|a, b| {
        let ta = match a {
            WaveThreadSummary::Comment {
                last_activity_time, ..
            } => *last_activity_time as i64,
            WaveThreadSummary::Mail {
                last_activity_time, ..
            } => *last_activity_time,
        };
        let tb = match b {
            WaveThreadSummary::Comment {
                last_activity_time, ..
            } => *last_activity_time as i64,
            WaveThreadSummary::Mail {
                last_activity_time, ..
            } => *last_activity_time,
        };
        if sort_order.eq_ignore_ascii_case("asc") {
            ta.cmp(&tb)
        } else {
            tb.cmp(&ta)
        }
    });

    let total = comment_res.total + mail_total;
    let start = ((page - 1) * per_page) as usize;
    let items: Vec<WaveThreadSummary> = items
        .into_iter()
        .skip(start)
        .take(per_page as usize)
        .collect();

    Ok(WavesThreadsResponse {
        items,
        total,
        page,
        per_page,
    })
}
