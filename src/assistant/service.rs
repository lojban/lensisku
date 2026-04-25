#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_web_lab::sse;
use futures::future::join_all;
use deadpool_postgres::Pool;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::sleep;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::utils::embeddings::get_batch_embeddings;
use crate::error::AppError;
use crate::jbovlaste::models::{DefinitionDetail, DefinitionResponse, SearchDefinitionsParams};
use crate::middleware::cache::{generate_assistant_semantic_cache_key, RedisCache};
use crate::utils::openrouter_models::{
    evict_openrouter_assistant_model_from_cache, fetch_latest_openrouter_models,
    load_or_fetch_openrouter_candidates, ModelIdName,
};
use crate::jbovlaste::service::semantic_search;
use std::borrow::Cow;

use super::context_compress;
use super::dto::{AssistantStep, ChatMessage, ChatRequest, ToolCallDto};
use super::persist::ChatPersistState;

/// When `true`, streaming runs two OpenRouter models in parallel when two candidates exist.
/// Doubles provider cost/latency for parallel redundancy; set to `false` to use a single model.
const ASSISTANT_PARALLEL_DUAL_MODEL: bool = true;

/// In-process cache for `languages.tag` → `langid` (clears on process restart; disable via `ASSISTANT_LANG_TAG_CACHE_DISABLE`).
static JBOVLASTE_LANG_TAG_CACHE: Lazy<RwLock<HashMap<String, i32>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Limits concurrent DB+embedding work per batched tool call (embedding is batched; this caps parallel `semantic_search` calls).
static ASSISTANT_SEMANTIC_SUBQUERY_SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| {
    let n = env::var("ASSISTANT_SEMANTIC_SUBQUERY_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24)
        .clamp(1, 64);
    Arc::new(Semaphore::new(n))
});

fn assistant_lang_tag_cache_enabled() -> bool {
    !env::var("ASSISTANT_LANG_TAG_CACHE_DISABLE")
        .ok()
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn assistant_semantic_cache_ttl() -> Duration {
    Duration::from_secs(
        env::var("ASSISTANT_SEMANTIC_CACHE_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600)
            .clamp(60, 86_400),
    )
}

fn assistant_semantic_cache_disabled() -> bool {
    env::var("ASSISTANT_SEMANTIC_CACHE_DISABLE")
        .ok()
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn agent_max_iterations() -> u32 {
    env::var("ASSISTANT_MAX_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(15)
        .clamp(1, 30)
}

#[derive(Debug, Clone, Serialize)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct Tool {
    r#type: String,
    function: ToolFunction,
}

#[derive(Debug, Clone, Serialize)]
struct ChatCompletionMessageRequest {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessageRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
    /// OpenAI-compatible: when true, the model may return several tool calls in one turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessageResponse,
}

/// OpenRouter/OpenAI-style error payload (e.g. 200 OK with {"error":{"message":"...","code":500}}).
#[derive(Debug, Deserialize)]
struct OpenRouterErrorPayload {
    #[serde(default)]
    error: OpenRouterErrorDetail,
}

#[derive(Debug, Deserialize, Default)]
struct OpenRouterErrorDetail {
    #[serde(default)]
    message: String,
    #[serde(default)]
    code: Option<u16>,
}

/// Ensure OpenRouter response is success; on HTTP error return body for debugging.
/// 5xx are returned as ExternalServiceRetryable so callers can retry.
async fn ensure_openrouter_status(
    res: reqwest::Response,
    label: &str,
) -> Result<reqwest::Response, AppError> {
    let status = res.status();
    if status.is_success() {
        return Ok(res);
    }
    let body = res
        .text()
        .await
        .unwrap_or_else(|_| String::from("(failed to read body)"));
    let message = format!(
        "{} returned {} {}",
        label,
        status,
        status.canonical_reason().unwrap_or("")
    );
    if status.is_server_error() {
        Err(AppError::ExternalServiceRetryable {
            message,
            raw_response: body,
        })
    } else {
        Err(AppError::ExternalServiceWithRaw {
            message,
            raw_response: body,
        })
    }
}

/// Deserialize OpenRouter response from response body; on error include raw body for debugging.
/// When the body is an error payload (e.g. 200 OK with {"error":{"message":"Internal Server Error","code":500}}),
/// returns ExternalServiceRetryable so callers can retry.
async fn parse_chat_response(
    res: reqwest::Response,
    label: &str,
) -> Result<ChatCompletionResponse, AppError> {
    let status = res.status();
    let body = res.text().await.map_err(|e| {
        AppError::ExternalService(format!("Failed to read {} response body: {}", label, e))
    })?;
    let body_trimmed = body.trim();
    match serde_json::from_str::<ChatCompletionResponse>(body_trimmed) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            // Check if body is an OpenRouter/OpenAI-style error (e.g. 200 with {"error":{...}}).
            let retryable = if let Ok(err_payload) =
                serde_json::from_str::<OpenRouterErrorPayload>(body_trimmed)
            {
                let code = err_payload.error.code;
                let msg = if err_payload.error.message.is_empty() {
                    format!("Invalid {} response: {}", label, e)
                } else {
                    format!("{}: {}", label, err_payload.error.message)
                };
                let is_server_error = code.map(|c| c >= 500).unwrap_or(true);
                if is_server_error {
                    log::warn!(
                        "OpenRouter {} returned error body (code {:?}), will retry: {}",
                        label,
                        code,
                        msg
                    );
                    Some((msg, body.clone()))
                } else {
                    None
                }
            } else {
                // Unrecognized shape; treat parse failure as retryable (transient malformed response).
                Some((format!("Invalid {} response: {}", label, e), body.clone()))
            };
            if let Some((message, raw_response)) = retryable {
                log::debug!(
                    "OpenRouter {} response (status {}): {}",
                    label,
                    status,
                    raw_response
                );
                return Err(AppError::ExternalServiceRetryable {
                    message,
                    raw_response,
                });
            }
            log::debug!(
                "OpenRouter {} response (status {}): {}",
                label,
                status,
                body
            );
            log::warn!(
                "OpenRouter {} parse error: {} (see debug log for raw body)",
                label,
                e
            );
            Err(AppError::ExternalServiceWithRaw {
                message: format!("Invalid {} response: {}", label, e),
                raw_response: body,
            })
        }
    }
}

const OPENROUTER_MAX_ATTEMPTS: u32 = 3;
const OPENROUTER_INITIAL_BACKOFF_MS: u64 = 500;

/// Runs an OpenRouter chat/completions request with retries on transient errors (5xx or error body).
async fn openrouter_chat_with_retry<F, Fut>(
    label: &str,
    mut run: F,
) -> Result<ChatCompletionResponse, AppError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<ChatCompletionResponse, AppError>>,
{
    let mut last_err = None;
    for attempt in 1..=OPENROUTER_MAX_ATTEMPTS {
        match run().await {
            Ok(r) => return Ok(r),
            Err(e) => {
                if let AppError::ExternalServiceRetryable { .. } = &e {
                    last_err = Some(e);
                    if attempt < OPENROUTER_MAX_ATTEMPTS {
                        let delay = Duration::from_millis(
                            OPENROUTER_INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1),
                        );
                        log::info!(
                            "OpenRouter {} retry {}/{} after {:?}",
                            label,
                            attempt,
                            OPENROUTER_MAX_ATTEMPTS,
                            delay
                        );
                        sleep(delay).await;
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }
    Err(last_err.unwrap())
}

#[derive(Debug, Deserialize, Clone)]
struct ChatCompletionMessageResponse {
    /// OpenRouter/OpenAI may send null for role or content (e.g. when message has tool_calls).
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct ToolCallFunction {
    /// Some providers send null for name or arguments.
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct ToolCall {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    r#type: Option<String>,
    function: ToolCallFunction,
}

fn tool_call_dto_to_internal(c: &ToolCallDto) -> ToolCall {
    ToolCall {
        id: c.id.clone(),
        r#type: c.r#type.clone(),
        function: ToolCallFunction {
            name: c.function.name.clone(),
            arguments: c.function.arguments.clone(),
        },
    }
}

fn error_indicates_context_limit(e: &AppError) -> bool {
    let mut text = String::new();
    match e {
        AppError::ExternalServiceWithRaw {
            message,
            raw_response,
        } => {
            text.push_str(message);
            text.push_str(raw_response);
        }
        AppError::ExternalServiceRetryable {
            message,
            raw_response,
        } => {
            text.push_str(message);
            text.push_str(raw_response);
        }
        AppError::ExternalService(m) => {
            text.push_str(m);
        }
        _ => return false,
    }
    let lower = text.to_lowercase();
    lower.contains("context length")
        || lower.contains("maximum context")
        || lower.contains("token limit")
        || lower.contains("too many tokens")
        || lower.contains("exceeds the context")
        || lower.contains("prompt is too long")
        || lower.contains("requested token")
}

/// Resolves jbovlaste `languages.tag` values (e.g. `en`, `ru`, `jbo`) to `langid` for DB filters.
/// Uses an in-process tag cache and a single `ANY($1::text[])` query for cache misses.
async fn resolve_jbovlaste_language_tags_to_langids(
    pool: &Pool,
    tags: &[String],
) -> Result<Option<Vec<i32>>, AppError> {
    let mut norm: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    for t in tags {
        let s = t.trim().to_lowercase();
        if !s.is_empty() && seen.insert(s.clone()) {
            norm.push(s);
        }
    }
    if norm.is_empty() {
        return Ok(None);
    }

    let mut resolved_map: HashMap<String, i32> = HashMap::with_capacity(norm.len());

    if assistant_lang_tag_cache_enabled() {
        let cache = JBOVLASTE_LANG_TAG_CACHE.read();
        for t in &norm {
            if let Some(&id) = cache.get(t) {
                resolved_map.insert(t.clone(), id);
            }
        }
    }

    let to_fetch: Vec<String> = norm
        .iter()
        .filter(|t| !resolved_map.contains_key(*t))
        .cloned()
        .collect();

    if !to_fetch.is_empty() {
        let client = pool
            .get()
            .await
            .map_err(|e| AppError::ExternalService(format!("Database pool error: {}", e)))?;
        let tag_refs: Vec<&str> = to_fetch.iter().map(|s| s.as_str()).collect();
        let rows = client
            .query(
                "SELECT lower(tag) AS tag, langid FROM languages WHERE lower(tag) = ANY($1::text[])",
                &[&tag_refs],
            )
            .await
            .map_err(|e| AppError::ExternalService(format!("language tag lookup failed: {}", e)))?;

        for row in rows {
            let tag: String = row.get("tag");
            let langid: i32 = row.get("langid");
            resolved_map.insert(tag.clone(), langid);
            if assistant_lang_tag_cache_enabled() {
                JBOVLASTE_LANG_TAG_CACHE.write().insert(tag, langid);
            }
        }

        for tag in &to_fetch {
            if !resolved_map.contains_key(tag) {
                return Err(AppError::BadRequest(format!(
                    "Unknown language tag `{}`. Use jbovlaste tags such as en, ru, es, jbo.",
                    tag
                )));
            }
        }
    }

    let mut ids = Vec::with_capacity(norm.len());
    for t in &norm {
        ids.push(
            *resolved_map
                .get(t)
                .expect("all norm tags resolved"),
        );
    }
    Ok(Some(ids))
}

/// Resolves optional `source_language` tag to `langid` (valsi source language). `None` = caller default.
async fn resolve_optional_source_language_tag(
    pool: &Pool,
    tag: Option<&str>,
) -> Result<Option<i32>, AppError> {
    let Some(s) = tag else {
        return Ok(None);
    };
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    let key = s.to_lowercase();
    if assistant_lang_tag_cache_enabled() {
        if let Some(&id) = JBOVLASTE_LANG_TAG_CACHE.read().get(&key) {
            return Ok(Some(id));
        }
    }
    let resolved = resolve_jbovlaste_language_tags_to_langids(pool, std::slice::from_ref(&key)).await?;
    let Some(v) = resolved else {
        return Err(AppError::Internal(
            "language tag resolution returned None unexpectedly".into(),
        ));
    };
    let id = v
        .first()
        .copied()
        .ok_or_else(|| AppError::Internal("empty language resolution".into()))?;
    Ok(Some(id))
}

#[derive(Clone)]
struct ResolvedSemanticFilters {
    languages_langids: Option<Vec<i32>>,
    source_langid: Option<i32>,
}

async fn resolve_semantic_search_language_filters(
    pool: &Pool,
    languages: Option<&[String]>,
    source_language: Option<&String>,
) -> Result<ResolvedSemanticFilters, AppError> {
    let languages_langids = match languages {
        None | Some([]) => None,
        Some(tags) => resolve_jbovlaste_language_tags_to_langids(pool, tags).await?,
    };
    let source_langid = resolve_optional_source_language_tag(
        pool,
        source_language.map(|s| s.as_str()),
    )
    .await?;
    Ok(ResolvedSemanticFilters {
        languages_langids,
        source_langid,
    })
}

fn truncate_utf8_prefix(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Full embedded system prompt (static instructions + reference dictionary). Rebuild with `scripts/build_assistant_core_dictionary.py`.
/// `ASSISTANT_CORE_DICT_MAX_CHARS` limits total UTF-8 bytes (name kept for compatibility).
fn assistant_embedded_system_prompt_cow() -> Cow<'static, str> {
    const EMBEDDED: &str = include_str!("assistant_system_prompt.txt");
    let max_chars: usize = env::var("ASSISTANT_CORE_DICT_MAX_CHARS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500_000);
    if EMBEDDED.len() <= max_chars {
        Cow::Borrowed(EMBEDDED)
    } else {
        let prefix = truncate_utf8_prefix(EMBEDDED, max_chars);
        let cut = prefix.rfind('\n').unwrap_or(prefix.len());
        let mut t = prefix[..cut].to_string();
        t.push_str("\n\n[System prompt truncated for context size; remaining content omitted.]");
        Cow::Owned(t)
    }
}

async fn system_prompt_with_dictionary(_pool: &Pool, locale: Option<&str>) -> String {
    let mut prompt = assistant_embedded_system_prompt_cow().into_owned();
    if prompt.trim().is_empty() {
        return String::new();
    }
    if let Some(loc) = locale {
        if !loc.is_empty() {
            prompt.push_str(&format!(
                "\n\nPrefer to explain things in locale `{}` where appropriate.",
                loc
            ));
        }
    }
    prompt
}

fn map_chat_messages(messages: &[ChatMessage]) -> Vec<ChatCompletionMessageRequest> {
    messages
        .iter()
        .map(|m| {
            let role = match m.role.as_str() {
                "user" | "assistant" | "system" | "tool" => m.role.clone(),
                other => {
                    log::warn!("Unknown chat role `{}`, mapping to `user`", other);
                    "user".to_string()
                }
            };
            let tool_calls = m.tool_calls.as_ref().map(|tc| {
                tc.iter()
                    .map(tool_call_dto_to_internal)
                    .collect::<Vec<ToolCall>>()
            });
            ChatCompletionMessageRequest {
                role,
                content: m.content.clone(),
                tool_call_id: m.tool_call_id.clone(),
                name: m.name.clone(),
                tool_calls,
            }
        })
        .collect()
}

fn jbovlaste_tool_schema() -> Tool {
    Tool {
        r#type: "function".to_string(),
        function: ToolFunction {
            name: "jbovlaste_semantic_search".to_string(),
            description: "Semantic search over jbovlaste definition text (embeddings). \
                          Your ONLY tool—call before stating facts about \
                          Lojban words or meanings, unless the system message **core reference \
                          dictionary** already answers. \
                          **Always pass every lookup in one call** via the **`queries` array** \
                          (even a single word is `[\"lorxu\"]`). \
                          Call when: new valsi/topic, or prior results were insufficient—\
                          then use **different** strings (no duplicate searches). \
                          Do NOT call when this thread already has search-backed answers. \
                          **Query language rules (critical):** \
                          1) Search in the **user's language** (e.g. English keywords like \
                          `fox`, `big thanks`, `logical connective`, `past tense`). \
                          2) Include a bare Lojban valsi ONLY if the user explicitly typed it. \
                          3) NEVER guess Lojban words from memory—your pretrained Lojban is \
                          untrusted and produces garbage results. \
                          4) NEVER combine Lojban words in one query, and NEVER mix \
                          Lojban words with English words (e.g. `\"mutce ki'e\"`, \
                          `\"zu'ai example\"`, `\"simxu usage\"` are all useless—each \
                          valsi must be a separate element). \
                          5) NEVER re-search a valsi whose definition already appeared \
                          in prior results—you already have it; the dictionary will \
                          not give you examples or usage guides on repeated queries. \
                          Bad queries: \"what is the Lojban word for fox\", \
                          \"definition of klama\", \"mutce ki'e\", \"zu'ai example\". \
                          Good queries: `[\"fox\"]`, `[\"big thanks\", \"thanks\"]`, \
                          `[\"klama\"]` (only if user typed klama). \
                          Strip meta-words (\"Lojban\", \"definition\", \"dictionary\", \
                          \"meaning\", \"word\", \"jbovlaste\"). \
                          **`languages`**: Always pass the user's language tag \
                          (e.g. `[\"en\"]` for English, `[\"ru\"]` for Russian). \
                          Omit only when language is unknown."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "queries": {
                        "type": "array",
                        "items": { "type": "string", "minLength": 1 },
                        "minItems": 1,
                        "maxItems": 24,
                        "description": "All jbovlaste lookups for this step. One string per \
                            distinct valsi or gloss-style search (parallel on the server). \
                            Single lookup: e.g. `[\"klama\"]`. Translation: e.g. \
                            `[\"fox\", \"run\", \"because\", \"i\"]`. Max 24 strings."
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 15,
                        "default": 15,
                        "description": "How many top matches **per query** (max 15). Use fewer \
                            for a known valsi or narrow term; use 15 for broad English concepts \
                            or when refining after weak results."
                    },
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional: restrict to definitions in these languages. \
                            Use **tags** from jbovlaste, not numeric IDs. Examples: \
                            `[\"en\"]` English glosses; `[\"ru\"]` Russian; `[\"es\"]` Spanish; \
                            `[\"jbo\"]` definitions written in Lojban. Combine e.g. `[\"en\",\"ru\"]` \
                            for bilingual glosses. Omit to include all indexed languages."
                    },
                    "source_language": {
                        "type": "string",
                        "description": "Optional: **language tag** of the head word (valsi) \
                            source language—usually omit (defaults to Lojban `jbo`). \
                            Set when filtering non-Lojban source languages (same `tag` as \
                            in jbovlaste `languages`). Example: `jbo` for standard Lojban valsi."
                    }
                },
                "required": ["queries"]
            }),
        },
    }
}

static LLM_CORNER_BRACKET_SEGMENTS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"【.*?】").expect("valid regex"));

fn strip_llm_corner_bracket_segments(s: &str) -> String {
    LLM_CORNER_BRACKET_SEGMENTS.replace_all(s, "").into_owned()
}

/// Some models (e.g. openrouter/free) emit tool calls as text instead of tool_calls.
/// Parse content for CALL>[...]</TOOLCALL> and return equivalent ToolCalls.
fn parse_tool_calls_from_content(content: &str) -> Option<Vec<ToolCall>> {
    const PREFIX: &str = "CALL>";
    const SUFFIX: &str = "</TOOLCALL>";
    let start = content.find(PREFIX)?;
    let rest = &content[start + PREFIX.len()..];
    let end = rest.find(SUFFIX)?;
    let json_str = rest[..end].trim();
    #[derive(Deserialize)]
    struct FallbackCall {
        name: String,
        #[serde(default)]
        arguments: Option<serde_json::Value>,
    }
    let arr: Vec<FallbackCall> = serde_json::from_str(json_str).ok()?;
    if arr.is_empty() {
        return None;
    }
    Some(
        arr.into_iter()
            .enumerate()
            .map(|(i, c)| {
                let args_string = c
                    .arguments
                    .as_ref()
                    .and_then(|v| serde_json::to_string(v).ok())
                    .unwrap_or_else(|| "{}".to_string());
                ToolCall {
                    id: Some(format!("fallback-{}", i)),
                    r#type: Some("function".to_string()),
                    function: ToolCallFunction {
                        name: Some(c.name),
                        arguments: Some(args_string),
                    },
                }
            })
            .collect(),
    )
}

/// Maximum results per semantic search for the assistant tool.
const SEMANTIC_SEARCH_MAX_LIMIT: u32 = 15;

/// Max parallel jbovlaste lookups bundled in **one** `jbovlaste_semantic_search` call (`queries` array).
const SEMANTIC_SEARCH_MAX_QUERIES_PER_CALL: usize = 24;

#[derive(Debug, Clone)]
struct SemanticSearchCore {
    query: String,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
struct ToolArgs {
    /// Primary API: batch all lookups here (one tool round-trip).
    #[serde(default)]
    queries: Vec<String>,
    /// Legacy single-query shape; accepted only if `queries` is empty (older clients / history).
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
    /// jbovlaste `languages.tag` values (e.g. en, ru, jbo), not numeric langids.
    #[serde(default)]
    languages: Option<Vec<String>>,
    #[serde(default)]
    source_language: Option<String>,
}

impl ToolArgs {
    fn normalized_queries(&self) -> Result<Vec<String>, String> {
        let mut v: Vec<String> = self
            .queries
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        if v.is_empty() {
            if let Some(ref q) = self.query {
                let t = q.trim();
                if !t.is_empty() {
                    v.push(t.to_string());
                }
            }
        }
        if v.is_empty() {
            return Err(
                "`queries` must be a non-empty array of search strings (non-empty after trimming)."
                    .to_string(),
            );
        }
        if v.len() > SEMANTIC_SEARCH_MAX_QUERIES_PER_CALL {
            return Err(format!(
                "At most {} queries per jbovlaste_semantic_search call.",
                SEMANTIC_SEARCH_MAX_QUERIES_PER_CALL
            ));
        }
        Ok(v)
    }

}

#[derive(Debug, Clone)]
struct SearchBatch {
    queries: Vec<String>,
    limit: Option<u32>,
    languages: Option<Vec<String>>,
    source_language: Option<String>,
}

/// Extract a jbovlaste language tag from a locale string (e.g. "en-US" → "en").
fn language_tag_from_locale(locale: &str) -> Option<String> {
    let tag = locale
        .split(['-', '_'])
        .next()?;
    let tag = tag.trim().to_lowercase();
    if tag.len() >= 2 {
        Some(tag)
    } else {
        None
    }
}

impl SearchBatch {
    fn from_tool_args(args: &ToolArgs, queries: Vec<String>, default_locale: Option<&str>) -> Self {
        let languages = args.languages.clone().or_else(|| {
            default_locale
                .and_then(language_tag_from_locale)
                .map(|tag| {
                    log::info!(
                        "Assistant: LLM omitted `languages`; defaulting to [\"{}\"] from locale",
                        tag
                    );
                    vec![tag]
                })
        });
        Self {
            queries,
            limit: args.limit,
            languages,
            source_language: args.source_language.clone(),
        }
    }

    fn call_core(&self, query: &str) -> SemanticSearchCore {
        SemanticSearchCore {
            query: query.to_string(),
            limit: self.limit,
        }
    }
}

/// One assistant turn may include several tool calls; each jbovlaste search slot runs a **batch** of queries in parallel.
#[derive(Debug)]
enum PreparedToolSlot {
    Immediate {
        tool_call_id: Option<String>,
        name: Option<String>,
        content: String,
    },
    Search {
        tool_call_id: Option<String>,
        name: Option<String>,
        batch: SearchBatch,
        assistant_reasoning: Option<String>,
        global_step_index: usize,
        action_desc: String,
    },
}

async fn run_jbovlaste_semantic_search_core(
    pool: &Pool,
    core: &SemanticSearchCore,
    filters: &ResolvedSemanticFilters,
    embedding: &[f32],
    redis: Option<&RedisCache>,
) -> Result<DefinitionResponse, AppError> {
    let query = core.query.trim().to_string();
    if query.is_empty() {
        return Err(AppError::BadRequest(
            "jbovlaste_semantic_search: query is empty after trimming".into(),
        ));
    }

    let limit = core
        .limit
        .unwrap_or(SEMANTIC_SEARCH_MAX_LIMIT)
        .clamp(1, SEMANTIC_SEARCH_MAX_LIMIT) as i64;

    // `sort_by` / `sort_order` are **not** read by `jbovlaste::service::semantic_search`—SQL fixes order:
    // `exact_match_rank`, then embedding distance. Same field values as `GET /jbovlaste/semantic-search` for parity only.
    let params = SearchDefinitionsParams {
        page: 1,
        per_page: limit,
        search_term: query.clone(),
        include_comments: false,
        sort_by: "similarity".to_string(),
        sort_order: "asc".to_string(),
        languages: filters.languages_langids.clone(),
        selmaho: None,
        username: None,
        word_type: None,
        source_langid: filters.source_langid,
        search_in_phrases: None,
        include_total_count: false,
    };

    let run_db = || async {
        semantic_search(pool, params.clone(), embedding.to_vec(), None)
            .await
            .map_err(|e| {
                AppError::ExternalService(format!(
                    "Semantic search failed for query \"{}\": {}",
                    query, e
                ))
            })
    };

    if assistant_semantic_cache_disabled() || redis.is_none() {
        return run_db().await;
    }

    let redis = redis.expect("checked");
    let cache_key = generate_assistant_semantic_cache_key(
        &query,
        limit,
        filters.languages_langids.as_deref(),
        filters.source_langid,
    );

    match redis.get::<DefinitionResponse>(&cache_key).await {
        Ok(Some(cached)) => return Ok(cached),
        Ok(None) => {}
        Err(e) => {
            log::warn!("Assistant semantic cache read failed ({}); running search", e);
        }
    }

    let response = run_db().await?;

    if let Err(e) = redis
        .set(
            &cache_key,
            &response,
            Some(assistant_semantic_cache_ttl()),
        )
        .await
    {
        log::warn!("Assistant semantic cache write failed: {}", e);
    }

    Ok(response)
}

/// Runs semantic search with retries on transient failure (embedding or DB/network).
async fn run_jbovlaste_semantic_search_with_retry(
    pool: &Pool,
    core: &SemanticSearchCore,
    filters: &ResolvedSemanticFilters,
    embedding: Vec<f32>,
    redis: Option<&RedisCache>,
) -> Result<DefinitionResponse, AppError> {
    for attempt in 1..=TOOL_MAX_ATTEMPTS {
        match run_jbovlaste_semantic_search_core(pool, core, filters, &embedding, redis).await {
            Ok(r) => return Ok(r),
            Err(e @ AppError::BadRequest(_)) => return Err(e),
            Err(e) => {
                if attempt < TOOL_MAX_ATTEMPTS {
                    let delay = Duration::from_millis(
                        TOOL_INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1),
                    );
                    log::info!(
                        "Assistant semantic search retry {}/{} for query \"{}\" after {:?}",
                        attempt,
                        TOOL_MAX_ATTEMPTS,
                        core.query,
                        delay
                    );
                    sleep(delay).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}

fn summarise_definition(def: &DefinitionDetail) -> serde_json::Value {
    json!({
        "valsi": def.valsiword,
        "definition": &def.definition,
        "notes": def.notes.as_deref(),
        "lang": def.langrealname,
        "score": def.score,
        "selmaho": def.selmaho.as_deref(),
        "jargon": def.jargon.as_deref(),
    })
}

/// Plain-text tool results for the LLM (avoids echoing JSON fragments in the final reply).
fn semantic_tool_results_plain_text_for_llm(query: &str, definitions: &[DefinitionDetail]) -> String {
    let mut out = String::new();
    out.push_str(&format!("Semantic search for: \"{}\"\n\n", query));
    if definitions.is_empty() {
        out.push_str("(No definitions returned.)");
        return out;
    }
    for (i, def) in definitions.iter().enumerate() {
        out.push_str(&format!("--- {} ---\n", i + 1));
        out.push_str(&format!("valsi: {}\n", def.valsiword));
        out.push_str(&format!("language: {}\n", def.langrealname));
        out.push_str("definition:\n");
        out.push_str(&def.definition);
        out.push('\n');
        if let Some(notes) = def.notes.as_ref() {
            if !notes.trim().is_empty() {
                out.push_str("notes:\n");
                out.push_str(notes);
                out.push('\n');
            }
        }
        if let Some(s) = def.selmaho.as_ref() {
            if !s.trim().is_empty() {
                out.push_str(&format!("selmaho: {}\n", s));
            }
        }
        out.push('\n');
    }
    out
}

fn combine_batch_search_outcomes(
    queries: &[String],
    outcomes: Vec<Result<DefinitionResponse, AppError>>,
) -> (String, serde_json::Value, String) {
    let mut searches = Vec::new();
    let mut plain = String::new();
    let mut total_defs = 0usize;
    let mut err_count = 0usize;

    for (q, res) in queries.iter().zip(outcomes) {
        match res {
            Ok(def_response) => {
                total_defs += def_response.definitions.len();
                let compact: Vec<serde_json::Value> = def_response
                    .definitions
                    .iter()
                    .map(summarise_definition)
                    .collect();
                searches.push(json!({
                    "query": q,
                    "results": compact,
                }));
                plain.push_str(&semantic_tool_results_plain_text_for_llm(
                    q,
                    &def_response.definitions,
                ));
                plain.push('\n');
            }
            Err(e) => {
                err_count += 1;
                let err_str = format!("{}", e);
                searches.push(json!({
                    "query": q,
                    "error": err_str.clone(),
                    "results": [],
                }));
                plain.push_str(&format!(
                    "Semantic search error for \"{}\": {}\n\n",
                    q, err_str
                ));
            }
        }
    }

    let summary = if err_count == 0 {
        format!(
            "{} quer{}; {} definition(s) total.",
            queries.len(),
            if queries.len() == 1 { "y" } else { "ies" },
            total_defs
        )
    } else {
        format!(
            "{} quer{}; {} definition(s); {} sub-search error(s).",
            queries.len(),
            if queries.len() == 1 { "y" } else { "ies" },
            total_defs,
            err_count
        )
    };

    let payload = json!({ "searches": searches });
    (summary, payload, plain.trim_end().to_string())
}

/// Max retries for a single tool call (e.g. semantic search) on transient failure.
const TOOL_MAX_ATTEMPTS: u32 = 3;
const TOOL_INITIAL_BACKOFF_MS: u64 = 400;

/// Max length of raw_response included in SSE error events (to avoid huge payloads).
const ERROR_RAW_RESPONSE_MAX_LEN: usize = 8000;

/// Builds the JSON payload for an SSE error event, including debugging info (e.g. raw_response when present).
fn sse_error_payload(e: &AppError) -> serde_json::Value {
    let mut obj = json!({
        "type": "error",
        "error": format!("{}", e),
    });
    if let AppError::ExternalServiceWithRaw { raw_response, .. }
    | AppError::ExternalServiceRetryable { raw_response, .. } = e
    {
        obj["raw_response"] = serde_json::Value::String(truncate_error_raw_response(raw_response));
    }
    obj
}

fn truncate_error_raw_response(raw_response: &str) -> String {
    if raw_response.len() > ERROR_RAW_RESPONSE_MAX_LEN {
        format!(
            "{}... [truncated, total {} bytes]",
            &raw_response[..ERROR_RAW_RESPONSE_MAX_LEN],
            raw_response.len()
        )
    } else {
        raw_response.to_string()
    }
}

async fn emit_sse_user_visible(
    persist: &Option<Arc<ChatPersistState>>,
    tx: &mpsc::Sender<sse::Event>,
    payload: serde_json::Value,
) -> Result<(), AppError> {
    if let Some(p) = persist {
        if payload.get("type").and_then(|t| t.as_str()) != Some("stream_debug") {
            p.apply_and_save(&payload).await?;
        }
    }
    if let Ok(data) = serde_json::to_string(&payload) {
        let _ = tx.send(sse::Data::new(data).into()).await;
    }
    Ok(())
}

/// Debug-only SSE events (`type: "stream_debug"`). Clients should ignore these; useful for inspecting
/// model selection and per-attempt failures in the Network tab or custom tooling.
async fn sse_send_stream_debug(tx: &mpsc::Sender<sse::Event>, debug: serde_json::Value) {
    let payload = json!({
        "type": "stream_debug",
        "debug": debug
    });
    if let Ok(data) = serde_json::to_string(&payload) {
        let _ = tx.send(sse::Data::new(data).into()).await;
    }
}

fn app_error_debug_object(e: &AppError) -> serde_json::Value {
    let mut o = json!({
        "message": format!("{}", e),
    });
    if let AppError::ExternalServiceWithRaw { raw_response, .. }
    | AppError::ExternalServiceRetryable { raw_response, .. } = e
    {
        o["raw_response"] = serde_json::Value::String(truncate_error_raw_response(raw_response));
    }
    o
}

/// Pre-creates reply stubs for parallel dual-model runs so `streamFinished` can be tracked per branch
/// (and reload / recovery does not treat the turn as complete after only one model finishes).
async fn emit_parallel_branches(
    tx: &mpsc::Sender<sse::Event>,
    persist: &Option<Arc<ChatPersistState>>,
    pair: &[ModelIdName],
) -> Result<(), AppError> {
    if pair.len() != 2 {
        return Ok(());
    }
    let payload = json!({
        "type": "parallel_branches",
        "models": [
            { "id": pair[0].0, "name": pair[0].1 },
            { "id": pair[1].0, "name": pair[1].1 },
        ],
    });
    emit_sse_user_visible(persist, tx, payload).await
}

async fn sse_stream_debug_models_plan(
    tx: &mpsc::Sender<sse::Event>,
    candidates: &[ModelIdName],
    run_parallel: bool,
    parallel_pair: &[ModelIdName],
) {
    let list: Vec<serde_json::Value> = candidates
        .iter()
        .map(|(id, name)| json!({ "id": id, "name": name }))
        .collect();
    let mut debug = json!({
        "kind": "models_plan",
        "mode": if run_parallel { "parallel" } else { "sequential" },
        "candidates": list,
    });
    if run_parallel && parallel_pair.len() == 2 {
        debug["parallel_pair"] = json!([
            { "id": parallel_pair[0].0, "name": parallel_pair[0].1 },
            { "id": parallel_pair[1].0, "name": parallel_pair[1].1 },
        ]);
    } else {
        debug["note"] = serde_json::Value::String(
            "Sequential: try each candidate in order until one succeeds.".into(),
        );
    }
    sse_send_stream_debug(tx, debug).await;
}

async fn sse_stream_debug_model_attempt_failed(
    tx: &mpsc::Sender<sse::Event>,
    model_id: &str,
    model_name: &str,
    e: &AppError,
) {
    let debug = json!({
        "kind": "model_attempt_failed",
        "model": model_id,
        "model_name": model_name,
        "error": app_error_debug_object(e),
    });
    sse_send_stream_debug(tx, debug).await;
}

async fn sse_stream_debug_parallel_branch_finished(
    tx: &mpsc::Sender<sse::Event>,
    model_id: &str,
    model_name: &str,
    result: &Result<(String, Vec<AssistantStep>), AppError>,
) {
    let mut debug = json!({
        "kind": "parallel_branch_finished",
        "model": model_id,
        "model_name": model_name,
        "ok": result.is_ok(),
    });
    if let Err(e) = result {
        debug["error"] = app_error_debug_object(e);
    }
    sse_send_stream_debug(tx, debug).await;
}

/// Runs the agent loop. If event_tx is Some, streams step/done/error events (with optional "model" key for parallel runs).
/// When event_tx is Some and we have 2 models, runs both in parallel and streams both; **if either branch fails**,
/// retries that slot with the next unused candidates from the same list (see
/// [`crate::utils::openrouter_models::OPENROUTER_MODEL_CANDIDATES_MAX`]).
/// Otherwise runs a single model at a time and tries the next catalog candidate on failure.
///
/// Uses two probed models from Redis when available ([`crate::utils::openrouter_models`]); on failure of that set,
/// falls back to a live catalog fetch once (same mechanism as before).
pub async fn run_agent_loop(
    pool: &Pool,
    request: &ChatRequest,
    event_tx: Option<mpsc::Sender<sse::Event>>,
    redis: Option<&RedisCache>,
    persist: Option<Arc<ChatPersistState>>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
        AppError::ExternalService("OPENROUTER_API_KEY is not set in the environment".into())
    })?;
    let base_url = env::var("OPENROUTER_API_BASE")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let (mut candidates, mut from_redis_only) =
        load_or_fetch_openrouter_candidates(redis, &base_url, &api_key).await?;

    loop {
        match run_agent_loop_with_candidates(
            pool,
            request,
            event_tx.clone(),
            &candidates,
            persist.clone(),
            redis,
        )
        .await
        {
            Ok(ok) => return Ok(ok),
            Err(e) if from_redis_only => {
                log::warn!(
                    "OpenRouter: Redis-cached assistant models failed ({}); loading full catalog once",
                    e
                );
                candidates = fetch_latest_openrouter_models(&base_url, &api_key).await?;
                from_redis_only = false;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn run_agent_loop_with_candidates(
    pool: &Pool,
    request: &ChatRequest,
    event_tx: Option<mpsc::Sender<sse::Event>>,
    candidates: &[ModelIdName],
    persist: Option<Arc<ChatPersistState>>,
    redis: Option<&RedisCache>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    let parallel_models: Vec<ModelIdName> = candidates.iter().take(2).cloned().collect();
    let is_streaming = event_tx.is_some();
    let run_parallel =
        ASSISTANT_PARALLEL_DUAL_MODEL && is_streaming && parallel_models.len() == 2;

    if let Some(ref tx) = event_tx {
        sse_stream_debug_models_plan(tx, candidates, run_parallel, &parallel_models).await;
        if run_parallel {
            emit_parallel_branches(tx, &persist, &parallel_models).await?;
        }
    }

    if run_parallel {
        let et = event_tx.expect("run_parallel implies streaming sender");
        let post_debug_tx = et.clone();
        let tx1 = et.clone();
        let tx2 = et;
        let pool1 = pool.clone();
        let pool2 = pool.clone();
        let req1 = request.clone();
        let req2 = request.clone();
        let (m1_id, m1_name) = parallel_models[0].clone();
        let (m2_id, m2_name) = parallel_models[1].clone();
        let p1 = persist.clone();
        let p2 = persist.clone();
        let (mut r1, mut r2) = tokio::join!(
            run_agent_loop_inner_health_checked(
                &pool1, &req1, &m1_id, &m1_name, Some(tx1), p1, redis
            ),
            run_agent_loop_inner_health_checked(
                &pool2, &req2, &m2_id, &m2_name, Some(tx2), p2, redis
            ),
        );
        sse_stream_debug_parallel_branch_finished(&post_debug_tx, &m1_id, &m1_name, &r1).await;
        sse_stream_debug_parallel_branch_finished(&post_debug_tx, &m2_id, &m2_name, &r2).await;

        // Spare candidates (index 2..) are shared between both branches: each round tries one retry for
        // branch 1 if still failing, then one for branch 2, so one branch cannot exhaust the list alone
        // while the other never gets a try (unless only one branch still fails).
        let mut next_idx = 2;
        while next_idx < candidates.len() && (r1.is_err() || r2.is_err()) {
            if r1.is_err() {
                let (nid, nname) = candidates[next_idx].clone();
                next_idx += 1;
                let res = run_agent_loop_inner_health_checked(
                    &pool1,
                    &req1,
                    &nid,
                    &nname,
                    Some(post_debug_tx.clone()),
                    persist.clone(),
                    redis,
                )
                .await;
                if let Err(ref e) = res {
                    sse_stream_debug_model_attempt_failed(&post_debug_tx, &nid, &nname, e).await;
                    log::warn!(
                        "Assistant: parallel branch 1 model {} failed; trying next candidate if any: {}",
                        nid,
                        e
                    );
                }
                sse_stream_debug_parallel_branch_finished(&post_debug_tx, &nid, &nname, &res).await;
                r1 = res;
            }
            if r2.is_err() && next_idx < candidates.len() {
                let (nid, nname) = candidates[next_idx].clone();
                next_idx += 1;
                let res = run_agent_loop_inner_health_checked(
                    &pool2,
                    &req2,
                    &nid,
                    &nname,
                    Some(post_debug_tx.clone()),
                    persist.clone(),
                    redis,
                )
                .await;
                if let Err(ref e) = res {
                    sse_stream_debug_model_attempt_failed(&post_debug_tx, &nid, &nname, e).await;
                    log::warn!(
                        "Assistant: parallel branch 2 model {} failed; trying next candidate if any: {}",
                        nid,
                        e
                    );
                }
                sse_stream_debug_parallel_branch_finished(&post_debug_tx, &nid, &nname, &res).await;
                r2 = res;
            }
        }

        drop(post_debug_tx);
        // Return first successful reply for API compatibility.
        if let Ok((reply, _)) = r1 {
            Ok((reply, vec![]))
        } else if let Ok((reply, _)) = r2 {
            Ok((reply, vec![]))
        } else {
            Err(r2
                .err()
                .unwrap_or_else(|| r1.expect_err("both parallel branches failed")))
        }
    } else {
        let mut event_tx = event_tx;
        let mut last_err: Option<AppError> = None;

        for (model_id, model_name) in candidates {
            let tx = event_tx.clone();
            match run_agent_loop_inner_health_checked(
                pool,
                request,
                model_id,
                model_name,
                tx,
                persist.clone(),
                redis,
            )
            .await
            {
                Ok(result) => {
                    // Inner consumed a clone of the sender; drop any remaining handle so the SSE stream ends.
                    drop(event_tx.take());
                    return Ok(result);
                }
                Err(e) => {
                    if let Some(ref tx) = event_tx {
                        sse_stream_debug_model_attempt_failed(tx, model_id, model_name, &e).await;
                    }
                    log::warn!(
                        "Assistant: model {} failed; trying next candidate if any: {}",
                        model_id,
                        e
                    );
                    last_err = Some(e);
                }
            }
        }

        if let Some(e) = last_err.take() {
            if let Some(tx) = event_tx.take() {
                let payload = sse_error_payload(&e);
                if let Err(send_err) = emit_sse_user_visible(&persist, &tx, payload).await {
                    drop(tx);
                    return Err(send_err);
                }
                drop(tx);
            }
            return Err(e);
        }

        Err(AppError::ExternalService(
            "Assistant: no OpenRouter model candidates".into(),
        ))
    }
}

/// Test phrase used as the user message when probing a candidate model with the real assistant
/// chat path. The probe expects any non-empty trimmed reply within
/// [`ASSISTANT_MODEL_HEALTH_TIMEOUT`] for the model to be considered healthy.
pub const ASSISTANT_MODEL_PROBE_PHRASE: &str = "The big brown fox jumps over the lazy dog";

/// Hard ceiling for a single `run_agent_loop_inner` invocation in both probe and real-chat paths;
/// candidates exceeding this are treated as unhealthy and (in real chat) evicted from the Redis
/// assistant model cache.
pub const ASSISTANT_MODEL_HEALTH_TIMEOUT: Duration = Duration::from_secs(60);

/// Runs the real assistant chat path against `model` with a fixed test prompt and verifies the
/// final reply (including any tool calls) is returned within [`ASSISTANT_MODEL_HEALTH_TIMEOUT`]
/// and is non-empty after trimming. Used by the background OpenRouter model cache refresh.
pub async fn probe_openrouter_model_full_chat(
    pool: &Pool,
    redis: Option<&RedisCache>,
    model_id: &str,
    model_name: &str,
) -> Result<(), AppError> {
    let request = ChatRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: ASSISTANT_MODEL_PROBE_PHRASE.to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }],
        locale: None,
    };
    let fut = run_agent_loop_inner(pool, &request, model_id, model_name, None, None, redis);
    let outcome = tokio::time::timeout(ASSISTANT_MODEL_HEALTH_TIMEOUT, fut).await;
    match outcome {
        Err(_) => Err(AppError::ExternalService(format!(
            "OpenRouter probe: model {} did not return a final reply within {}s",
            model_id,
            ASSISTANT_MODEL_HEALTH_TIMEOUT.as_secs()
        ))),
        Ok(Err(e)) => Err(e),
        Ok(Ok((reply, _steps))) => {
            if reply.trim().is_empty() {
                Err(AppError::ExternalService(format!(
                    "OpenRouter probe: model {} returned empty reply (trimmed)",
                    model_id
                )))
            } else {
                Ok(())
            }
        }
    }
}

/// Wraps [`run_agent_loop_inner`] for real-chat use: enforces the same timeout / empty-reply
/// health check used by the probe, and on failure removes the offending model id from the
/// Redis assistant model cache so the next request picks a different candidate.
async fn run_agent_loop_inner_health_checked(
    pool: &Pool,
    request: &ChatRequest,
    model: &str,
    model_name: &str,
    event_tx: Option<mpsc::Sender<sse::Event>>,
    persist: Option<Arc<ChatPersistState>>,
    redis: Option<&RedisCache>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    let fut = run_agent_loop_inner(pool, request, model, model_name, event_tx, persist, redis);
    let outcome = tokio::time::timeout(ASSISTANT_MODEL_HEALTH_TIMEOUT, fut).await;
    let result = match outcome {
        Err(_) => Err(AppError::ExternalService(format!(
            "Assistant: model {} did not return a final reply within {}s",
            model,
            ASSISTANT_MODEL_HEALTH_TIMEOUT.as_secs()
        ))),
        Ok(Err(e)) => Err(e),
        Ok(Ok((reply, steps))) => {
            if reply.trim().is_empty() {
                Err(AppError::ExternalService(format!(
                    "Assistant: model {} returned empty reply (trimmed)",
                    model
                )))
            } else {
                Ok((reply, steps))
            }
        }
    };
    if result.is_err() {
        if let Some(r) = redis {
            if let Err(e) = evict_openrouter_assistant_model_from_cache(r, model).await {
                log::warn!(
                    "Assistant: failed to evict failing model `{}` from Redis cache: {}",
                    model,
                    e
                );
            }
        }
    }
    result
}

pub(crate) async fn run_agent_loop_inner(
    pool: &Pool,
    request: &ChatRequest,
    model: &str,
    model_name: &str,
    event_tx: Option<mpsc::Sender<sse::Event>>,
    persist: Option<Arc<ChatPersistState>>,
    redis: Option<&RedisCache>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
        AppError::ExternalService("OPENROUTER_API_KEY is not set in the environment".into())
    })?;

    let base_url = env::var("OPENROUTER_API_BASE")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let client = reqwest::Client::new();

    let system_content = system_prompt_with_dictionary(pool, request.locale.as_deref()).await;
    let context_budget =
        context_compress::ContextBudget::from_env_and_system_prompt(system_content.len());
    let mut client_round = context_compress::compress_chat_history_for_request(
        &request.messages,
        &context_budget,
    );
    let mut messages: Vec<ChatCompletionMessageRequest> = Vec::new();
    messages.push(ChatCompletionMessageRequest {
        role: "system".to_string(),
        content: system_content.clone(),
        tool_call_id: None,
        name: None,
        tool_calls: None,
    });
    messages.extend(map_chat_messages(&client_round));

    let tools = vec![jbovlaste_tool_schema()];
    let mut steps = Vec::new();
    let mut aggressive_context_retry = false;
    // Per-query repetition counter: if the model calls the same search query too many
    // times in a row, inject a tool-level reminder instead of running it again.
    // This mirrors Roo-Code's ToolRepetitionDetector pattern.
    let mut query_seen_count: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    const MAX_QUERY_REPETITIONS: u32 = 2;

    let agent_max_iter = agent_max_iterations();

    // Agent loop: call LLM until it returns a final reply (no tool_calls).
    for iteration in 1..=agent_max_iter {
        let label = format!("chat/completions iteration {}", iteration);
        let response = loop {
            let request_body = ChatCompletionRequest {
                model: model.to_string(),
                messages: messages.clone(),
                tools: Some(tools.clone()),
                tool_choice: Some(json!("auto")),
                parallel_tool_calls: Some(true),
            };
            match openrouter_chat_with_retry(&label, {
                let client = client.clone();
                let base_url = base_url.clone();
                let api_key = api_key.clone();
                let request_body = request_body.clone();
                let label = label.clone();
                move || {
                    let client = client.clone();
                    let base_url = base_url.clone();
                    let api_key = api_key.clone();
                    let request_body = request_body.clone();
                    let label = label.clone();
                    async move {
                        let res = client
                            .post(format!("{}/chat/completions", base_url))
                            .header("Authorization", format!("Bearer {}", api_key))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await?;
                        let ok = ensure_openrouter_status(res, &label).await?;
                        parse_chat_response(ok, &label).await
                    }
                }
            })
            .await
            {
                Ok(r) => break r,
                Err(e) => {
                    if iteration == 1
                        && !aggressive_context_retry
                        && error_indicates_context_limit(&e)
                    {
                        aggressive_context_retry = true;
                        log::warn!(
                            "Assistant: context limit error on first iteration; retrying with aggressive history compression"
                        );
                        client_round = context_compress::compress_chat_history_aggressive(
                            &request.messages,
                            &context_budget,
                        );
                        messages = vec![ChatCompletionMessageRequest {
                            role: "system".to_string(),
                            content: system_content.clone(),
                            tool_call_id: None,
                            name: None,
                            tool_calls: None,
                        }];
                        messages.extend(map_chat_messages(&client_round));
                        continue;
                    }
                    return Err(e);
                }
            }
        };

        let choice = response.choices.into_iter().next().ok_or_else(|| {
            AppError::ExternalService(format!(
                "No choices returned from OpenRouter (iteration {})",
                iteration
            ))
        })?;

        let msg = &choice.message;
        let content_str = msg.content.as_deref().unwrap_or("");
        // Some models emit tool calls in content as CALL>[...]</TOOLCALL> instead of tool_calls.
        let tool_calls = msg
            .tool_calls
            .clone()
            .filter(|c| !c.is_empty())
            .or_else(|| parse_tool_calls_from_content(content_str));

        // Append assistant message (with optional tool_calls) to history.
        messages.push(ChatCompletionMessageRequest {
            role: msg.role.clone().unwrap_or_else(|| "assistant".to_string()),
            content: content_str.to_string(),
            tool_call_id: None,
            name: None,
            tool_calls: tool_calls.clone(),
        });

        if let Some(calls) = tool_calls {
            if let Some(ref tx) = event_tx {
                let payload = json!({
                    "type": "assistant_tool_calls",
                    "model": model,
                    "model_name": model_name,
                    "content": content_str,
                    "tool_calls": calls,
                });
                emit_sse_user_visible(&persist, tx, payload).await?;
            }

            if calls.len() > 1 {
                log::warn!(
                    "Assistant: model emitted {} tool calls in one turn; expected one batched jbovlaste_semantic_search",
                    calls.len()
                );
                let bail = "Use exactly one jbovlaste_semantic_search call per assistant turn. \
                     Put every lookup in a single `queries` array (e.g. [\"klama\",\"fox\"]), \
                     not multiple tool calls.";
                for call in calls.iter() {
                    messages.push(ChatCompletionMessageRequest {
                        role: "tool".to_string(),
                        content: bail.to_string(),
                        tool_call_id: call.id.clone(),
                        name: call
                            .function
                            .name
                            .clone()
                            .or_else(|| Some("jbovlaste_semantic_search".to_string())),
                        tool_calls: None,
                    });
                }
                continue;
            }

            // Prepare tool slots (validation, repetition guard), then run semantic searches in
            // parallel while preserving tool_result order to match assistant tool_calls (OpenAI protocol).
            let base_step_index = steps.len();
            let mut prepared: Vec<PreparedToolSlot> = Vec::with_capacity(calls.len());
            let mut pending_search_ordinal = 0usize;
            let mut is_first_semantic_in_batch = true;

            for call in calls.iter() {
                if call.function.name.as_deref() != Some("jbovlaste_semantic_search") {
                    log::error!(
                        "Assistant: unexpected tool call '{}' — not in schema",
                        call.function.name.as_deref().unwrap_or("unknown")
                    );
                    prepared.push(PreparedToolSlot::Immediate {
                        tool_call_id: call.id.clone(),
                        name: call.function.name.clone(),
                        content: "Unknown tool. Use jbovlaste_semantic_search.".to_string(),
                    });
                    continue;
                }

                let assistant_reasoning = if is_first_semantic_in_batch {
                    is_first_semantic_in_batch = false;
                    let t = content_str.trim();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t.to_string())
                    }
                } else {
                    None
                };

                let global_step_index = base_step_index + pending_search_ordinal;

                let args_json: &str = match call.function.arguments.as_deref() {
                    None | Some("") => "{}",
                    Some(s) => s,
                };

                let args: ToolArgs = match serde_json::from_str(args_json) {
                    Ok(a) => a,
                    Err(e) => {
                        log::warn!(
                            "Tool call arguments JSON parse error: {}; raw arguments: {:?}",
                            e,
                            call.function.arguments
                        );
                        let err_msg = format!(
                            "Invalid tool arguments (invalid JSON). Please call jbovlaste_semantic_search again with valid JSON. Error: {}",
                            e
                        );
                        prepared.push(PreparedToolSlot::Immediate {
                            tool_call_id: call.id.clone(),
                            name: call.function.name.clone(),
                            content: format!("Tool error: {}", err_msg),
                        });
                        continue;
                    }
                };

                let queries = match args.normalized_queries() {
                    Ok(q) => q,
                    Err(msg) => {
                        prepared.push(PreparedToolSlot::Immediate {
                            tool_call_id: call.id.clone(),
                            name: call.function.name.clone(),
                            content: format!("Tool error: jbovlaste_semantic_search: {}", msg),
                        });
                        continue;
                    }
                };

                let mut repetition_block: Option<(String, u32)> = None;
                for q in &queries {
                    let prior = query_seen_count.get(q).copied().unwrap_or(0);
                    if prior >= MAX_QUERY_REPETITIONS {
                        repetition_block = Some((q.clone(), prior));
                        break;
                    }
                }
                if let Some((search_query_for_llm, prior)) = repetition_block {
                    log::warn!(
                        "Assistant: query '{}' already used {} time(s); injecting loop-break tool result",
                        search_query_for_llm,
                        prior
                    );
                    prepared.push(PreparedToolSlot::Immediate {
                        tool_call_id: call.id.clone(),
                        name: call.function.name.clone(),
                        content: format!(
                            "You have already searched for \"{}\" {} time(s). \
                             The results are already in this conversation. \
                             Stop searching and formulate your answer now.",
                            search_query_for_llm,
                            prior
                        ),
                    });
                    continue;
                }
                for q in &queries {
                    *query_seen_count.entry(q.clone()).or_insert(0) += 1;
                }

                pending_search_ordinal += 1;
                let batch = SearchBatch::from_tool_args(&args, queries, request.locale.as_deref());
                let action_desc = if batch.queries.len() == 1 {
                    format!("Semantic search: \"{}\"", batch.queries[0])
                } else {
                    let preview = batch.queries[..batch.queries.len().min(4)].join(", ");
                    let suffix = if batch.queries.len() > 4 { ", …" } else { "" };
                    format!(
                        "Semantic search ({} queries): {}{}",
                        batch.queries.len(),
                        preview,
                        suffix
                    )
                };
                prepared.push(PreparedToolSlot::Search {
                    tool_call_id: call.id.clone(),
                    name: call.function.name.clone(),
                    batch,
                    assistant_reasoning,
                    global_step_index,
                    action_desc,
                });
            }

            let mut batch_outcomes_by_slot: HashMap<
                usize,
                Vec<Result<DefinitionResponse, AppError>>,
            > = HashMap::new();

            if prepared
                .iter()
                .any(|s| matches!(s, PreparedToolSlot::Search { .. }))
            {
                if let Some(ref tx) = event_tx {
                    for slot in &prepared {
                        if let PreparedToolSlot::Search {
                            global_step_index,
                            action_desc,
                            tool_call_id,
                            assistant_reasoning,
                            ..
                        } = slot
                        {
                            let mut start_payload = json!({
                                "type": "step_start",
                                "model": model,
                                "model_name": model_name,
                                "index": global_step_index,
                                "action": action_desc,
                                "tool_call_id": tool_call_id,
                            });
                            if let Some(ref ar) = assistant_reasoning {
                                start_payload["assistant_reasoning"] =
                                    serde_json::Value::String(ar.clone());
                            }
                            emit_sse_user_visible(&persist, tx, start_payload).await?;
                        }
                    }
                }

                let pool_clone = pool.clone();
                let sem = ASSISTANT_SEMANTIC_SUBQUERY_SEMAPHORE.clone();
                for (slot_i, slot) in prepared.iter().enumerate() {
                    if let PreparedToolSlot::Search { batch, .. } = slot {
                        let filters = resolve_semantic_search_language_filters(
                            pool,
                            batch.languages.as_deref(),
                            batch.source_language.as_ref(),
                        )
                        .await?;

                        let trimmed: Vec<String> =
                            batch.queries.iter().map(|q| q.trim().to_string()).collect();
                        for q in &trimmed {
                            if q.is_empty() {
                                return Err(AppError::BadRequest(
                                    "jbovlaste_semantic_search: query is empty after trimming"
                                        .into(),
                                ));
                            }
                        }

                        let embeddings = get_batch_embeddings(trimmed.clone()).await?;

                        let outcomes = join_all(
                            trimmed.iter().zip(embeddings).map(|(q, emb)| {
                                let pool = pool_clone.clone();
                                let filters = filters.clone();
                                let sem = sem.clone();
                                let core = batch.call_core(q);
                                async move {
                                    let _permit = sem
                                        .acquire()
                                        .await
                                        .expect("assistant semantic subquery semaphore");
                                    run_jbovlaste_semantic_search_with_retry(
                                        &pool, &core, &filters, emb, redis,
                                    )
                                    .await
                                }
                            }),
                        )
                        .await;
                        batch_outcomes_by_slot.insert(slot_i, outcomes);
                    }
                }
            }

            for (slot_i, slot) in prepared.into_iter().enumerate() {
                match slot {
                    PreparedToolSlot::Immediate {
                        tool_call_id,
                        name,
                        content,
                    } => {
                        messages.push(ChatCompletionMessageRequest {
                            role: "tool".to_string(),
                            content,
                            tool_call_id,
                            name,
                            tool_calls: None,
                        });
                    }
                    PreparedToolSlot::Search {
                        tool_call_id,
                        name,
                        batch,
                        assistant_reasoning,
                        global_step_index,
                        action_desc,
                        ..
                    } => {
                        let outcomes = batch_outcomes_by_slot
                            .remove(&slot_i)
                            .expect("search slot must have batch outcomes");

                        for res in &outcomes {
                            if let Err(e) = res {
                                log::warn!(
                                    "Assistant semantic search sub-query failed after retries: {}",
                                    e
                                );
                            }
                        }

                        let (result_summary, tool_payload_value, tool_content_for_llm) =
                            combine_batch_search_outcomes(&batch.queries, outcomes);

                        let tool_content_json =
                            serde_json::to_string(&tool_payload_value).unwrap_or_else(|_| {
                                "{}".to_string()
                            });

                        let step = AssistantStep {
                            action: action_desc.clone(),
                            result: result_summary.clone(),
                            tool_output: Some(tool_content_json.clone()),
                            assistant_reasoning: assistant_reasoning.clone(),
                        };
                        steps.push(step.clone());

                        if let Some(ref tx) = event_tx {
                            let mut payload = json!({
                                "type": "step",
                                "model": model,
                                "model_name": model_name,
                                "index": global_step_index,
                                "action": step.action,
                                "result": step.result,
                                "tool_call_id": tool_call_id,
                                "tool_content_plain": tool_content_for_llm,
                            });
                            if let Some(ref ar) = assistant_reasoning {
                                payload["assistant_reasoning"] =
                                    serde_json::Value::String(ar.clone());
                            }
                            if let Some(ref out) = step.tool_output {
                                payload["tool_output"] = serde_json::Value::String(out.clone());
                            }
                            emit_sse_user_visible(&persist, tx, payload).await?;
                        }

                        messages.push(ChatCompletionMessageRequest {
                            role: "tool".to_string(),
                            content: tool_content_for_llm,
                            tool_call_id,
                            name,
                            tool_calls: None,
                        });
                    }
                }
            }
            // Loop again so the model can see tool results and either call more tools or reply.
        } else {
            // No tool calls: this is the final assistant reply.
            let reply = strip_llm_corner_bracket_segments(
                &choice
                    .message
                    .content
                    .clone()
                    .unwrap_or_else(String::new),
            );

            // Stuck-recovery: if the model returned empty content with no tools while search
            // results exist in this conversation, inject a brief nudge message and continue the
            // loop. This mirrors Roo's noToolsUsed / recovery pattern for stalled agents.
            if reply.trim().is_empty() && iteration < agent_max_iter && !steps.is_empty() {
                log::warn!(
                    "Assistant: empty reply with no tool calls at iteration {}; injecting recovery nudge",
                    iteration
                );
                messages.push(ChatCompletionMessageRequest {
                    role: "user".to_string(),
                    content: "You have search results above. Please use them to answer my question now.".to_string(),
                    tool_call_id: None,
                    name: None,
                    tool_calls: None,
                });
                continue;
            }

            if let Some(tx) = event_tx {
                let payload = json!({
                    "type": "done",
                    "model": model,
                    "model_name": model_name,
                    "reply": reply
                });
                emit_sse_user_visible(&persist, &tx, payload).await?;
                drop(tx);
            }
            return Ok((reply, steps));
        }
    }

    // Max iterations reached without a final reply; return last assistant content if any.
    let last_content = messages
        .iter()
        .rev()
        .find(|m| m.role == "assistant")
        .map(|m| m.content.clone())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| "I need more time to look that up. Please try again.".to_string());
    let last_content = strip_llm_corner_bracket_segments(&last_content);

    if let Some(tx) = event_tx {
        let payload = json!({
            "type": "done",
            "model": model,
            "model_name": model_name,
            "reply": &last_content
        });
        emit_sse_user_visible(&persist, &tx, payload).await?;
        drop(tx);
    }
    Ok((last_content, steps))
}

#[cfg(test)]
mod chat_message_map_tests {
    use super::map_chat_messages;
    use crate::assistant::dto::{ChatMessage, ToolCallDto, ToolCallFunctionDto};

    #[test]
    fn map_passes_tool_role_and_ids() {
        let messages = vec![
            ChatMessage {
                role: "assistant".into(),
                content: "".into(),
                tool_calls: Some(vec![ToolCallDto {
                    id: Some("c1".into()),
                    r#type: Some("function".into()),
                    function: ToolCallFunctionDto {
                        name: Some("jbovlaste_semantic_search".into()),
                        arguments: Some(r#"{"queries":["test"]}"#.into()),
                    },
                }]),
                tool_call_id: None,
                name: None,
            },
            ChatMessage {
                role: "tool".into(),
                content: "tool body".into(),
                tool_calls: None,
                tool_call_id: Some("c1".into()),
                name: Some("jbovlaste_semantic_search".into()),
            },
        ];
        let out = map_chat_messages(&messages);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].role, "assistant");
        assert!(out[0].tool_calls.is_some());
        assert_eq!(out[1].role, "tool");
        assert_eq!(out[1].tool_call_id.as_deref(), Some("c1"));
        assert_eq!(out[1].content, "tool body");
    }

    #[test]
    fn error_indicates_context_detects_keywords() {
        use crate::error::AppError;
        assert!(super::error_indicates_context_limit(&AppError::ExternalServiceWithRaw {
            message: "x".into(),
            raw_response: "prompt is too long".into(),
        }));
        assert!(!super::error_indicates_context_limit(&AppError::BadRequest("nope".into())));
    }
}
