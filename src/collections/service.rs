use super::dto::SkippedItemInfo;
use super::dto::*;
use crate::jbovlaste::service::get_valsi_sound_urls_from_db;
use crate::utils::remove_html_tags;
use crate::{
    auth_utils::verify_collection_ownership, export::models::CollectionExportItem,
    flashcards::models::FlashcardDirection, middleware::cache::RedisCache,
    middleware::cache::FILTER_PICKER_POPULAR_CACHE_KEY, middleware::image::ImageProcessor,
    users::dto::ProfileImageRequest, utils::validate_item_audio, utils::validate_item_image,
    utils::MAX_ITEM_IMAGE_BYTES, AppError, AppResult,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use super::images::*;

use chrono::{DateTime, Utc};
use deadpool_postgres::{GenericClient, Pool, Transaction};
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use std::path::Path;
use tokio_postgres::types::ToSql;
use zip::ZipArchive;

use crate::utils::{detect_image_mime_from_content, validate_item_image_bytes};

pub async fn create_collection(
    pool: &Pool,
    redis: &RedisCache,
    user_id: i32,
    req: &CreateCollectionRequest,
) -> AppResult<CollectionResponse> {
    let sanitized_name = sanitize_html(&req.name);
    let sanitized_description = req.description.as_ref().map(|d| sanitize_html(d));

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let row = transaction
        .query_one(
            "INSERT INTO collections (user_id, name, description, is_public)
             VALUES ($1, $2, $3, $4)
             RETURNING collection_id, created_at, updated_at",
            &[
                &user_id,
                &sanitized_name,
                &sanitized_description,
                &req.is_public.unwrap_or(true),
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let username = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = CollectionResponse {
        collection_id: row.get("collection_id"),
        name: sanitized_name,
        description: sanitized_description,
        is_public: req.is_public.unwrap_or(true),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        item_count: 0,
        has_flashcards: false,
        has_cover_image: false,
        has_collection_image: false,
        comment_count: 0,
        monthly_drill_users: 0,
        owner: CollectionOwner { user_id, username },
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;
    Ok(response)
}

pub fn sanitize_html(html: &str) -> String {
    remove_html_tags(html)
}

pub async fn list_collections(
    pool: &Pool,
    user_id: i32,
    query: &ListCollectionsQuery,
) -> AppResult<CollectionListResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(user_id)];
    let where_clause = build_collection_where_clause(
        "c.user_id = $1",
        &mut params,
        query.search.as_deref(),
        query.has_flashcards_only.unwrap_or(false),
        query.has_levels_only.unwrap_or(false),
    );

    query_collections(
        &client,
        &where_clause,
        &mut params,
        query.sort.as_deref(),
        query.search.as_deref(),
        pagination_bounds(query.page, query.per_page),
        Some(user_id),
    )
    .await
}

/// Valsi + definition text + source language + target language for content matching.
struct ItemContentIdentity {
    valsi: Option<String>,
    definition: Option<String>,
    source_langid: Option<i32>,
    langid: Option<i32>,
}

impl ItemContentIdentity {
    fn is_empty(&self) -> bool {
        self.valsi.as_deref().unwrap_or("").is_empty()
            && self.definition.as_deref().unwrap_or("").is_empty()
    }
}

async fn load_item_content_identity(
    client: &impl GenericClient,
    item_id: i32,
) -> AppResult<Option<ItemContentIdentity>> {
    let src = client
        .query_opt(
            "SELECT
                COALESCE(d.cached_valsiword, ci.free_content_front) AS valsi,
                COALESCE(d.definition, ci.free_content_back) AS definition,
                d.cached_source_langid AS source_langid,
                COALESCE(d.langid, ci.langid) AS langid
             FROM collection_items ci
             LEFT JOIN definitions d ON d.definitionid = ci.definition_id
             WHERE ci.item_id = $1",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(src.map(|row| ItemContentIdentity {
        valsi: row.get("valsi"),
        definition: row.get("definition"),
        source_langid: row.get("source_langid"),
        langid: row.get("langid"),
    }))
}

/// Predicate that matches dictionary-backed items on denormalized definition columns and
/// custom-text items on `free_content_*` (avoids wrapping both sides in COALESCE, which
/// blocks index use on `definitions`).
fn append_content_identity_match(
    sql: &mut String,
    params: &mut Vec<Box<dyn ToSql + Sync>>,
    param_count: &mut usize,
    ident: &ItemContentIdentity,
) {
    let valsi_idx = *param_count;
    let def_idx = *param_count + 1;
    let src_idx = *param_count + 2;
    let lang_idx = *param_count + 3;
    sql.push_str(&format!(
        " AND (
            (
                ci.definition_id IS NOT NULL
                AND d.cached_valsiword IS NOT DISTINCT FROM ${valsi_idx}
                AND d.definition IS NOT DISTINCT FROM ${def_idx}
                AND d.langid IS NOT DISTINCT FROM ${lang_idx}
                AND d.cached_source_langid IS NOT DISTINCT FROM ${src_idx}
            )
            OR (
                ci.definition_id IS NULL
                AND ci.free_content_front IS NOT DISTINCT FROM ${valsi_idx}
                AND ci.free_content_back IS NOT DISTINCT FROM ${def_idx}
                AND ci.langid IS NOT DISTINCT FROM ${lang_idx}
                AND ${src_idx}::int4 IS NULL
            )
         )"
    ));
    params.push(Box::new(ident.valsi.clone()));
    params.push(Box::new(ident.definition.clone()));
    params.push(Box::new(ident.source_langid));
    params.push(Box::new(ident.langid));
    *param_count += 4;
}

/// Collections owned by `user_id` that already contain this definition or a full-content
/// match of this collection item (valsi, definition text, source language, target language).
pub async fn collections_containing_item(
    pool: &Pool,
    user_id: i32,
    req: &CollectionMembershipRequest,
) -> AppResult<CollectionMembershipResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if let Some(item_id) = req.item_id {
        if item_id <= 0 {
            return Err(AppError::BadRequest("item_id must be positive".to_string()));
        }
        let src = client
            .query_opt(
                "SELECT
                    COALESCE(d.cached_valsiword, ci.free_content_front) AS valsi,
                    COALESCE(d.definition, ci.free_content_back) AS definition,
                    d.cached_source_langid AS source_langid,
                    COALESCE(d.langid, ci.langid) AS langid
                 FROM collection_items ci
                 JOIN collections owner_c ON owner_c.collection_id = ci.collection_id
                 LEFT JOIN definitions d ON d.definitionid = ci.definition_id
                 WHERE ci.item_id = $1
                   AND (owner_c.is_public = true OR owner_c.user_id = $2)",
                &[&item_id, &user_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        let Some(row) = src else {
            return Ok(CollectionMembershipResponse {
                collection_ids: vec![],
            });
        };
        let ident = ItemContentIdentity {
            valsi: row.get("valsi"),
            definition: row.get("definition"),
            source_langid: row.get("source_langid"),
            langid: row.get("langid"),
        };
        if ident.is_empty() {
            return Ok(CollectionMembershipResponse {
                collection_ids: vec![],
            });
        }

        // Scan the caller's items once (nested-loop from their collections) instead of a
        // correlated EXISTS per collection that joins `valsi` and wraps columns in COALESCE.
        let mut sql = String::from(
            "SELECT c.collection_id
             FROM collections c
             INNER JOIN collection_items ci ON ci.collection_id = c.collection_id
             LEFT JOIN definitions d ON d.definitionid = ci.definition_id
             WHERE c.user_id = $1",
        );
        let mut params: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(user_id)];
        let mut param_count = 2;
        append_content_identity_match(&mut sql, &mut params, &mut param_count, &ident);
        sql.push_str(" GROUP BY c.collection_id, c.updated_at ORDER BY c.updated_at DESC");
        let param_refs: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|p| &**p as &(dyn ToSql + Sync)).collect();
        let rows = client
            .query(&sql, &param_refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        return Ok(CollectionMembershipResponse {
            collection_ids: rows.iter().map(|row| row.get("collection_id")).collect(),
        });
    }

    let Some(definition_id) = req.definition_id else {
        return Err(AppError::BadRequest(
            "definition_id or item_id is required".to_string(),
        ));
    };
    if definition_id <= 0 {
        return Err(AppError::BadRequest(
            "definition_id must be positive".to_string(),
        ));
    }

    // UNIQUE (collection_id, definition_id) makes this an index lookup per owned collection.
    let rows = client
        .query(
            "SELECT c.collection_id
             FROM collections c
             WHERE c.user_id = $1
               AND EXISTS (
                 SELECT 1 FROM collection_items ci
                 WHERE ci.collection_id = c.collection_id
                   AND ci.definition_id = $2
               )
             ORDER BY c.updated_at DESC",
            &[&user_id, &definition_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(CollectionMembershipResponse {
        collection_ids: rows.iter().map(|row| row.get("collection_id")).collect(),
    })
}

const MAX_MEMBERSHIP_BATCH: usize = 100;

/// Batch variant of [`collections_containing_item`]. Results are parallel to `req.items`.
pub async fn collections_containing_items_batch(
    pool: &Pool,
    user_id: i32,
    req: &CollectionMembershipBatchRequest,
) -> AppResult<CollectionMembershipBatchResponse> {
    let items: Vec<&CollectionMembershipRequest> =
        req.items.iter().take(MAX_MEMBERSHIP_BATCH).collect();
    if items.is_empty() {
        return Ok(CollectionMembershipBatchResponse { results: vec![] });
    }

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut definition_ids: Vec<i32> = Vec::new();
    let mut item_ids: Vec<i32> = Vec::new();
    for item in &items {
        if let Some(item_id) = item.item_id.filter(|id| *id > 0) {
            item_ids.push(item_id);
        } else if let Some(definition_id) = item.definition_id.filter(|id| *id > 0) {
            definition_ids.push(definition_id);
        }
    }
    definition_ids.sort_unstable();
    definition_ids.dedup();
    item_ids.sort_unstable();
    item_ids.dedup();

    let mut by_definition: HashMap<i32, Vec<i32>> = HashMap::new();
    if !definition_ids.is_empty() {
        let rows = client
            .query(
                "SELECT ci.definition_id, c.collection_id
                 FROM collections c
                 INNER JOIN collection_items ci ON ci.collection_id = c.collection_id
                 WHERE c.user_id = $1
                   AND ci.definition_id = ANY($2)
                 ORDER BY ci.definition_id, c.updated_at DESC",
                &[&user_id, &definition_ids],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        for row in rows {
            let definition_id: i32 = row.get("definition_id");
            let collection_id: i32 = row.get("collection_id");
            let entry = by_definition.entry(definition_id).or_default();
            if !entry.contains(&collection_id) {
                entry.push(collection_id);
            }
        }
    }

    let mut by_item: HashMap<i32, Vec<i32>> = HashMap::new();
    if !item_ids.is_empty() {
        let rows = client
            .query(
                "WITH src AS (
                    SELECT
                        ci.item_id,
                        COALESCE(d.cached_valsiword, ci.free_content_front) AS valsi,
                        COALESCE(d.definition, ci.free_content_back) AS definition,
                        d.cached_source_langid AS source_langid,
                        COALESCE(d.langid, ci.langid) AS langid
                    FROM collection_items ci
                    JOIN collections owner_c ON owner_c.collection_id = ci.collection_id
                    LEFT JOIN definitions d ON d.definitionid = ci.definition_id
                    WHERE ci.item_id = ANY($2)
                      AND (owner_c.is_public = true OR owner_c.user_id = $1)
                 )
                 SELECT src.item_id, c.collection_id
                 FROM src
                 JOIN collections c ON c.user_id = $1
                 JOIN collection_items ci ON ci.collection_id = c.collection_id
                 LEFT JOIN definitions d ON d.definitionid = ci.definition_id
                 WHERE (
                    (
                        ci.definition_id IS NOT NULL
                        AND d.cached_valsiword IS NOT DISTINCT FROM src.valsi
                        AND d.definition IS NOT DISTINCT FROM src.definition
                        AND d.langid IS NOT DISTINCT FROM src.langid
                        AND d.cached_source_langid IS NOT DISTINCT FROM src.source_langid
                    )
                    OR (
                        ci.definition_id IS NULL
                        AND ci.free_content_front IS NOT DISTINCT FROM src.valsi
                        AND ci.free_content_back IS NOT DISTINCT FROM src.definition
                        AND ci.langid IS NOT DISTINCT FROM src.langid
                        AND src.source_langid IS NULL
                    )
                 )
                 ORDER BY src.item_id, c.updated_at DESC",
                &[&user_id, &item_ids],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        for row in rows {
            let item_id: i32 = row.get("item_id");
            let collection_id: i32 = row.get("collection_id");
            let entry = by_item.entry(item_id).or_default();
            if !entry.contains(&collection_id) {
                entry.push(collection_id);
            }
        }
    }

    let results = items
        .iter()
        .map(|item| {
            let collection_ids = if let Some(item_id) = item.item_id.filter(|id| *id > 0) {
                by_item.get(&item_id).cloned().unwrap_or_default()
            } else if let Some(definition_id) = item.definition_id.filter(|id| *id > 0) {
                by_definition
                    .get(&definition_id)
                    .cloned()
                    .unwrap_or_default()
            } else {
                vec![]
            };
            CollectionMembershipResponse { collection_ids }
        })
        .collect();

    Ok(CollectionMembershipBatchResponse { results })
}

/// Identity used to treat collection items as the same content.
pub(crate) fn collection_item_content_key(
    item: &CollectionItemResponse,
) -> (String, String, i32, i32) {
    let valsi = item
        .word
        .as_deref()
        .or(item.free_content_front.as_deref())
        .unwrap_or("")
        .to_string();
    let definition = item
        .definition
        .as_deref()
        .or(item.free_content_back.as_deref())
        .unwrap_or("")
        .to_string();
    let source = item.source_langid.unwrap_or(-1);
    let lang = item.lang_id.or(item.language_id).unwrap_or(-1);
    (valsi, definition, source, lang)
}

/// Keep one representative per valsi+definition+source+target language.
/// Prefers items from `filter_collection_ids`, then the oldest matching collection
/// item (`added_at`). Filter order is a tie-breaker only.
/// Filtered search now dedupes in SQL (`dedupe_by_content`); kept for tests/parity.
#[allow(dead_code)]
pub(crate) fn dedupe_collection_items_by_content(
    items: Vec<CollectionItemResponse>,
    filter_collection_ids: &[i32],
) -> Vec<CollectionItemResponse> {
    use std::collections::HashMap;
    let mut groups: HashMap<(String, String, i32, i32), Vec<CollectionItemResponse>> =
        HashMap::new();
    let mut order: Vec<(String, String, i32, i32)> = Vec::new();
    for item in items {
        let key = collection_item_content_key(&item);
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        groups.entry(key).or_default().push(item);
    }
    order
        .into_iter()
        .filter_map(|key| {
            let mut group = groups.remove(&key)?;
            group.sort_by_key(|item| {
                let filter_pos = item
                    .collection_id
                    .and_then(|id| filter_collection_ids.iter().position(|&fid| fid == id))
                    .unwrap_or(usize::MAX);
                let outside_filter = if filter_pos == usize::MAX { 1 } else { 0 };
                // Oldest matching item first; filter order is only a tie-breaker.
                (outside_filter, item.added_at, filter_pos, item.item_id)
            });
            let count = i32::try_from(group.len()).unwrap_or(i32::MAX);
            let mut chosen = group.remove(0);
            if count > 1 {
                chosen.match_count = Some(count);
            }
            Some(chosen)
        })
        .collect()
}

pub async fn list_public_collections(
    pool: &Pool,
    redis: &RedisCache,
    query: &ListCollectionsQuery,
    viewer_user_id: Option<i32>,
) -> AppResult<CollectionListResponse> {
    let requested = query.sort.as_deref().unwrap_or("active_week");
    // Personal sort needs a viewer; anonymous callers get the public default.
    let sort_key = if requested == "recently_studied" && viewer_user_id.is_none() {
        "active_week"
    } else {
        requested
    };
    let cache_key = format!("collections_public:{}", sort_key);
    let has_search = query
        .search
        .as_deref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let has_flashcards_only = query.has_flashcards_only.unwrap_or(false);
    let has_levels_only = query.has_levels_only.unwrap_or(false);
    let pagination = pagination_bounds(query.page, query.per_page);
    let has_pagination = pagination.is_some();
    let is_personal_sort = sort_key == "recently_studied";

    // For plain listing requests (sort-only), serve from cache and paginate in memory if requested.
    // Skip Redis for personal sorts — they depend on the caller.
    if !is_personal_sort && !has_search && !has_flashcards_only && !has_levels_only {
        if let Ok(Some(response)) = redis.get::<CollectionListResponse>(&cache_key).await {
            if let Some((_, per_page, offset)) = pagination {
                let start = offset as usize;
                let page_items = response
                    .collections
                    .into_iter()
                    .skip(start)
                    .take(per_page as usize)
                    .collect::<Vec<_>>();
                return Ok(CollectionListResponse {
                    collections: page_items,
                    total: response.total,
                });
            }
            return Ok(response);
        }
    }

    // Cache miss or filtered query – compute live
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    log::info!("collection cache miss/filtered query for sort_key={sort_key}; computing live");
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];
    let where_clause = build_collection_where_clause(
        "c.is_public = true",
        &mut params,
        query.search.as_deref(),
        has_flashcards_only,
        has_levels_only,
    );

    let response = query_collections(
        &client,
        &where_clause,
        &mut params,
        Some(sort_key),
        query.search.as_deref(),
        pagination,
        viewer_user_id,
    )
    .await?;

    // Keep existing sort-cache behavior only for plain (non-filtered, unpaginated) requests.
    if !is_personal_sort
        && !has_search
        && !has_flashcards_only
        && !has_levels_only
        && !has_pagination
    {
        let _ = redis
            .set(
                &cache_key,
                &response,
                Some(std::time::Duration::from_secs(6 * 3600)),
            )
            .await;
    }

    Ok(response)
}

const MAX_PICKER_PER_KIND: i64 = 40;
const POPULAR_PICKER_PER_KIND: i64 = 10;
const FILTER_PICKER_POPULAR_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(3600);

/// Combined picker for Home/Fast Search: public collections, then authors (users).
///
/// Empty search returns the top popular public collections (flashcard activity, last 7 days)
/// and the top users by most recent definition submission. That default snapshot is Redis-cached.
pub async fn list_users_and_collections(
    pool: &Pool,
    redis: &RedisCache,
    search: Option<String>,
    per_kind: i64,
) -> AppResult<CollectionUserPickerResponse> {
    let per_kind = per_kind.clamp(1, MAX_PICKER_PER_KIND);
    let search_pattern = search
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s));

    if search_pattern.is_none() {
        return list_popular_users_and_collections(pool, redis).await;
    }

    query_users_and_collections(pool, search_pattern.as_deref(), per_kind).await
}

async fn list_popular_users_and_collections(
    pool: &Pool,
    redis: &RedisCache,
) -> AppResult<CollectionUserPickerResponse> {
    if let Ok(Some(cached)) = redis
        .get::<CollectionUserPickerResponse>(FILTER_PICKER_POPULAR_CACHE_KEY)
        .await
    {
        return Ok(cached);
    }

    let response = query_popular_users_and_collections(pool, POPULAR_PICKER_PER_KIND).await?;
    if let Err(e) = redis
        .set(
            FILTER_PICKER_POPULAR_CACHE_KEY,
            &response,
            Some(FILTER_PICKER_POPULAR_CACHE_TTL),
        )
        .await
    {
        log::warn!("Failed to cache filter picker popular list: {e}");
    }
    Ok(response)
}

async fn query_popular_users_and_collections(
    pool: &Pool,
    per_kind: i64,
) -> AppResult<CollectionUserPickerResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Popular = most distinct flashcard reviewers in the last 7 days (same as public list default).
    let collection_rows = client
        .query(
            "SELECT c.collection_id, c.name, u.username AS owner_username
             FROM collections c
             JOIN users u ON u.userid = c.user_id
             LEFT JOIN flashcards f ON f.collection_id = c.collection_id
             LEFT JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id
                 AND ufp.last_reviewed_at >= NOW() - INTERVAL '7 days'
             WHERE c.is_public = true
             GROUP BY c.collection_id, u.userid, u.username
             ORDER BY COUNT(DISTINCT ufp.user_id) DESC, c.updated_at DESC
             LIMIT $1",
            &[&per_kind],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Authors who most recently added (or updated) a definition.
    let user_rows = client
        .query(
            "SELECT u.userid, u.username, u.realname
             FROM users u
             INNER JOIN definitions d ON d.userid = u.userid
             WHERE u.password IS DISTINCT FROM 'DISABLED'
             GROUP BY u.userid, u.username, u.realname
             ORDER BY MAX(d.time) DESC NULLS LAST
             LIMIT $1",
            &[&per_kind],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(picker_response_from_rows(collection_rows, user_rows))
}

async fn query_users_and_collections(
    pool: &Pool,
    search_pattern: Option<&str>,
    per_kind: i64,
) -> AppResult<CollectionUserPickerResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection_rows = client
        .query(
            "SELECT c.collection_id, c.name, u.username AS owner_username
             FROM collections c
             JOIN users u ON u.userid = c.user_id
             WHERE c.is_public = true
               AND (
                    $1::text IS NULL
                    OR c.name ILIKE $1
                    OR COALESCE(c.description, '') ILIKE $1
                    OR u.username ILIKE $1
               )
             ORDER BY c.updated_at DESC
             LIMIT $2",
            &[&search_pattern, &per_kind],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let user_rows = client
        .query(
            "SELECT u.userid, u.username, u.realname
             FROM users u
             WHERE u.password IS DISTINCT FROM 'DISABLED'
               AND (
                    $1::text IS NULL
                    OR u.username ILIKE $1
                    OR COALESCE(u.realname, '') ILIKE $1
               )
             ORDER BY u.username ASC
             LIMIT $2",
            &[&search_pattern, &per_kind],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(picker_response_from_rows(collection_rows, user_rows))
}

/// Top public collections by distinct flashcards learnt in the last 3 months.
/// A collection counts only when more than one card was learnt in that window.
pub const POPULAR_LEARNT_COLLECTION_LIMIT: i64 = 100;
const POPULAR_LEARNT_CACHE_KEY: &str = "popular_collections:learnt_3m_v1";
const POPULAR_LEARNT_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(3600);

pub async fn popular_learnt_collection_ids(pool: &Pool, redis: &RedisCache) -> AppResult<Vec<i32>> {
    if let Ok(Some(cached)) = redis.get::<Vec<i32>>(POPULAR_LEARNT_CACHE_KEY).await {
        return Ok(cached);
    }
    let ids = query_popular_learnt_collection_ids(pool).await?;
    if let Err(e) = redis
        .set(
            POPULAR_LEARNT_CACHE_KEY,
            &ids,
            Some(POPULAR_LEARNT_CACHE_TTL),
        )
        .await
    {
        log::warn!("Failed to cache popular learnt collections: {e}");
    }
    Ok(ids)
}

pub async fn query_popular_learnt_collection_ids(pool: &Pool) -> AppResult<Vec<i32>> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = client
        .query(
            "SELECT c.collection_id
             FROM collections c
             JOIN flashcards f ON f.collection_id = c.collection_id
             JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id
             WHERE c.is_public = true
               AND ufp.last_reviewed_at >= NOW() - INTERVAL '3 months'
               AND ufp.review_count > 0
             GROUP BY c.collection_id
             HAVING COUNT(DISTINCT f.id) > 1
             ORDER BY COUNT(DISTINCT f.id) DESC, c.collection_id DESC
             LIMIT $1",
            &[&POPULAR_LEARNT_COLLECTION_LIMIT],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(rows.iter().map(|row| row.get("collection_id")).collect())
}

pub fn merge_collection_ids(mut ids: Vec<i32>, popular: Vec<i32>) -> Vec<i32> {
    let mut seen: std::collections::HashSet<i32> = ids.iter().copied().collect();
    for id in popular {
        if seen.insert(id) {
            ids.push(id);
        }
    }
    ids
}

fn picker_response_from_rows(
    collection_rows: Vec<tokio_postgres::Row>,
    user_rows: Vec<tokio_postgres::Row>,
) -> CollectionUserPickerResponse {
    let mut items = Vec::with_capacity(collection_rows.len() + user_rows.len());
    for row in collection_rows {
        items.push(CollectionUserPickerItem::Collection {
            collection_id: row.get("collection_id"),
            name: row.get("name"),
            owner_username: row.get("owner_username"),
        });
    }
    for row in user_rows {
        items.push(CollectionUserPickerItem::User {
            user_id: row.get("userid"),
            username: row.get("username"),
            realname: row.get("realname"),
        });
    }
    CollectionUserPickerResponse { items }
}

/// Distinct logged-in users who reviewed at least one card in the collection during the last 30 days.
/// Same window as the public catalog “Popular this month” sort.
const MONTHLY_DRILL_USERS_SQL: &str = "(SELECT COUNT(DISTINCT ufp_month.user_id)
 FROM flashcards f_month
 JOIN user_flashcard_progress ufp_month ON ufp_month.flashcard_id = f_month.id
 WHERE f_month.collection_id = c.collection_id
   AND ufp_month.last_reviewed_at >= NOW() - INTERVAL '30 days') AS monthly_drill_users";

fn pagination_bounds(page: Option<i64>, per_page: Option<i64>) -> Option<(i64, i64, i64)> {
    if page.is_none() && per_page.is_none() {
        return None;
    }
    Some(crate::pagination::page_window(page, per_page, 24, 100))
}

fn build_collection_where_clause(
    base_condition: &str,
    params: &mut Vec<Box<dyn ToSql + Sync>>,
    search: Option<&str>,
    has_flashcards_only: bool,
    has_levels_only: bool,
) -> String {
    let mut conditions = vec![base_condition.to_string()];
    if has_flashcards_only {
        conditions.push(
            "EXISTS(SELECT 1 FROM flashcards f WHERE f.collection_id = c.collection_id)"
                .to_string(),
        );
    }
    if has_levels_only {
        conditions.push(
            "EXISTS(SELECT 1 FROM flashcard_levels fl WHERE fl.collection_id = c.collection_id)"
                .to_string(),
        );
    }

    if let Some(raw_search) = search {
        let trimmed = raw_search.trim();
        if !trimmed.is_empty() {
            let like = format!("%{}%", trimmed);
            params.push(Box::new(like));
            let like_idx = params.len();
            conditions.push(format!(
                "(c.name ILIKE ${idx} OR COALESCE(c.description, '') ILIKE ${idx})",
                idx = like_idx
            ));
        }
    }

    format!("WHERE {}", conditions.join(" AND "))
}

async fn query_collections(
    client: &tokio_postgres::Client,
    where_clause: &str,
    params: &mut Vec<Box<dyn ToSql + Sync>>,
    sort: Option<&str>,
    search: Option<&str>,
    pagination: Option<(i64, i64, i64)>,
    viewer_user_id: Option<i32>,
) -> AppResult<CollectionListResponse> {
    // COUNT only uses where-clause placeholders; sort/search-rank params are added after this.
    let where_param_count = params.len();

    let (active_join, sort_order) = match sort.unwrap_or("active_week") {
        "active_month" => (
            "LEFT JOIN flashcards f ON f.collection_id = c.collection_id
             LEFT JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id
                 AND ufp.last_reviewed_at >= NOW() - INTERVAL '30 days'"
                .to_string(),
            "COUNT(DISTINCT ufp.user_id) DESC, c.updated_at DESC".to_string(),
        ),
        "active_all" => (
            "LEFT JOIN flashcards f ON f.collection_id = c.collection_id
             LEFT JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id"
                .to_string(),
            "COUNT(DISTINCT ufp.user_id) DESC, c.updated_at DESC".to_string(),
        ),
        "newest" => (String::new(), "c.created_at DESC".to_string()),
        "recently_studied" => {
            // Caller’s last review on any card in the course; never-studied last.
            let uid = viewer_user_id.unwrap_or(-1);
            params.push(Box::new(uid));
            let uid_idx = params.len();
            (
                format!(
                    "LEFT JOIN flashcards f ON f.collection_id = c.collection_id
                     LEFT JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id
                         AND ufp.user_id = ${uid_idx} AND NOT ufp.archived"
                ),
                "MAX(ufp.last_reviewed_at) DESC NULLS LAST, c.updated_at DESC".to_string(),
            )
        }
        _ => (
            "LEFT JOIN flashcards f ON f.collection_id = c.collection_id
             LEFT JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id
                 AND ufp.last_reviewed_at >= NOW() - INTERVAL '7 days'"
                .to_string(),
            "COUNT(DISTINCT ufp.user_id) DESC, c.updated_at DESC".to_string(),
        ),
    };

    let group_by = if active_join.is_empty() {
        String::new()
    } else {
        "GROUP BY c.collection_id, u.userid".to_string()
    };

    let mut search_rank_select = "0 AS search_rank,".to_string();
    let mut search_rank_order = String::new();
    if let Some(raw_search) = search {
        let trimmed = raw_search.trim();
        if !trimmed.is_empty() {
            params.push(Box::new(trimmed.to_string()));
            let term_idx = params.len();
            // Prefer title full-word matches, then title substring, then description matches.
            search_rank_select = format!(
                "(CASE WHEN to_tsvector('simple', COALESCE(c.name, '')) @@ plainto_tsquery('simple', ${term}) THEN 100 ELSE 0 END
                  + CASE WHEN c.name ILIKE ${like} THEN 20 ELSE 0 END
                  + CASE WHEN to_tsvector('simple', COALESCE(c.description, '')) @@ plainto_tsquery('simple', ${term}) THEN 10 ELSE 0 END
                  + CASE WHEN COALESCE(c.description, '') ILIKE ${like} THEN 1 ELSE 0 END) AS search_rank,",
                term = term_idx,
                like = term_idx - 1,
            );
            search_rank_order = "search_rank DESC, ".to_string();
        }
    }

    let count_param_refs: Vec<&(dyn ToSql + Sync)> = params[..where_param_count]
        .iter()
        .map(|p| &**p as &(dyn ToSql + Sync))
        .collect();
    let count_sql = format!("SELECT COUNT(*) AS total FROM collections c {where_clause}",);
    let total: i64 = client
        .query_one(&count_sql, &count_param_refs)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .get("total");

    let mut sql = format!(
        "SELECT c.*, u.userid, u.username,
                (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.collection_id) AS item_count,
                EXISTS(SELECT 1 FROM flashcards f2 WHERE f2.collection_id = c.collection_id) AS has_flashcards,
                (c.cover_collection_image_id IS NOT NULL) AS has_cover_image,
                (
                    (c.cover_collection_image_id IS NOT NULL)
                    OR EXISTS (
                        SELECT 1 FROM collection_item_images cii
                        INNER JOIN collection_items ci2 ON ci2.item_id = cii.item_id
                        WHERE ci2.collection_id = c.collection_id AND cii.side IN ('front', 'back')
                    )
                ) AS has_collection_image,
                (SELECT COUNT(*) FROM comments cm
                    JOIN threads t ON cm.threadid = t.threadid
                    WHERE t.collection_id = c.collection_id) AS comment_count,
                {MONTHLY_DRILL_USERS_SQL},
                {search_rank_select}
                c.updated_at AS _rank_tiebreak
         FROM collections c
         JOIN users u ON c.user_id = u.userid
         {active_join}
         {where_clause}
         {group_by}
         ORDER BY {search_rank_order}{sort_order}, _rank_tiebreak DESC"
    );

    if let Some((_, per_page, offset)) = pagination {
        params.push(Box::new(per_page));
        let per_page_idx = params.len();
        params.push(Box::new(offset));
        let offset_idx = params.len();
        sql.push_str(&format!(" LIMIT ${per_page_idx} OFFSET ${offset_idx}"));
    }

    let param_refs: Vec<&(dyn ToSql + Sync)> =
        params.iter().map(|p| &**p as &(dyn ToSql + Sync)).collect();
    let rows = client
        .query(&sql, &param_refs)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collections = rows
        .iter()
        .map(|row| CollectionResponse {
            collection_id: row.get("collection_id"),
            name: row.get("name"),
            description: row.get("description"),
            is_public: row.get("is_public"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            item_count: row.get("item_count"),
            has_flashcards: row.get("has_flashcards"),
            has_cover_image: row.get("has_cover_image"),
            has_collection_image: row.get("has_collection_image"),
            comment_count: row.get("comment_count"),
            monthly_drill_users: row.get("monthly_drill_users"),
            owner: CollectionOwner {
                user_id: row.get("userid"),
                username: row
                    .try_get("username")
                    .unwrap_or_else(|_| "unknown".to_string()),
            },
        })
        .collect();

    Ok(CollectionListResponse { collections, total })
}

/// Recompute all four sort-order snapshots for the public collections list and
/// store them in Redis. Called by the background job every few hours.
pub async fn refresh_collection_sort_cache(pool: &Pool, redis: &RedisCache) -> AppResult<()> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    for sort_key in &["active_week", "active_month", "active_all", "newest"] {
        let sql = build_collections_query("WHERE c.is_public = true", Some(sort_key), false);
        let rows = client.query(&sql, &[]).await.map_err(|e| {
            AppError::Database(format!("Cache refresh query failed for {sort_key}: {e}"))
        })?;

        let collections: Vec<CollectionResponse> = rows
            .iter()
            .map(|row| CollectionResponse {
                collection_id: row.get("collection_id"),
                name: row.get("name"),
                description: row.get("description"),
                is_public: row.get("is_public"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                item_count: row.get("item_count"),
                has_flashcards: row.get("has_flashcards"),
                has_cover_image: row.get("has_cover_image"),
                has_collection_image: row.get("has_collection_image"),
                comment_count: row.get("comment_count"),
                monthly_drill_users: row.get("monthly_drill_users"),
                owner: CollectionOwner {
                    user_id: row.get("userid"),
                    username: row
                        .try_get("username")
                        .unwrap_or_else(|_| "unknown".to_string()),
                },
            })
            .collect();

        let response = CollectionListResponse {
            total: collections.len() as i64,
            collections,
        };

        let cache_key = format!("collections_public:{}", sort_key);
        redis
            .set(
                &cache_key,
                &response,
                Some(std::time::Duration::from_secs(6 * 3600)),
            )
            .await
            .map_err(|e| {
                AppError::ExternalService(format!(
                    "Cache refresh Redis set failed for {sort_key}: {e}"
                ))
            })?;

        log::info!("collection cache refreshed in Redis for sort_key={sort_key}");
    }

    Ok(())
}

/// Drop Redis snapshots for [`list_public_collections`] (all sort keys). Call after any change
/// that can affect public listing rows, ordering, or tie-breakers (`updated_at`, activity, counts).
pub async fn invalidate_public_collections_cache(redis: &RedisCache) {
    if let Err(e) = redis.invalidate("collections_public:*").await {
        log::warn!("Failed to invalidate public collections cache: {e}");
    }
    invalidate_filter_picker_popular_cache(redis).await;
}

/// Drop the cached default collections+authors picker (popular collections / recent authors).
pub async fn invalidate_filter_picker_popular_cache(redis: &RedisCache) {
    if let Err(e) = redis.invalidate_filter_picker_popular().await {
        log::warn!("Failed to invalidate filter picker popular cache: {e}");
    }
}

/// Build the SQL query for listing collections with a configurable ORDER BY.
///
/// `where_clause` – a WHERE clause already containing any necessary conditions
///    (but **no** $N placeholders for user_id – the parameter index is handled
///    by the caller).
/// `sort`         – sort key: "active_week" | "active_month" | "active_all" | "newest"
///    (`recently_studied` is not cached — it needs a viewer user id).
/// `user_owned`   – if true the query selects `u.username` only (no `u.userid`),
///    matching the shape used by `list_collections`.
fn build_collections_query(where_clause: &str, sort: Option<&str>, user_owned: bool) -> String {
    let extra_select = if user_owned { "" } else { "u.userid, " };

    let (active_join, order_by) = match sort.unwrap_or("active_week") {
        "active_month" => (
            "LEFT JOIN flashcards f ON f.collection_id = c.collection_id
             LEFT JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id
                 AND ufp.last_reviewed_at >= NOW() - INTERVAL '30 days'",
            "ORDER BY COUNT(DISTINCT ufp.user_id) DESC, c.updated_at DESC",
        ),
        "active_all" => (
            "LEFT JOIN flashcards f ON f.collection_id = c.collection_id
             LEFT JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id",
            "ORDER BY COUNT(DISTINCT ufp.user_id) DESC, c.updated_at DESC",
        ),
        "newest" => ("", "ORDER BY c.created_at DESC"),
        // default: active_week (and any unknown / personal keys skipped by cache refresh)
        _ => (
            "LEFT JOIN flashcards f ON f.collection_id = c.collection_id
             LEFT JOIN user_flashcard_progress ufp ON ufp.flashcard_id = f.id
                 AND ufp.last_reviewed_at >= NOW() - INTERVAL '7 days'",
            "ORDER BY COUNT(DISTINCT ufp.user_id) DESC, c.updated_at DESC",
        ),
    };

    let group_by = if active_join.is_empty() {
        String::new()
    } else {
        "GROUP BY c.collection_id, u.userid".to_string()
    };

    format!(
        "SELECT c.*, {extra_select}u.username,
                (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.collection_id) AS item_count,
                EXISTS(SELECT 1 FROM flashcards f WHERE f.collection_id = c.collection_id) AS has_flashcards,
                (c.cover_collection_image_id IS NOT NULL) AS has_cover_image,
                (
                    (c.cover_collection_image_id IS NOT NULL)
                    OR EXISTS (
                        SELECT 1 FROM collection_item_images cii
                        INNER JOIN collection_items ci2 ON ci2.item_id = cii.item_id
                        WHERE ci2.collection_id = c.collection_id AND cii.side IN ('front', 'back')
                    )
                ) AS has_collection_image,
                (SELECT COUNT(*) FROM comments cm
                    JOIN threads t ON cm.threadid = t.threadid
                    WHERE t.collection_id = c.collection_id) AS comment_count,
                {MONTHLY_DRILL_USERS_SQL}
         FROM collections c
         JOIN users u ON c.user_id = u.userid
         {active_join}
         {where_clause}
         {group_by}
         {order_by}",
        extra_select = extra_select,
        active_join = active_join,
        where_clause = where_clause,
        group_by = group_by,
        order_by = order_by,
    )
}

pub async fn get_collection(
    pool: &Pool,
    collection_id: i32,
    user_id: Option<i32>,
) -> AppResult<CollectionResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get collection details
    let collection_row = client
    .query_one(
        "SELECT c.*, u.userid, u.username, 
        (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.collection_id) as item_count,
        EXISTS(SELECT 1 FROM flashcards f WHERE f.collection_id = c.collection_id) as has_flashcards,
        (c.cover_collection_image_id IS NOT NULL) as has_cover_image,
        (
            (c.cover_collection_image_id IS NOT NULL)
            OR EXISTS (
                SELECT 1 FROM collection_item_images cii
                INNER JOIN collection_items ci2 ON ci2.item_id = cii.item_id
                WHERE ci2.collection_id = c.collection_id AND cii.side IN ('front', 'back')
            )
        ) as has_collection_image,
        (SELECT COUNT(*) FROM comments cm
            JOIN threads t ON cm.threadid = t.threadid
            WHERE t.collection_id = c.collection_id) as comment_count
        FROM collections c
        JOIN users u ON c.user_id = u.userid
             WHERE c.collection_id = $1",
            &[&collection_id],
        )
        .await.map_err(|e| AppError::Database(e.to_string()))?;

    let is_public: bool = collection_row.get("is_public");
    let owner_id: i32 = collection_row.get("user_id");

    // Check access
    if !is_public && Some(owner_id) != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    Ok(CollectionResponse {
        collection_id,
        name: collection_row.get("name"),
        description: collection_row.get("description"),
        is_public,
        created_at: collection_row.get("created_at"),
        updated_at: collection_row.get("updated_at"),
        item_count: collection_row.get("item_count"),
        has_flashcards: collection_row.get("has_flashcards"),
        has_cover_image: collection_row.get("has_cover_image"),
        has_collection_image: collection_row.get("has_collection_image"),
        comment_count: collection_row.try_get("comment_count").unwrap_or(0),
        monthly_drill_users: 0,
        owner: CollectionOwner {
            user_id: owner_id,
            username: collection_row
                .try_get("username")
                .map_err(|e| AppError::Database(e.to_string()))?,
        },
    })
}

pub async fn update_collection(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
    req: &UpdateCollectionRequest,
) -> AppResult<CollectionResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let sanitized_name = req.name.as_ref().map(|n| sanitize_html(n));
    let sanitized_description = req.description.as_ref().map(|d| sanitize_html(d));

    // Update collection
    let row = transaction
        .query_one(
            "UPDATE collections 
             SET name = COALESCE($1, name),
                 description = COALESCE($2, description),
                 is_public = COALESCE($3, is_public),
                 updated_at = $4
             WHERE collection_id = $5
             RETURNING *",
            &[
                &sanitized_name,
                &sanitized_description,
                &req.is_public,
                &Utc::now(),
                &collection_id,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let username = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item_count = transaction
        .query_one(
            "SELECT COUNT(*) FROM collection_items WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get::<_, i64>(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let has_flashcards = transaction
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM flashcards WHERE collection_id = $1)",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get::<_, bool>(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let image_flags = transaction
        .query_one(
            "SELECT
                (c.cover_collection_image_id IS NOT NULL) AS has_cover_image,
                (
                    c.cover_collection_image_id IS NOT NULL
                    OR EXISTS (
                        SELECT 1 FROM collection_item_images cii
                        INNER JOIN collection_items ci2 ON ci2.item_id = cii.item_id
                        WHERE ci2.collection_id = $1 AND cii.side IN ('front', 'back')
                    )
                ) AS has_collection_image
             FROM collections c WHERE c.collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(CollectionResponse {
        collection_id,
        name: row.get("name"),
        description: row.get("description"),
        is_public: row.get("is_public"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        item_count,
        has_flashcards,
        has_cover_image: image_flags.get("has_cover_image"),
        has_collection_image: image_flags.get("has_collection_image"),
        comment_count: 0,
        monthly_drill_users: 0,
        owner: CollectionOwner { user_id, username },
    })
}

pub async fn delete_collection(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // Delete in dependency order: flashcards reference collection_items, so delete flashcard
    // data first, then levels, then items, then the collection.

    // 1. Review history references flashcards without CASCADE
    transaction
        .execute(
            "DELETE FROM flashcard_review_history
             WHERE flashcard_id IN (SELECT id FROM flashcards WHERE collection_id = $1)",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 2. Flashcards (CASCADEs: user_flashcard_progress, flashcard_level_items)
    transaction
        .execute(
            "DELETE FROM flashcards WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 3. Levels (CASCADEs: level_prerequisites, flashcard_level_items, user_level_progress)
    transaction
        .execute(
            "DELETE FROM flashcard_levels WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 4. Collection items (CASCADEs: collection_item_images, etc.)
    transaction
        .execute(
            "DELETE FROM collection_items WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 5. Collection
    transaction
        .execute(
            "DELETE FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;
    Ok(())
}

async fn mark_progress_graduated(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> AppResult<()> {
    transaction
        .execute(
            "UPDATE user_flashcard_progress SET status = 'graduated', next_review_at = NULL
         WHERE user_id = $1 AND flashcard_id = $2 AND card_side = $3 AND NOT archived",
            &[&user_id, &flashcard_id, &side],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn import_json(
    pool: &Pool,
    redis: &RedisCache,
    user_id: i32,
    req: &ImportJsonRequest,
) -> AppResult<ImportJsonResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Create the collection
    let row = transaction
        .query_one(
            "INSERT INTO collections (user_id, name, description, is_public)
             VALUES ($1, $2, $3, $4)
             RETURNING collection_id, created_at, updated_at",
            &[
                &user_id,
                &req.name,
                &req.description,
                &req.is_public.unwrap_or(true),
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection_id: i32 = row.get("collection_id");
    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut warnings = Vec::new();

    // Which referenced definitions exist (one round trip instead of one per item).
    let requested_def_ids: Vec<i32> = req.items.iter().filter_map(|i| i.definition_id).collect();
    let existing_def_ids: std::collections::HashSet<i32> = transaction
        .query(
            "SELECT definitionid FROM definitions WHERE definitionid = ANY($1)",
            &[&requested_def_ids],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .iter()
        .map(|r| r.get(0))
        .collect();

    let max_position: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut ins_def_ids: Vec<i32> = Vec::new();
    let mut ins_notes: Vec<Option<String>> = Vec::new();
    let mut ins_positions: Vec<i32> = Vec::new();
    let mut ins_canonical: Vec<Option<String>> = Vec::new();

    for item in &req.items {
        if let Some(def_id) = item.definition_id {
            if !existing_def_ids.contains(&def_id) {
                warnings.push(format!(
                    "Definition ID {} not found for word '{}'",
                    def_id, item.word
                ));
                skipped_count += 1;
                continue;
            }
            if ins_def_ids.contains(&def_id) {
                warnings.push(format!(
                    "Definition ID {} listed more than once, skipped duplicate for word '{}'",
                    def_id, item.word
                ));
                skipped_count += 1;
                continue;
            }
            ins_def_ids.push(def_id);
            ins_notes.push(item.collection_note.clone());
            ins_positions.push(max_position + 1 + imported_count);
            ins_canonical.push(crate::utils::canonical::get_canonical_form(&item.word));
            imported_count += 1;
        } else {
            warnings.push(format!(
                "Skipping word '{}' - no definition ID provided",
                item.word
            ));
            skipped_count += 1;
        }
    }

    if !ins_def_ids.is_empty() {
        transaction
            .execute(
                "INSERT INTO collection_items (collection_id, definition_id, notes, position, canonical_form)
                 SELECT $1, t.definition_id, t.notes, t.position, t.canonical_form
                 FROM UNNEST($2::int[], $3::text[], $4::int[], $5::text[])
                      WITH ORDINALITY AS t(definition_id, notes, position, canonical_form, ord)
                 ORDER BY t.ord",
                &[
                    &collection_id,
                    &ins_def_ids,
                    &ins_notes,
                    &ins_positions,
                    &ins_canonical,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    let username = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection = CollectionResponse {
        collection_id,
        name: req.name.clone(),
        description: req.description.clone(),
        is_public: req.is_public.unwrap_or(true),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        item_count: imported_count as i64,
        has_flashcards: false,
        has_cover_image: false,
        has_collection_image: false,
        comment_count: 0,
        monthly_drill_users: 0,
        owner: CollectionOwner { user_id, username },
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(ImportJsonResponse {
        collection,
        imported_count,
        skipped_count,
        warnings,
    })
}

// Helper function to parse data URL and decode base64
fn decode_data_url(url: &str) -> AppResult<(String, Vec<u8>)> {
    if !url.starts_with("data:") {
        return Err(AppError::BadRequest("Invalid data URL format".to_string()));
    }
    let parts: Vec<&str> = url[5..].splitn(2, ';').collect();
    if parts.len() != 2 || parts[1].splitn(2, ',').count() != 2 {
        return Err(AppError::BadRequest("Invalid data URL format".to_string()));
    }
    let mime_type = parts[0].to_string();
    let data_part = parts[1].split_once(',').map(|x| x.1).unwrap_or("");

    if !parts[1].starts_with("base64,") {
        return Err(AppError::BadRequest(
            "Only base64 encoded data URLs are supported".to_string(),
        ));
    }

    let decoded = BASE64
        .decode(data_part)
        .map_err(|e| AppError::BadRequest(format!("Invalid base64 data: {}", e)))?;

    Ok((mime_type, decoded))
}

pub async fn import_collection_from_json(
    pool: &Pool,
    redis: &RedisCache,
    target_collection_id: i32,
    user_id: i32,
    items: &[CollectionExportItem],
) -> AppResult<ImportCollectionJsonResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Verify ownership of the target collection
    verify_collection_ownership(&transaction, target_collection_id, user_id).await?;

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut skipped_items = Vec::new();

    // Get current max position in the target collection
    let mut current_max_position: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
            &[&target_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Prefetch what already exists in the target collection, then track what this
    // import adds, so duplicate detection matches the old per-item queries.
    let requested_def_ids: Vec<i32> = items.iter().filter_map(|i| i.definition_id).collect();
    let mut present_def_ids: std::collections::HashSet<i32> = transaction
        .query(
            "SELECT definition_id FROM collection_items
             WHERE collection_id = $1 AND definition_id = ANY($2)",
            &[&target_collection_id, &requested_def_ids],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .iter()
        .map(|r| r.get(0))
        .collect();
    let mut present_free: std::collections::HashSet<(String, String)> = transaction
        .query(
            "SELECT free_content_front, free_content_back FROM collection_items
             WHERE collection_id = $1
               AND free_content_front IS NOT NULL AND free_content_back IS NOT NULL",
            &[&target_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .iter()
        .map(|r| (r.get(0), r.get(1)))
        .collect();

    // Rows to insert, in input order, plus the export item each came from.
    let mut ins_items: Vec<&CollectionExportItem> = Vec::new();
    let mut ins_def_ids: Vec<Option<i32>> = Vec::new();
    let mut ins_fronts: Vec<Option<String>> = Vec::new();
    let mut ins_backs: Vec<Option<String>> = Vec::new();
    let mut ins_notes: Vec<Option<String>> = Vec::new();
    let mut ins_positions: Vec<i32> = Vec::new();
    let mut ins_canonical: Vec<Option<String>> = Vec::new();

    for item in items {
        let mut skip_reason: Option<String> = None;
        let identifier: String;

        if let Some(def_id) = item.definition_id {
            identifier = format!("definition_id: {}", def_id);
            if present_def_ids.contains(&def_id) {
                skip_reason = Some("Definition already exists in target collection".to_string());
            }
        } else if let (Some(front), Some(back)) =
            (&item.free_content_front, &item.free_content_back)
        {
            identifier = format!(
                "free_content_front: {}",
                front.chars().take(30).collect::<String>()
            );
            if present_free.contains(&(sanitize_html(front), sanitize_html(back))) {
                skip_reason =
                    Some("Free content item already exists in target collection".to_string());
            }
        } else {
            identifier = format!("item_id: {}", item.item_id);
            skip_reason =
                Some("Invalid item format (missing definition_id or free content)".to_string());
        }

        if let Some(reason) = skip_reason {
            skipped_count += 1;
            skipped_items.push(SkippedItemInfo { identifier, reason });
            continue;
        }

        current_max_position += 1;
        let sanitized_front = item.free_content_front.as_ref().map(|f| sanitize_html(f));
        let sanitized_back = item.free_content_back.as_ref().map(|b| sanitize_html(b));
        let sanitized_note = item.collection_note.as_ref().map(|n| sanitize_html(n));

        let canonical_form = sanitized_front
            .as_ref()
            .and_then(|front| crate::utils::canonical::get_canonical_form(front))
            .or_else(|| {
                item.word
                    .as_ref()
                    .and_then(|w| crate::utils::canonical::get_canonical_form(w))
            });

        if let Some(def_id) = item.definition_id {
            present_def_ids.insert(def_id);
        }
        if let (Some(f), Some(b)) = (&sanitized_front, &sanitized_back) {
            present_free.insert((f.clone(), b.clone()));
        }

        ins_items.push(item);
        ins_def_ids.push(item.definition_id);
        ins_fronts.push(sanitized_front);
        ins_backs.push(sanitized_back);
        ins_notes.push(sanitized_note);
        ins_positions.push(current_max_position);
        ins_canonical.push(canonical_form);
        imported_count += 1;
    }

    let mut item_id_by_position: HashMap<i32, i32> = HashMap::new();
    if !ins_items.is_empty() {
        for row in transaction
            .query(
                "INSERT INTO collection_items (collection_id, definition_id, free_content_front, free_content_back, notes, position, canonical_form)
                 SELECT $1, t.definition_id, t.front, t.back, t.notes, t.position, t.canonical_form
                 FROM UNNEST($2::int[], $3::text[], $4::text[], $5::text[], $6::int[], $7::text[])
                      WITH ORDINALITY AS t(definition_id, front, back, notes, position, canonical_form, ord)
                 ORDER BY t.ord
                 RETURNING item_id, position",
                &[
                    &target_collection_id,
                    &ins_def_ids,
                    &ins_fronts,
                    &ins_backs,
                    &ins_notes,
                    &ins_positions,
                    &ins_canonical,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            item_id_by_position.insert(row.get("position"), row.get("item_id"));
        }
    }

    // Images, in input order (only items that carry one).
    for (item, position) in ins_items.iter().zip(&ins_positions) {
        let new_item_id = item_id_by_position[position];
        for (side, url_option) in [
            ("front", &item.front_image_url),
            ("back", &item.back_image_url),
        ] {
            if let Some(url) = url_option {
                let (mime_type, image_data) = decode_data_url(url)?;
                if image_data.len() > MAX_ITEM_IMAGE_BYTES {
                    return Err(AppError::BadRequest(format!(
                        "Image for item {} ({}) exceeds {}MB limit",
                        new_item_id,
                        side,
                        MAX_ITEM_IMAGE_BYTES / (1024 * 1024)
                    )));
                }
                let image_id =
                    get_or_insert_collection_image_id(&transaction, &image_data, &mime_type)
                        .await?;
                transaction
                    .execute(
                        "INSERT INTO collection_item_images (item_id, collection_image_id, side) VALUES ($1, $2, $3)",
                        &[&new_item_id, &image_id, &side],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }
        }
    }

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(ImportCollectionJsonResponse {
        imported_count,
        skipped_count,
        skipped_items,
    })
}

/// Parse direction string from export; defaults to Both if missing or invalid.
fn parse_export_direction(s: Option<&String>) -> FlashcardDirection {
    let s = match s {
        Some(x) => x.to_lowercase(),
        None => return FlashcardDirection::Both,
    };
    match s.as_str() {
        "direct" => FlashcardDirection::Direct,
        "reverse" => FlashcardDirection::Reverse,
        "both" => FlashcardDirection::Both,
        "fillin" => FlashcardDirection::FillIn,
        "fillin_reverse" => FlashcardDirection::FillInReverse,
        "fillin_both" => FlashcardDirection::FillInBoth,
        "wordbank" => FlashcardDirection::Wordbank,
        "wordbank_reverse" => FlashcardDirection::WordbankReverse,
        "wordbank_both" => FlashcardDirection::WordbankBoth,
        "listen_wordbank" => FlashcardDirection::ListenWordbank,
        "listen_wordbank_reverse" => FlashcardDirection::ListenWordbankReverse,
        "listen_wordbank_both" => FlashcardDirection::ListenWordbankBoth,
        "just_information" => FlashcardDirection::JustInformation,
        "quiz_direct" => FlashcardDirection::QuizDirect,
        "quiz_reverse" => FlashcardDirection::QuizReverse,
        "quiz_both" => FlashcardDirection::QuizBoth,
        _ => FlashcardDirection::Both,
    }
}

/// Store an imported `data:audio/...;base64,` sound for an item (same limits as item upload).
async fn attach_imported_sound(
    transaction: &Transaction<'_>,
    item_id: i32,
    url: &str,
) -> Result<(), String> {
    let (mime_type, data) = url
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(";base64,"))
        .ok_or_else(|| "not a base64 data URL".to_string())?;
    let sound = crate::collections::models::SoundData {
        data: data.to_string(),
        mime_type: mime_type.to_string(),
    };
    validate_item_audio(&sound)?;
    let bytes = BASE64.decode(data).map_err(|e| e.to_string())?;
    transaction
        .execute(
            "INSERT INTO collection_item_sounds (item_id, sound_data, mime_type) VALUES ($1, $2, $3)",
            &[&item_id, &bytes, &mime_type],
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn is_quiz_direction(d: &FlashcardDirection) -> bool {
    matches!(
        d,
        FlashcardDirection::QuizDirect
            | FlashcardDirection::QuizReverse
            | FlashcardDirection::QuizBoth
    )
}

/// Challenge-mix directions for imported level items that carry no `direction`.
///
/// Without this every such card became `both` (plain flip cards) and a course had no
/// Duolingo-style variety. Levels where every item already has a direction are left alone.
/// The listen slot needs the item's imported `sound_url`; without one it is a flip card.
fn auto_mix_directions_for_import(
    items: &[CollectionExportItem],
    levels: &[LevelExport],
) -> HashMap<usize, &'static str> {
    let mut out = HashMap::new();
    for level in levels {
        let positions: Vec<usize> = level
            .item_positions
            .iter()
            .copied()
            .filter(|&i| i < items.len())
            .collect();
        if positions.iter().all(|&i| items[i].direction.is_some()) {
            continue;
        }
        let cards: Vec<crate::utils::challenge_mix::ChallengeMixCard<'_>> = positions
            .iter()
            .map(|&i| crate::utils::challenge_mix::ChallengeMixCard {
                front: items[i]
                    .free_content_front
                    .as_deref()
                    .or(items[i].word.as_deref())
                    .unwrap_or(""),
                has_sound: items[i].sound_url.is_some(),
            })
            .collect();
        for (&i, dir) in positions
            .iter()
            .zip(crate::utils::challenge_mix::assign_challenge_mix(&cards))
        {
            if items[i].direction.is_none() {
                out.entry(i).or_insert(dir);
            }
        }
    }
    out
}

/// Full import: create collection, items, then (if levels present) flashcards and levels.
pub async fn import_full(
    pool: &Pool,
    redis: &RedisCache,
    user_id: i32,
    req: &ImportFullRequest,
) -> AppResult<ImportFullResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let sanitized_name = sanitize_html(&req.collection.name);
    let sanitized_description = req
        .collection
        .description
        .as_ref()
        .map(|d| sanitize_html(d));
    let is_public = req.collection.is_public.unwrap_or(true);

    let row = transaction
        .query_one(
            "INSERT INTO collections (user_id, name, description, is_public)
             VALUES ($1, $2, $3, $4)
             RETURNING collection_id, created_at, updated_at",
            &[
                &user_id,
                &sanitized_name,
                &sanitized_description,
                &is_public,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection_id: i32 = row.get("collection_id");
    let mut imported_count = 0i32;
    let mut skipped_count = 0i32;
    let mut warnings: Vec<String> = Vec::new();
    // For each export index, Some(item_id) if we inserted that item, None if skipped
    let mut inserted_item_ids_by_export_index: Vec<Option<i32>> =
        Vec::with_capacity(req.items.len());

    // Pass 1: decide, in input order, which items are inserted. Definition existence
    // is checked in one query instead of one per item.
    let requested_def_ids: Vec<i32> = req.items.iter().filter_map(|i| i.definition_id).collect();
    let existing_def_ids: HashSet<i32> = transaction
        .query(
            "SELECT definitionid FROM definitions WHERE definitionid = ANY($1)",
            &[&requested_def_ids],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .iter()
        .map(|r| r.get(0))
        .collect();

    // Some(skip warning) for skipped items, None for items to insert.
    let mut skip_warning_by_index: Vec<Option<String>> = Vec::with_capacity(req.items.len());
    let mut ins_def_ids: Vec<Option<i32>> = Vec::new();
    let mut ins_fronts: Vec<Option<String>> = Vec::new();
    let mut ins_backs: Vec<Option<String>> = Vec::new();
    let mut ins_notes: Vec<Option<String>> = Vec::new();
    let mut ins_positions: Vec<i32> = Vec::new();
    let mut ins_canonical: Vec<Option<String>> = Vec::new();

    for (pos, item) in req.items.iter().enumerate() {
        let has_definition = item.definition_id.is_some();
        let has_free_content =
            item.free_content_front.is_some() && item.free_content_back.is_some();

        if !has_definition && !has_free_content {
            skip_warning_by_index.push(Some(format!(
                "Item at position {}: missing definition_id and free content, skipped",
                pos
            )));
            continue;
        }
        if let Some(def_id) = item.definition_id {
            if !existing_def_ids.contains(&def_id) {
                skip_warning_by_index.push(Some(format!(
                    "Definition ID {} not found for word '{}', skipped",
                    def_id,
                    item.word.as_deref().unwrap_or("")
                )));
                continue;
            }
            if ins_def_ids.contains(&Some(def_id)) {
                skip_warning_by_index.push(Some(format!(
                    "Item at position {}: definition ID {} listed more than once, skipped",
                    pos, def_id
                )));
                continue;
            }
        }

        let sanitized_front = item.free_content_front.as_ref().map(|f| sanitize_html(f));
        let sanitized_back = item.free_content_back.as_ref().map(|b| sanitize_html(b));
        let sanitized_note = item.collection_note.as_ref().map(|n| sanitize_html(n));
        let canonical_form = sanitized_front
            .as_ref()
            .and_then(|f| crate::utils::canonical::get_canonical_form(f))
            .or_else(|| {
                item.word
                    .as_ref()
                    .and_then(|w| crate::utils::canonical::get_canonical_form(w))
            });

        skip_warning_by_index.push(None);
        ins_def_ids.push(item.definition_id);
        ins_fronts.push(sanitized_front);
        ins_backs.push(sanitized_back);
        ins_notes.push(sanitized_note);
        ins_positions.push(pos as i32);
        ins_canonical.push(canonical_form);
    }

    // One INSERT for all items; position is the export index, which is unique here.
    let mut item_id_by_position: HashMap<i32, i32> = HashMap::new();
    if !ins_positions.is_empty() {
        for row in transaction
            .query(
                "INSERT INTO collection_items (collection_id, definition_id, free_content_front, free_content_back, notes, position, canonical_form)
                 SELECT $1, t.definition_id, t.front, t.back, t.notes, t.position, t.canonical_form
                 FROM UNNEST($2::int[], $3::text[], $4::text[], $5::text[], $6::int[], $7::text[])
                      WITH ORDINALITY AS t(definition_id, front, back, notes, position, canonical_form, ord)
                 ORDER BY t.ord
                 RETURNING item_id, position",
                &[
                    &collection_id,
                    &ins_def_ids,
                    &ins_fronts,
                    &ins_backs,
                    &ins_notes,
                    &ins_positions,
                    &ins_canonical,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            item_id_by_position.insert(row.get("position"), row.get("item_id"));
        }
    }

    // Pass 2, in input order: skip warnings and images (keeps the warning order).
    for (pos, item) in req.items.iter().enumerate() {
        if let Some(w) = skip_warning_by_index[pos].take() {
            warnings.push(w);
            skipped_count += 1;
            inserted_item_ids_by_export_index.push(None);
            continue;
        }
        let new_item_id = item_id_by_position[&(pos as i32)];

        for (side, url_option) in [
            ("front", &item.front_image_url),
            ("back", &item.back_image_url),
        ] {
            if let Some(url) = url_option {
                match decode_data_url(url) {
                    Ok((mime_type, image_data)) => {
                        if image_data.len() > MAX_ITEM_IMAGE_BYTES {
                            warnings.push(format!(
                                "Item at position {}: {} image exceeds {}MB after decode, skipped",
                                pos,
                                side,
                                MAX_ITEM_IMAGE_BYTES / (1024 * 1024)
                            ));
                        } else {
                            match get_or_insert_collection_image_id(
                                &transaction,
                                &image_data,
                                &mime_type,
                            )
                            .await
                            {
                                Ok(image_id) => {
                                    if let Err(e) = transaction
                                        .execute(
                                            "INSERT INTO collection_item_images (item_id, collection_image_id, side) VALUES ($1, $2, $3)",
                                            &[&new_item_id, &image_id, &side],
                                        )
                                        .await
                                    {
                                        warnings.push(format!(
                                            "Item at position {}: failed to attach {} image: {}",
                                            pos, side, e
                                        ));
                                    }
                                }
                                Err(e) => warnings.push(format!(
                                    "Item at position {}: failed to store {} image: {}",
                                    pos, side, e
                                )),
                            }
                        }
                    }
                    Err(e) => warnings.push(format!(
                        "Item at position {}: invalid {} image data URL: {}",
                        pos, side, e
                    )),
                }
            }
        }

        if let Some(url) = &item.sound_url {
            if let Err(e) = attach_imported_sound(&transaction, new_item_id, url).await {
                warnings.push(format!("Item at position {}: sound skipped: {}", pos, e));
            }
        }

        inserted_item_ids_by_export_index.push(Some(new_item_id));
        imported_count += 1;
    }

    // When no explicit levels array is provided, infer levels from per-item level_index/position_in_level
    let levels_to_use: Vec<LevelExport> = if !req.levels.is_empty() {
        req.levels.clone()
    } else {
        let mut level_to_items: std::collections::BTreeMap<u32, Vec<(usize, i32)>> =
            std::collections::BTreeMap::new();
        for (item_idx, item) in req.items.iter().enumerate() {
            if let Some(li) = item.level_index {
                let pos = item.position_in_level.unwrap_or(i32::MAX);
                level_to_items.entry(li).or_default().push((item_idx, pos));
            }
        }
        level_to_items
            .into_iter()
            .enumerate()
            .map(|(pos_index, (level_index, mut items))| {
                items.sort_by_key(|&(idx, pos)| (pos, idx));
                // Chain prerequisites: level 0 has none, level 1 requires 0, level 2 requires 1, etc.
                let prerequisite_positions: Vec<usize> = if pos_index == 0 {
                    vec![]
                } else {
                    vec![pos_index - 1]
                };
                LevelExport {
                    name: format!("Level {}", level_index + 1),
                    description: None,
                    min_cards: 5,
                    min_success_rate: 0.8,
                    position: level_index as i32,
                    prerequisite_positions,
                    item_positions: items.into_iter().map(|(idx, _)| idx).collect(),
                }
            })
            .collect()
    };

    let mut levels_created = 0i32;
    let mut quiz_flashcard_positions: Vec<i32> = Vec::new();
    let mut quiz_flashcard_ids: Vec<i32> = Vec::new();
    if !levels_to_use.is_empty() {
        // One flashcard per inserted item (position = export index), then the progress
        // rows for the sides each direction needs, each in a single statement.
        let mut fc_positions: Vec<i32> = Vec::new();
        let mut fc_item_ids: Vec<i32> = Vec::new();
        let mut fc_directions: Vec<FlashcardDirection> = Vec::new();
        let auto_directions = auto_mix_directions_for_import(&req.items, &levels_to_use);
        for (idx, item_id_opt) in inserted_item_ids_by_export_index.iter().enumerate() {
            if let Some(item_id) = *item_id_opt {
                fc_positions.push(idx as i32);
                fc_item_ids.push(item_id);
                let explicit = req.items.get(idx).and_then(|i| i.direction.as_ref());
                let direction = match (explicit, auto_directions.get(&idx)) {
                    (None, Some(auto)) => parse_export_direction(Some(&auto.to_string())),
                    _ => parse_export_direction(explicit),
                };
                if is_quiz_direction(&direction) {
                    quiz_flashcard_positions.push(idx as i32);
                }
                fc_directions.push(direction);
            }
        }
        let mut flashcard_id_by_position: HashMap<i32, i32> = HashMap::new();
        if !fc_positions.is_empty() {
            for row in transaction
                .query(
                    "INSERT INTO flashcards (collection_id, position, item_id, direction)
                     SELECT $1, t.position, t.item_id, t.direction::flashcard_direction
                     FROM UNNEST($2::int[], $3::int[], $4::text[]) WITH ORDINALITY AS t(position, item_id, direction, ord)
                     ORDER BY t.ord
                     RETURNING id, position",
                    &[&collection_id, &fc_positions, &fc_item_ids, &fc_directions],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
            {
                flashcard_id_by_position.insert(row.get("position"), row.get("id"));
            }
        }
        quiz_flashcard_ids.extend(
            quiz_flashcard_positions
                .iter()
                .filter_map(|p| flashcard_id_by_position.get(p).copied()),
        );
        // Seed progress sides to match direction (same as create_flashcard).
        let mut progress_ids: Vec<i32> = Vec::new();
        let mut progress_sides: Vec<&str> = Vec::new();
        for (position, direction) in fc_positions.iter().zip(&fc_directions) {
            let flashcard_id = flashcard_id_by_position[position];
            let sides: &[&str] = match direction {
                FlashcardDirection::Reverse
                | FlashcardDirection::FillInReverse
                | FlashcardDirection::WordbankReverse
                | FlashcardDirection::ListenWordbankReverse => &["reverse"],
                FlashcardDirection::Both
                | FlashcardDirection::FillInBoth
                | FlashcardDirection::WordbankBoth
                | FlashcardDirection::ListenWordbankBoth => &["direct", "reverse"],
                _ => &["direct"],
            };
            for side in sides {
                progress_ids.push(flashcard_id);
                progress_sides.push(side);
            }
        }
        if !progress_ids.is_empty() {
            transaction
                .execute(
                    "INSERT INTO user_flashcard_progress
                     (user_id, flashcard_id, card_side, status, next_review_at)
                     SELECT $1, t.flashcard_id, t.card_side, 'new', CURRENT_TIMESTAMP
                     FROM UNNEST($2::int[], $3::text[]) AS t(flashcard_id, card_side)
                     ON CONFLICT (user_id, flashcard_id, card_side) WHERE NOT archived DO NOTHING",
                    &[&user_id, &progress_ids, &progress_sides],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        let flashcard_id_by_export_index: Vec<Option<i32>> = (0..inserted_item_ids_by_export_index
            .len())
            .map(|idx| flashcard_id_by_position.get(&(idx as i32)).copied())
            .collect();

        let mut new_level_ids: Vec<i32> = Vec::with_capacity(levels_to_use.len());
        for level in &levels_to_use {
            let level_id: i32 = transaction
                .query_one(
                    "INSERT INTO flashcard_levels (collection_id, name, description, min_cards, min_success_rate, position)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     RETURNING level_id",
                    &[
                        &collection_id,
                        &level.name,
                        &level.description,
                        &level.min_cards,
                        &(level.min_success_rate as f64),
                        &level.position,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
                .try_get(0)
                .map_err(|e| AppError::Database(e.to_string()))?;
            new_level_ids.push(level_id);
            levels_created += 1;
        }

        // Prerequisites and level membership, one statement each. For a flashcard listed
        // twice in a level the last position wins, as with the old per-row upsert.
        let mut pre_level_ids: Vec<i32> = Vec::new();
        let mut pre_ids: Vec<i32> = Vec::new();
        let mut member_pos: HashMap<(i32, i32), i32> = HashMap::new();
        let mut member_order: Vec<(i32, i32)> = Vec::new();
        for (level_idx, level) in levels_to_use.iter().enumerate() {
            let level_id = new_level_ids[level_idx];
            for &prereq_idx in &level.prerequisite_positions {
                if prereq_idx < new_level_ids.len() {
                    let prereq_id = new_level_ids[prereq_idx];
                    if level_id != prereq_id {
                        pre_level_ids.push(level_id);
                        pre_ids.push(prereq_id);
                    }
                }
            }
            for (pos_in_level, &item_idx) in level.item_positions.iter().enumerate() {
                if let Some(Some(flashcard_id)) = flashcard_id_by_export_index.get(item_idx) {
                    let key = (level_id, *flashcard_id);
                    if member_pos.insert(key, pos_in_level as i32).is_none() {
                        member_order.push(key);
                    }
                }
            }
        }
        if !pre_level_ids.is_empty() {
            if let Err(e) = transaction
                .execute(
                    "INSERT INTO level_prerequisites (level_id, prerequisite_id)
                     SELECT * FROM UNNEST($1::int[], $2::int[])
                     ON CONFLICT (level_id, prerequisite_id) DO NOTHING",
                    &[&pre_level_ids, &pre_ids],
                )
                .await
            {
                warnings.push(format!("Failed to save level prerequisites: {}", e));
            }
        }
        if !member_order.is_empty() {
            let m_levels: Vec<i32> = member_order.iter().map(|k| k.0).collect();
            let m_cards: Vec<i32> = member_order.iter().map(|k| k.1).collect();
            let m_pos: Vec<i32> = member_order.iter().map(|k| member_pos[k]).collect();
            if let Err(e) = transaction
                .execute(
                    "INSERT INTO flashcard_level_items (level_id, flashcard_id, position)
                     SELECT * FROM UNNEST($1::int[], $2::int[], $3::int[])
                     ON CONFLICT (level_id, flashcard_id) DO UPDATE SET position = EXCLUDED.position",
                    &[&m_levels, &m_cards, &m_pos],
                )
                .await
            {
                warnings.push(format!("Failed to save level memberships: {}", e));
            }
        }
    }

    let username: String = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection_resp = CollectionResponse {
        collection_id,
        name: sanitized_name,
        description: sanitized_description,
        is_public,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        item_count: imported_count as i64,
        has_flashcards: false,
        has_cover_image: false,
        has_collection_image: false,
        comment_count: 0,
        monthly_drill_users: 0,
        owner: CollectionOwner { user_id, username },
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    // Quiz cards need their correct answer stored, or the learner gets no options.
    // Seeded after commit (the helper opens its own transaction); a failure only warns.
    for flashcard_id in quiz_flashcard_ids {
        if let Err(e) =
            crate::flashcards::generate_and_set_quiz_options(pool, user_id, flashcard_id).await
        {
            warnings.push(format!(
                "Could not seed quiz options for flashcard {}: {}",
                flashcard_id, e
            ));
        }
    }

    Ok(ImportFullResponse {
        collection: collection_resp,
        imported_count,
        skipped_count,
        levels_created,
        warnings,
    })
}

/// Full collection export: collection metadata, items (with flashcard direction when present), and levels.
pub async fn export_collection_full(
    pool: &Pool,
    collection_id: i32,
    user_id: Option<i32>,
) -> AppResult<CollectionFullExport> {
    let collection_resp = get_collection(pool, collection_id, user_id).await?;
    let collection_meta = CollectionExportMeta {
        name: collection_resp.name,
        description: collection_resp.description,
        is_public: Some(collection_resp.is_public),
    };

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item_rows = client
        .query(
            "SELECT
                ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
                ci.free_content_front, ci.free_content_back,
                ci.langid as language_id, ci.owner_user_id, ci.license,
                v.word, d.definition, d.notes as definition_notes, d.jargon, t.descriptor as word_type,
                c.rafsi, c.experimental_rafsi, c.selmaho,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'front') as front_image_mime,
                (SELECT img.image_data FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_data,
                (SELECT img.mime_type FROM collection_item_images cii
                    INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
                    WHERE cii.item_id = ci.item_id AND cii.side = 'back') as back_image_mime,
                (SELECT s.sound_data FROM collection_item_sounds s WHERE s.item_id = ci.item_id) as sound_data,
                (SELECT s.mime_type FROM collection_item_sounds s WHERE s.item_id = ci.item_id) as sound_mime,
                f.direction::text as flashcard_direction
            FROM collection_items ci
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN valsitypes t ON v.typeid = t.typeid
            LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid
            LEFT JOIN flashcards f ON f.item_id = ci.item_id AND f.collection_id = ci.collection_id
            WHERE ci.collection_id = $1
            ORDER BY ci.position",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut item_id_to_index: std::collections::HashMap<i32, usize> =
        std::collections::HashMap::new();
    let items: Vec<CollectionExportItem> = item_rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let item_id: i32 = row.get("item_id");
            item_id_to_index.insert(item_id, idx);
            let front_image_url =
                row.get::<_, Option<Vec<u8>>>("front_image_data")
                    .and_then(|data| {
                        row.get::<_, Option<String>>("front_image_mime")
                            .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(&data)))
                    });
            let back_image_url =
                row.get::<_, Option<Vec<u8>>>("back_image_data")
                    .and_then(|data| {
                        row.get::<_, Option<String>>("back_image_mime")
                            .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(&data)))
                    });
            let sound_url = row
                .get::<_, Option<Vec<u8>>>("sound_data")
                .and_then(|data| {
                    row.get::<_, Option<String>>("sound_mime")
                        .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(&data)))
                });
            let direction: Option<String> = row.get("flashcard_direction");
            CollectionExportItem {
                item_id: row.get("item_id"),
                position: row.get("position"),
                collection_note: row.get("collection_note"),
                definition_id: row.get("definition_id"),
                language_id: row.get("language_id"),
                owner_user_id: row.get("owner_user_id"),
                license: row.get("license"),
                word: row.get("word"),
                word_type: row.get("word_type"),
                rafsi: row.get("rafsi"),
                experimental_rafsi: row.try_get("experimental_rafsi").ok().flatten(),
                selmaho: row.get("selmaho"),
                definition: row.get("definition"),
                definition_notes: row.get("definition_notes"),
                jargon: row.get("jargon"),
                free_content_front: row.get("free_content_front"),
                free_content_back: row.get("free_content_back"),
                front_image_url,
                back_image_url,
                sound_url,
                direction,
                level_index: None,
                position_in_level: None,
            }
        })
        .collect();

    let level_rows = client
        .query(
            "SELECT level_id, name, description, min_cards, min_success_rate, position
             FROM flashcard_levels
             WHERE collection_id = $1
             ORDER BY position",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let level_ids: Vec<i32> = level_rows.iter().map(|r| r.get("level_id")).collect();
    let level_id_to_index: std::collections::HashMap<i32, usize> = level_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    // Load prerequisites and level items for every level up front (two queries
    // in total) instead of 2 + items queries per level.
    let mut prereqs_by_level: HashMap<i32, Vec<i32>> = HashMap::new();
    let mut item_ids_by_level: HashMap<i32, Vec<i32>> = HashMap::new();
    if !level_ids.is_empty() {
        for r in client
            .query(
                "SELECT level_id, prerequisite_id FROM level_prerequisites
                 WHERE level_id = ANY($1)",
                &[&level_ids],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            prereqs_by_level
                .entry(r.get("level_id"))
                .or_default()
                .push(r.get("prerequisite_id"));
        }
        for r in client
            .query(
                "SELECT fli.level_id, f.item_id
                 FROM flashcard_level_items fli
                 JOIN flashcards f ON f.id = fli.flashcard_id
                 WHERE fli.level_id = ANY($1)
                 ORDER BY fli.level_id, fli.position",
                &[&level_ids],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        {
            item_ids_by_level
                .entry(r.get("level_id"))
                .or_default()
                .push(r.get("item_id"));
        }
    }

    let mut levels: Vec<LevelExport> = Vec::with_capacity(level_rows.len());
    for row in &level_rows {
        let level_id: i32 = row.get("level_id");
        let prerequisite_positions: Vec<usize> = prereqs_by_level
            .get(&level_id)
            .into_iter()
            .flatten()
            .filter_map(|pid| level_id_to_index.get(pid).copied())
            .collect();
        let item_positions_for_level: Vec<usize> = item_ids_by_level
            .get(&level_id)
            .into_iter()
            .flatten()
            .filter_map(|item_id| item_id_to_index.get(item_id).copied())
            .collect();

        levels.push(LevelExport {
            name: row.get("name"),
            description: row.get("description"),
            min_cards: row.get("min_cards"),
            min_success_rate: row.get::<_, f64>("min_success_rate") as f32,
            position: row.get("position"),
            prerequisite_positions,
            item_positions: item_positions_for_level,
        });
    }

    Ok(CollectionFullExport {
        collection: collection_meta,
        items,
        levels,
    })
}

// Helper function to initialize flashcard progress
async fn initialize_flashcard_progress(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> AppResult<()> {
    transaction
        .execute(
            "INSERT INTO user_flashcard_progress
             (user_id, flashcard_id, card_side, status, next_review_at)
             VALUES ($1, $2, $3, 'new', CURRENT_TIMESTAMP)
             ON CONFLICT (user_id, flashcard_id, card_side) WHERE NOT archived DO NOTHING",
            &[&user_id, &flashcard_id, &side],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn upsert_item(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
    req: &AddItemRequest,
) -> AppResult<CollectionItemResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection ownership
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Validate images if present
    if let Some(img) = &req.front_image {
        validate_item_image(img).map_err(|e| AppError::BadRequest(e.clone()))?;
    }
    if let Some(img) = &req.back_image {
        validate_item_image(img).map_err(|e| AppError::BadRequest(e.clone()))?;
    }

    // Get highest current position
    let max_position: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let sanitized_notes = req.notes.as_ref().map(|n| sanitize_html(n));
    let sanitized_front = req.free_content_front.as_ref().map(|f| sanitize_html(f));
    let sanitized_back = req.free_content_back.as_ref().map(|b| sanitize_html(b));

    // Check if item exists either by specified ID or definition ID
    let existing_item = if let Some(item_id) = req.item_id {
        transaction
            .query_opt(
                "SELECT item_id, notes, added_at, position 
                 FROM collection_items 
                 WHERE collection_id = $1 AND item_id = $2",
                &[&collection_id, &item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else if let Some(def_id) = req.definition_id {
        transaction
            .query_opt(
                "SELECT item_id, notes, added_at, position 
                 FROM collection_items 
                 WHERE collection_id = $1 AND definition_id = $2",
                &[&collection_id, &def_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        None
    };

    // Use explicit position, or when updating existing item keep its current position; otherwise append
    let position = req
        .position
        .or_else(|| {
            existing_item
                .as_ref()
                .map(|row| row.get::<_, i32>("position"))
        })
        .unwrap_or(max_position + 1);

    // Validate item_id exists if provided
    if req.item_id.is_some() && existing_item.is_none() {
        return Err(AppError::NotFound("Item not found".to_string()));
    }

    // Create or update item
    let mut canonical_form = sanitized_front
        .as_ref()
        .and_then(|front| crate::utils::canonical::get_canonical_form(front));

    // For dictionary items without free content, try to use the word from the definition
    if canonical_form.is_none() {
        if let Some(def_id) = req.definition_id {
            if let Ok(row) = transaction.query_one(
                "SELECT v.word FROM definitions d JOIN valsi v ON d.valsiid = v.valsiid WHERE d.definitionid = $1",
                &[&def_id]
            ).await {
                let word: String = row.get(0);
                canonical_form = crate::utils::canonical::get_canonical_form(&word);
            }
        }
    }

    let (item_id, notes, added_at): (i32, Option<String>, DateTime<Utc>) =
        if let Some(row) = existing_item {
            let item_id: i32 = row.get("item_id");
            // Update existing item
            let old_position: i32 = row.get("position");

            if position != old_position {
                // Shift items if position changed
                let (start, end, shift) = if position > old_position {
                    (old_position + 1, position + 1, -1)
                } else {
                    (position, old_position, 1)
                };

                transaction
                    .execute(
                        "UPDATE collection_items 
                     SET position = position + $1
                     WHERE collection_id = $2 
                     AND position >= $3 AND position < $4
                     AND item_id != $5",
                        &[
                            &shift as &(dyn tokio_postgres::types::ToSql + Sync),
                            &collection_id,
                            &start,
                            &end,
                            &item_id,
                        ],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }

            transaction
                .execute(
                    "UPDATE collection_items 
                 SET notes = $1, position = $2, canonical_form = $3,
                     free_content_front = $4, free_content_back = $5
                 WHERE item_id = $6",
                    &[
                        &sanitized_notes,
                        &position,
                        &canonical_form,
                        &sanitized_front,
                        &sanitized_back,
                        &item_id,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            (
                item_id,
                sanitized_notes,
                row.get::<_, DateTime<Utc>>("added_at"),
            )
        } else {
            // Add new item
            let row = transaction
                .query_one(
                    "INSERT INTO collection_items (
                    collection_id, definition_id, 
                    free_content_front, free_content_back, 
                    langid, owner_user_id, license, script, is_original,
                    notes, position, auto_progress, canonical_form
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING item_id, added_at",
                    &[
                        &collection_id,
                        &req.definition_id,
                        &sanitized_front,
                        &sanitized_back,
                        &req.language_id,
                        &req.owner_user_id,
                        &req.license,
                        &req.script,
                        &req.is_original.unwrap_or(true),
                        &sanitized_notes,
                        &position,
                        &req.auto_progress.unwrap_or(true),
                        &canonical_form,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            (
                row.get::<_, i32>("item_id"),
                sanitized_notes,
                row.get::<_, DateTime<Utc>>("added_at"),
            )
        };

    // Handle front image
    if let Some(image) = &req.front_image {
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid front image base64: {}", e)))?;
        let image_id =
            get_or_insert_collection_image_id(&transaction, &image_data, &image.mime_type).await?;
        transaction
            .execute(
                "INSERT INTO collection_item_images (item_id, collection_image_id, side)
                 VALUES ($1, $2, 'front')
                 ON CONFLICT (item_id, side) DO UPDATE SET
                   collection_image_id = EXCLUDED.collection_image_id",
                &[&item_id, &image_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle back image
    if let Some(image) = &req.back_image {
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid back image base64: {}", e)))?;
        let image_id =
            get_or_insert_collection_image_id(&transaction, &image_data, &image.mime_type).await?;
        transaction
            .execute(
                "INSERT INTO collection_item_images (item_id, collection_image_id, side)
                 VALUES ($1, $2, 'back')
                 ON CONFLICT (item_id, side) DO UPDATE SET
                   collection_image_id = EXCLUDED.collection_image_id",
                &[&item_id, &image_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle sound: overwrite (delete then insert) or remove
    if req.remove_sound.unwrap_or(false) || req.sound.is_some() {
        transaction
            .execute(
                "DELETE FROM collection_item_sounds WHERE item_id = $1",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }
    if let Some(sound) = &req.sound {
        validate_item_audio(sound).map_err(|e| AppError::BadRequest(e.clone()))?;
        let sound_data = BASE64
            .decode(&sound.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid sound base64: {}", e)))?;
        transaction
            .execute(
                "INSERT INTO collection_item_sounds (item_id, sound_data, mime_type)
                 VALUES ($1, $2, $3)",
                &[&item_id, &sound_data, &sound.mime_type],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle flashcard creation/update
    if req.direction.is_some() {
        // Get existing flashcard if it exists
        let existing_flashcard = transaction
            .query_opt(
                "SELECT id, direction FROM flashcards 
                 WHERE collection_id = $1 AND item_id = $2",
                &[&collection_id, &item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Parse requested direction
        let direction = if let Some(dir_str) = &req.direction {
            match dir_str.to_lowercase().as_str() {
                "direct" => FlashcardDirection::Direct,
                "reverse" => FlashcardDirection::Reverse,
                "fillin" => FlashcardDirection::FillIn,
                "fillin_reverse" => FlashcardDirection::FillInReverse,
                "fillin_both" => FlashcardDirection::FillInBoth,
                "wordbank" => FlashcardDirection::Wordbank,
                "wordbank_reverse" => FlashcardDirection::WordbankReverse,
                "wordbank_both" => FlashcardDirection::WordbankBoth,
                "listen_wordbank" => FlashcardDirection::ListenWordbank,
                "listen_wordbank_reverse" => FlashcardDirection::ListenWordbankReverse,
                "listen_wordbank_both" => FlashcardDirection::ListenWordbankBoth,
                "just_information" => FlashcardDirection::JustInformation,
                _ => FlashcardDirection::Both, // Default to Both if unspecified or invalid
            }
        } else {
            // Default direction if not specified in request
            FlashcardDirection::Both
        };

        match existing_flashcard {
            Some(row) => {
                let existing_id: i32 = row.get("id");
                let existing_direction: FlashcardDirection = row.get("direction");

                // Handle direction change
                if existing_direction != direction {
                    // Archive existing progress
                    transaction
                        .execute(
                            "UPDATE user_flashcard_progress
                             SET archived = true
                             WHERE flashcard_id = $1 AND user_id = $2 AND NOT archived",
                            &[&existing_id, &user_id],
                        )
                        .await
                        .map_err(|e| AppError::Database(e.to_string()))?;

                    // Update flashcard direction
                    transaction
                        .execute(
                            "UPDATE flashcards SET direction = $1 WHERE id = $2",
                            &[&direction, &existing_id],
                        )
                        .await
                        .map_err(|e| AppError::Database(e.to_string()))?;

                    // Store correct answer for quiz-image when switching to image-quiz
                    if matches!(
                        direction,
                        FlashcardDirection::QuizImageDirect
                            | FlashcardDirection::QuizImageReverse
                            | FlashcardDirection::QuizImageBoth
                    ) {
                        let item_row = transaction
                            .query_one(
                                "SELECT item_id FROM flashcards WHERE id = $1",
                                &[&existing_id],
                            )
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                        let flashcard_item_id: i32 = item_row.get("item_id");
                        let correct = match direction {
                            FlashcardDirection::QuizImageDirect => {
                                format!("{}:front", flashcard_item_id)
                            }
                            FlashcardDirection::QuizImageReverse => {
                                format!("{}:back", flashcard_item_id)
                            }
                            FlashcardDirection::QuizImageBoth => flashcard_item_id.to_string(),
                            _ => unreachable!(),
                        };
                        let _ = transaction
                            .execute(
                                "INSERT INTO flashcard_quiz_options (flashcard_id, correct_answer_text)
                                 VALUES ($1, $2)
                                 ON CONFLICT (flashcard_id) DO UPDATE SET correct_answer_text = EXCLUDED.correct_answer_text",
                                &[&existing_id, &correct],
                            )
                            .await;
                    }

                    // Initialize new progress based on new direction
                    match direction {
                        FlashcardDirection::Direct => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?
                        }
                        FlashcardDirection::Reverse => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?
                        }
                        FlashcardDirection::Both => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?;
                        }
                        FlashcardDirection::JustInformation => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                            mark_progress_graduated(&transaction, user_id, existing_id, "direct")
                                .await?;
                        }
                        FlashcardDirection::FillIn
                        | FlashcardDirection::Wordbank
                        | FlashcardDirection::ListenWordbank => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                        }
                        FlashcardDirection::FillInReverse
                        | FlashcardDirection::WordbankReverse
                        | FlashcardDirection::ListenWordbankReverse => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?;
                        }
                        FlashcardDirection::FillInBoth
                        | FlashcardDirection::WordbankBoth
                        | FlashcardDirection::ListenWordbankBoth => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?;
                        }
                        FlashcardDirection::QuizDirect
                        | FlashcardDirection::QuizReverse
                        | FlashcardDirection::QuizBoth
                        | FlashcardDirection::QuizImageDirect
                        | FlashcardDirection::QuizImageReverse
                        | FlashcardDirection::QuizImageBoth => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?;
                        }
                    }
                }
                existing_id
            }
            None => {
                // Create new flashcard
                let max_position: i32 = transaction
                    .query_one(
                        "SELECT COALESCE(MAX(position), -1) FROM flashcards WHERE collection_id = $1",
                        &[&collection_id],
                    )
                    .await.map_err(|e| AppError::Database(e.to_string()))?
                    .try_get(0).map_err(|e| AppError::Database(e.to_string()))?;

                let row = transaction
                    .query_one(
                        "INSERT INTO flashcards (
                            collection_id, item_id, position, direction
                        )
                        VALUES ($1, $2, $3, $4)
                        RETURNING id",
                        &[&collection_id, &item_id, &(max_position + 1), &direction],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;

                let new_id: i32 = row.get("id");

                // Store correct answer for quiz-image types so study can show image options
                if matches!(
                    direction,
                    FlashcardDirection::QuizImageDirect
                        | FlashcardDirection::QuizImageReverse
                        | FlashcardDirection::QuizImageBoth
                ) {
                    let correct = match direction {
                        FlashcardDirection::QuizImageDirect => format!("{}:front", item_id),
                        FlashcardDirection::QuizImageReverse => format!("{}:back", item_id),
                        FlashcardDirection::QuizImageBoth => item_id.to_string(),
                        _ => unreachable!(),
                    };
                    let _ = transaction
                        .execute(
                            "INSERT INTO flashcard_quiz_options (flashcard_id, correct_answer_text)
                             VALUES ($1, $2)
                             ON CONFLICT (flashcard_id) DO UPDATE SET correct_answer_text = EXCLUDED.correct_answer_text",
                            &[&new_id, &correct],
                        )
                        .await;
                }

                // Initialize progress based on direction
                match direction {
                    FlashcardDirection::Direct => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                    }
                    FlashcardDirection::Reverse => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                    FlashcardDirection::Both => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                    FlashcardDirection::FillIn
                    | FlashcardDirection::Wordbank
                    | FlashcardDirection::ListenWordbank => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                    }
                    FlashcardDirection::FillInReverse
                    | FlashcardDirection::WordbankReverse
                    | FlashcardDirection::ListenWordbankReverse => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                    FlashcardDirection::FillInBoth
                    | FlashcardDirection::WordbankBoth
                    | FlashcardDirection::ListenWordbankBoth => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                    FlashcardDirection::JustInformation => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                        mark_progress_graduated(&transaction, user_id, new_id, "direct").await?;
                    }
                    FlashcardDirection::QuizDirect
                    | FlashcardDirection::QuizReverse
                    | FlashcardDirection::QuizBoth
                    | FlashcardDirection::QuizImageDirect
                    | FlashcardDirection::QuizImageReverse
                    | FlashcardDirection::QuizImageBoth => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                }
                new_id
            }
        };
    }

    // Resolve has_sound from DB after any sound updates
    let has_sound: bool = transaction
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = $1)",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .get(0);

    // Get item details
    let response = if let Some(def_id) = req.definition_id {
        // Get definition details
        let def_row = transaction
            .query_one(
                "SELECT d.*, v.word, v.valsiid, u.username
                 FROM definitions d
                 JOIN valsi v ON d.valsiid = v.valsiid
                 JOIN users u ON d.userid = u.userid
                 WHERE d.definitionid = $1",
                &[&def_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        CollectionItemResponse {
            item_id,
            definition_id: Some(def_id),
            word: Some(def_row.get("word")),
            definition: Some(def_row.get("definition")),
            username: Some(def_row.get("username")),
            valsi_id: Some(def_row.get("valsiid")),
            lang_id: Some(def_row.get("langid")),
            free_content_front: None,
            free_content_back: None,
            notes: def_row.get("notes"),
            language_id: req.language_id,
            owner_user_id: req.owner_user_id,
            license: req.license.clone(),
            script: req.script.clone(),
            is_original: req.is_original.unwrap_or(false),
            ci_notes: notes,
            position,
            auto_progress: req.auto_progress.unwrap_or(true),
            added_at,
            has_front_image: req.front_image.is_some(),
            has_back_image: req.back_image.is_some(),
            has_sound,
            sound_url: if has_sound {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id, item_id
                ))
            } else {
                None
            },
            canonical_form,
            flashcard: None,
            collection_id: None,
            collection_name: None,
            source_langid: None,
            collection_created_at: None,
            match_count: None,
        }
    } else {
        // Free content item
        CollectionItemResponse {
            item_id,
            definition_id: None,
            word: None,
            definition: None,
            username: None,
            valsi_id: None,
            lang_id: None,
            free_content_front: sanitized_front,
            free_content_back: sanitized_back,
            notes: None,
            language_id: req.language_id,
            owner_user_id: req.owner_user_id,
            license: req.license.clone(),
            script: req.script.clone(),
            is_original: req.is_original.unwrap_or(true),
            ci_notes: notes,
            position,
            auto_progress: req.auto_progress.unwrap_or(true),
            added_at,
            has_front_image: req.front_image.is_some(),
            has_back_image: req.back_image.is_some(),
            has_sound,
            sound_url: if has_sound {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id, item_id
                ))
            } else {
                None
            },
            canonical_form,
            flashcard: None,
            collection_id: None,
            collection_name: None,
            source_langid: None,
            collection_created_at: None,
            match_count: None,
        }
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;
    Ok(response)
}

async fn restore_or_initialize_progress(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> AppResult<()> {
    // Check for archived progress
    let archived_exists = transaction
        .query_opt(
            "SELECT 1 FROM user_flashcard_progress
             WHERE user_id = $1 AND flashcard_id = $2
             AND card_side = $3 AND archived = true",
            &[&user_id, &flashcard_id, &side],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .is_some();

    if archived_exists {
        // Unarchive existing progress
        transaction
            .execute(
                "UPDATE user_flashcard_progress
                 SET archived = false
                 WHERE user_id = $1 AND flashcard_id = $2
                 AND card_side = $3 AND archived = true",
                &[&user_id, &flashcard_id, &side],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    } else {
        // Initialize new progress
        initialize_flashcard_progress(transaction, user_id, flashcard_id, side).await?;
    }

    Ok(())
}

pub async fn update_item_position(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    item_id: i32,
    user_id: i32,
    new_position: i32,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // Get current item position
    let current_position: i32 = transaction
        .query_one(
            "SELECT position FROM collection_items 
             WHERE collection_id = $1 AND item_id = $2",
            &[&collection_id, &item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update positions
    let (start, end, shift) = if new_position > current_position {
        (current_position + 1, new_position + 1, -1)
    } else {
        (new_position, current_position, 1)
    };

    // Shift other items
    transaction
        .execute(
            "UPDATE collection_items 
             SET position = position + $1
             WHERE collection_id = $2 
             AND position >= $3 AND position < $4
             AND item_id != $5",
            &[
                &shift as &(dyn tokio_postgres::types::ToSql + Sync),
                &collection_id,
                &start,
                &end,
                &item_id,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update item position
    transaction
        .execute(
            "UPDATE collection_items 
             SET position = $1
             WHERE item_id = $2",
            &[&new_position, &item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update collection timestamp
    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;
    Ok(())
}

pub async fn remove_item(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    item_id: i32,
    user_id: i32,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // First delete any associated flashcard history and progress
    transaction
        .execute(
            "DELETE FROM flashcard_review_history
             WHERE flashcard_id IN (SELECT id FROM flashcards WHERE item_id = $1)",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .execute(
            "DELETE FROM user_flashcard_progress
             WHERE flashcard_id IN (SELECT id FROM flashcards WHERE item_id = $1)",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Then delete the flashcards
    transaction
        .execute(
            "DELETE FROM flashcards 
             WHERE item_id = $1",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Then remove the item and verify it belonged to the correct collection
    let result = transaction
        .execute(
            "DELETE FROM collection_items 
             WHERE item_id = $1 AND collection_id = $2",
            &[&item_id, &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result == 0 {
        return Err(AppError::NotFound("Item not found".to_string()));
    }

    // Update collection's updated_at timestamp
    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;
    Ok(())
}

/// Removes many collection items in one transaction (flashcard history, progress, flashcards, then rows).
/// Same semantics as repeated [`remove_item`], but atomic and bounded by [`MAX_BULK_REMOVE_ITEMS`].
pub async fn remove_items_bulk(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
    item_ids: &[i32],
) -> AppResult<BulkRemoveItemsResponse> {
    const MAX_BULK_REMOVE_ITEMS: usize = 500;

    let unique_set: HashSet<i32> = item_ids.iter().copied().filter(|&id| id > 0).collect();
    if unique_set.is_empty() {
        return Err(AppError::BadRequest(
            "item_ids must contain at least one positive id".to_string(),
        ));
    }
    if unique_set.len() > MAX_BULK_REMOVE_ITEMS {
        return Err(AppError::BadRequest(format!(
            "At most {} items per request",
            MAX_BULK_REMOVE_ITEMS
        )));
    }

    let mut unique: Vec<i32> = unique_set.into_iter().collect();
    unique.sort_unstable();

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    let rows = transaction
        .query(
            "SELECT item_id FROM collection_items WHERE collection_id = $1 AND item_id = ANY($2)",
            &[&collection_id, &unique],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if rows.len() != unique.len() {
        return Err(AppError::BadRequest(
            "One or more items are not in this collection".to_string(),
        ));
    }

    transaction
        .execute(
            "DELETE FROM flashcard_review_history
             WHERE flashcard_id IN (SELECT id FROM flashcards WHERE item_id = ANY($1))",
            &[&unique],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .execute(
            "DELETE FROM user_flashcard_progress
             WHERE flashcard_id IN (SELECT id FROM flashcards WHERE item_id = ANY($1))",
            &[&unique],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .execute("DELETE FROM flashcards WHERE item_id = ANY($1)", &[&unique])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let deleted = transaction
        .execute(
            "DELETE FROM collection_items WHERE collection_id = $1 AND item_id = ANY($2)",
            &[&collection_id, &unique],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if deleted as usize != unique.len() {
        return Err(AppError::Database(
            "Bulk delete removed fewer rows than expected".to_string(),
        ));
    }

    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(BulkRemoveItemsResponse {
        deleted: unique.len() as i32,
    })
}

/// Bulk-copies existing dictionary definitions into a collection.
///
/// Backing the "Add all to collection" action that sweeps every page of a dictionary
/// search result. Idempotent by `(collection_id, definition_id)`: definitions that
/// already exist in the collection are counted as `skipped` and not duplicated.
///
/// Definitions that do not exist in `definitions` are returned in
/// `invalid_definition_ids` but do not fail the request. The auto_progress flag is set
/// to true (same default as single-add).
pub async fn add_items_bulk_by_definition_ids(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
    definition_ids: &[i32],
    notes: Option<&str>,
) -> AppResult<BulkAddDefinitionsResponse> {
    let unique_set: HashSet<i32> = definition_ids
        .iter()
        .copied()
        .filter(|&id| id > 0)
        .collect();
    if unique_set.is_empty() {
        return Err(AppError::BadRequest(
            "definition_ids must contain at least one positive id".to_string(),
        ));
    }

    let mut unique: Vec<i32> = unique_set.into_iter().collect();
    unique.sort_unstable();

    let sanitized_notes = notes.map(sanitize_html);

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Split requested ids into (valid definition ids present in DB) and invalid ones.
    let existing_def_rows = transaction
        .query(
            "SELECT d.definitionid, v.word
               FROM definitions d
               JOIN valsi v ON v.valsiid = d.valsiid
              WHERE d.definitionid = ANY($1)",
            &[&unique],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut valid_ids: Vec<i32> = Vec::with_capacity(existing_def_rows.len());
    let mut word_by_def_id: HashMap<i32, String> = HashMap::with_capacity(existing_def_rows.len());
    for row in &existing_def_rows {
        let did: i32 = row.get("definitionid");
        let word: String = row.get("word");
        word_by_def_id.insert(did, word);
        valid_ids.push(did);
    }
    valid_ids.sort_unstable();

    let valid_set: HashSet<i32> = valid_ids.iter().copied().collect();
    let invalid_definition_ids: Vec<i32> = unique
        .iter()
        .copied()
        .filter(|id| !valid_set.contains(id))
        .collect();

    if valid_ids.is_empty() {
        transaction
            .rollback()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        return Ok(BulkAddDefinitionsResponse {
            added: 0,
            skipped: 0,
            invalid_definition_ids,
        });
    }

    // Existing memberships so we report `skipped` and only insert new rows.
    let already_rows = transaction
        .query(
            "SELECT definition_id
               FROM collection_items
              WHERE collection_id = $1
                AND definition_id = ANY($2)",
            &[&collection_id, &valid_ids],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let already_set: HashSet<i32> = already_rows
        .into_iter()
        .map(|r| r.get::<_, i32>("definition_id"))
        .collect();

    let to_insert: Vec<i32> = valid_ids
        .iter()
        .copied()
        .filter(|id| !already_set.contains(id))
        .collect();
    let skipped_already_present = already_set.len() as i32;

    if to_insert.is_empty() {
        transaction
            .commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        return Ok(BulkAddDefinitionsResponse {
            added: 0,
            skipped: skipped_already_present,
            invalid_definition_ids,
        });
    }

    let max_position: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Release the DB connection during CPU-bound Lojban parsing. Holding a transaction
    // open while we spin up tersmu for thousands of words would both pin a pool slot
    // and risk a statement timeout. We'll re-verify ownership in the write transaction.
    transaction
        .rollback()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    drop(client);

    // Build the input for the tersmu batch (one element per row we're about to insert,
    // in the same order as `to_insert`).
    let words_for_canonical: Vec<Option<String>> = to_insert
        .iter()
        .map(|id| word_by_def_id.get(id).cloned())
        .collect();

    let canonical_forms: Vec<Option<String>> = tokio::task::spawn_blocking(move || {
        crate::utils::canonical::get_canonical_forms_batch(&words_for_canonical)
    })
    .await
    .map_err(|e| AppError::Internal(format!("canonical-form task failed: {e}")))?;

    // Positions are assigned contiguously after the current max. A concurrent
    // single-item add could race, but collection_items has UNIQUE(collection_id,
    // definition_id) so the worst case is a few position duplicates (not crashes).
    let positions: Vec<i32> = (0..to_insert.len() as i32)
        .map(|i| max_position + 1 + i)
        .collect();

    // Write phase: one round-trip, atomic, idempotent via ON CONFLICT. This replaces
    // the previous 5000-iteration `execute` loop.
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let write_tx = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&write_tx, collection_id, user_id).await?;

    let inserted_rows = write_tx
        .execute(
            "INSERT INTO collection_items (
                 collection_id, definition_id, notes, position,
                 auto_progress, canonical_form, is_original
             )
             SELECT $1, x.def_id, $2, x.pos, true, x.canonical, true
               FROM UNNEST($3::int4[], $4::int4[], $5::text[])
                    AS x(def_id, pos, canonical)
             ON CONFLICT (collection_id, definition_id) DO NOTHING",
            &[
                &collection_id,
                &sanitized_notes,
                &to_insert,
                &positions,
                &canonical_forms,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    write_tx
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    write_tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    let added = inserted_rows as i32;
    // Rows we *planned* to insert but `ON CONFLICT` skipped — a concurrent add
    // beat us to them, so count them as skipped for an accurate response.
    let raced_skipped = to_insert.len() as i32 - added;
    let skipped = skipped_already_present + raced_skipped.max(0);

    Ok(BulkAddDefinitionsResponse {
        added,
        skipped,
        invalid_definition_ids,
    })
}

pub async fn clone_collection(
    pool: &Pool,
    redis: &RedisCache,
    source_collection_id: i32,
    user_id: i32,
) -> AppResult<CollectionResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    crate::auth_utils::verify_collection_read_access(
        &transaction,
        source_collection_id,
        Some(user_id),
    )
    .await?;

    // Get source collection (including shared cover blob reference)
    let source = transaction
        .query_one(
            "SELECT name, description, is_public, cover_collection_image_id
             FROM collections
             WHERE collection_id = $1",
            &[&source_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Create new collection (reuse source cover_collection_image_id — no blob copy)
    let new_collection = transaction
        .query_one(
            "INSERT INTO collections (user_id, name, description, is_public, cover_collection_image_id)
             VALUES ($1, $2, $3, false, $4)
             RETURNING collection_id, created_at, updated_at",
            &[
                &user_id,
                &format!("Copy of {}", source.get::<_, String>("name")),
                &source.get::<_, Option<String>>("description"),
                &source.get::<_, Option<i32>>("cover_collection_image_id"),
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let new_collection_id: i32 = new_collection.get("collection_id");

    // Copy items that have either definition_id or free content (stable order for pairing with clones)
    transaction
        .execute(
            "INSERT INTO collection_items (collection_id, definition_id,
                free_content_front, free_content_back,
                langid, owner_user_id, license, script, is_original,
                notes, position, auto_progress, canonical_form)
            SELECT $1, definition_id,
                   free_content_front, free_content_back,
                   langid, owner_user_id, license, script, is_original,
                   notes, position, auto_progress, canonical_form
            FROM collection_items
            WHERE collection_id = $2
            AND (definition_id IS NOT NULL
                 OR free_content_front IS NOT NULL
                 OR free_content_back IS NOT NULL)
            ORDER BY position NULLS LAST, item_id",
            &[&new_collection_id, &source_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Reuse the same collection_image_id rows for card images (no BYTEA duplication)
    transaction
        .execute(
            "INSERT INTO collection_item_images (item_id, side, collection_image_id)
             SELECT paired.new_item_id, cii.side, cii.collection_image_id
             FROM collection_item_images cii
             INNER JOIN (
                 WITH old_items AS (
                     SELECT item_id,
                            row_number() OVER (ORDER BY position NULLS LAST, item_id) AS rn
                     FROM collection_items
                     WHERE collection_id = $1
                       AND (definition_id IS NOT NULL
                            OR free_content_front IS NOT NULL
                            OR free_content_back IS NOT NULL)
                 ),
                 new_items AS (
                     SELECT item_id,
                            row_number() OVER (ORDER BY position NULLS LAST, item_id) AS rn
                     FROM collection_items
                     WHERE collection_id = $2
                       AND (definition_id IS NOT NULL
                            OR free_content_front IS NOT NULL
                            OR free_content_back IS NOT NULL)
                 )
                 SELECT o.item_id AS old_item_id, n.item_id AS new_item_id
                 FROM old_items o
                 INNER JOIN new_items n ON n.rn = o.rn
             ) AS paired ON paired.old_item_id = cii.item_id",
            &[&source_collection_id, &new_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let username = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item_count = transaction
        .query_one(
            "SELECT COUNT(*) FROM collection_items WHERE collection_id = $1",
            &[&new_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get::<_, i64>(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let image_flags = transaction
        .query_one(
            "SELECT (c.cover_collection_image_id IS NOT NULL) AS has_cover_image,
                    (c.cover_collection_image_id IS NOT NULL OR EXISTS (
                        SELECT 1 FROM collection_item_images cii
                        INNER JOIN collection_items ci2 ON ci2.item_id = cii.item_id
                        WHERE ci2.collection_id = $1 AND cii.side IN ('front', 'back')
                    )) AS has_collection_image
             FROM collections c WHERE c.collection_id = $1",
            &[&new_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(CollectionResponse {
        collection_id: new_collection_id,
        name: format!("Copy of {}", source.get::<_, String>("name")),
        description: source.get("description"),
        is_public: source.get("is_public"),
        created_at: new_collection.get("created_at"),
        updated_at: new_collection.get("updated_at"),
        item_count,
        has_flashcards: false,
        has_cover_image: image_flags.get("has_cover_image"),
        has_collection_image: image_flags.get("has_collection_image"),
        comment_count: 0,
        monthly_drill_users: 0,
        owner: CollectionOwner { user_id, username },
    })
}

pub async fn merge_collections(
    pool: &Pool,
    redis: &RedisCache,
    user_id: i32,
    req: &MergeCollectionsRequest,
) -> AppResult<CollectionResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check ownership of source collections
    for collection_id in &[req.source_collection_id, req.target_collection_id] {
        let owner_id: i32 = transaction
            .query_one(
                "SELECT user_id FROM collections WHERE collection_id = $1",
                &[collection_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .try_get("user_id")
            .map_err(|e| AppError::Database(e.to_string()))?;

        if owner_id != user_id {
            return Err(AppError::Unauthorized("Access denied".to_string()));
        }
    }

    // Create new collection if name provided, otherwise use target
    let target_id = if let Some(name) = &req.new_collection_name {
        let new_collection = transaction
            .query_one(
                "INSERT INTO collections (user_id, name)
                VALUES ($1, $2)
                RETURNING collection_id",
                &[&user_id, name],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        new_collection.get("collection_id")
    } else {
        req.target_collection_id
    };

    // Merge items handling duplicates
    transaction
        .execute(
            "INSERT INTO collection_items (collection_id, definition_id, notes, canonical_form)
            SELECT $1, definition_id, notes, canonical_form 
            FROM collection_items 
            WHERE collection_id = $2
            ON CONFLICT (collection_id, definition_id) DO NOTHING",
            &[&target_id, &req.source_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update target collection's timestamp
    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &target_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get collection details
    let collection_row = transaction
        .query_one(
            "SELECT c.*, u.userid, u.username,
            (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.collection_id) as item_count,
            EXISTS(SELECT 1 FROM flashcards f WHERE f.collection_id = c.collection_id) as has_flashcards,
            (c.cover_collection_image_id IS NOT NULL) as has_cover_image,
            (
                (c.cover_collection_image_id IS NOT NULL)
                OR EXISTS (
                    SELECT 1 FROM collection_item_images cii
                    INNER JOIN collection_items ci2 ON ci2.item_id = cii.item_id
                    WHERE ci2.collection_id = c.collection_id AND cii.side IN ('front', 'back')
                )
            ) as has_collection_image
            FROM collections c
            JOIN users u ON c.user_id = u.userid
            WHERE c.collection_id = $1",
            &[&target_id],
        )
        .await.map_err(|e| AppError::Database(e.to_string()))?;

    let result = CollectionResponse {
        collection_id: collection_row.get("collection_id"),
        name: collection_row.get("name"),
        description: collection_row.get("description"),
        is_public: collection_row.get("is_public"),
        created_at: collection_row.get("created_at"),
        updated_at: collection_row.get("updated_at"),
        item_count: collection_row.get("item_count"),
        has_flashcards: collection_row.get("has_flashcards"),
        has_cover_image: collection_row.get("has_cover_image"),
        has_collection_image: collection_row.get("has_collection_image"),
        comment_count: collection_row.try_get("comment_count").unwrap_or(0),
        monthly_drill_users: 0,
        owner: CollectionOwner {
            user_id: collection_row.get("userid"),
            username: collection_row.get("username"),
        },
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub async fn list_collection_items(
    pool: &Pool,
    collection_id: i32,
    user_id: Option<i32>,
    page: i64,
    per_page: i64,
    search: Option<String>,
    item_id: Option<i32>,
    exclude_with_flashcards: Option<bool>,
    has_card_image_only: Option<bool>,
    filters: ListCollectionItemsFilters,
) -> AppResult<CollectionItemListResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection access
    let collection = transaction
        .query_one(
            "SELECT user_id, is_public FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let is_public: bool = collection.get("is_public");
    let owner_id: i32 = collection.get("user_id");

    if !is_public && Some(owner_id) != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let offset = (page - 1) * per_page;

    // Build base query
    let mut query = String::from(
        "SELECT ci.item_id, ci.definition_id, ci.notes as ci_notes, ci.added_at, ci.auto_progress, 
                ci.free_content_front, ci.free_content_back, 
                ci.canonical_form,
                ci.langid, ci.owner_user_id, ci.license, ci.script, ci.is_original,
                d.langid as lang_id,
                coalesce(u.username,'') as username,
                d.definition, d.notes as notes, v.valsiid, v.word, ci.position,
                EXISTS(SELECT 1 FROM collection_item_images cii 
                       WHERE cii.item_id = ci.item_id AND cii.side = 'front') as has_front_image,
                EXISTS(SELECT 1 FROM collection_item_images cii 
                       WHERE cii.item_id = ci.item_id AND cii.side = 'back') as has_back_image,
                EXISTS(SELECT 1 FROM collection_item_sounds cis 
                       WHERE cis.item_id = ci.item_id) as has_sound,
                f.id as flashcard_id, f.direction::text as flashcard_direction, f.created_at as flashcard_created_at
         FROM collection_items ci
         LEFT JOIN definitions d ON ci.definition_id = d.definitionid
         LEFT JOIN valsi v ON d.valsiid = v.valsiid
         LEFT JOIN users u ON d.userid = u.userid
         LEFT JOIN flashcards f ON ci.item_id = f.item_id
         WHERE ci.collection_id = $1 
           AND ($2::int IS NULL OR ci.item_id = $2)
           AND ($3::boolean IS NULL OR ($3::boolean = true AND f.id IS NULL))
           AND ($4::boolean IS DISTINCT FROM true OR EXISTS (
               SELECT 1 FROM collection_item_images cii_img
               WHERE cii_img.item_id = ci.item_id AND cii_img.side IN ('front', 'back')
           ))",
    );

    // Create vectors to store parameters and the search pattern
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = vec![
        Box::new(collection_id),
        Box::new(item_id),
        Box::new(exclude_with_flashcards),
        Box::new(has_card_image_only),
    ];
    let mut param_count = 5;

    // Store search pattern if search is provided. Searching also matches custom-text items
    // (free_content_front/back) so they remain visible alongside dictionary-backed rows.
    crate::jbovlaste::service::validate_valsi_regex(&transaction, search.as_deref().unwrap_or(""))
        .await?;

    let search_pattern = search
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| crate::search_helpers::CollectionSearchPattern::from_query(&s));

    // Build a helper that records a filter clause with the "OR ci.definition_id IS NULL" escape
    // hatch so custom-text-only rows are not dropped by dictionary-targeted filters.
    let mut extra_clauses: Vec<String> = Vec::new();

    if let Some(pattern) = &search_pattern {
        extra_clauses.push(if pattern.is_regex {
            format!(" AND v.word ~* ${}", param_count)
        } else {
            format!(
                " AND (
                ci.notes ILIKE ${idx} OR
                v.word ILIKE ${idx} OR
                d.definition ILIKE ${idx} OR
                d.notes ILIKE ${idx} OR
                ci.free_content_front ILIKE ${idx} OR
                ci.free_content_back ILIKE ${idx}
            )",
                idx = param_count
            )
        });
        params.push(Box::new(pattern.pattern.clone()));
        param_count += 1;
    }

    if let Some(selmaho) = filters.selmaho.as_ref().filter(|s| !s.is_empty()) {
        extra_clauses.push(format!(
            " AND (d.selmaho = ${idx} OR ci.definition_id IS NULL)",
            idx = param_count
        ));
        params.push(Box::new(selmaho.clone()));
        param_count += 1;
    }

    if let Some(names) = filters.usernames.as_ref().filter(|v| !v.is_empty()) {
        extra_clauses.push(format!(
            " AND (u.username = ANY(${idx}) OR ci.definition_id IS NULL)",
            idx = param_count
        ));
        params.push(Box::new(names.clone()));
        param_count += 1;
    }
    if let Some(names) = filters.exclude_usernames.as_ref().filter(|v| !v.is_empty()) {
        extra_clauses.push(format!(
            " AND (ci.definition_id IS NULL OR NOT (u.username = ANY(${idx})))",
            idx = param_count
        ));
        params.push(Box::new(names.clone()));
        param_count += 1;
    }

    if let Some(word_types) = filters.word_type.as_ref().filter(|ids| !ids.is_empty()) {
        extra_clauses.push(format!(
            " AND (v.typeid = ANY(${idx}::int2[]) OR ci.definition_id IS NULL)",
            idx = param_count
        ));
        params.push(Box::new(word_types.clone()));
        param_count += 1;
    }

    // source_langid: only apply when explicitly non-default to keep behaviour additive.
    if let Some(source_langid) = filters.source_langid.filter(|&v| v != 1) {
        extra_clauses.push(format!(
            " AND (v.source_langid = ${idx} OR ci.definition_id IS NULL)",
            idx = param_count
        ));
        params.push(Box::new(source_langid));
        param_count += 1;
    }

    if let Some(langs) = filters.languages.as_ref().filter(|l| !l.is_empty()) {
        extra_clauses.push(format!(
            " AND (d.langid = ANY(${idx}) OR ci.definition_id IS NULL)",
            idx = param_count
        ));
        params.push(Box::new(langs.clone()));
        param_count += 1;
    }

    // search_in_phrases=false → exclude phrase-typed valsi (typeid 15) like dictionary search.
    // word_type takes precedence (matches the dictionary search behaviour).
    if !crate::jbovlaste::dto::has_word_type_filter(&filters.word_type)
        && filters.search_in_phrases == Some(false)
    {
        extra_clauses
            .push(" AND (v.typeid IS DISTINCT FROM 15 OR ci.definition_id IS NULL)".to_string());
    }

    for clause in &extra_clauses {
        query.push_str(clause);
    }

    // Ordering: when semantic mode is requested with a non-empty search and an embedding was
    // computed, rank rows by cosine distance to the embedding. Custom-text items (no embedding)
    // sort to the bottom in their natural order.
    let semantic_vector_idx = if let Some(ref embedding) = filters.semantic_embedding {
        if search_pattern.is_some() {
            let idx = param_count;
            params.push(Box::new(embedding.clone()));
            param_count += 1;
            Some(idx)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(idx) = semantic_vector_idx {
        query.push_str(&format!(
            " ORDER BY (CASE WHEN d.embedding IS NULL THEN 1 ELSE 0 END) ASC, \
              d.embedding <=> ${idx}::vector ASC NULLS LAST, ci.position ASC, ci.added_at DESC",
            idx = idx
        ));
    } else {
        query.push_str(" ORDER BY ci.position ASC, ci.added_at DESC");
    }
    query.push_str(&format!(
        " LIMIT ${} OFFSET ${}",
        param_count,
        param_count + 1
    ));
    params.push(Box::new(per_page));
    params.push(Box::new(offset));

    // Execute query
    let rows = transaction
        .query(
            &query,
            &params.iter().map(|p| &**p as _).collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Map results: set sound_url to custom when has_sound, else None (fallback filled below)
    let mut items: Vec<CollectionItemResponse> = rows
        .iter()
        .map(|row| {
            let has_sound: bool = row.get("has_sound");
            let item_id: i32 = row.get("item_id");
            let sound_url = if has_sound {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id, item_id
                ))
            } else {
                None
            };
            CollectionItemResponse {
                lang_id: row.get("lang_id"),
                item_id,
                definition_id: row.get("definition_id"),
                valsi_id: row.get("valsiid"),
                word: row.get("word"),
                username: row.get("username"),
                definition: row.get("definition"),
                notes: row.get("notes"),
                ci_notes: row.get("ci_notes"),
                position: row.get("position"),
                auto_progress: row.get("auto_progress"),
                added_at: row.get("added_at"),
                free_content_front: row.get("free_content_front"),
                free_content_back: row.get("free_content_back"),
                has_front_image: exists_front_image(row),
                language_id: row.get("langid"),
                owner_user_id: row.get("owner_user_id"),
                license: row.get("license"),
                script: row.get("script"),
                is_original: row.get("is_original"),
                has_back_image: exists_back_image(row),
                has_sound,
                sound_url,
                canonical_form: row.get("canonical_form"),
                flashcard: row
                    .get::<_, Option<i32>>("flashcard_id")
                    .map(|flashcard_id| FlashcardResponse {
                        id: flashcard_id,
                        direction: row.get("flashcard_direction"),
                        created_at: row.get("flashcard_created_at"),
                        canonical_form: row.get("canonical_form"),
                    }),
                collection_id: Some(collection_id),
                collection_name: None,
                source_langid: None,
                collection_created_at: None,
                match_count: None,
            }
        })
        .collect();

    // Fallback sound_url from DB (valsi_sounds) for items without custom sound
    let words_to_check: Vec<String> = rows
        .iter()
        .filter(|r| !r.get::<_, bool>("has_sound"))
        .filter_map(|r| {
            let w: Option<String> = r.get("word");
            let f: Option<String> = r.get("free_content_front");
            w.or(f).filter(|s| !s.trim().is_empty())
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let sound_urls_map = if words_to_check.is_empty() {
        std::collections::HashMap::new()
    } else {
        get_valsi_sound_urls_from_db(pool, &words_to_check)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    };
    for (item, row) in items.iter_mut().zip(rows.iter()) {
        if !item.has_sound {
            let key: Option<String> = row
                .get::<_, Option<String>>("word")
                .or(row.get("free_content_front"));
            item.sound_url = key.and_then(|k| sound_urls_map.get(&k).cloned().flatten());
        }
    }

    // Count mirrors list filters (join flashcards when excluding flashcard rows)
    let mut count_query = String::from(
        "SELECT COUNT(DISTINCT ci.item_id)
         FROM collection_items ci
         LEFT JOIN definitions d ON ci.definition_id = d.definitionid
         LEFT JOIN valsi v ON d.valsiid = v.valsiid
         LEFT JOIN users u ON d.userid = u.userid
         LEFT JOIN flashcards f ON ci.item_id = f.item_id
         WHERE ci.collection_id = $1
           AND ($2::int IS NULL OR ci.item_id = $2)
           AND ($3::boolean IS NULL OR ($3::boolean = true AND f.id IS NULL))
           AND ($4::boolean IS DISTINCT FROM true OR EXISTS (
               SELECT 1 FROM collection_item_images cii_img
               WHERE cii_img.item_id = ci.item_id AND cii_img.side IN ('front', 'back')
           ))",
    );

    let mut count_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = vec![
        Box::new(collection_id),
        Box::new(item_id),
        Box::new(exclude_with_flashcards),
        Box::new(has_card_image_only),
    ];
    let mut count_param_idx = 5;

    if let Some(pattern) = &search_pattern {
        count_query.push_str(&if pattern.is_regex {
            format!(" AND v.word ~* ${}", count_param_idx)
        } else {
            format!(
                " AND (
                ci.notes ILIKE ${idx} OR
                v.word ILIKE ${idx} OR
                d.definition ILIKE ${idx} OR
                d.notes ILIKE ${idx} OR
                ci.free_content_front ILIKE ${idx} OR
                ci.free_content_back ILIKE ${idx}
            )",
                idx = count_param_idx
            )
        });
        count_params.push(Box::new(pattern.pattern.clone()));
        count_param_idx += 1;
    }

    if let Some(selmaho) = filters.selmaho.as_ref().filter(|s| !s.is_empty()) {
        count_query.push_str(&format!(
            " AND (d.selmaho = ${idx} OR ci.definition_id IS NULL)",
            idx = count_param_idx
        ));
        count_params.push(Box::new(selmaho.clone()));
        count_param_idx += 1;
    }

    if let Some(names) = filters.usernames.as_ref().filter(|v| !v.is_empty()) {
        count_query.push_str(&format!(
            " AND (u.username = ANY(${idx}) OR ci.definition_id IS NULL)",
            idx = count_param_idx
        ));
        count_params.push(Box::new(names.clone()));
        count_param_idx += 1;
    }
    if let Some(names) = filters.exclude_usernames.as_ref().filter(|v| !v.is_empty()) {
        count_query.push_str(&format!(
            " AND (ci.definition_id IS NULL OR NOT (u.username = ANY(${idx})))",
            idx = count_param_idx
        ));
        count_params.push(Box::new(names.clone()));
        count_param_idx += 1;
    }

    if let Some(word_types) = filters.word_type.as_ref().filter(|ids| !ids.is_empty()) {
        count_query.push_str(&format!(
            " AND (v.typeid = ANY(${idx}::int2[]) OR ci.definition_id IS NULL)",
            idx = count_param_idx
        ));
        count_params.push(Box::new(word_types.clone()));
        count_param_idx += 1;
    }

    if let Some(source_langid) = filters.source_langid.filter(|&v| v != 1) {
        count_query.push_str(&format!(
            " AND (v.source_langid = ${idx} OR ci.definition_id IS NULL)",
            idx = count_param_idx
        ));
        count_params.push(Box::new(source_langid));
        count_param_idx += 1;
    }

    if let Some(langs) = filters.languages.as_ref().filter(|l| !l.is_empty()) {
        count_query.push_str(&format!(
            " AND (d.langid = ANY(${idx}) OR ci.definition_id IS NULL)",
            idx = count_param_idx
        ));
        count_params.push(Box::new(langs.clone()));
        let _ = count_param_idx; // silence unused-after-last-push lint when no later push
        count_param_idx += 1;
    }

    if !crate::jbovlaste::dto::has_word_type_filter(&filters.word_type)
        && filters.search_in_phrases == Some(false)
    {
        count_query.push_str(" AND (v.typeid IS DISTINCT FROM 15 OR ci.definition_id IS NULL)");
    }
    let _ = count_param_idx;

    let total: i64 = transaction
        .query_one(
            &count_query,
            &count_params.iter().map(|p| &**p as _).collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(CollectionItemListResponse {
        items,
        total,
        page,
        per_page,
    })
}

/// Explicit picker ids are capped at 50; popular-collections adds up to 100 more.
const MAX_MULTI_COLLECTION_IDS: usize = 150;
pub(crate) const MAX_MULTI_COLLECTION_PER_PAGE: i64 = 200;

/// Search items across a set of **public** collections in one query. Private collection ids
/// are skipped. Per-collection listing (`list_collection_items`) still allows owners to search
/// their own private collections.
///
/// Plan: materialize the (tiny) public-collection set first so Postgres nested-loops into
/// `collection_items` via `idx_collection_items_collection` instead of GIN-scanning the whole
/// dictionary. Text/author/type filters use the same denormalized `definitions.cached_*`
/// columns as jbovlaste fast search, so valsi/users joins are unnecessary. Media EXISTS
/// runs only for the paginated rows.
pub async fn search_items_in_collections(
    pool: &Pool,
    collection_ids: Vec<i32>,
    page: i64,
    per_page: i64,
    search: Option<String>,
    filters: ListCollectionItemsFilters,
    include_total: bool,
) -> AppResult<SearchItemsResponse> {
    if collection_ids.is_empty() {
        return Err(AppError::BadRequest(
            "collection_ids is required".to_string(),
        ));
    }
    let collection_ids: Vec<i32> = collection_ids
        .into_iter()
        .filter(|id| *id > 0)
        .take(MAX_MULTI_COLLECTION_IDS)
        .collect();
    if collection_ids.is_empty() {
        return Err(AppError::BadRequest(
            "collection_ids is required".to_string(),
        ));
    }

    let page = page.max(1);
    let per_page = per_page.clamp(1, MAX_MULTI_COLLECTION_PER_PAGE);
    let offset = (page - 1) * per_page;

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let content_match = if let Some(match_item_id) = filters.match_item_id.filter(|id| *id > 0) {
        match load_item_content_identity(&client, match_item_id).await? {
            Some(ident) if !ident.is_empty() => Some(ident),
            _ => {
                return Ok(SearchItemsResponse {
                    items: vec![],
                    total: 0,
                });
            }
        }
    } else {
        None
    };

    crate::jbovlaste::service::validate_valsi_regex(&client, search.as_deref().unwrap_or(""))
        .await?;

    let search_pattern = search
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| crate::search_helpers::CollectionSearchPattern::from_query(&s));

    const ACCESSIBLE_CTE: &str = "WITH accessible AS MATERIALIZED (
            SELECT c.collection_id, c.name, c.updated_at
            FROM collections c
            WHERE c.collection_id = ANY($1::int[])
              AND c.is_public = true
         )";
    const FROM_WHERE_HEAD: &str = "
            FROM accessible c
            JOIN collection_items ci ON ci.collection_id = c.collection_id
            LEFT JOIN definitions d ON d.definitionid = ci.definition_id
            WHERE TRUE";

    let mut from_where = String::from(FROM_WHERE_HEAD);
    let mut count_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> =
        vec![Box::new(collection_ids.clone())];
    let mut count_param_idx = 2;
    append_multi_collection_search_clauses(
        &mut from_where,
        &mut count_params,
        &mut count_param_idx,
        &search_pattern,
        &filters,
        content_match.as_ref(),
    );

    let total = if include_total {
        let count_query = format!(
            "{cte}
             SELECT COUNT(*)::bigint
             {from_where}",
            cte = ACCESSIBLE_CTE,
            from_where = from_where,
        );
        client
            .query_one(
                &count_query,
                &count_params.iter().map(|p| &**p as _).collect::<Vec<_>>(),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .try_get(0)
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        0
    };

    let mut list_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> =
        vec![Box::new(collection_ids)];
    let mut list_param_idx = 2;
    append_multi_collection_search_clauses(
        &mut String::new(),
        &mut list_params,
        &mut list_param_idx,
        &search_pattern,
        &filters,
        content_match.as_ref(),
    );

    let sim_param_idx = match (filters.semantic_embedding.as_ref(), search_pattern.as_ref()) {
        (Some(embedding), Some(_)) => {
            let idx = list_param_idx;
            list_params.push(Box::new(embedding.clone()));
            list_param_idx += 1;
            Some(idx)
        }
        _ => None,
    };

    let select_item_cols = "
                ci.item_id, ci.definition_id, ci.notes as ci_notes, ci.added_at, ci.auto_progress,
                ci.free_content_front, ci.free_content_back,
                ci.canonical_form,
                ci.langid, ci.owner_user_id, ci.license, ci.script, ci.is_original,
                d.langid as lang_id,
                coalesce(d.cached_username, '') as username,
                d.definition, d.notes as notes, d.valsiid, d.cached_valsiword as word, ci.position,
                c.collection_id, c.name as collection_name,
                d.cached_source_langid as source_langid";

    let media_select = "
            EXISTS(SELECT 1 FROM collection_item_images cii
                   WHERE cii.item_id = page.item_id AND cii.side = 'front') as has_front_image,
            EXISTS(SELECT 1 FROM collection_item_images cii
                   WHERE cii.item_id = page.item_id AND cii.side = 'back') as has_back_image,
            EXISTS(SELECT 1 FROM collection_item_sounds cis
                   WHERE cis.item_id = page.item_id) as has_sound";

    let query = if filters.dedupe_by_content {
        let sim_select = if let Some(idx) = sim_param_idx {
            format!(", d.embedding <=> ${idx}::vector AS _sim")
        } else {
            ", NULL::float8 AS _sim".to_string()
        };
        let page_order = if sim_param_idx.is_some() {
            "(CASE WHEN _sim IS NULL THEN 1 ELSE 0 END) ASC, \
             _sim ASC NULLS LAST, \
             collection_updated_at DESC, position ASC, added_at DESC"
        } else {
            "collection_updated_at DESC, position ASC, added_at DESC"
        };
        format!(
            "{cte},
             candidates AS MATERIALIZED (
                SELECT
                    {select_item_cols},
                    c.updated_at AS collection_updated_at,
                    COALESCE(d.cached_valsiword, ci.free_content_front, '') AS _valsi,
                    COALESCE(d.definition, ci.free_content_back, '') AS _def,
                    COALESCE(d.cached_source_langid, -1) AS _src,
                    COALESCE(d.langid, ci.langid, -1) AS _tgt
                    {sim_select}
                {from_where}
             ),
             uniq AS MATERIALIZED (
                SELECT * FROM (
                    SELECT
                        candidates.*,
                        COUNT(*) OVER (PARTITION BY _valsi, _def, _src, _tgt)::int AS match_count,
                        ROW_NUMBER() OVER (
                            PARTITION BY _valsi, _def, _src, _tgt
                            ORDER BY added_at ASC,
                                     array_position($1::int[], collection_id) ASC NULLS LAST,
                                     item_id ASC
                        ) AS _rn
                    FROM candidates
                ) ranked
                WHERE _rn = 1
             ),
             page AS MATERIALIZED (
                SELECT * FROM uniq
                ORDER BY {page_order}
                LIMIT ${lim} OFFSET ${off}
             )
             SELECT
                page.item_id, page.definition_id, page.ci_notes, page.added_at, page.auto_progress,
                page.free_content_front, page.free_content_back,
                page.canonical_form,
                page.langid, page.owner_user_id, page.license, page.script, page.is_original,
                page.lang_id, page.username, page.definition, page.notes, page.valsiid, page.word,
                page.position, page.collection_id, page.collection_name,
                page.source_langid,
                page.match_count,
                {media_select}
             FROM page",
            cte = ACCESSIBLE_CTE,
            select_item_cols = select_item_cols,
            sim_select = sim_select,
            from_where = from_where,
            page_order = page_order,
            media_select = media_select,
            lim = list_param_idx,
            off = list_param_idx + 1,
        )
    } else {
        let order_by = if let Some(idx) = sim_param_idx {
            format!(
                "(CASE WHEN d.embedding IS NULL THEN 1 ELSE 0 END) ASC, \
                 d.embedding <=> ${idx}::vector ASC NULLS LAST, \
                 c.updated_at DESC, ci.position ASC, ci.added_at DESC",
                idx = idx
            )
        } else {
            "c.updated_at DESC, ci.position ASC, ci.added_at DESC".to_string()
        };
        format!(
            "{cte},
             page AS MATERIALIZED (
                SELECT
                    {select_item_cols}
                {from_where}
                ORDER BY {order_by}
                LIMIT ${lim} OFFSET ${off}
             )
             SELECT
                page.item_id, page.definition_id, page.ci_notes, page.added_at, page.auto_progress,
                page.free_content_front, page.free_content_back,
                page.canonical_form,
                page.langid, page.owner_user_id, page.license, page.script, page.is_original,
                page.lang_id, page.username, page.definition, page.notes, page.valsiid, page.word,
                page.position, page.collection_id, page.collection_name,
                page.source_langid,
                {media_select}
             FROM page",
            cte = ACCESSIBLE_CTE,
            select_item_cols = select_item_cols,
            from_where = from_where,
            order_by = order_by,
            media_select = media_select,
            lim = list_param_idx,
            off = list_param_idx + 1,
        )
    };
    list_params.push(Box::new(per_page));
    list_params.push(Box::new(offset));

    let rows = client
        .query(
            &query,
            &list_params.iter().map(|p| &**p as _).collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut items: Vec<CollectionItemResponse> = rows
        .iter()
        .map(|row| {
            let has_sound: bool = row.get("has_sound");
            let item_id: i32 = row.get("item_id");
            let cid: i32 = row.get("collection_id");
            let sound_url = if has_sound {
                Some(format!("/api/collections/{}/items/{}/sound", cid, item_id))
            } else {
                None
            };
            CollectionItemResponse {
                lang_id: row.get("lang_id"),
                item_id,
                definition_id: row.get("definition_id"),
                valsi_id: row.get("valsiid"),
                word: row.get("word"),
                username: row.get("username"),
                definition: row.get("definition"),
                notes: row.get("notes"),
                ci_notes: row.get("ci_notes"),
                position: row.get("position"),
                auto_progress: row.get("auto_progress"),
                added_at: row.get("added_at"),
                free_content_front: row.get("free_content_front"),
                free_content_back: row.get("free_content_back"),
                has_front_image: exists_front_image(row),
                language_id: row.get("langid"),
                owner_user_id: row.get("owner_user_id"),
                license: row.get("license"),
                script: row.get("script"),
                is_original: row.get("is_original"),
                has_back_image: exists_back_image(row),
                has_sound,
                sound_url,
                canonical_form: row.get("canonical_form"),
                flashcard: None,
                collection_id: Some(cid),
                collection_name: Some(row.get("collection_name")),
                source_langid: row.try_get("source_langid").ok().flatten(),
                collection_created_at: None,
                match_count: row.try_get::<_, i32>("match_count").ok().filter(|c| *c > 1),
            }
        })
        .collect();

    let words_to_check: Vec<String> = rows
        .iter()
        .filter(|r| !r.get::<_, bool>("has_sound"))
        .filter_map(|r| {
            let w: Option<String> = r.get("word");
            let f: Option<String> = r.get("free_content_front");
            w.or(f).filter(|s| !s.trim().is_empty())
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let sound_urls_map = if words_to_check.is_empty() {
        std::collections::HashMap::new()
    } else {
        get_valsi_sound_urls_from_db(pool, &words_to_check)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    };
    for (item, row) in items.iter_mut().zip(rows.iter()) {
        if !item.has_sound {
            let key: Option<String> = row
                .get::<_, Option<String>>("word")
                .or(row.get("free_content_front"));
            item.sound_url = key.and_then(|k| sound_urls_map.get(&k).cloned().flatten());
        }
    }

    Ok(SearchItemsResponse { items, total })
}

/// Search public collection items for a filtered export: extra type/rafsi/selmaho joins,
/// no media blobs, and a higher row cap than the interactive search endpoint.
pub async fn search_items_in_collections_for_export(
    pool: &Pool,
    collection_ids: Vec<i32>,
    search: Option<String>,
    filters: ListCollectionItemsFilters,
    max_rows: i64,
) -> AppResult<(Vec<CollectionExportItem>, i64)> {
    if collection_ids.is_empty() {
        return Err(AppError::BadRequest(
            "collection_ids is required".to_string(),
        ));
    }
    let collection_ids: Vec<i32> = collection_ids
        .into_iter()
        .filter(|id| *id > 0)
        .take(MAX_MULTI_COLLECTION_IDS)
        .collect();
    if collection_ids.is_empty() {
        return Err(AppError::BadRequest(
            "collection_ids is required".to_string(),
        ));
    }

    let max_rows = max_rows.clamp(1, crate::export::models::SEARCH_EXPORT_ROW_CAP);

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    crate::jbovlaste::service::validate_valsi_regex(&client, search.as_deref().unwrap_or(""))
        .await?;

    let search_pattern = search
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| crate::search_helpers::CollectionSearchPattern::from_query(&s));

    const ACCESSIBLE_CTE: &str = "WITH accessible AS MATERIALIZED (
            SELECT c.collection_id, c.name, c.updated_at
            FROM collections c
            WHERE c.collection_id = ANY($1::int[])
              AND c.is_public = true
         )";
    const FROM_WHERE_HEAD: &str = "
            FROM accessible c
            JOIN collection_items ci ON ci.collection_id = c.collection_id
            LEFT JOIN definitions d ON d.definitionid = ci.definition_id
            LEFT JOIN valsitypes t ON t.typeid = d.cached_typeid
            LEFT JOIN convenientdefinitions conv ON conv.definitionid = d.definitionid
            WHERE TRUE";

    let mut from_where = String::from(FROM_WHERE_HEAD);
    let mut count_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> =
        vec![Box::new(collection_ids.clone())];
    let mut count_param_idx = 2;
    append_multi_collection_search_clauses(
        &mut from_where,
        &mut count_params,
        &mut count_param_idx,
        &search_pattern,
        &filters,
        None,
    );

    let count_query = format!(
        "{cte}
         SELECT COUNT(*)::bigint
         {from_where}",
        cte = ACCESSIBLE_CTE,
        from_where = from_where,
    );
    let total: i64 = client
        .query_one(
            &count_query,
            &count_params.iter().map(|p| &**p as _).collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    if total == 0 || total > max_rows {
        return Ok((Vec::new(), total));
    }

    let mut list_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> =
        vec![Box::new(collection_ids)];
    let mut list_param_idx = 2;
    append_multi_collection_search_clauses(
        &mut String::new(),
        &mut list_params,
        &mut list_param_idx,
        &search_pattern,
        &filters,
        None,
    );

    let order_by = if let Some(ref embedding) = filters.semantic_embedding {
        if search_pattern.is_some() {
            let idx = list_param_idx;
            list_params.push(Box::new(embedding.clone()));
            list_param_idx += 1;
            format!(
                "(CASE WHEN d.embedding IS NULL THEN 1 ELSE 0 END) ASC, \
                 d.embedding <=> ${idx}::vector ASC NULLS LAST, \
                 c.updated_at DESC, ci.position ASC, ci.added_at DESC",
                idx = idx
            )
        } else {
            "c.updated_at DESC, ci.position ASC, ci.added_at DESC".to_string()
        }
    } else {
        "c.updated_at DESC, ci.position ASC, ci.added_at DESC".to_string()
    };

    let query = format!(
        "{cte}
         SELECT
            ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
            ci.free_content_front, ci.free_content_back,
            ci.langid as language_id, ci.owner_user_id, ci.license,
            COALESCE(d.cached_valsiword, ci.free_content_front) as word,
            d.definition, d.notes as definition_notes, d.jargon,
            t.descriptor as word_type,
            conv.rafsi, conv.experimental_rafsi, conv.selmaho
         {from_where}
         ORDER BY {order_by}
         LIMIT ${lim}",
        cte = ACCESSIBLE_CTE,
        from_where = from_where,
        order_by = order_by,
        lim = list_param_idx,
    );
    list_params.push(Box::new(max_rows));

    let rows = client
        .query(
            &query,
            &list_params.iter().map(|p| &**p as _).collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let items = rows
        .into_iter()
        .map(|row| CollectionExportItem {
            item_id: row.get("item_id"),
            position: row.get("position"),
            collection_note: row.get("collection_note"),
            definition_id: row.get("definition_id"),
            language_id: row.get("language_id"),
            owner_user_id: row.get("owner_user_id"),
            license: row.get("license"),
            word: row.get("word"),
            word_type: row.get("word_type"),
            rafsi: row.get("rafsi"),
            experimental_rafsi: row.try_get("experimental_rafsi").ok().flatten(),
            selmaho: row.get("selmaho"),
            definition: row.get("definition"),
            definition_notes: row.get("definition_notes"),
            jargon: row.get("jargon"),
            free_content_front: row.get("free_content_front"),
            free_content_back: row.get("free_content_back"),
            front_image_url: None,
            sound_url: None,
            back_image_url: None,
            direction: None,
            level_index: None,
            position_in_level: None,
        })
        .collect();

    Ok((items, total))
}

fn append_multi_collection_search_clauses(
    sql: &mut String,
    params: &mut Vec<Box<dyn tokio_postgres::types::ToSql + Sync>>,
    param_count: &mut usize,
    search_pattern: &Option<crate::search_helpers::CollectionSearchPattern>,
    filters: &ListCollectionItemsFilters,
    content_match: Option<&ItemContentIdentity>,
) {
    if let Some(pattern) = search_pattern {
        // `cached_search_text` already concatenates word/rafsi/definition/notes/selmaho/glosswords
        // (GIN-indexed for dictionary search). Collection custom-text rows have no definition, so
        // they still match on item notes / free_content.
        sql.push_str(&if pattern.is_regex {
            format!(" AND d.cached_valsiword ~* ${}", *param_count)
        } else {
            format!(
                " AND (
                d.cached_search_text ILIKE ${idx}
                OR d.cached_valsiword ILIKE ${idx}
                OR ci.notes ILIKE ${idx}
                OR ci.free_content_front ILIKE ${idx}
                OR ci.free_content_back ILIKE ${idx}
            )",
                idx = *param_count
            )
        });
        params.push(Box::new(pattern.pattern.clone()));
        *param_count += 1;
    }

    if let Some(selmaho) = filters.selmaho.as_ref().filter(|s| !s.is_empty()) {
        sql.push_str(&format!(
            " AND (d.selmaho = ${idx} OR ci.definition_id IS NULL)",
            idx = *param_count
        ));
        params.push(Box::new(selmaho.clone()));
        *param_count += 1;
    }

    if let Some(names) = filters.usernames.as_ref().filter(|v| !v.is_empty()) {
        sql.push_str(&format!(
            " AND (d.cached_username = ANY(${idx}) OR ci.definition_id IS NULL)",
            idx = *param_count
        ));
        params.push(Box::new(names.clone()));
        *param_count += 1;
    }
    if let Some(names) = filters.exclude_usernames.as_ref().filter(|v| !v.is_empty()) {
        sql.push_str(&format!(
            " AND (ci.definition_id IS NULL OR NOT (d.cached_username = ANY(${idx})))",
            idx = *param_count
        ));
        params.push(Box::new(names.clone()));
        *param_count += 1;
    }

    if let Some(word_types) = filters.word_type.as_ref().filter(|ids| !ids.is_empty()) {
        sql.push_str(&format!(
            " AND (d.cached_typeid = ANY(${idx}::int2[]) OR ci.definition_id IS NULL)",
            idx = *param_count
        ));
        params.push(Box::new(word_types.clone()));
        *param_count += 1;
    }

    if let Some(source_langid) = filters.source_langid.filter(|&v| v != 1) {
        sql.push_str(&format!(
            " AND (d.cached_source_langid = ${idx} OR ci.definition_id IS NULL)",
            idx = *param_count
        ));
        params.push(Box::new(source_langid));
        *param_count += 1;
    }

    if let Some(langs) = filters.languages.as_ref().filter(|l| !l.is_empty()) {
        sql.push_str(&format!(
            " AND (d.langid = ANY(${idx}::int[]) OR ci.definition_id IS NULL)",
            idx = *param_count
        ));
        params.push(Box::new(langs.clone()));
        *param_count += 1;
    }

    if !crate::jbovlaste::dto::has_word_type_filter(&filters.word_type)
        && filters.search_in_phrases == Some(false)
    {
        sql.push_str(" AND (d.cached_typeid IS DISTINCT FROM 15 OR ci.definition_id IS NULL)");
    }

    if let Some(ident) = content_match {
        append_content_identity_match(sql, params, param_count, ident);
    }
}

pub async fn update_item_notes(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    item_id: i32,
    user_id: i32,
    req: &UpdateItemNotesRequest,
) -> AppResult<CollectionItemResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // Update item notes and auto_progress flag
    let item = transaction
        .query_opt(
            "WITH updated AS (
                UPDATE collection_items 
                SET notes = $1,
                    auto_progress = COALESCE($4, auto_progress)
                WHERE collection_id = $2 AND item_id = $3
                RETURNING *
            )
            SELECT u.*, 
                   EXISTS(SELECT 1 FROM collection_item_images i 
                         WHERE i.item_id = u.item_id AND i.side = 'front') as has_front_image,
                   EXISTS(SELECT 1 FROM collection_item_images i 
                         WHERE i.item_id = u.item_id AND i.side = 'back') as has_back_image
            FROM updated u",
            &[&req.notes, &collection_id, &item_id, &req.auto_progress],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or(AppError::NotFound("Item not found".to_string()))?;

    // Get related definition info if this is a definition-based item
    let definition = if let Some(def_id) = item.get::<_, Option<i32>>("definition_id") {
        Some(transaction
            .query_one(
                "SELECT d.definition, d.notes, v.word, v.valsiid, u.username, d.langid as lang_id
                 FROM definitions d
                 JOIN valsi v ON d.valsiid = v.valsiid
                 JOIN users u ON d.userid = u.userid
                 WHERE d.definitionid = $1",
                &[&def_id],
            )
            .await.map_err(|e| AppError::Database(e.to_string()))?)
    } else {
        None
    };

    // Update collection's updated_at timestamp
    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let has_sound: bool = transaction
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = $1)",
            &[&item_id],
        )
        .await
        .map(|r| r.get(0))
        .unwrap_or(false);

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(CollectionItemResponse {
        item_id,
        definition_id: item.get("definition_id"),
        valsi_id: definition.as_ref().map(|d| d.get("valsiid")),
        word: definition.as_ref().map(|d| d.get("word")),
        definition: definition.as_ref().map(|d| d.get("definition")),
        notes: definition.as_ref().map(|d| d.get("notes")),
        ci_notes: item.get("notes"),
        position: item.get("position"),
        auto_progress: item.get("auto_progress"),
        added_at: item.get("added_at"),
        lang_id: definition.as_ref().map(|d| d.get("lang_id")),
        username: definition.as_ref().map(|d| d.get("username")),
        free_content_front: item.get("free_content_front"),
        free_content_back: item.get("free_content_back"),
        has_front_image: item.get("has_front_image"),
        language_id: item.get("langid"),
        owner_user_id: item.get("owner_user_id"),
        license: item.get("license"),
        script: item.get("script"),
        is_original: item.get("is_original"),
        has_back_image: item.get("has_back_image"),
        has_sound,
        sound_url: if has_sound {
            Some(format!(
                "/api/collections/{}/items/{}/sound",
                collection_id, item_id
            ))
        } else {
            None
        },
        canonical_form: item.get("canonical_form"),
        flashcard: None,
        collection_id: Some(collection_id),
        collection_name: None,
        source_langid: None,
        collection_created_at: None,
        match_count: None,
    })
}

pub async fn get_item_image(
    pool: &Pool,
    item_id: i32,
    side: &str,
    user_id: Option<i32>,
) -> AppResult<Option<(Vec<u8>, String)>> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let accessible: bool = client.query_one(
        "SELECT EXISTS(SELECT 1 FROM collection_items ci JOIN collections c USING(collection_id)
         WHERE ci.item_id = $1 AND (c.is_public OR c.user_id = $2))",
        &[&item_id, &user_id],
    ).await.map_err(|e| AppError::Database(e.to_string()))?.get(0);
    if !accessible {
        return Err(AppError::Unauthorized("Access denied".into()));
    }

    let result = client
        .query_opt(
            "SELECT img.image_data, img.mime_type
             FROM collection_item_images cii
             INNER JOIN collection_images img ON img.collection_image_id = cii.collection_image_id
             JOIN collection_items ci ON ci.item_id = cii.item_id
             JOIN collections c ON c.collection_id = ci.collection_id
             WHERE cii.item_id = $1 AND cii.side = $2 AND (c.is_public OR c.user_id = $3)",
            &[&item_id, &side, &user_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    result
        .map(|row| {
            normalize_collection_image(
                &row.get::<_, Vec<u8>>("image_data"),
                &row.get::<_, String>("mime_type"),
            )
        })
        .transpose()
}

pub async fn get_item_sound(
    pool: &Pool,
    item_id: i32,
    user_id: Option<i32>,
) -> AppResult<Option<(Vec<u8>, String)>> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let accessible: bool = client.query_one(
        "SELECT EXISTS(SELECT 1 FROM collection_items ci JOIN collections c USING(collection_id)
         WHERE ci.item_id = $1 AND (c.is_public OR c.user_id = $2))",
        &[&item_id, &user_id],
    ).await.map_err(|e| AppError::Database(e.to_string()))?.get(0);
    if !accessible {
        return Err(AppError::Unauthorized("Access denied".into()));
    }

    let result = client
        .query_opt(
            "SELECT cis.sound_data, cis.mime_type
             FROM collection_item_sounds cis JOIN collection_items ci ON ci.item_id = cis.item_id
             JOIN collections c ON c.collection_id = ci.collection_id
             WHERE cis.item_id = $1 AND (c.is_public OR c.user_id = $2)",
            &[&item_id, &user_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(result.map(|row| (row.get("sound_data"), row.get("mime_type"))))
}

pub async fn update_item_images(
    pool: &Pool,
    collection_id: i32,
    item_id: i32,
    user_id: i32,
    req: &UpdateItemRequest,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;
    if transaction.query_opt(
        "SELECT item_id FROM collection_items WHERE item_id = $1 AND collection_id = $2 FOR UPDATE",
        &[&item_id, &collection_id],
    ).await.map_err(|e| AppError::Database(e.to_string()))?.is_none() {
        return Err(AppError::Unauthorized("Item does not belong to this collection".into()));
    }

    // Update notes if provided
    if let Some(notes) = &req.notes {
        transaction
            .execute(
                "UPDATE collection_items SET notes = $1 WHERE item_id = $2",
                &[notes, &item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle front image
    if req.remove_front_image.unwrap_or(false) || req.front_image.is_some() {
        transaction
            .execute(
                "DELETE FROM collection_item_images WHERE item_id = $1 AND side = 'front'",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    if let Some(image) = &req.front_image {
        validate_item_image(image).map_err(|e| AppError::BadRequest(e.clone()))?;
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid front image base64: {}", e)))?;
        let image_id =
            get_or_insert_collection_image_id(&transaction, &image_data, &image.mime_type).await?;
        transaction
            .execute(
                "INSERT INTO collection_item_images (item_id, collection_image_id, side)
             VALUES ($1, $2, 'front')",
                &[&item_id, &image_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle back image
    if req.remove_back_image.unwrap_or(false) || req.back_image.is_some() {
        transaction
            .execute(
                "DELETE FROM collection_item_images WHERE item_id = $1 AND side = 'back'",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    if let Some(image) = &req.back_image {
        validate_item_image(image).map_err(|e| AppError::BadRequest(e.clone()))?;
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid back image base64: {}", e)))?;
        let image_id =
            get_or_insert_collection_image_id(&transaction, &image_data, &image.mime_type).await?;
        transaction
            .execute(
                "INSERT INTO collection_item_images (item_id, collection_image_id, side)
             VALUES ($1, $2, 'back')",
                &[&item_id, &image_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle sound
    if req.remove_sound.unwrap_or(false) || req.sound.is_some() {
        transaction
            .execute(
                "DELETE FROM collection_item_sounds WHERE item_id = $1",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    if let Some(sound) = &req.sound {
        validate_item_audio(sound).map_err(|e| AppError::BadRequest(e.clone()))?;
        let sound_data = BASE64
            .decode(&sound.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid sound base64: {}", e)))?;
        transaction
            .execute(
                "INSERT INTO collection_item_sounds (item_id, sound_data, mime_type)
             VALUES ($1, $2, $3)",
                &[&item_id, &sound_data, &sound.mime_type],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn exists_front_image(row: &tokio_postgres::Row) -> bool {
    row.get("has_front_image")
}

fn exists_back_image(row: &tokio_postgres::Row) -> bool {
    row.get("has_back_image")
}

pub async fn search_items(
    pool: &Pool,
    current_user_id: i32,
    query: &str,
    owner_id: Option<i32>,
) -> AppResult<SearchItemsResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let search_pattern = format!("%{}%", query);
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> =
        vec![Box::new(&search_pattern)];
    let mut param_count = 2;

    let mut sql = String::from(
        "WITH accessible_collections AS (
            SELECT collection_id 
            FROM collections 
            WHERE is_public = true 
            OR user_id = $",
    );
    sql.push_str(&param_count.to_string());
    params.push(Box::new(current_user_id));
    param_count += 1;

    if let Some(uid) = owner_id {
        sql.push_str(" AND user_id = $");
        sql.push_str(&param_count.to_string());
        params.push(Box::new(uid));
    }

    sql.push_str(
        ")
        SELECT ci.item_id, ci.definition_id, ci.notes as ci_notes, 
               ci.added_at, ci.position, ci.auto_progress, 
               ci.langid, ci.owner_user_id, ci.license, ci.script, ci.is_original,
               ci.free_content_front, ci.free_content_back, ci.canonical_form,
               d.langid as lang_id, d.definition, d.notes,
               v.valsiid, v.word, u.username,
               c.collection_id,
               EXISTS(SELECT 1 FROM collection_item_images cii 
                      WHERE cii.item_id = ci.item_id AND cii.side = 'front') as has_front_image,
               EXISTS(SELECT 1 FROM collection_item_images cii 
                      WHERE cii.item_id = ci.item_id AND cii.side = 'back') as has_back_image,
               EXISTS(SELECT 1 FROM collection_item_sounds cis 
                      WHERE cis.item_id = ci.item_id) as has_sound
        FROM collection_items ci
        JOIN accessible_collections ac ON ci.collection_id = ac.collection_id
        JOIN collections c ON ci.collection_id = c.collection_id
        LEFT JOIN definitions d ON ci.definition_id = d.definitionid
        LEFT JOIN valsi v ON d.valsiid = v.valsiid
        LEFT JOIN users u ON d.userid = u.userid
        WHERE v.word ILIKE $1
           OR d.definition ILIKE $1
           OR d.notes ILIKE $1
           OR ci.notes ILIKE $1
           OR ci.free_content_front ILIKE $1
           OR ci.free_content_back ILIKE $1
        ORDER BY c.updated_at DESC, ci.position ASC",
    );

    let rows = client
        .query(&sql, &params.iter().map(|p| &**p as _).collect::<Vec<_>>())
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let items = rows
        .iter()
        .map(|row| {
            let has_sound: bool = row.get("has_sound");
            let item_id: i32 = row.get("item_id");
            let cid: i32 = row.get("collection_id");
            let sound_url = if has_sound {
                Some(format!("/api/collections/{}/items/{}/sound", cid, item_id))
            } else {
                None
            };
            CollectionItemResponse {
                item_id,
                definition_id: row.get("definition_id"),
                word: row.get("word"),
                username: row.get("username"),
                valsi_id: row.get("valsiid"),
                definition: row.get("definition"),
                notes: row.get("notes"),
                ci_notes: row.get("ci_notes"),
                position: row.get("position"),
                auto_progress: row.get("auto_progress"),
                added_at: row.get("added_at"),
                lang_id: row.get("lang_id"),
                free_content_front: row.get("free_content_front"),
                free_content_back: row.get("free_content_back"),
                has_front_image: row.get("has_front_image"),
                language_id: row.get("langid"),
                owner_user_id: row.get("owner_user_id"),
                license: row.get("license"),
                script: row.get("script"),
                is_original: row.get("is_original"),
                has_back_image: row.get("has_back_image"),
                has_sound,
                sound_url,
                canonical_form: row.get("canonical_form"),
                flashcard: None,
                collection_id: Some(cid),
                collection_name: None,
                source_langid: None,
                collection_created_at: None,
                match_count: None,
            }
        })
        .collect();

    Ok(SearchItemsResponse {
        items,
        total: rows.len() as i64,
    })
}

const MAX_CUSTOM_TEXT_BULK_ITEMS: usize = 500;

pub async fn list_custom_text_bulk_items(
    pool: &Pool,
    collection_id: i32,
    user_id: i32,
) -> AppResult<CustomTextBulkListResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    let rows = transaction
        .query(
            "SELECT ci.item_id, ci.position, ci.free_content_front, ci.free_content_back, ci.langid as language_id
             FROM collection_items ci
             WHERE ci.collection_id = $1
               AND ci.definition_id IS NULL
             ORDER BY ci.position ASC, ci.added_at DESC",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let items = rows
        .iter()
        .map(|row| CustomTextBulkItemRow {
            item_id: row.get("item_id"),
            position: row.get("position"),
            free_content_front: row
                .get::<_, Option<String>>("free_content_front")
                .unwrap_or_default(),
            free_content_back: row
                .get::<_, Option<String>>("free_content_back")
                .unwrap_or_default(),
            language_id: row.get("language_id"),
        })
        .collect();

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(CustomTextBulkListResponse { items })
}

pub async fn bulk_update_custom_text_items(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
    req: &CustomTextBulkUpdateRequest,
) -> AppResult<CustomTextBulkUpdateResponse> {
    let total = req.items.len() + req.new_items.len();
    if total > MAX_CUSTOM_TEXT_BULK_ITEMS {
        return Err(AppError::BadRequest(format!(
            "At most {} items per request",
            MAX_CUSTOM_TEXT_BULK_ITEMS
        )));
    }

    let mut seen = HashSet::new();
    for item in &req.items {
        if !seen.insert(item.item_id) {
            return Err(AppError::BadRequest(
                "Duplicate item_id in request".to_string(),
            ));
        }
    }

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    let mut updated: i32 = 0;

    for item in &req.items {
        let sanitized_front = sanitize_html(&item.free_content_front);
        let sanitized_back = sanitize_html(&item.free_content_back);

        let canonical_form = if sanitized_front.trim().is_empty() {
            None
        } else {
            crate::utils::canonical::get_canonical_form(sanitized_front.as_str())
        };

        let n = transaction
            .execute(
                "UPDATE collection_items
                 SET free_content_front = $1,
                     free_content_back = $2,
                     langid = $3,
                     canonical_form = $4
                 WHERE collection_id = $5
                   AND item_id = $6
                   AND definition_id IS NULL",
                &[
                    &sanitized_front,
                    &sanitized_back,
                    &item.language_id,
                    &canonical_form,
                    &collection_id,
                    &item.item_id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if n == 0 {
            return Err(AppError::BadRequest(format!(
                "Item {} not found or is not a custom-text-only item in this collection",
                item.item_id
            )));
        }
        updated += 1;
    }

    let mut inserted: i32 = 0;

    if !req.new_items.is_empty() {
        let max_position: i32 = transaction
            .query_one(
                "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
                &[&collection_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .try_get(0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut next_position: i32 = max_position + 1;
        let definition_id: Option<i32> = None;
        let owner_user_id: Option<i32> = None;
        let license: Option<String> = None;
        let script: Option<String> = None;
        let notes: Option<String> = None;

        for new_item in &req.new_items {
            let sanitized_front = sanitize_html(&new_item.free_content_front);
            let sanitized_back = sanitize_html(&new_item.free_content_back);

            let free_front = if sanitized_front.trim().is_empty() {
                None
            } else {
                Some(sanitized_front)
            };
            let free_back = if sanitized_back.trim().is_empty() {
                None
            } else {
                Some(sanitized_back)
            };

            if free_front.is_none() && free_back.is_none() {
                continue;
            }

            let canonical_form = free_front
                .as_ref()
                .and_then(|front| crate::utils::canonical::get_canonical_form(front.as_str()));

            transaction
                .execute(
                    "INSERT INTO collection_items (
                    collection_id, definition_id,
                    free_content_front, free_content_back,
                    langid, owner_user_id, license, script, is_original,
                    notes, position, auto_progress, canonical_form
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                    &[
                        &collection_id,
                        &definition_id,
                        &free_front,
                        &free_back,
                        &new_item.language_id,
                        &owner_user_id,
                        &license,
                        &script,
                        &true,
                        &notes,
                        &next_position,
                        &true,
                        &canonical_form,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            next_position += 1;
            inserted += 1;
        }
    }

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(CustomTextBulkUpdateResponse { updated, inserted })
}

/// Returns `None` when the collection has no image row (caller may respond with 404).
pub async fn get_collection_image_bytes(
    pool: &Pool,
    collection_id: i32,
    user_id: Option<i32>,
) -> AppResult<Option<(Vec<u8>, String)>> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let row = client
        .query_opt(
            "SELECT user_id, is_public FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let Some(row) = row else {
        return Err(AppError::NotFound("Collection not found".to_string()));
    };

    let is_public: bool = row.get("is_public");
    let owner_id: i32 = row.get("user_id");

    if !is_public && Some(owner_id) != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let img = client
        .query_opt(
            "SELECT img.image_data, img.mime_type
             FROM collections c
             INNER JOIN collection_images img ON img.collection_image_id = c.cover_collection_image_id
             WHERE c.collection_id = $1 AND (c.is_public OR c.user_id = $2)",
            &[&collection_id, &user_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    img.map(|row| {
        normalize_collection_image(
            &row.get::<_, Vec<u8>>("image_data"),
            &row.get::<_, String>("mime_type"),
        )
    })
    .transpose()
}

pub async fn upsert_collection_image(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
    req: &ProfileImageRequest,
) -> AppResult<()> {
    let image_data = validate_collection_logo(req).map_err(AppError::BadRequest)?;
    let (stored_data, new_mime_type) =
        if mime_base(&req.mime_type).eq_ignore_ascii_case("image/svg+xml") {
            (image_data, "image/svg+xml".to_string())
        } else {
            ImageProcessor::compress_avatar(&image_data, mime_base(&req.mime_type))
                .map_err(AppError::BadRequest)?
        };

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    let image_id =
        get_or_insert_collection_image_id(&transaction, &stored_data, &new_mime_type).await?;
    transaction
        .execute(
            "UPDATE collections
             SET cover_collection_image_id = $1,
                 updated_at = CURRENT_TIMESTAMP
             WHERE collection_id = $2",
            &[&image_id, &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;
    Ok(())
}

pub async fn remove_collection_image(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    transaction
        .execute(
            "UPDATE collections
             SET cover_collection_image_id = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;
    Ok(())
}

// --- Bulk media import (multipart + ZIP) ------------------------------------

type MediaBulkFileMap = HashMap<String, Vec<u8>>;

/// Max rows in manifest (multipart or ZIP).
const MAX_MEDIA_BULK_MANIFEST_ENTRIES: usize = 250;
/// Max compressed ZIP body size.
const MAX_MEDIA_BULK_ZIP_BYTES: usize = 100 * 1024 * 1024;
/// Cap sum of declared uncompressed sizes (zip bomb mitigation).
const MAX_MEDIA_BULK_ZIP_UNCOMPRESSED_TOTAL: u64 = 128 * 1024 * 1024;
const MAX_MEDIA_BULK_MANIFEST_JSON_BYTES: usize = 512 * 1024;

pub(crate) fn media_bulk_safe_basename(name: &str) -> Option<String> {
    let base = Path::new(name).file_name()?.to_str()?;
    if base.is_empty() || base == "." || base == ".." {
        return None;
    }
    if base.contains('/') || base.contains('\\') || base.contains("..") {
        return None;
    }
    Some(base.to_string())
}

pub fn parse_media_bulk_manifest_str(raw: &str) -> AppResult<Vec<MediaBulkManifestEntry>> {
    if raw.len() > MAX_MEDIA_BULK_MANIFEST_JSON_BYTES {
        return Err(AppError::BadRequest(format!(
            "manifest JSON exceeds {} KiB",
            MAX_MEDIA_BULK_MANIFEST_JSON_BYTES / 1024
        )));
    }
    let v: Vec<MediaBulkManifestEntry> = serde_json::from_str(raw)
        .map_err(|e| AppError::BadRequest(format!("Invalid manifest JSON: {}", e)))?;
    if v.is_empty() {
        return Err(AppError::BadRequest("manifest array is empty".to_string()));
    }
    if v.len() > MAX_MEDIA_BULK_MANIFEST_ENTRIES {
        return Err(AppError::BadRequest(format!(
            "At most {} manifest entries allowed",
            MAX_MEDIA_BULK_MANIFEST_ENTRIES
        )));
    }
    Ok(v)
}

/// Reads `manifest.json` (exact basename) and image files; keys in the map are entry basenames.
pub fn load_media_bulk_zip(
    bytes: &[u8],
) -> AppResult<(Vec<MediaBulkManifestEntry>, MediaBulkFileMap)> {
    if bytes.len() > MAX_MEDIA_BULK_ZIP_BYTES {
        return Err(AppError::BadRequest(format!(
            "ZIP file exceeds {} MiB (compressed)",
            MAX_MEDIA_BULK_ZIP_BYTES / (1024 * 1024)
        )));
    }
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| AppError::BadRequest(format!("Invalid or unsupported ZIP: {}", e)))?;
    if archive.len() > MAX_MEDIA_BULK_MANIFEST_ENTRIES + 64 {
        return Err(AppError::BadRequest(
            "ZIP contains too many entries".to_string(),
        ));
    }

    let mut manifest: Option<Vec<MediaBulkManifestEntry>> = None;
    let mut files: MediaBulkFileMap = HashMap::new();
    let mut total_uncompressed: u64 = 0;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::BadRequest(format!("ZIP read error: {}", e)))?;
        if file.is_dir() {
            continue;
        }
        let Some(enclosed) = file.enclosed_name() else {
            return Err(AppError::BadRequest(
                "ZIP contains an entry with an unsafe path".to_string(),
            ));
        };
        let path_str = enclosed.to_string_lossy();
        if path_str.starts_with('/') || path_str.contains("..") {
            return Err(AppError::BadRequest(
                "ZIP path traversal is not allowed".to_string(),
            ));
        }

        let declared = file.size();
        total_uncompressed = total_uncompressed.saturating_add(declared);
        if total_uncompressed > MAX_MEDIA_BULK_ZIP_UNCOMPRESSED_TOTAL {
            return Err(AppError::BadRequest(
                "ZIP uncompressed size budget exceeded (rejected as possible zip bomb)".to_string(),
            ));
        }

        let cap = if path_str.to_ascii_lowercase().ends_with("manifest.json") {
            MAX_MEDIA_BULK_MANIFEST_JSON_BYTES
        } else {
            MAX_ITEM_IMAGE_BYTES
        };
        if declared > cap as u64 {
            return Err(AppError::BadRequest(format!(
                "ZIP entry too large: {}",
                path_str
            )));
        }

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| AppError::BadRequest(format!("ZIP decompress failed: {}", e)))?;
        if buf.len() > cap {
            return Err(AppError::BadRequest(format!(
                "ZIP entry expanded past limit: {}",
                path_str
            )));
        }

        let basename = enclosed
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if basename.eq_ignore_ascii_case("manifest.json") {
            if manifest.is_some() {
                return Err(AppError::BadRequest(
                    "ZIP must contain exactly one manifest.json".to_string(),
                ));
            }
            let s = std::str::from_utf8(&buf).map_err(|e| {
                AppError::BadRequest(format!("manifest.json is not valid UTF-8: {}", e))
            })?;
            manifest = Some(parse_media_bulk_manifest_str(s)?);
            continue;
        }

        let key = media_bulk_safe_basename(&basename).ok_or_else(|| {
            AppError::BadRequest(format!("Invalid ZIP entry basename: {}", basename))
        })?;
        files.insert(key, buf);
    }

    let manifest_vec = manifest.ok_or_else(|| {
        AppError::BadRequest("ZIP must include a manifest.json file (any folder depth)".to_string())
    })?;

    Ok((manifest_vec, files))
}

pub async fn bulk_import_collection_item_media(
    pool: &Pool,
    redis: &RedisCache,
    collection_id: i32,
    user_id: i32,
    manifest: Vec<MediaBulkManifestEntry>,
    files: &MediaBulkFileMap,
) -> AppResult<MediaBulkImportResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    let mut attached: u32 = 0;
    let mut created_items: u32 = 0;
    let mut warnings: Vec<String> = Vec::new();

    for (idx, entry) in manifest.iter().enumerate() {
        let row_no = idx + 1;
        let key = match media_bulk_safe_basename(&entry.filename) {
            Some(k) => k,
            None => {
                warnings.push(format!(
                    "Row {row_no}: invalid or unsafe filename {:?}",
                    entry.filename
                ));
                continue;
            }
        };
        let data = match files.get(&key) {
            Some(d) => d.as_slice(),
            None => {
                warnings.push(format!(
                    "Row {row_no}: no uploaded file with basename {:?} (manifest filename must match)",
                    key
                ));
                continue;
            }
        };

        let side_lc = entry.side.to_lowercase();
        if side_lc != "front" && side_lc != "back" {
            warnings.push(format!(
                "Row {row_no}: side must be \"front\" or \"back\", got {:?}",
                entry.side
            ));
            continue;
        }

        let mime = match detect_image_mime_from_content(data, &entry.filename) {
            Ok(m) => m,
            Err(e) => {
                warnings.push(format!("Row {row_no}: {e}"));
                continue;
            }
        };
        if let Err(e) = validate_item_image_bytes(mime.as_str(), data) {
            warnings.push(format!("Row {row_no}: {e}"));
            continue;
        }

        let sp = format!("mb_{idx}");
        if let Err(e) = transaction.batch_execute(&format!("SAVEPOINT {sp}")).await {
            return Err(AppError::Database(e.to_string()));
        }

        let row_result: Result<bool, AppError> = async {
            let (target_item_id, created_new): (i32, bool) = if let Some(iid) = entry.item_id {
                let ok = transaction
                    .query_opt(
                        "SELECT 1 FROM collection_items WHERE collection_id = $1 AND item_id = $2",
                        &[&collection_id, &iid],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?
                    .is_some();
                if !ok {
                    return Err(AppError::BadRequest(format!(
                        "item_id {iid} not found in this collection"
                    )));
                }
                (iid, false)
            } else if let Some(pos) = entry.position {
                let row_opt = transaction
                    .query_opt(
                        "SELECT item_id FROM collection_items WHERE collection_id = $1 AND position = $2",
                        &[&collection_id, &pos],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                let Some(r) = row_opt else {
                    return Err(AppError::BadRequest(format!(
                        "no item at position {pos} in this collection"
                    )));
                };
                (r.get::<_, i32>(0), false)
            } else {
                let Some(ref ff) = entry.free_content_front else {
                    return Err(AppError::BadRequest(
                        "create entry requires free_content_front (or set item_id / position)"
                            .to_string(),
                    ));
                };
                let Some(ref fb) = entry.free_content_back else {
                    return Err(AppError::BadRequest(
                        "create entry requires free_content_back (or set item_id / position)"
                            .to_string(),
                    ));
                };
                let sanitized_front = sanitize_html(ff);
                let sanitized_back = sanitize_html(fb);
                let canonical_form =
                    crate::utils::canonical::get_canonical_form(sanitized_front.as_str());
                let max_position: i32 = transaction
                    .query_one(
                        "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
                        &[&collection_id],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?
                    .get(0);
                let next_position = max_position + 1;
                let free_front = Some(sanitized_front);
                let free_back = Some(sanitized_back);
                let definition_id: Option<i32> = None;
                let notes: Option<String> = None;
                let owner_user_id: Option<i32> = None;
                let license: Option<String> = None;
                let script: Option<String> = None;
                let row = transaction
                    .query_one(
                        "INSERT INTO collection_items (
                    collection_id, definition_id,
                    free_content_front, free_content_back,
                    langid, owner_user_id, license, script, is_original,
                    notes, position, auto_progress, canonical_form
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING item_id",
                        &[
                            &collection_id,
                            &definition_id,
                            &free_front,
                            &free_back,
                            &entry.language_id,
                            &owner_user_id,
                            &license,
                            &script,
                            &true,
                            &notes,
                            &next_position,
                            &true,
                            &canonical_form,
                        ],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                (row.get::<_, i32>("item_id"), true)
            };

            transaction
                .execute(
                    "DELETE FROM collection_item_images WHERE item_id = $1 AND side = $2",
                    &[&target_item_id, &side_lc],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            let image_id =
                get_or_insert_collection_image_id(&transaction, data, mime.as_str()).await?;
            transaction
                .execute(
                    "INSERT INTO collection_item_images (item_id, collection_image_id, side)
                     VALUES ($1, $2, $3)",
                    &[&target_item_id, &image_id, &side_lc],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(created_new)
        }
        .await;

        match row_result {
            Ok(created_new) => {
                if let Err(e) = transaction
                    .batch_execute(&format!("RELEASE SAVEPOINT {sp}"))
                    .await
                {
                    return Err(AppError::Database(e.to_string()));
                }
                attached += 1;
                if created_new {
                    created_items += 1;
                }
            }
            Err(e) => {
                if let Err(rb) = transaction
                    .batch_execute(&format!("ROLLBACK TO SAVEPOINT {sp}"))
                    .await
                {
                    return Err(AppError::Database(rb.to_string()));
                }
                warnings.push(format!("Row {row_no}: {e}"));
            }
        }
    }

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    invalidate_public_collections_cache(redis).await;

    Ok(MediaBulkImportResponse {
        attached,
        created_items,
        warnings,
    })
}

#[cfg(test)]
mod security_media_tests {
    #[test]
    fn reject_active_image_documents_and_preserve_rasters_and_vectors() {
        assert!(
            super::normalize_collection_image(b"<script>alert(1)</script>", "text/html").is_err()
        );
        assert!(
            super::normalize_collection_image(b"<script>alert(1)</script>", "image/png").is_err()
        );
        for svg in [
            r#"<svg xmlns="http://www.w3.org/2000/svg" onload = "alert(1)"/>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><foreignObject/></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><use href="&#106;avascript:alert(1)"/></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><use href="//evil.example/image"/></svg>"#,
        ] {
            assert!(super::normalize_collection_image(svg.as_bytes(), "image/svg+xml").is_err());
        }
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M0 0L10 10"/></svg>"#;
        assert!(super::normalize_collection_image(svg, "image/svg+xml").is_ok());
        let image = image::DynamicImage::new_rgb8(2, 2);
        let mut data = std::io::Cursor::new(Vec::new());
        image.write_to(&mut data, image::ImageFormat::Png).unwrap();
        let (bytes, mime) = super::normalize_collection_image(data.get_ref(), "image/png").unwrap();
        assert_eq!(mime, "image/webp");
        assert!(image::load_from_memory(&bytes).is_ok());
    }
}

#[cfg(test)]
mod import_auto_mix_tests {
    use super::{auto_mix_directions_for_import, CollectionExportItem, LevelExport};

    fn item(front: &str, direction: Option<&str>) -> CollectionExportItem {
        serde_json::from_value(serde_json::json!({
            "free_content_front": front,
            "free_content_back": "x",
            "direction": direction,
        }))
        .unwrap()
    }

    fn level(item_positions: Vec<usize>) -> LevelExport {
        LevelExport {
            name: "L".into(),
            description: None,
            min_cards: 5,
            min_success_rate: 0.8,
            position: 0,
            prerequisite_positions: vec![],
            item_positions,
        }
    }

    #[test]
    fn level_without_directions_gets_a_mix_not_all_both() {
        let items: Vec<_> = ["coi do", "coi ro do", "co'o"]
            .iter()
            .map(|f| item(f, None))
            .collect();
        let dirs = auto_mix_directions_for_import(&items, &[level(vec![0, 1, 2])]);
        // No audio on these items, so the listen slot is a flip card.
        assert_eq!(dirs[&0], "quiz_direct");
        assert_eq!(dirs[&1], "wordbank_reverse");
        assert_eq!(dirs[&2], "both");
    }

    #[test]
    fn imported_audio_enables_the_listen_slot() {
        let mut items: Vec<_> = ["a", "b", "c"].iter().map(|f| item(f, None)).collect();
        items[2].sound_url = Some("data:audio/mpeg;base64,SUQz".into());
        let dirs = auto_mix_directions_for_import(&items, &[level(vec![0, 1, 2])]);
        assert_eq!(dirs[&2], "listen_wordbank");
    }

    #[test]
    fn explicit_directions_are_kept() {
        let items = vec![
            item("a", Some("listen_wordbank")),
            item("b", Some("wordbank")),
        ];
        assert!(auto_mix_directions_for_import(&items, &[level(vec![0, 1])]).is_empty());
        // Mixed level: only the item without a direction is filled in.
        let items = vec![item("a", Some("wordbank")), item("b", None)];
        let dirs = auto_mix_directions_for_import(&items, &[level(vec![0, 1, 9])]);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[&1], "wordbank");
    }
}
