//! Hybrid ranking across every public discussion source, with document-level RRF.
use deadpool_postgres::Pool;
use pgvector::Vector;
use serde::Deserialize;
use std::collections::HashMap;

use super::dto::{WaveRelevance, WaveSearchHit, WavesSearchQuery, WavesSearchResponse};
use crate::comments::service::search_comment_hits;
use crate::utils::embeddings::{embeddings_disabled, get_embedding};

type SearchError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Deserialize)]
struct RankedHit {
    kind: String,
    source_id: i32,
    source: String,
    title: String,
    edited_at: Option<chrono::DateTime<chrono::Utc>>,
    score: f64,
    lexical_match: bool,
    semantic_match: bool,
    excerpt: String,
}

fn order_clause(sort_by: &str, asc: bool) -> String {
    let direction = if asc { "ASC" } else { "DESC" };
    match sort_by {
        "time" => format!("edited_at {direction} NULLS LAST, score DESC, document_id ASC"),
        "reactions" => format!("reactions {direction}, score DESC, document_id ASC"),
        "replies" => format!("replies {direction}, score DESC, document_id ASC"),
        _ => format!("match_priority {direction}, score {direction}, document_id ASC"),
    }
}

pub(super) async fn search(
    pool: &Pool,
    query: &WavesSearchQuery,
    current_user_id: Option<i32>,
) -> Result<WavesSearchResponse, SearchError> {
    let embedding = if embeddings_disabled() {
        None
    } else {
        match get_embedding(query.search.as_deref().unwrap_or("").trim()).await {
            Ok(vector) => Some(Vector::from(vector)),
            Err(e) => {
                log::warn!("Discussion query embedding unavailable; using lexical search: {e}");
                None
            }
        }
    };
    search_with_embedding(pool, query, current_user_id, embedding).await
}

async fn search_with_embedding(
    pool: &Pool,
    query: &WavesSearchQuery,
    current_user_id: Option<i32>,
    embedding: Option<Vector>,
) -> Result<WavesSearchResponse, SearchError> {
    let term = query.search.as_deref().unwrap_or("").trim();
    let source = query.source.as_deref().unwrap_or("all");
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = page.saturating_sub(1).saturating_mul(per_page);
    let minimum_similarity = std::env::var("WAVE_SEARCH_MIN_SIMILARITY")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
        .unwrap_or(0.4);
    let order = order_clause(
        query.sort_by.as_deref().unwrap_or("relevance"),
        query.sort_order.as_deref() == Some("asc"),
    );
    let sql = include_str!("relevance.sql")
        .replace("/*CANONICAL*/", include_str!("canonical.sql"))
        .replace("/*ORDER*/", &order);
    let mut client = pool.get().await?;
    let tx = client.transaction().await?;
    // pgvector iterative scans refill filtered ANN lists instead of starving small sources.
    tx.batch_execute(
        "SET LOCAL hnsw.iterative_scan = 'strict_order'; SET LOCAL hnsw.ef_search = 200",
    )
    .await?;
    let row = tx
        .query_one(
            &sql,
            &[
                &term,
                &source,
                &query.collection_id,
                &embedding,
                &minimum_similarity,
                &per_page,
                &offset,
            ],
        )
        .await?;
    let total: i64 = row.get("total");
    let hits: Vec<RankedHit> = serde_json::from_value(row.get("hits"))?;
    tx.commit().await?;
    // Return the connection before batch hydration (also works with a small pool).
    drop(client);

    let comment_ids: Vec<i32> = hits
        .iter()
        .filter(|h| h.kind == "comment")
        .map(|h| h.source_id)
        .collect();
    let mut comments: HashMap<_, _> = if comment_ids.is_empty() {
        HashMap::new()
    } else {
        search_comment_hits(pool, &comment_ids, current_user_id)
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .into_iter()
            .map(|c| (c.comment_id, c))
            .collect()
    };
    let mail_ids: Vec<i32> = hits
        .iter()
        .filter(|h| h.kind == "mail")
        .map(|h| h.source_id)
        .collect();
    let client = pool.get().await?;
    let mut messages: HashMap<i32, crate::mailarchive::Message> = if mail_ids.is_empty() {
        HashMap::new()
    } else {
        client.query("SELECT m.*, (SELECT count(*) FROM message_spam_votes s WHERE s.message_id = m.id) AS spam_vote_count
                      FROM messages m WHERE m.id = ANY($1)", &[&mail_ids]).await?
            .into_iter().map(|row| { let message = crate::mailarchive::Message::from(row); (message.id, message) }).collect()
    };
    let wiki_ids: Vec<i32> = hits
        .iter()
        .filter(|h| h.kind == "wiki")
        .map(|h| h.source_id)
        .collect();
    let mut wiki_previews: HashMap<i32, (i32, String)> = if wiki_ids.is_empty() {
        HashMap::new()
    } else {
        client
            .query(
                "SELECT page_id, namespace, markdown FROM wiki_articles WHERE page_id = ANY($1)",
                &[&wiki_ids],
            )
            .await?
            .iter()
            .map(|r| {
                let markdown: String = r.get("markdown");
                (
                    r.get("page_id"),
                    (
                        r.get("namespace"),
                        crate::wiki::markdown::rewrite_wiki_links_for_lensisku(&markdown),
                    ),
                )
            })
            .collect()
    };
    let native_wiki_ids: Vec<i32> = hits
        .iter()
        .filter(|h| h.kind == "native_wiki")
        .map(|h| h.source_id)
        .collect();
    let mut native_wiki_previews: HashMap<i32, String> = if native_wiki_ids.is_empty() {
        HashMap::new()
    } else {
        client
            .query(
                "SELECT definitionid, definition FROM definitions WHERE definitionid = ANY($1)",
                &[&native_wiki_ids],
            )
            .await?
            .iter()
            .map(|r| (r.get("definitionid"), r.get("definition")))
            .collect()
    };
    let mut items = Vec::with_capacity(hits.len());
    for hit in hits {
        let relevance = Some(WaveRelevance {
            score: hit.score,
            lexical_match: hit.lexical_match,
            semantic_match: hit.semantic_match,
            excerpt: hit.excerpt.clone(),
        });
        match hit.kind.as_str() {
            "comment" => {
                if let Some(comment) = comments.remove(&hit.source_id) {
                    items.push(WaveSearchHit::Comment {
                        comment,
                        import_source: if hit.source == "comments" {
                            None
                        } else {
                            Some(hit.source)
                        },
                        relevance,
                    });
                }
            }
            "mail" => {
                if let Some(message) = messages.remove(&hit.source_id) {
                    items.push(WaveSearchHit::Mail { message, relevance });
                }
            }
            "wiki" | "native_wiki" => {
                // Ranking excerpts can be plain text; previews need the page Markdown.
                let (namespace, markdown) = if hit.kind == "wiki" {
                    wiki_previews.remove(&hit.source_id).unwrap_or_default()
                } else {
                    (
                        0,
                        native_wiki_previews
                            .remove(&hit.source_id)
                            .unwrap_or_default(),
                    )
                };
                items.push(WaveSearchHit::Wiki {
                    article: crate::wiki::dto::WikiSearchHit {
                        page_id: hit.source_id,
                        namespace,
                        article_url: format!("/wiki/{}", urlencoding::encode(&hit.title)),
                        title: hit.title,
                        last_edited: hit.edited_at,
                        content_preview: crate::wiki::service::truncate_preview(&markdown),
                    },
                    relevance,
                });
            }
            _ => {}
        }
    }
    Ok(WavesSearchResponse {
        items,
        total,
        page,
        per_page,
    })
}
