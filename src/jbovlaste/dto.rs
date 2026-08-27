use super::{models::KeywordMapping, DefinitionDetail, RecentChange};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

#[derive(Deserialize, ToSchema)]
pub struct SearchDefinitionsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    /// When set, semantic search uses this definition's stored embedding instead of embedding `search`.
    pub definition_id: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub include_comments: Option<bool>,
    pub languages: Option<String>,
    pub selmaho: Option<String>,
    pub word_type: Option<i16>,
    /// Comma-separated definition author usernames. When set, only those authors' definitions are returned.
    pub username: Option<String>,
    /// Comma-separated usernames whose definitions should be excluded.
    pub exclude_usernames: Option<String>,
    pub source_langid: Option<i32>,
    pub fast: Option<bool>,
    pub search_in_phrases: Option<bool>,
    /// Comma-separated public collection ids. With `include_global_group`, these hits
    /// appear in `filtered_collection_items` (not AND’d with `username`).
    pub collection_ids: Option<String>,
    /// When true, `username` / `collection_ids` form a priority filtered group, and
    /// `definitions` is the unscoped global group (excluding ids already in the filtered group).
    pub include_global_group: Option<bool>,
    /// Expand a content-deduped collection hit: return every matching collection item
    /// (same valsi + definition + source language + target language) instead of one card.
    pub expand_collection_item: Option<i32>,
}

/// Parse a comma-separated username query param into a non-empty list.
pub fn parse_username_list(value: &Option<String>) -> Option<Vec<String>> {
    let names: Vec<String> = value
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if names.is_empty() {
        None
    } else {
        Some(names)
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NonLojbanDefinitionsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub languages: Option<String>, // Filter by definition language
    /// Comma-separated definition author usernames. When set, only those authors' definitions are returned.
    pub username: Option<String>,
    /// Comma-separated usernames whose definitions should be excluded.
    pub exclude_usernames: Option<String>,
    pub source_langid: Option<i32>, // Filter by the source language of the valsi
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListDefinitionsQuery {
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
    pub search: Option<String>,
    #[schema(default = "created_at", example = "updated_at")]
    pub sort_by: Option<String>,
    #[schema(default = "desc", example = "asc")]
    pub sort_order: Option<String>,
    pub languages: Option<String>, // Comma-separated list of langids
    pub selmaho: Option<String>,
    pub word_type: Option<i16>,
    pub user_id: Option<i32>,
    pub source_langid: Option<i32>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DefinitionListResponse {
    /// Global (or sole) dictionary page. When `include_global_group` was set, this group
    /// is not filtered by include-authors / collections and omits ids already in the
    /// filtered groups below.
    pub definitions: Vec<DefinitionDetail>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decomposition: Vec<String>,
    /// Hits from `collection_ids` (authors∪collections OR — not AND’d with `username`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filtered_collection_items: Vec<FilteredCollectionHit>,
    /// Hits from include-authors when `include_global_group` is on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filtered_definitions: Vec<DefinitionDetail>,
}

/// Compact collection-item row for the priority filtered group on dictionary search.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilteredCollectionHit {
    pub item_id: i32,
    pub definition_id: Option<i32>,
    pub word: Option<String>,
    pub definition: Option<String>,
    pub notes: Option<String>,
    pub lang_id: Option<i32>,
    pub username: Option<String>,
    pub valsi_id: Option<i32>,
    pub free_content_front: Option<String>,
    pub free_content_back: Option<String>,
    pub has_front_image: bool,
    pub has_back_image: bool,
    pub has_sound: bool,
    pub sound_url: Option<String>,
    pub canonical_form: Option<String>,
    pub collection_id: Option<i32>,
    pub collection_name: Option<String>,
    pub source_langid: Option<i32>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub collection_created_at: Option<DateTime<Utc>>,
    /// Present when this row stands in for several identical collection items.
    pub match_count: Option<i32>,
}

impl From<&crate::collections::dto::CollectionItemResponse> for FilteredCollectionHit {
    fn from(item: &crate::collections::dto::CollectionItemResponse) -> Self {
        Self {
            item_id: item.item_id,
            definition_id: item.definition_id,
            word: item.word.clone(),
            definition: item.definition.clone(),
            notes: item.notes.clone().or_else(|| item.ci_notes.clone()),
            lang_id: item.lang_id.or(item.language_id),
            username: item.username.clone(),
            valsi_id: item.valsi_id,
            free_content_front: item.free_content_front.clone(),
            free_content_back: item.free_content_back.clone(),
            has_front_image: item.has_front_image,
            has_back_image: item.has_back_image,
            has_sound: item.has_sound,
            sound_url: item.sound_url.clone(),
            canonical_form: item.canonical_form.clone(),
            collection_id: item.collection_id,
            collection_name: item.collection_name.clone(),
            source_langid: item.source_langid,
            collection_created_at: item.collection_created_at,
            match_count: item.match_count,
        }
    }
}
#[derive(Debug, Deserialize, ToSchema)]
pub struct ValsiDefinitionsQuery {
    pub langid: Option<i32>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetImageDefinitionQuery {
    pub image_id: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddDefinitionRequest {
    pub word: String,
    pub definition: String,
    pub notes: Option<String>,
    pub etymology: Option<String>,
    pub lang_id: i32,
    pub source_langid: Option<i32>,
    pub selmaho: Option<String>,
    pub jargon: Option<String>,
    pub rafsi: Option<String>,
    pub gloss_keywords: Option<Vec<KeywordMapping>>,
    pub place_keywords: Option<Vec<KeywordMapping>>,
    pub owner_only: Option<bool>,
    #[schema(format = "binary")]
    pub image: Option<ImageData>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// When true, the entry is a native wiki page: it uses valsi type `wiki`,
    /// skips Lojban morphology validation, and stores only the markdown body.
    #[serde(default)]
    pub is_wiki: Option<bool>,
    /// Optional version commit message (wiki and definitions).
    #[serde(default)]
    pub commit_message: Option<String>,
    /// Optimistic concurrency: reject if `definitions.time` differs.
    #[serde(default)]
    pub expected_time: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AddValsiResponse {
    pub success: bool,
    pub word_type: String,
    pub definition_id: i32,
    pub error: Option<String>,
    /// Soft warning (e.g. rafsi also used by another valsi). Does not block save.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDefinitionRequest {
    pub lang_id: i32,
    pub definition: String,
    pub notes: Option<String>,
    pub etymology: Option<String>,
    pub gloss_keywords: Option<Vec<KeywordMapping>>,
    pub place_keywords: Option<Vec<KeywordMapping>>,
    pub selmaho: Option<String>,
    pub jargon: Option<String>,
    pub rafsi: Option<String>,
    pub owner_only: Option<bool>,
    #[schema(format = "binary")]
    pub image: Option<ImageData>,
    pub remove_image: Option<bool>,
    /// When true, marks the updated entry as a native wiki page.
    #[serde(default)]
    pub is_wiki: Option<bool>,
    /// Optional version commit message (wiki and definitions).
    #[serde(default)]
    pub commit_message: Option<String>,
    /// Optimistic concurrency: reject if `definitions.time` differs.
    #[serde(default)]
    pub expected_time: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RenameWikiRequest {
    pub new_word: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RenameWikiResponse {
    pub success: bool,
    pub old_word: String,
    pub new_word: String,
    pub definition_id: i32,
    pub redirect_stub_definition_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WikiByDefinitionResponse {
    pub word: String,
    pub definition_id: i32,
    pub valsiid: i32,
    pub is_redirect: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_to: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateDefinitionResponse {
    pub success: bool,
    pub error: Option<String>,
    /// Soft warning (e.g. rafsi also used by another valsi). Does not block save.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RafsiOverlapQuery {
    /// Space-separated rafsi to check.
    pub rafsi: String,
    /// Current entry word; overlaps on this same valsi are ignored.
    pub word: Option<String>,
    /// Current valsi id when known (edit / existing word).
    pub valsi_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RafsiOverlapHit {
    pub word: String,
    pub word_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RafsiOverlapResponse {
    pub overlap: Option<RafsiOverlapHit>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VoteRequest {
    pub definition_id: i32,
    pub downvote: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VoteResponse {
    pub success: bool,
    pub message: String,
    pub word: Option<String>,
    pub score: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserVoteResponse {
    pub vote: Option<i32>, // 1 for upvote, -1 for downvote, None if no vote
    pub definition_id: i32,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct BulkVoteRequest {
    #[validate(
        length(
            max = 1000,
            message = "Cannot request more than 1000 definitions at once"
        ),
        custom(function = "validate_unique_sorted")
    )]
    pub definition_ids: Vec<i32>,
}

fn validate_unique_sorted(ids: &[i32]) -> Result<(), ValidationError> {
    let mut prev = None;
    for id in ids {
        if prev >= Some(id) {
            return Err(ValidationError::new(
                "Definition IDs must be unique and sorted",
            ));
        }
        prev = Some(id);
    }
    Ok(())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkVoteResponse {
    pub votes: std::collections::HashMap<String, Option<i32>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClientIdGroup {
    pub client_id: String,
    pub count: i64, // Using i64 for count, consistent with pagination totals
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VoteError {
    pub error: String,
    pub code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecentChangesResponse {
    pub changes: Vec<RecentChange>,
    /// Total count (0 when using cursor-based pagination).
    pub total: i64,
    /// Opaque cursor for the next page when using cursor-based pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecentChangesQuery {
    pub limit: Option<i64>,
    pub types: Option<String>,
    /// Cursor for keyset pagination (when present, overrides page and no time window is used).
    pub after: Option<String>,
    /// When true, excludes new valsi (entry) changes. Intended for the home page.
    #[serde(default)]
    pub home: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkImportRequest {
    /// CSV content with columns: gismu,definition,notes,glosswords
    #[schema(format = "binary")]
    pub csv: String,
    /// Target language ID for all definitions
    pub lang_id: i32,
}

#[derive(Debug)]
pub struct BulkImportParams<'a> {
    pub csv_data: &'a str,
    pub lang_id: i32,
    pub client_id: String,
    pub import_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageUploadRequest {
    #[schema(format = "binary")]
    pub image: ImageData,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageData {
    #[schema(format = "byte")]
    pub data: String, // Base64 encoded image data
    pub mime_type: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkDefinitionsRequest {
    pub definition_id: i32,
    pub translation_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DefinitionTranslation {
    pub definitionid: i32,
    pub valsiword: String,
    pub definition: String,
    pub langid: i32,
    pub lang_name: String,
    pub link_id: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExportPairsQuery {
    pub from_lang: i32,
    pub to_lang: i32,
}

/// Query parameters for [`GET /jbovlaste/semantic-graph`](crate::jbovlaste::controller::semantic_graph).
#[derive(Debug, Deserialize, ToSchema)]
pub struct SemanticGraphQuery {
    /// When true, returns a stratified preview (no `search` or embedding): top definitions per word type by vote score, then k-NN edges. Cached separately from anchored graphs.
    #[serde(default)]
    pub preview: Option<bool>,
    /// When `true` or omitted, the anchor vector is computed from the raw `search` text. When `false`, the server resolves `search` as a valsi and uses the stored embedding of its **English** (`langid` 2) definition as the anchor (no on-the-fly embedding of the query string).
    #[serde(default)]
    pub semantic: Option<bool>,
    pub search: Option<String>,
    pub languages: Option<String>,
    pub selmaho: Option<String>,
    pub word_type: Option<i16>,
    /// Comma-separated definition author usernames. When set, only those authors' definitions are returned.
    pub username: Option<String>,
    /// Comma-separated usernames whose definitions should be excluded.
    pub exclude_usernames: Option<String>,
    pub source_langid: Option<i32>,
    pub search_in_phrases: Option<bool>,
    /// Minimum aggregate vote score (sum of votes per definition). Default 1 (matches classic semantic search `score > 0`).
    pub min_vote: Option<i32>,
    /// Max nodes (hard-capped server-side).
    pub limit: Option<i64>,
    /// Per-node top-k neighbors used to build sparse edges.
    pub k_neighbors: Option<i64>,
    /// Drop edges with pairwise cosine similarity below this (0..=1).
    pub min_similarity: Option<f32>,
    /// When set (e.g. the valsi under the current viewport center after zoom), the graph is the
    /// top-`limit` neighborhood of that word's stored embedding — not the stratified preview and
    /// not the `search` text embedding.
    pub focus: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SemanticGraphNode {
    pub id: String,
    pub definitionid: i32,
    pub word: String,
    /// Short label for the node (word plus optional hint).
    pub label: String,
    pub type_name: String,
    pub lang_name: String,
    pub score: i32,
    /// Similarity to the anchor query (1 - cosine distance), when available.
    pub query_similarity: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SemanticGraphEdge {
    pub source: String,
    pub target: String,
    pub similarity: f32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SemanticGraphResponse {
    pub nodes: Vec<SemanticGraphNode>,
    pub edges: Vec<SemanticGraphEdge>,
}
