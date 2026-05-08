//! Pulls articles from `mw.lojban.org` via the MediaWiki API and stores them in
//! `wiki_articles`. Run from `src/background/service.rs` on startup and weekly.
//!
//! Only the latest revision is mirrored. Namespaces 0 (Main) and 2 (User) are
//! included per product decision. Templates are NOT expanded.

use std::time::Duration;

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use log::{debug, info, warn};
use serde::Deserialize;

use super::markdown::wikitext_to_markdown;

const API_URL: &str = "https://mw.lojban.org/api.php";
const USER_AGENT: &str = "lensisku-wiki-importer/0.1 (https://lojban.org)";
const NAMESPACES: &[i32] = &[0, 2];

/// Returned from `?action=query&list=allpages`.
#[derive(Debug, Deserialize)]
struct AllPagesEnvelope {
    #[serde(default)]
    query: Option<AllPagesQuery>,
    #[serde(rename = "continue", default)]
    cont: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AllPagesQuery {
    #[serde(default)]
    allpages: Vec<PageRef>,
}

#[derive(Debug, Deserialize)]
struct PageRef {
    pageid: i64,
    #[serde(default)]
    #[allow(dead_code)]
    ns: i32,
    #[allow(dead_code)]
    title: String,
}

/// Returned from `?action=query&prop=revisions&rvslots=main`.
#[derive(Debug, Deserialize)]
struct RevisionsEnvelope {
    #[serde(default)]
    query: Option<RevisionsQuery>,
}

#[derive(Debug, Deserialize)]
struct RevisionsQuery {
    #[serde(default)]
    pages: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct PageWithRev {
    pageid: i64,
    #[serde(default)]
    ns: i32,
    title: String,
    #[serde(default)]
    missing: Option<bool>,
    #[serde(default)]
    revisions: Vec<RevEntry>,
}

#[derive(Debug, Deserialize)]
struct RevEntry {
    #[serde(default)]
    revid: Option<i64>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    slots: Option<RevSlots>,
}

#[derive(Debug, Deserialize)]
struct RevSlots {
    #[serde(default)]
    main: Option<RevSlotMain>,
}

#[derive(Debug, Deserialize)]
struct RevSlotMain {
    #[serde(rename = "*", default)]
    star: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RecentChangesEnvelope {
    #[serde(default)]
    query: Option<RecentChangesQuery>,
    #[serde(rename = "continue", default)]
    cont: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RecentChangesQuery {
    #[serde(default)]
    recentchanges: Vec<RecentChange>,
}

#[derive(Debug, Deserialize)]
struct RecentChange {
    #[serde(default)]
    pageid: Option<i64>,
    #[serde(default)]
    ns: Option<i32>,
    #[serde(default)]
    #[allow(dead_code)]
    title: Option<String>,
    #[serde(default, rename = "type")]
    rc_type: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    timestamp: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum WikiSyncError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("db error: {0}")]
    Db(String),
}

fn http_client() -> Result<reqwest::Client, WikiSyncError> {
    Ok(reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(60))
        .build()?)
}

fn sync_disabled() -> bool {
    matches!(std::env::var("DISABLE_WIKI_SYNC").ok().as_deref(), Some("1") | Some("true"))
}

/// Decide on startup whether to do a full sync (table empty) or incremental.
pub async fn sync_on_startup(pool: &Pool) -> Result<(), WikiSyncError> {
    if sync_disabled() {
        info!("DISABLE_WIKI_SYNC set; skipping wiki sync");
        return Ok(());
    }
    let client = pool.get().await.map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let row = client
        .query_one("SELECT COUNT(*)::BIGINT AS c FROM wiki_articles", &[])
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let count: i64 = row.get("c");
    drop(client);
    if count == 0 {
        info!("wiki_articles empty -> running full wiki sync");
        run_full_sync(pool).await
    } else {
        debug!("wiki_articles has {count} rows -> running incremental wiki sync");
        run_incremental_sync(pool).await
    }
}

/// Full crawl across all configured namespaces. Inserts/updates every page.
pub async fn run_full_sync(pool: &Pool) -> Result<(), WikiSyncError> {
    if sync_disabled() {
        return Ok(());
    }
    let http = http_client()?;
    let mut total = 0usize;
    for &ns in NAMESPACES {
        let pages = list_all_pages(&http, ns).await?;
        info!("wiki: namespace {ns} has {} pages", pages.len());
        for chunk in pages.chunks(50) {
            let ids: Vec<i64> = chunk.iter().map(|p| p.pageid).collect();
            match fetch_revisions(&http, &ids).await {
                Ok(pages) => {
                    for p in pages {
                        if let Err(e) = upsert_page(pool, &p).await {
                            warn!("wiki: upsert {} failed: {e}", p.title);
                        } else {
                            total += 1;
                        }
                    }
                }
                Err(e) => warn!("wiki: fetch_revisions failed for chunk: {e}"),
            }
        }
    }
    mark_full_sync_done(pool).await?;
    info!("wiki: full sync stored {total} pages");
    Ok(())
}

/// Pull only pages changed since the last sync timestamp.
pub async fn run_incremental_sync(pool: &Pool) -> Result<(), WikiSyncError> {
    if sync_disabled() {
        return Ok(());
    }
    let http = http_client()?;
    let since = last_sync_at(pool).await?;
    let now = Utc::now();
    let changes = list_recent_changes(&http, since).await?;
    if changes.is_empty() {
        info!("wiki: no recent changes since {since:?}");
        mark_incremental_sync_done(pool, now).await?;
        return Ok(());
    }
    let mut to_fetch: Vec<i64> = Vec::new();
    let mut to_delete: Vec<i64> = Vec::new();
    for ch in changes {
        let ns = ch.ns.unwrap_or(-1);
        if !NAMESPACES.contains(&ns) {
            continue;
        }
        match (ch.rc_type.as_deref(), ch.pageid) {
            (Some("delete"), Some(pid)) => to_delete.push(pid),
            (_, Some(pid)) if pid > 0 => to_fetch.push(pid),
            _ => {}
        }
    }
    to_fetch.sort_unstable();
    to_fetch.dedup();
    info!(
        "wiki: incremental sync — {} pages to fetch, {} deletes",
        to_fetch.len(),
        to_delete.len()
    );
    for chunk in to_fetch.chunks(50) {
        match fetch_revisions(&http, chunk).await {
            Ok(pages) => {
                for p in pages {
                    if let Err(e) = upsert_page(pool, &p).await {
                        warn!("wiki: upsert {} failed: {e}", p.title);
                    }
                }
            }
            Err(e) => warn!("wiki: fetch_revisions failed: {e}"),
        }
    }
    if !to_delete.is_empty() {
        let client = pool.get().await.map_err(|e| WikiSyncError::Db(e.to_string()))?;
        let _ = client
            .execute(
                "DELETE FROM wiki_articles WHERE page_id = ANY($1::int[])",
                &[&to_delete.iter().map(|x| *x as i32).collect::<Vec<_>>()],
            )
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    }
    mark_incremental_sync_done(pool, now).await?;
    Ok(())
}

async fn list_all_pages(http: &reqwest::Client, namespace: i32) -> Result<Vec<PageRef>, WikiSyncError> {
    let mut out: Vec<PageRef> = Vec::new();
    let mut apcontinue: Option<String> = None;
    loop {
        let mut params: Vec<(&str, String)> = vec![
            ("action", "query".into()),
            ("format", "json".into()),
            ("formatversion", "2".into()),
            ("list", "allpages".into()),
            ("apnamespace", namespace.to_string()),
            ("aplimit", "max".into()),
        ];
        if let Some(c) = &apcontinue {
            params.push(("apcontinue", c.clone()));
        }
        let resp: AllPagesEnvelope = http
            .get(API_URL)
            .query(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if let Some(q) = resp.query {
            out.extend(q.allpages);
        }
        match resp.cont.as_ref().and_then(|v| v.get("apcontinue")).and_then(|v| v.as_str()) {
            Some(s) => apcontinue = Some(s.to_string()),
            None => break,
        }
    }
    Ok(out)
}

async fn fetch_revisions(
    http: &reqwest::Client,
    page_ids: &[i64],
) -> Result<Vec<PageWithRev>, WikiSyncError> {
    if page_ids.is_empty() {
        return Ok(vec![]);
    }
    let ids = page_ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("|");
    let params: Vec<(&str, String)> = vec![
        ("action", "query".into()),
        ("format", "json".into()),
        ("formatversion", "2".into()),
        ("prop", "revisions".into()),
        ("rvprop", "ids|timestamp|content".into()),
        ("rvslots", "main".into()),
        ("pageids", ids),
    ];
    let resp: RevisionsEnvelope = http
        .get(API_URL)
        .query(&params)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let pages_value = resp.query.map(|q| q.pages).unwrap_or(serde_json::Value::Null);
    let mut out: Vec<PageWithRev> = Vec::new();
    // formatversion=2 returns an array; older formats returned an object map.
    if let Some(arr) = pages_value.as_array() {
        for v in arr {
            if let Ok(p) = serde_json::from_value::<PageWithRev>(v.clone()) {
                out.push(p);
            }
        }
    } else if let Some(obj) = pages_value.as_object() {
        for (_, v) in obj {
            if let Ok(p) = serde_json::from_value::<PageWithRev>(v.clone()) {
                out.push(p);
            }
        }
    }
    Ok(out)
}

async fn list_recent_changes(
    http: &reqwest::Client,
    since: Option<DateTime<Utc>>,
) -> Result<Vec<RecentChange>, WikiSyncError> {
    let nsfilter = NAMESPACES
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("|");
    let mut out: Vec<RecentChange> = Vec::new();
    let mut rccontinue: Option<String> = None;
    let rcend = since.map(|d| d.to_rfc3339());
    loop {
        let mut params: Vec<(&str, String)> = vec![
            ("action", "query".into()),
            ("format", "json".into()),
            ("formatversion", "2".into()),
            ("list", "recentchanges".into()),
            ("rcnamespace", nsfilter.clone()),
            ("rcprop", "ids|title|timestamp".into()),
            ("rctype", "edit|new|delete|move".into()),
            ("rclimit", "max".into()),
            ("rcdir", "older".into()),
        ];
        if let Some(end) = &rcend {
            params.push(("rcend", end.clone()));
        }
        if let Some(c) = &rccontinue {
            params.push(("rccontinue", c.clone()));
        }
        let resp: RecentChangesEnvelope = http
            .get(API_URL)
            .query(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if let Some(q) = resp.query {
            out.extend(q.recentchanges);
        }
        match resp.cont.as_ref().and_then(|v| v.get("rccontinue")).and_then(|v| v.as_str()) {
            Some(s) => rccontinue = Some(s.to_string()),
            None => break,
        }
    }
    Ok(out)
}

async fn upsert_page(pool: &Pool, p: &PageWithRev) -> Result<(), WikiSyncError> {
    if p.missing.unwrap_or(false) {
        let client = pool.get().await.map_err(|e| WikiSyncError::Db(e.to_string()))?;
        let _ = client
            .execute(
                "DELETE FROM wiki_articles WHERE page_id = $1",
                &[&(p.pageid as i32)],
            )
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
        return Ok(());
    }
    let rev = match p.revisions.first() {
        Some(r) => r,
        None => return Ok(()), // no revision data, skip
    };
    let wikitext = rev
        .slots
        .as_ref()
        .and_then(|s| s.main.as_ref())
        .and_then(|m| m.content.clone().or_else(|| m.star.clone()))
        .unwrap_or_default();
    let (md, plain) = wikitext_to_markdown(&wikitext);
    let is_redirect = wikitext.trim_start().to_lowercase().starts_with("#redirect");
    let revid = rev.revid;
    let last_edited: Option<DateTime<Utc>> = rev
        .timestamp
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc)));
    let client = pool.get().await.map_err(|e| WikiSyncError::Db(e.to_string()))?;
    client
        .execute(
            "INSERT INTO wiki_articles
                (page_id, namespace, title, revision_id, wikitext, markdown, plain_text, is_redirect, last_edited, fetched_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
             ON CONFLICT (page_id) DO UPDATE SET
                namespace   = EXCLUDED.namespace,
                title       = EXCLUDED.title,
                revision_id = EXCLUDED.revision_id,
                wikitext    = EXCLUDED.wikitext,
                markdown    = EXCLUDED.markdown,
                plain_text  = EXCLUDED.plain_text,
                is_redirect = EXCLUDED.is_redirect,
                last_edited = EXCLUDED.last_edited,
                fetched_at  = now()",
            &[
                &(p.pageid as i32),
                &p.ns,
                &p.title,
                &revid,
                &wikitext,
                &md,
                &plain,
                &is_redirect,
                &last_edited,
            ],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    Ok(())
}

async fn last_sync_at(pool: &Pool) -> Result<Option<DateTime<Utc>>, WikiSyncError> {
    let client = pool.get().await.map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let row = client
        .query_one(
            "SELECT GREATEST(
                COALESCE(last_incremental_sync, '-infinity'::timestamptz),
                COALESCE(last_full_sync, '-infinity'::timestamptz)
             ) AS ts FROM wiki_sync_state WHERE id = 1",
            &[],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let ts: Option<DateTime<Utc>> = row.try_get("ts").ok();
    // GREATEST of two -infinity becomes -infinity; treat that as None.
    Ok(ts.filter(|d| d.timestamp() > 0))
}

async fn mark_full_sync_done(pool: &Pool) -> Result<(), WikiSyncError> {
    let client = pool.get().await.map_err(|e| WikiSyncError::Db(e.to_string()))?;
    client
        .execute(
            "UPDATE wiki_sync_state
             SET last_full_sync = now(), last_incremental_sync = now()
             WHERE id = 1",
            &[],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    Ok(())
}

async fn mark_incremental_sync_done(pool: &Pool, ts: DateTime<Utc>) -> Result<(), WikiSyncError> {
    let client = pool.get().await.map_err(|e| WikiSyncError::Db(e.to_string()))?;
    client
        .execute(
            "UPDATE wiki_sync_state SET last_incremental_sync = $1 WHERE id = 1",
            &[&ts],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_allpages_response() {
        let json = r#"{
            "query": { "allpages": [
                {"pageid": 1, "ns": 0, "title": "Lojban"},
                {"pageid": 2, "ns": 0, "title": "Gismu"}
            ]},
            "continue": {"apcontinue": "Z", "continue": "-||"}
        }"#;
        let env: AllPagesEnvelope = serde_json::from_str(json).unwrap();
        let pages = env.query.unwrap().allpages;
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].title, "Lojban");
    }

    #[test]
    fn parse_revisions_v2_response() {
        let json = r#"{
            "query": { "pages": [
                {
                    "pageid": 5, "ns": 0, "title": "X",
                    "revisions": [
                        {"revid": 99, "timestamp": "2024-01-02T03:04:05Z",
                         "slots": {"main": {"content": "hello"}}}
                    ]
                }
            ]}
        }"#;
        let env: RevisionsEnvelope = serde_json::from_str(json).unwrap();
        let pages_val = env.query.unwrap().pages;
        let arr = pages_val.as_array().unwrap();
        let p: PageWithRev = serde_json::from_value(arr[0].clone()).unwrap();
        assert_eq!(p.pageid, 5);
        assert_eq!(p.revisions[0].slots.as_ref().unwrap().main.as_ref().unwrap().content.as_deref(), Some("hello"));
    }
}
