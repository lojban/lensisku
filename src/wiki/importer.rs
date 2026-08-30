//! Pulls articles from `mw.lojban.org` via the MediaWiki API and stores them in
//! `wiki_articles`. Run from `src/background/service.rs` on startup and hourly.
//!
//! Latest revision is mirrored into `wiki_articles`. Each incremental upsert
//! also writes new MediaWiki revisions into `definition_versions`. A batch
//! `import_revision_histories` pass backfills remaining history.

use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use deadpool_postgres::Pool;
use log::{info, warn};
use serde::Deserialize;

use super::markdown::wikitext_to_markdown;

const API_URL: &str = "https://mw.lojban.org/api.php";
const USER_AGENT: &str = "lensisku-wiki-importer/0.1 (https://lojban.org)";
const NAMESPACES: &[i32] = &[0, 2];

/// Returned from `?action=query&list=allpages`.
#[derive(Debug, Deserialize)]
struct AllPagesEnvelope {
    #[serde(default)]
    error: Option<MwApiError>,
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
    error: Option<MwApiError>,
    #[serde(default)]
    query: Option<RevisionsQuery>,
    #[serde(rename = "continue", default)]
    cont: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RevisionsQuery {
    #[serde(default)]
    pages: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
struct RevEntry {
    #[serde(default)]
    revid: Option<i64>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    comment: Option<String>,
    #[serde(default)]
    slots: Option<RevSlots>,
}

#[derive(Debug, Deserialize, Clone)]
struct RevSlots {
    #[serde(default)]
    main: Option<RevSlotMain>,
}

#[derive(Debug, Deserialize, Clone)]
struct RevSlotMain {
    #[serde(rename = "*", default)]
    star: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MwApiError {
    code: Option<String>,
    info: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RecentChangesEnvelope {
    #[serde(default)]
    error: Option<MwApiError>,
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
    logtype: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    timestamp: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum WikiSyncError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("mediawiki api error: {0}")]
    Api(String),
    #[error("db error: {0}")]
    Db(String),
}

fn mw_api_err(err: Option<MwApiError>) -> Result<(), WikiSyncError> {
    let Some(e) = err else {
        return Ok(());
    };
    Err(WikiSyncError::Api(format!(
        "{}: {}",
        e.code.as_deref().unwrap_or("unknown"),
        e.info.as_deref().unwrap_or("no info")
    )))
}

/// MediaWiki timestamp params reject 9-digit fractional seconds from `DateTime::to_rfc3339()`.
fn mediawiki_timestamp(dt: DateTime<Utc>) -> String {
    dt.format("%Y%m%d%H%M%S").to_string()
}

fn http_client() -> Result<reqwest::Client, WikiSyncError> {
    Ok(reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(60))
        .build()?)
}

fn sync_disabled() -> bool {
    matches!(
        std::env::var("DISABLE_WIKI_SYNC").ok().as_deref(),
        Some("1") | Some("true")
    )
}

/// Startup re-render of all stored wiki markdown is expensive and can freeze the app on
/// boot in environments with many articles. It is opt-in via `WIKI_RERENDER_ON_STARTUP`.
fn rerender_enabled_on_startup() -> bool {
    matches!(
        std::env::var("WIKI_RERENDER_ON_STARTUP").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

/// Re-render `markdown` and `plain_text` for every row from its stored `wikitext`,
/// without hitting the network.  Called after converter improvements so existing
/// articles pick up the new rendering on the next startup.
pub async fn rerender_all_markdown(pool: &Pool) -> Result<(), WikiSyncError> {
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let rows = client
        .query("SELECT id, wikitext FROM wiki_articles", &[])
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let total = rows.len();
    info!("wiki: re-rendering markdown for {total} articles");
    let mut updated = 0usize;
    for row in &rows {
        let id: i32 = row.get("id");
        let wikitext: &str = row.get("wikitext");
        let (md, plain) = wikitext_to_markdown(wikitext);
        let c = pool
            .get()
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
        c.execute(
            "UPDATE wiki_articles SET markdown = $1, plain_text = $2 WHERE id = $3",
            &[&md, &plain, &id],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
        updated += 1;
    }
    info!("wiki: re-render complete ({updated}/{total})");
    Ok(())
}

/// Decide on startup whether to do a full sync (table empty) or incremental.
pub async fn sync_on_startup(pool: &Pool) -> Result<(), WikiSyncError> {
    if sync_disabled() {
        info!("DISABLE_WIKI_SYNC set; skipping wiki sync");
        return Ok(());
    }
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let row = client
        .query_one("SELECT COUNT(*)::BIGINT AS c FROM wiki_articles", &[])
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let count: i64 = row.get("c");
    drop(client);
    if count == 0 {
        info!("wiki_articles empty -> running full wiki sync");
        run_full_sync(pool).await?;
    } else {
        if rerender_enabled_on_startup() {
            info!("wiki_articles has {count} rows -> re-rendering + incremental sync on startup");
            rerender_all_markdown(pool).await?;
        } else {
            info!("wiki_articles has {count} rows -> skipping startup re-render (set WIKI_RERENDER_ON_STARTUP=1 to enable)");
        }
        run_incremental_sync(pool).await?;
    }
    import_revision_histories(pool).await
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
    let since = content_watermark(pool).await?;
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
        match (ch.rc_type.as_deref(), ch.logtype.as_deref(), ch.pageid) {
            (Some("log"), Some("delete"), Some(pid)) if pid > 0 => to_delete.push(pid),
            (Some("log"), _, _) => {}
            (_, _, Some(pid)) if pid > 0 => to_fetch.push(pid),
            _ => {}
        }
    }
    to_fetch.sort_unstable();
    to_fetch.dedup();
    info!(
        "wiki: incremental sync — {} pages to fetch, {} deletes (since {since:?})",
        to_fetch.len(),
        to_delete.len()
    );
    let mut fetch_errors = 0usize;
    let history_user = match officialdata_user_id(pool).await {
        Ok(id) => Some(id),
        Err(e) => {
            warn!("wiki: cannot sync definition_versions ({e})");
            None
        }
    };
    for chunk in to_fetch.chunks(50) {
        match fetch_revisions(&http, chunk).await {
            Ok(pages) => {
                for p in pages {
                    if let Err(e) = upsert_page(pool, &p).await {
                        warn!("wiki: upsert {} failed: {e}", p.title);
                        continue;
                    }
                    if let Some(uid) = history_user {
                        if let Err(e) = sync_page_history_after_upsert(pool, &http, &p, uid).await
                        {
                            warn!(
                                "wiki: definition_versions sync for {} failed: {e}",
                                p.title
                            );
                        }
                    }
                }
            }
            Err(e) => {
                fetch_errors += 1;
                warn!("wiki: fetch_revisions failed: {e}");
            }
        }
    }
    if !to_delete.is_empty() {
        let client = pool
            .get()
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
        let _ = client
            .execute(
                "DELETE FROM wiki_articles WHERE page_id = ANY($1::int[])",
                &[&to_delete.iter().map(|x| *x as i32).collect::<Vec<_>>()],
            )
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    }
    if fetch_errors > 0 {
        return Err(WikiSyncError::Api(format!(
            "{fetch_errors} revision chunk(s) failed; not advancing wiki sync watermark"
        )));
    }
    mark_incremental_sync_done(pool, now).await?;
    Ok(())
}

async fn list_all_pages(
    http: &reqwest::Client,
    namespace: i32,
) -> Result<Vec<PageRef>, WikiSyncError> {
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
        mw_api_err(resp.error)?;
        if let Some(q) = resp.query {
            out.extend(q.allpages);
        }
        match resp
            .cont
            .as_ref()
            .and_then(|v| v.get("apcontinue"))
            .and_then(|v| v.as_str())
        {
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
    let ids = page_ids
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("|");
    let params: Vec<(&str, String)> = vec![
        ("action", "query".into()),
        ("format", "json".into()),
        ("formatversion", "2".into()),
        ("prop", "revisions".into()),
            ("rvprop", "ids|timestamp|user|comment|content".into()),
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
    mw_api_err(resp.error)?;
    let pages_value = resp
        .query
        .map(|q| q.pages)
        .unwrap_or(serde_json::Value::Null);
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
    let rcend = since;
    loop {
        let mut params: Vec<(&str, String)> = vec![
            ("action", "query".into()),
            ("format", "json".into()),
            ("formatversion", "2".into()),
            ("list", "recentchanges".into()),
            ("rcnamespace", nsfilter.clone()),
            ("rcprop", "ids|title|timestamp|loginfo".into()),
            ("rctype", "edit|new|log".into()),
            ("rclimit", "max".into()),
            ("rcdir", "older".into()),
        ];
        if let Some(end) = &rcend {
            params.push(("rcend", mediawiki_timestamp(*end)));
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
        mw_api_err(resp.error)?;
        if let Some(q) = resp.query {
            out.extend(q.recentchanges);
        }
        match resp
            .cont
            .as_ref()
            .and_then(|v| v.get("rccontinue"))
            .and_then(|v| v.as_str())
        {
            Some(s) => rccontinue = Some(s.to_string()),
            None => break,
        }
    }
    Ok(out)
}

async fn upsert_page(pool: &Pool, p: &PageWithRev) -> Result<(), WikiSyncError> {
    if p.missing.unwrap_or(false) {
        let client = pool
            .get()
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
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
    let is_redirect = wikitext
        .trim_start()
        .to_lowercase()
        .starts_with("#redirect");
    let revid = rev.revid;
    let last_edited: Option<DateTime<Utc>> = rev.timestamp.as_deref().and_then(|s| {
        DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    });
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
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

/// Pull missing MediaWiki revisions for a page we just mirrored into `definition_versions`.
async fn sync_page_history_after_upsert(
    pool: &Pool,
    http: &reqwest::Client,
    p: &PageWithRev,
    fallback_user_id: i32,
) -> Result<(), WikiSyncError> {
    let latest_revid = p.revisions.first().and_then(|r| r.revid).unwrap_or(0);
    if latest_revid <= 0 {
        return Ok(());
    }
    let imported_until = page_history_imported_until(pool, p.pageid as i32).await?;
    import_one_page_history(
        pool,
        http,
        p.pageid as i32,
        &p.title,
        latest_revid,
        imported_until,
        fallback_user_id,
        p.revisions.first().cloned(),
    )
    .await
}

async fn page_history_imported_until(pool: &Pool, page_id: i32) -> Result<i64, WikiSyncError> {
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let row = client
        .query_opt(
            "SELECT COALESCE(history_imported_until, 0) AS imported
             FROM wiki_articles WHERE page_id = $1",
            &[&page_id],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    Ok(row.map(|r| r.get("imported")).unwrap_or(0))
}

/// Import every MediaWiki revision of mirrored pages into `definition_versions`.
///
/// Imported pages use the real MediaWiki title. If that title is already a
/// dictionary valsi (or another wiki page) at `source_langid = 1`, a different
/// `source_langid` is used so the visible word can still be the real title.
pub async fn import_revision_histories(pool: &Pool) -> Result<(), WikiSyncError> {
    if sync_disabled() {
        return Ok(());
    }
    let importer_user_id = officialdata_user_id(pool).await?;
    let http = http_client()?;
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let rows = client
        .query(
            "SELECT page_id, title, revision_id, COALESCE(history_imported_until, 0) AS imported
             FROM wiki_articles
             WHERE revision_id IS NOT NULL
               AND COALESCE(history_imported_until, 0) IS DISTINCT FROM revision_id
             ORDER BY last_edited DESC NULLS LAST",
            &[],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    drop(client);
    let pending = rows.len();
    if pending == 0 {
        info!("wiki: revision history already caught up");
        return Ok(());
    }
    info!("wiki: importing MediaWiki history for {pending} pages");
    let mut done = 0usize;
    for row in rows {
        let page_id: i32 = row.get("page_id");
        let title: String = row.get("title");
        let latest_revid: i64 = row.get("revision_id");
        let imported_until: i64 = row.get("imported");
        match import_one_page_history(
            pool,
            &http,
            page_id,
            &title,
            latest_revid,
            imported_until,
            importer_user_id,
            None,
        )
        .await
        {
            Ok(()) => {
                done += 1;
                if done.is_multiple_of(25) {
                    info!("wiki: history import {done}/{pending}");
                }
            }
            Err(e) => warn!("wiki: history import for {title} (page {page_id}) failed: {e}"),
        }
    }
    info!("wiki: history import finished ({done}/{pending} pages)");
    Ok(())
}

async fn officialdata_user_id(pool: &Pool) -> Result<i32, WikiSyncError> {
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let row = client
        .query_opt(
            "SELECT userid FROM users WHERE username = 'officialdata' LIMIT 1",
            &[],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    row.map(|r| r.get("userid")).ok_or_else(|| {
        WikiSyncError::Db("users.username='officialdata' is required to import wiki history".into())
    })
}

#[allow(clippy::too_many_arguments)]
async fn import_one_page_history(
    pool: &Pool,
    http: &reqwest::Client,
    page_id: i32,
    title: &str,
    _latest_revid: i64,
    imported_until: i64,
    fallback_user_id: i32,
    known_latest: Option<RevEntry>,
) -> Result<(), WikiSyncError> {
    let mut revisions = fetch_page_revision_history(http, page_id as i64, imported_until).await?;
    if let Some(rev) = known_latest {
        let extra_id = rev.revid.unwrap_or(0);
        if extra_id > imported_until && !revisions.iter().any(|r| r.revid == Some(extra_id)) {
            revisions.push(rev);
        }
    }
    if revisions.is_empty() {
        // Do not advance the watermark: the new edit may not be in the RC/history
        // API yet, and we must retry on the next sync.
        return Ok(());
    }
    let (definition_id, valsi_id, lang_id) =
        ensure_mw_wiki_definition(pool, page_id, title, fallback_user_id).await?;
    let mut newest: Option<&RevEntry> = None;
    let mut imported_max = imported_until;
    for rev in &revisions {
        let Some(revid) = rev.revid else {
            continue;
        };
        if revid <= imported_until {
            continue;
        }
        let user_id = resolve_mw_user_id(pool, rev.user.as_deref(), fallback_user_id).await?;
        let wikitext = rev
            .slots
            .as_ref()
            .and_then(|s| s.main.as_ref())
            .and_then(|m| m.content.clone().or_else(|| m.star.clone()))
            .unwrap_or_default();
        let (md, _plain) = wikitext_to_markdown(&wikitext);
        let created_at: DateTime<Utc> = rev
            .timestamp
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let comment = rev
            .comment
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("mw.lojban.org r{revid}"));
        insert_mw_version(
            pool,
            definition_id,
            lang_id,
            valsi_id,
            &md,
            user_id,
            created_at,
            &comment,
            revid,
        )
        .await?;
        imported_max = imported_max.max(revid);
        newest = Some(rev);
    }
    if let Some(rev) = newest {
        let wikitext = rev
            .slots
            .as_ref()
            .and_then(|s| s.main.as_ref())
            .and_then(|m| m.content.clone().or_else(|| m.star.clone()))
            .unwrap_or_default();
        let (md, plain) = wikitext_to_markdown(&wikitext);
        let last_edited: Option<DateTime<Utc>> = rev.timestamp.as_deref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|d| d.with_timezone(&Utc))
        });
        let unix = last_edited.map(|d| d.timestamp() as i32).unwrap_or(0);
        let client = pool
            .get()
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
        client
            .execute(
                "UPDATE definitions
                 SET definition = $1, time = $2, embedding = NULL
                 WHERE definitionid = $3",
                &[&md, &unix, &definition_id],
            )
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
        client
            .execute(
                "UPDATE wiki_articles
                 SET markdown = $1, plain_text = $2, wikitext = $3,
                     last_edited = COALESCE($4, last_edited)
                 WHERE page_id = $5",
                &[&md, &plain, &wikitext, &last_edited, &page_id],
            )
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    }
    // Only advance to revids we actually wrote. Never jump to the mirrored
    // latest revid if that revision was missing from the history API.
    if imported_max > imported_until {
        mark_history_imported(pool, page_id, imported_max).await?;
    }
    Ok(())
}

async fn mark_history_imported(
    pool: &Pool,
    page_id: i32,
    until: i64,
) -> Result<(), WikiSyncError> {
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    client
        .execute(
            "UPDATE wiki_articles SET history_imported_until = $1 WHERE page_id = $2",
            &[&until, &page_id],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    Ok(())
}

async fn resolve_mw_user_id(
    pool: &Pool,
    mw_user: Option<&str>,
    fallback: i32,
) -> Result<i32, WikiSyncError> {
    let Some(local) = mw_user.and_then(super::mw_import_username) else {
        return Ok(fallback);
    };
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    if let Some(row) = client
        .query_opt(
            "SELECT userid FROM users WHERE username = $1 LIMIT 1",
            &[&local],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?
    {
        return Ok(row.get("userid"));
    }
    // username is varchar, email is text — Postgres cannot type a shared `$1`.
    let row = client
        .query_one(
            "INSERT INTO users (
                username, email, password, created_at, role, email_confirmed, votesize
             ) VALUES ($1, $2, 'DISABLED', NOW(), 'blocked', false, 0)
             ON CONFLICT (username) DO UPDATE SET username = users.username
             RETURNING userid",
            &[&local, &local],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    Ok(row.get("userid"))
}

async fn free_source_langid_for_wiki_word(
    client: &deadpool_postgres::Object,
    word: &str,
) -> Result<i32, WikiSyncError> {
    let row = client
        .query_opt(
            "SELECT l.langid
             FROM languages l
             WHERE NOT EXISTS (
                 SELECT 1 FROM valsi v
                 WHERE v.word = $1
                   AND v.source_langid = l.langid
             )
             ORDER BY CASE WHEN l.langid = 1 THEN 0 ELSE 1 END, l.langid
             LIMIT 1",
            &[&word],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    row.map(|r| r.get("langid")).ok_or_else(|| {
        WikiSyncError::Db(format!(
            "no free source_langid for wiki title '{word}'"
        ))
    })
}

async fn ensure_mw_wiki_definition(
    pool: &Pool,
    page_id: i32,
    title: &str,
    user_id: i32,
) -> Result<(i32, i32, i32), WikiSyncError> {
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let page_id_str = page_id.to_string();
    if let Some(row) = client
        .query_opt(
            "SELECT d.definitionid, d.valsiid, d.langid
             FROM definitions d
             JOIN valsi v ON v.valsiid = d.valsiid
             WHERE v.typeid = 16
               AND d.metadata->>'mw_page_id' = $1
             LIMIT 1",
            &[&page_id_str],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?
    {
        let definition_id: i32 = row.get(0);
        let valsi_id: i32 = row.get(1);
        let lang_id: i32 = row.get(2);
        return Ok((definition_id, valsi_id, lang_id));
    }

    let lang_id: i32 = 1;
    let typeid: i16 = 16;
    let now = Utc::now().timestamp() as i32;
    let metadata = serde_json::json!({
        "source": "mw.lojban.org",
        "mw_page_id": page_id,
        "mw_title": title,
    });
    let source_langid = free_source_langid_for_wiki_word(&client, title).await?;

    let valsi_id: i32 = client
        .query_one(
            "INSERT INTO valsi (word, typeid, userid, time, source_langid)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING valsiid",
            &[&title, &typeid, &user_id, &now, &source_langid],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?
        .get(0);

    let definition_id: i32 = if let Some(row) = client
        .query_opt(
            "SELECT definitionid FROM definitions WHERE valsiid = $1 ORDER BY definitionid LIMIT 1",
            &[&valsi_id],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?
    {
        let id: i32 = row.get(0);
        client
            .execute(
                "UPDATE definitions SET metadata = COALESCE(metadata, '{}'::jsonb) || $2::jsonb
                 WHERE definitionid = $1",
                &[&id, &metadata],
            )
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
        id
    } else {
        let id = client
            .query_one("SELECT nextval('definitions_definitionid_seq')", &[])
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?
            .get::<_, i64>(0) as i32;
        client
            .execute(
                "INSERT INTO definitions
                    (definitionid, langid, valsiid, definitionnum, definition, userid, time, metadata)
                 VALUES ($1, $2, $3, 1, '', $4, $5, $6)",
                &[&id, &lang_id, &valsi_id, &user_id, &now, &metadata],
            )
            .await
            .map_err(|e| WikiSyncError::Db(e.to_string()))?;
        id
    };
    Ok((definition_id, valsi_id, lang_id))
}

#[allow(clippy::too_many_arguments)]
async fn insert_mw_version(
    pool: &Pool,
    definition_id: i32,
    lang_id: i32,
    valsi_id: i32,
    markdown: &str,
    user_id: i32,
    created_at: DateTime<Utc>,
    message: &str,
    mw_revid: i64,
) -> Result<(), WikiSyncError> {
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    client
        .execute(
            "INSERT INTO definition_versions (
                created_at, definition_id, langid, valsiid, definition,
                notes, etymology, selmaho, jargon, rafsi,
                gloss_keywords, place_keywords, user_id, message, mw_revid
             )
             VALUES ($1, $2, $3, $4, $5, NULL, NULL, NULL, NULL, NULL,
                     '[]'::jsonb, '[]'::jsonb, $6, $7, $8)
             ON CONFLICT (mw_revid) WHERE mw_revid IS NOT NULL DO NOTHING",
            &[
                &created_at,
                &definition_id,
                &lang_id,
                &valsi_id,
                &markdown,
                &user_id,
                &message,
                &mw_revid,
            ],
        )
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    Ok(())
}

async fn fetch_page_revision_history(
    http: &reqwest::Client,
    page_id: i64,
    imported_until: i64,
) -> Result<Vec<RevEntry>, WikiSyncError> {
    let mut out: Vec<RevEntry> = Vec::new();
    let mut rvcontinue: Option<String> = None;
    loop {
        let mut params: Vec<(&str, String)> = vec![
            ("action", "query".into()),
            ("format", "json".into()),
            ("formatversion", "2".into()),
            ("prop", "revisions".into()),
            ("rvprop", "ids|timestamp|user|comment|content".into()),
            ("rvslots", "main".into()),
            ("rvlimit", "50".into()),
            ("pageids", page_id.to_string()),
        ];
        if imported_until > 0 {
            params.push(("rvdir", "newer".into()));
            params.push(("rvstartid", imported_until.to_string()));
        } else {
            params.push(("rvdir", "older".into()));
        }
        if let Some(c) = &rvcontinue {
            params.push(("rvcontinue", c.clone()));
        }
        let resp: RevisionsEnvelope = http
            .get(API_URL)
            .query(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        mw_api_err(resp.error)?;
        let pages_value = resp
            .query
            .as_ref()
            .map(|q| q.pages.clone())
            .unwrap_or(serde_json::Value::Null);
        let page_obj = if let Some(arr) = pages_value.as_array() {
            arr.first().cloned()
        } else if let Some(obj) = pages_value.as_object() {
            obj.values().next().cloned()
        } else {
            None
        };
        if let Some(p) = page_obj {
            if let Ok(parsed) = serde_json::from_value::<PageWithRev>(p) {
                out.extend(parsed.revisions);
            }
        }
        match resp
            .cont
            .as_ref()
            .and_then(|v| v.get("rvcontinue"))
            .and_then(|v| v.as_str())
        {
            Some(s) => rvcontinue = Some(s.to_string()),
            None => break,
        }
    }
    Ok(out)
}

/// Lower bound for incremental RC queries.
///
/// Use the newest mirrored `last_edited`, not `wiki_sync_state`. That table was
/// advanced even when MediaWiki returned an API error body (HTTP 200, empty
/// `query`), which skipped real edits forever.
async fn content_watermark(pool: &Pool) -> Result<Option<DateTime<Utc>>, WikiSyncError> {
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let row = client
        .query_one("SELECT MAX(last_edited) AS ts FROM wiki_articles", &[])
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
    let ts: Option<DateTime<Utc>> = row.get("ts");
    let ts = ts.filter(|d| d.timestamp() > 0);
    Ok(ts.map(|d| d - TimeDelta::hours(2)))
}

async fn mark_full_sync_done(pool: &Pool) -> Result<(), WikiSyncError> {
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
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
    let client = pool
        .get()
        .await
        .map_err(|e| WikiSyncError::Db(e.to_string()))?;
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
#[allow(clippy::unwrap_used)]
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
        assert_eq!(
            p.revisions[0]
                .slots
                .as_ref()
                .unwrap()
                .main
                .as_ref()
                .unwrap()
                .content
                .as_deref(),
            Some("hello")
        );
    }

    #[test]
    fn mediawiki_timestamp_has_no_fractional_seconds() {
        let dt = DateTime::parse_from_rfc3339("2026-08-23T01:15:52.123456789Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(mediawiki_timestamp(dt), "20260823011552");
        assert!(
            dt.to_rfc3339().contains('.'),
            "chrono RFC3339 keeps nanos, which MediaWiki rejects"
        );
    }

    #[test]
    fn parse_recentchanges_api_error() {
        let json = r#"{
            "error": {
                "code": "badtimestamp",
                "info": "Invalid value for timestamp parameter \"rcend\"."
            }
        }"#;
        let env: RecentChangesEnvelope = serde_json::from_str(json).unwrap();
        let err = mw_api_err(env.error).unwrap_err().to_string();
        assert!(err.contains("badtimestamp"));
    }
}
