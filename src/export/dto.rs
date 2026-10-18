use serde::{Deserialize, Serialize};

use super::models::{CollectionExportItem, DictionaryEntry};

#[derive(Debug, Default, Deserialize)]
pub struct ExportOptions {
    pub format: Option<String>,
    pub positive_scores_only: Option<bool>,
    pub collection_id: Option<i32>,
    /// Language tag for the source/word language (defaults to Lojban if omitted)
    pub source_lang: Option<String>,
}

/// Query for `GET /export/search` — same filters as dictionary / collection search.
/// Authors∪collections is a union: collection matches plus author-scoped dictionary.
/// When `collection_ids` is set without include-authors, the unscoped global dictionary
/// fallback is omitted. When `collection_only` is true, only collection items matching
/// the filters are exported (authors filter collection items; no dictionary union).
#[derive(Debug, Default, Deserialize)]
pub struct SearchExportQuery {
    pub format: Option<String>,
    pub search: Option<String>,
    pub languages: Option<String>,
    pub selmaho: Option<String>,
    /// Comma-separated valsi type ids (`1` or `1,4`).
    pub word_type: Option<String>,
    pub username: Option<String>,
    pub exclude_usernames: Option<String>,
    pub source_langid: Option<i32>,
    pub search_in_phrases: Option<bool>,
    pub semantic: Option<bool>,
    pub collection_ids: Option<String>,
    /// When true, also export matches from the top 100 public collections by cards learnt in the last 3 months.
    pub include_popular_collections: Option<bool>,
    pub collection_only: Option<bool>,
    /// When true with a single `collection_ids` value and `format=json`, export the full
    /// collection backup (`CollectionFullExport`: metadata, all items with images, levels).
    pub full_collection: Option<bool>,
}

/// Combined JSON body for a filtered search export.
#[derive(Serialize)]
pub struct SearchExportJson {
    pub collection_items: Vec<CollectionExportItem>,
    pub definitions: Vec<DictionaryEntry>,
}

/// True when the query is constrained enough to export (not a full-corpus dump).
/// Language-only filters do not count; use `/export/dictionary/{lang}` for that.
pub fn has_search_export_constraint(query: &SearchExportQuery) -> bool {
    if !query.search.as_deref().unwrap_or("").trim().is_empty() {
        return true;
    }
    if !query.selmaho.as_deref().unwrap_or("").trim().is_empty() {
        return true;
    }
    if crate::jbovlaste::dto::parse_username_list(&query.username).is_some() {
        return true;
    }
    if crate::jbovlaste::dto::parse_username_list(&query.exclude_usernames).is_some() {
        return true;
    }
    if crate::jbovlaste::dto::parse_word_type_list(&query.word_type).is_some() {
        return true;
    }
    if query.include_popular_collections == Some(true) {
        return true;
    }
    if query.full_collection.unwrap_or(false) {
        return query
            .collection_ids
            .as_deref()
            .map(|s| crate::collections::dto::parse_positive_id_list(s, 50))
            .is_some_and(|ids| ids.len() == 1);
    }
    if query
        .collection_ids
        .as_deref()
        .map(|s| crate::collections::dto::parse_positive_id_list(s, 50))
        .is_some_and(|ids| !ids.is_empty())
    {
        return true;
    }
    if query.source_langid.is_some_and(|id| id != 1) {
        return true;
    }
    query.search_in_phrases == Some(false)
}
