use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use utoipa::ToSchema;

use crate::comments::dto::ReactionResponse;
use crate::versions::VersionDiff;

#[derive(Debug, Clone)]
pub struct SearchDefinitionsParams {
    pub page: i64,
    pub per_page: i64,
    pub search_term: String,
    pub include_comments: bool,
    pub sort_by: String,
    pub sort_order: String,
    pub languages: Option<Vec<i32>>,
    pub selmaho: Option<String>,
    /// When set, only definitions by these authors are returned.
    pub usernames: Option<Vec<String>>,
    /// When set, definitions by these authors are omitted.
    pub exclude_usernames: Option<Vec<String>>,
    pub word_type: Option<Vec<i16>>,
    pub source_langid: Option<i32>,
    pub search_in_phrases: Option<bool>,
    /// When false, semantic search skips the extra COUNT query and sets `total` to the number of rows returned.
    pub include_total_count: bool,
    /// Exclude this definition from semantic neighbors (find-similar mode).
    pub exclude_definition_id: Option<i32>,
}

/// Parameters for building the semantic similarity graph (embedding neighborhood + k-NN edges).
#[derive(Debug, Clone)]
pub struct SemanticGraphParams {
    pub search_term: String,
    pub languages: Option<Vec<i32>>,
    pub selmaho: Option<String>,
    /// When set, only definitions by these authors are returned.
    pub usernames: Option<Vec<String>>,
    /// When set, definitions by these authors are omitted.
    pub exclude_usernames: Option<Vec<String>>,
    pub word_type: Option<Vec<i16>>,
    pub source_langid: Option<i32>,
    pub search_in_phrases: Option<bool>,
    pub min_vote: i32,
    pub limit: i64,
    pub k_neighbors: usize,
    pub min_similarity: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValsiEntry {
    pub valsiid: i32,
    pub word: String,
    pub type_id: i16,
    pub type_name: String,
    pub rafsi: Option<String>,
    pub langid: Option<i32>,
    pub comment_count: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RelatedValsi {
    pub valsiid: i32,
    pub word: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValsiDetail {
    pub valsiid: i32,
    pub word: String,
    pub type_name: String,
    pub rafsi: Option<String>,
    pub comment_count: Option<i64>,
    pub source_langid: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub decomposition: Option<Vec<String>>,
    /// Score-optimal spelling when this is a non-canonical lujvo.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub canonical_word: Option<String>,
    pub related_forms: Vec<RelatedValsi>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DefinitionDetail {
    pub definitionid: i32,
    pub valsiword: String,
    pub valsiid: i32,
    pub langid: i32,
    pub definition: String,
    pub notes: Option<String>,
    pub etymology: Option<String>,
    pub selmaho: Option<String>,
    pub jargon: Option<String>,
    pub definitionnum: i32,
    pub langrealname: String,
    pub username: String,
    pub time: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub type_name: String,
    /// Inherited official assignments stored on the word.
    #[serde(default)]
    pub official_rafsi: Option<String>,
    /// Proposals supplied by this definition.
    #[serde(default)]
    pub experimental_rafsi: Option<String>,
    pub rafsi: Option<String>,
    pub score: f32,
    pub comment_count: Option<i64>,
    pub gloss_keywords: Option<Vec<KeywordMapping>>,
    pub place_keywords: Option<Vec<KeywordMapping>>,
    pub user_vote: Option<i32>,
    pub owner_only: bool,
    pub can_edit: bool,
    pub has_image: bool,
    pub sound_url: Option<String>,
    #[schema(value_type = Vec<f32>)]
    pub embedding: Option<Vec<f32>>,
    pub similarity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<Example>>,
    /// Cached lujvo decomposition (source words). Set for lujvo / non-canonical lujvo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decomposition: Option<Vec<String>>,
    /// Score-optimal spelling when `type_name` is non-canonical lujvo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_word: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Example {
    pub exampleid: i32,
    pub content: String,
    pub examplenum: i32,
    pub username: String,
    pub time: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DefinitionResponse {
    pub definitions: Vec<DefinitionDetail>,
    pub decomposition: Vec<String>,
    pub total: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filtered_collection_items: Vec<crate::jbovlaste::dto::FilteredCollectionHit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filtered_definitions: Vec<DefinitionDetail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collection_membership: Vec<crate::collections::dto::SearchHitMembership>,
}

impl DefinitionResponse {
    pub fn plain(
        definitions: Vec<DefinitionDetail>,
        decomposition: Vec<String>,
        total: i64,
    ) -> Self {
        Self {
            definitions,
            decomposition,
            total,
            filtered_collection_items: Vec::new(),
            filtered_definitions: Vec::new(),
            collection_membership: Vec::new(),
        }
    }
}

/// Vote aggregates should be selected as `::bigint` (`definitionvotes.value` is REAL, so `SUM` is float8 otherwise).
#[inline]
pub(crate) fn row_vote_score_i32(row: &tokio_postgres::Row, column: &str) -> i32 {
    let v: i64 = row.get(column);
    v.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

#[inline]
pub(crate) fn row_vote_score_f32(row: &tokio_postgres::Row, column: &str) -> f32 {
    row_vote_score_i32(row, column) as f32
}

/// Display order for a space-separated rafsi list: CVC, CCV, CVV, CV'V, then long rafsi.
/// Tokens of the same shape keep their original order.
pub(crate) fn ordered_rafsi(rafsi: Option<String>) -> Option<String> {
    let raw = rafsi?;
    if raw.split_whitespace().nth(1).is_none() {
        return Some(raw);
    }
    let mut parts: Vec<&str> = raw.split_whitespace().collect();
    parts.sort_by_key(|token| rafsi_shape_rank(token));
    Some(parts.join(" "))
}

pub(crate) fn row_ordered_rafsi(row: &tokio_postgres::Row, column: &str) -> Option<String> {
    ordered_rafsi(row.try_get(column).ok().flatten())
}

fn rafsi_shape_rank(rafsi: &str) -> u8 {
    let chars: Vec<char> = rafsi.chars().collect();
    let is_vowel = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u');
    let is_consonant = |c: char| c.is_ascii_alphabetic() && !is_vowel(c);
    match chars.as_slice() {
        [c1, v, c2] if is_consonant(*c1) && is_vowel(*v) && is_consonant(*c2) => 0, // CVC
        [c1, c2, v] if is_consonant(*c1) && is_consonant(*c2) && is_vowel(*v) => 1, // CCV
        [c, v1, v2] if is_consonant(*c) && is_vowel(*v1) && is_vowel(*v2) => 2,     // CVV
        [c, v1, '\'', v2] if is_consonant(*c) && is_vowel(*v1) && is_vowel(*v2) => 3, // CV'V
        _ => 4,                                                                     // long rafsi
    }
}

impl From<tokio_postgres::Row> for DefinitionDetail {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            similarity: row
                .get::<_, Option<f64>>("similarity")
                .map(|d| (1.0 - d).max(0.0)),
            definitionid: row.get("definitionid"),
            valsiword: row.get("valsiword"),
            valsiid: row.get("valsiid"),
            langid: row.get("langid"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            etymology: row.get("etymology"),
            selmaho: row.get("selmaho"),
            jargon: row.get("jargon"),
            definitionnum: row.get("definitionnum"), // Note the field name matches DB
            langrealname: row.get("langrealname"),
            username: row.get("username"),
            time: row.get("time"),
            type_name: row.get("type_name"),
            official_rafsi: row_ordered_rafsi(&row, "official_rafsi"),
            experimental_rafsi: row_ordered_rafsi(&row, "experimental_rafsi"),
            rafsi: row_ordered_rafsi(&row, "rafsi"),
            score: row_vote_score_f32(&row, "score"),
            user_vote: row.get("user_vote"),
            comment_count: row.get("comment_count"),
            gloss_keywords: None, // These get filled in separately
            place_keywords: None,
            owner_only: row.get("owner_only"),
            can_edit: row.get("can_edit"),
            created_at: row.get("created_at"),
            has_image: row.get("has_image"),
            sound_url: row.get("sound_url"),
            embedding: None,
            metadata: row.get("metadata"),
            examples: None,
            decomposition: None,
            canonical_word: row
                .try_get::<_, Option<String>>("cached_canonical_word")
                .ok()
                .flatten()
                .or_else(|| {
                    row.try_get::<_, Option<String>>("canonical_word")
                        .ok()
                        .flatten()
                }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct KeywordMapping {
    pub word: String,
    pub meaning: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecentChange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    /// Distinguishes wiki discussion context from dictionary definitions.
    #[serde(default)]
    pub is_wiki: bool,
    pub change_type: String, // valsi, definition, comment
    pub word: String,
    pub content: serde_json::Value,
    pub valsi_id: Option<i32>,
    pub lang_id: Option<i32>,
    pub natlang_word_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub thread_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub username: String,
    /// When set, this is a mw.lojban.org account — not a lensisku profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_url: Option<String>,
    pub time: i32,
    pub language_name: Option<String>,
    pub language_english_name: Option<String>,
    pub language_lojban_name: Option<String>,
    pub diff: Option<VersionDiff>,
    // Comment-type only (optional so non-comment changes stay small)
    pub comment_num: Option<i32>,
    pub valsi_word: Option<String>,
    pub parent_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reactions: Option<Vec<ReactionResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bookmarked: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValsiType {
    pub type_id: i16,
    pub descriptor: String,
}

#[cfg(test)]
mod tests {
    use super::ordered_rafsi;

    #[test]
    fn rafsi_display_order_is_cvc_ccv_cvv_apostrophe_then_long() {
        let sorted = ordered_rafsi(Some("bai bap bapl ba'u bla".into()));
        assert_eq!(sorted.as_deref(), Some("bap bla bai ba'u bapl"));

        let condi = ordered_rafsi(Some("cno coi con cond".into()));
        assert_eq!(condi.as_deref(), Some("con cno coi cond"));

        let already = ordered_rafsi(Some("dav dza".into()));
        assert_eq!(already.as_deref(), Some("dav dza"));
    }
}

impl From<tokio_postgres::Row> for ValsiType {
    fn from(row: tokio_postgres::Row) -> Self {
        ValsiType {
            type_id: row.get("typeid"),
            descriptor: row.get("descriptor"),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValsiTypeListResponse {
    pub types: Vec<ValsiType>,
}

#[derive(Debug)]
pub struct DeleteDefinitionResult {
    pub definition_deleted: bool,
    pub valsi_deleted: bool,
    pub has_remaining_definitions: bool,
}
