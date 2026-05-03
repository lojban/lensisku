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

async fn fetch_thread_root_import_sources(
    pool: &Pool,
    thread_ids: &[i32],
) -> Result<std::collections::HashMap<i32, Option<String>>, Box<dyn std::error::Error + Send + Sync>>
{
    use std::collections::HashMap;
    if thread_ids.is_empty() {
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
            "SELECT DISTINCT ON (threadid) threadid, import_source
             FROM comments
             WHERE threadid = ANY($1)
             ORDER BY threadid, time ASC, commentid ASC",
            &[&thread_ids],
        )
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(io::Error::other(e.to_string()))
        })?;
    let mut out = HashMap::new();
    for row in rows {
        let tid: i32 = row.get("threadid");
        let import_source: Option<String> = row.get("import_source");
        out.insert(tid, import_source);
    }
    Ok(out)
}

/// Parse message date string to unix timestamp for sorting. Returns 0 on parse failure.
fn message_timestamp(date_str: Option<&String>) -> i64 {
    let s = match date_str {
        Some(s) if !s.is_empty() => s,
        _ => return 0,
    };
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
        return dt.with_timezone(&Utc).timestamp();
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return dt.with_timezone(&Utc).timestamp();
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc).timestamp();
    }
    0
}

fn wave_hit_sort_key(hit: &WaveSearchHit, sort_by: &str) -> i64 {
    match (sort_by, hit) {
        ("reactions", WaveSearchHit::Comment { comment: c, .. }) => c.total_reactions,
        ("replies", WaveSearchHit::Comment { comment: c, .. }) => c.total_replies,
        ("time", WaveSearchHit::Comment { comment: c, .. }) => c.time as i64,
        ("reactions", WaveSearchHit::Mail { .. }) => 0,
        ("replies", WaveSearchHit::Mail { .. }) => 0,
        ("time", WaveSearchHit::Mail { message: m }) => message_timestamp(m.date.as_ref()),
        _ => match hit {
            WaveSearchHit::Comment { comment: c, .. } => c.time as i64,
            WaveSearchHit::Mail { message: m } => message_timestamp(m.date.as_ref()),
        },
    }
}

fn thread_summary_sort_key(summary: &WaveThreadSummary, sort_by: &str) -> i64 {
    match (sort_by, summary) {
        (
            "reactions",
            WaveThreadSummary::Comment {
                last_comment_reactions,
                ..
            },
        ) => *last_comment_reactions,
        ("replies", WaveThreadSummary::Comment { total_replies, .. }) => *total_replies,
        ("time", WaveThreadSummary::Comment {
            last_activity_time,
            ..
        }) => *last_activity_time as i64,
        ("reactions", WaveThreadSummary::Mail {
            last_comment_reactions,
            ..
        }) => *last_comment_reactions,
        ("replies", WaveThreadSummary::Mail { message_count, .. }) => *message_count,
        ("time", WaveThreadSummary::Mail {
            last_activity_time,
            ..
        }) => *last_activity_time,
        _ => match summary {
            WaveThreadSummary::Comment {
                last_activity_time, ..
            } => *last_activity_time as i64,
            WaveThreadSummary::Mail {
                last_activity_time, ..
            } => *last_activity_time,
        },
    }
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
    let sort_by = query.sort_by.as_deref().unwrap_or("time");
    let source = query.source.as_deref().unwrap_or("all");

    let fetch_per_source = (page * per_page).min(100);

    let wave_source = match source {
        "jbotcan" => Some("jbotcan".to_string()),
        "comments" => Some("comments".to_string()),
        _ => None,
    };

    let comments_params = SearchCommentsParams {
        page: 1,
        per_page: fetch_per_source,
        search_term: search_term.clone(),
        sort_by: sort_by.to_string(),
        sort_order: sort_order.to_string(),
        username: None,
        valsi_id: None,
        definition_id: None,
        definition_link_id: None,
        target_user_id: None,
        collection_id: query.collection_id,
        wave_source: wave_source.clone(),
    };

    // Mail messages are not collection-scoped, so when the caller restricts to a collection we
    // silently drop the mail half of the unified search regardless of `source`.
    let collection_scoped = query.collection_id.is_some();

    let mail_query = mailarchive::SearchQuery {
        query: search_term.clone(),
        page: Some(1),
        per_page: Some(fetch_per_source),
        sort_by: Some("date".to_string()),
        sort_order: Some(sort_order.to_string()),
        include_content: Some(true),
        group_by_thread: Some(false),
    };

    let comments_res;
    let mail_res;

    match source {
        "mail" if !collection_scoped => {
            comments_res = None;
            mail_res = Some(
                mailarchive_service::search_messages(pool, mail_query)
                    .await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                        Box::new(io::Error::other(e.to_string()))
                    })?,
            );
        }
        "mail" => {
            // Collection scope makes mail-only search empty.
            comments_res = None;
            mail_res = None;
        }
        "jbotcan" | "comments" => {
            mail_res = None;
            comments_res = Some(
                comments_service::search_comments(pool, comments_params, current_user_id)
                    .await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                        Box::new(io::Error::other(e.to_string()))
                    })?,
            );
        }
        _ if collection_scoped => {
            mail_res = None;
            comments_res = Some(
                comments_service::search_comments(pool, comments_params, current_user_id)
                    .await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                        Box::new(io::Error::other(e.to_string()))
                    })?,
            );
        }
        _ => {
            let (c, m) = tokio::try_join!(
                comments_service::search_comments(pool, comments_params, current_user_id),
                mailarchive_service::search_messages(pool, mail_query),
            )
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(io::Error::other(e.to_string()))
            })?;
            comments_res = Some(c);
            mail_res = Some(m);
        }
    }

    let total = comments_res.as_ref().map(|c| c.total).unwrap_or(0)
        + mail_res.as_ref().map(|m| m.total).unwrap_or(0);

    let comment_ids: Vec<i32> = comments_res
        .as_ref()
        .map(|c| c.comments.iter().map(|x| x.comment_id).collect())
        .unwrap_or_default();
    let import_sources = fetch_comment_import_sources(pool, &comment_ids).await?;

    let mut merged: Vec<WaveSearchHit> = Vec::new();
    if let Some(c) = comments_res {
        for c in c.comments {
            let import_source = import_sources.get(&c.comment_id).cloned().flatten();
            merged.push(WaveSearchHit::Comment {
                comment: c,
                import_source,
            });
        }
    }
    if let Some(m) = mail_res {
        for msg in m.messages {
            merged.push(WaveSearchHit::Mail { message: msg });
        }
    }

    merged.sort_by(|a, b| {
        let ka = wave_hit_sort_key(a, sort_by);
        let kb = wave_hit_sort_key(b, sort_by);
        if sort_order.eq_ignore_ascii_case("asc") {
            ka.cmp(&kb)
        } else {
            kb.cmp(&ka)
        }
    });
    let start = ((page - 1) * per_page) as usize;
    let items: Vec<WaveSearchHit> = merged
        .into_iter()
        .skip(start)
        .take(per_page as usize)
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
    let sort_by_raw = query.sort_by.as_deref().unwrap_or("time");
    let sort_order = query.sort_order.as_deref().unwrap_or("desc");
    let source = query.source.as_deref().unwrap_or("all");

    let threads_sort_by = match sort_by_raw {
        "reactions" => "reactions",
        "replies" => "replies",
        _ => "time",
    };
    let mail_sort_by = match sort_by_raw {
        "reactions" => "reactions",
        "replies" => "replies",
        _ => "time",
    };

    let fetch_per_source = (page * per_page).min(100);

    let wave_source = match source {
        "jbotcan" => Some("jbotcan"),
        "comments" => Some("comments"),
        _ => None,
    };

    let comment_res;
    let mail_threads;
    let mail_total;

    match source {
        "mail" => {
            comment_res = None;
            let (mt, mt_total) = mailarchive_service::list_mail_threads(
                pool,
                1,
                fetch_per_source,
                sort_order,
                mail_sort_by,
            )
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(io::Error::other(e.to_string()))
            })?;
            mail_threads = mt;
            mail_total = mt_total;
        }
        "jbotcan" | "comments" => {
            mail_threads = Vec::new();
            mail_total = 0;
            comment_res = Some(
                comments_service::list_threads(
                    pool,
                    1,
                    fetch_per_source,
                    threads_sort_by,
                    sort_order,
                    wave_source,
                )
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(io::Error::other(e.to_string()))
                })?,
            );
        }
        _ => {
            let (cr, (mt, mt_total)) = tokio::try_join!(
                comments_service::list_threads(
                    pool,
                    1,
                    fetch_per_source,
                    threads_sort_by,
                    sort_order,
                    wave_source,
                ),
                mailarchive_service::list_mail_threads(
                    pool,
                    1,
                    fetch_per_source,
                    sort_order,
                    mail_sort_by,
                ),
            )
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(io::Error::other(e.to_string()))
            })?;
            comment_res = Some(cr);
            mail_threads = mt;
            mail_total = mt_total;
        }
    }

    let mut items: Vec<WaveThreadSummary> = Vec::new();

    if let Some(ref cres) = comment_res {
        let thread_ids: Vec<i32> = cres.comments.iter().map(|c| c.thread_id).collect();
        let root_imports = fetch_thread_root_import_sources(pool, &thread_ids).await?;

        for c in &cres.comments {
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
                import_source: root_imports.get(&c.thread_id).cloned().flatten(),
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
                valsi_word: c.valsi_word.clone(),
                definition: c.definition.clone(),
                last_comment_reactions: c.total_reactions,
                last_comment_parent_id: c.parent_id,
                comment_num: c.comment_num,
            });
        }
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
            last_comment_reactions: 0,
        });
    }

    items.sort_by(|a, b| {
        let ka = thread_summary_sort_key(a, sort_by_raw);
        let kb = thread_summary_sort_key(b, sort_by_raw);
        if sort_order.eq_ignore_ascii_case("asc") {
            ka.cmp(&kb)
        } else {
            kb.cmp(&ka)
        }
    });

    let total = comment_res.as_ref().map(|c| c.total).unwrap_or(0) + mail_total;
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
