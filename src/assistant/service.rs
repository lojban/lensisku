#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_web_lab::sse;
use deadpool_postgres::Pool;
use futures::future::join_all;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::sleep;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::AppError;
use crate::jbovlaste::models::{DefinitionDetail, DefinitionResponse, Example, SearchDefinitionsParams};
use crate::jbovlaste::service::{get_definition, semantic_search};
use crate::middleware::cache::{generate_assistant_semantic_cache_key, RedisCache};
use crate::utils::embeddings::get_batch_embeddings;
use crate::utils::openrouter_models::{
    evict_openrouter_assistant_model_from_cache, fetch_latest_openrouter_models,
    load_or_fetch_openrouter_candidates, ModelIdName,
};
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
        .unwrap_or(25)
        .clamp(1, 50)
}

fn assistant_iteration_timeout() -> Duration {
    Duration::from_secs(
        env::var("ASSISTANT_ITERATION_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60)
            .clamp(10, 300),
    )
}

/// Mutable conversation and loop state for a single assistant turn.
///
/// Grouping fields here makes failover, context compression retries, and
/// tests easier to reason about.
struct AgentState {
    messages: Vec<ChatCompletionMessageRequest>,
    steps: Vec<AssistantStep>,
    query_seen_count: HashMap<String, u32>,
    current_model_idx: usize,
    model: String,
    model_name: String,
    aggressive_context_retry: bool,
    client_round: Vec<ChatMessage>,
    context_budget: context_compress::ContextBudget,
    tools: Vec<Tool>,
    system_content: String,
    candidates: Vec<ModelIdName>,
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    iteration_timeout: Duration,
    max_iterations: u32,
}

impl AgentState {
    async fn new(
        pool: &Pool,
        request: &ChatRequest,
        candidates: &[ModelIdName],
    ) -> Result<Self, AppError> {
        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
            AppError::ExternalService("OPENROUTER_API_KEY is not set in the environment".into())
        })?;
        let base_url = env::var("OPENROUTER_API_BASE")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
        let client = reqwest::Client::new();
        let system_content = system_prompt_with_dictionary(pool, request.locale.as_deref()).await;
        let context_budget =
            context_compress::ContextBudget::from_env_and_system_prompt(system_content.len());
        let client_round =
            context_compress::compress_chat_history_for_request(&request.messages, &context_budget);
        let mut messages = Vec::new();
        messages.push(ChatCompletionMessageRequest {
            role: "system".to_string(),
            content: system_content.clone(),
            tool_call_id: None,
            name: None,
            tool_calls: None,
        });
        messages.extend(map_chat_messages(&client_round));

        if candidates.is_empty() {
            return Err(AppError::ExternalService(
                "Assistant: no OpenRouter model candidates provided".into(),
            ));
        }

        Ok(Self {
            messages,
            steps: Vec::new(),
            query_seen_count: HashMap::new(),
            current_model_idx: 0,
            model: candidates[0].0.clone(),
            model_name: candidates[0].1.clone(),
            aggressive_context_retry: false,
            client_round,
            context_budget,
            tools: vec![jbovlaste_tool_schema(), jbovlaste_resolve_results_tool_schema()],
            system_content,
            candidates: candidates.to_vec(),
            client,
            api_key,
            base_url,
            iteration_timeout: assistant_iteration_timeout(),
            max_iterations: agent_max_iterations(),
        })
    }
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
        ids.push(*resolved_map.get(t).expect("all norm tags resolved"));
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
    let resolved =
        resolve_jbovlaste_language_tags_to_langids(pool, std::slice::from_ref(&key)).await?;
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
    let source_langid =
        resolve_optional_source_language_tag(pool, source_language.map(|s| s.as_str())).await?;
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

fn jbovlaste_resolve_results_tool_schema() -> Tool {
    Tool {
        r#type: "function".to_string(),
        function: ToolFunction {
            name: "jbovlaste_resolve_results".to_string(),
            description: "Submit the final set of jbovlaste references that answer the user's question. \
                          This is the **only** way to finish a turn. The backend will validate every \
                          reference (definitionid, field, and exact_text) against the real database and, \
                          if all references are valid, build the printable answer. If any reference is \
                          wrong, the backend will return a list of errors—fix them and call this tool again. \
                          Do **not** output Markdown or prose; use this tool. \
                          For fields: `definition`, `notes`, `etymology`, `rafsi`, `example` (requires exampleid), \
                          `decomposition`. Copy `exact_text` exactly from the tool output, preserving `$...$` delimiters."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "references": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "definitionid": {
                                    "type": "integer",
                                    "description": "jbovlaste definitionid from the search results."
                                },
                                "field": {
                                    "type": "string",
                                    "enum": ["definition", "notes", "etymology", "rafsi", "example", "decomposition"],
                                    "description": "Which part of the definition entry the exact_text comes from."
                                },
                                "exampleid": {
                                    "type": "integer",
                                    "description": "Required when field is `example`; the exampleid from the search results."
                                },
                                "exact_text": {
                                    "type": "string",
                                    "minLength": 1,
                                    "description": "The exact substring to quote, copied verbatim from the search result. Preserve $...$ math delimiters."
                                }
                            },
                            "required": ["definitionid", "field", "exact_text"]
                        },
                        "minItems": 1,
                        "description": "Ordered list of references that form the answer."
                    },
                    "message": {
                        "type": "string",
                        "minLength": 1,
                        "description": "Use instead of `references` when no Lojban evidence is found. Briefly explain why no answer is possible."
                    }
                }
            }),
        },
    }
}

#[derive(Debug, Clone)]
struct ValidatedReference {
    definitionid: i32,
    valsiword: String,
    type_name: String,
    langrealname: String,
    selmaho: Option<String>,
    rafsi: Option<String>,
    field: String,
    exampleid: Option<i32>,
    exact_text: String,
}

#[derive(Debug, Clone)]
struct ReferenceError {
    index: usize,
    definitionid: i32,
    field: String,
    exact_text: String,
    reason: String,
}

enum ResolveOutcome {
    Valid(Vec<ValidatedReference>),
    Invalid(Vec<ReferenceError>),
}

/// Validates that every reference points to a real definition (and, for examples, a real example)
/// and that `exact_text` actually occurs in the requested field.
async fn resolve_references(
    pool: &Pool,
    references: &[ResolveReference],
) -> Result<ResolveOutcome, AppError> {
    let mut validated = Vec::with_capacity(references.len());
    let mut errors = Vec::new();

    for (index, r) in references.iter().enumerate() {
        let field = r.field.trim().to_lowercase();
        let needle = r.exact_text.as_deref().unwrap_or("").trim();
        if needle.is_empty() {
            errors.push(ReferenceError {
                index,
                definitionid: r.definitionid,
                field: r.field.clone(),
                exact_text: r.exact_text.clone().unwrap_or_default(),
                reason: "exact_text is empty after trimming".to_string(),
            });
            continue;
        }

        let def = match get_definition(pool, r.definitionid, None).await {
            Ok(Some(d)) => d,
            Ok(None) => {
                errors.push(ReferenceError {
                    index,
                    definitionid: r.definitionid,
                    field: r.field.clone(),
                    exact_text: r.exact_text.clone().unwrap_or_default(),
                    reason: format!("definitionid {} not found", r.definitionid),
                });
                continue;
            }
            Err(e) => {
                return Err(AppError::ExternalService(format!(
                    "Failed to load definition {}: {}",
                    r.definitionid, e
                )));
            }
        };

        let valid = match field.as_str() {
            "definition" => def.definition.contains(needle),
            "notes" => match def.notes.as_ref() {
                Some(notes) => notes.contains(needle),
                None => false,
            },
            "etymology" => match def.etymology.as_ref() {
                Some(et) => et.contains(needle),
                None => false,
            },
            "rafsi" => match def.rafsi.as_ref() {
                Some(r) => r.contains(needle),
                None => false,
            },
            "decomposition" => match def.decomposition.as_ref() {
                Some(d) => d.iter().any(|s| s.contains(needle)),
                None => false,
            },
            "example" => match def.examples.as_ref() {
                Some(examples) => match examples.iter().find(|e| Some(e.exampleid) == r.exampleid) {
                    Some(ex) => ex.content.contains(needle),
                    None => {
                        errors.push(ReferenceError {
                            index,
                            definitionid: r.definitionid,
                            field: r.field.clone(),
                            exact_text: r.exact_text.clone().unwrap_or_default(),
                            reason: format!(
                                "exampleid {} not found for definitionid {}",
                                r.exampleid.unwrap_or(-1),
                                r.definitionid
                            ),
                        });
                        continue;
                    }
                },
                None => {
                    errors.push(ReferenceError {
                        index,
                        definitionid: r.definitionid,
                        field: r.field.clone(),
                        exact_text: r.exact_text.clone().unwrap_or_default(),
                        reason: format!(
                            "definitionid {} has no examples",
                            r.definitionid
                        ),
                    });
                    continue;
                }
            },
            _ => {
                errors.push(ReferenceError {
                    index,
                    definitionid: r.definitionid,
                    field: r.field.clone(),
                    exact_text: r.exact_text.clone().unwrap_or_default(),
                    reason: format!("unknown field `{}`", r.field),
                });
                continue;
            }
        };

        if !valid {
            errors.push(ReferenceError {
                index,
                definitionid: r.definitionid,
                field: r.field.clone(),
                exact_text: r.exact_text.clone().unwrap_or_default(),
                reason: format!(
                    "exact_text not found in {} for definitionid {}",
                    field, r.definitionid
                ),
            });
            continue;
        }

        validated.push(ValidatedReference {
            definitionid: r.definitionid,
            valsiword: def.valsiword,
            type_name: def.type_name,
            langrealname: def.langrealname,
            selmaho: def.selmaho,
            rafsi: def.rafsi,
            field,
            exampleid: r.exampleid,
            exact_text: needle.to_string(),
        });
    }

    if !errors.is_empty() {
        return Ok(ResolveOutcome::Invalid(errors));
    }
    Ok(ResolveOutcome::Valid(validated))
}

fn build_printable_markdown(validated: &[ValidatedReference]) -> String {
    if validated.is_empty() {
        return "No references to display.".to_string();
    }

    let mut groups: Vec<(i32, Vec<&ValidatedReference>)> = Vec::new();
    let mut group_index: HashMap<i32, usize> = HashMap::new();
    for r in validated {
        match group_index.get(&r.definitionid) {
            Some(&idx) => groups[idx].1.push(r),
            None => {
                group_index.insert(r.definitionid, groups.len());
                groups.push((r.definitionid, vec![r]));
            }
        }
    }

    let mut out = String::new();
    for (did, refs) in groups {
        let first = refs[0];
        out.push_str(&format!("## {} ({})\n\n", first.valsiword, first.type_name));
        out.push_str(&format!("- **definitionid:** {}\n", did));
        out.push_str(&format!("- **language:** {}\n", first.langrealname));
        if let Some(s) = first.selmaho.as_ref() {
            out.push_str(&format!("- **selmaho:** {}\n", s));
        }
        if let Some(r) = first.rafsi.as_ref() {
            out.push_str(&format!("- **rafsi:** {}\n", r));
        }
        out.push('\n');
        for r in refs {
            let label = match r.field.as_str() {
                "definition" => "Definition".to_string(),
                "notes" => "Notes".to_string(),
                "etymology" => "Etymology".to_string(),
                "rafsi" => "Rafsi".to_string(),
                "decomposition" => "Decomposition".to_string(),
                "example" => format!("Example (exampleid: {})", r.exampleid.unwrap_or(-1)),
                _ => r.field.clone(),
            };
            out.push_str(&format!("### {}\n\n", label));
            out.push_str(&r.exact_text);
            out.push_str("\n\n");
        }
        out.push_str("---\n\n");
    }
    out
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

#[derive(Debug, Deserialize, Clone)]
struct ResolveReference {
    definitionid: i32,
    field: String,
    #[serde(default)]
    exampleid: Option<i32>,
    #[serde(default)]
    exact_text: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct ResolveArgs {
    #[serde(default)]
    references: Vec<ResolveReference>,
    #[serde(default)]
    message: Option<String>,
}

impl ResolveArgs {
    fn normalized(&self) -> Result<(Vec<ResolveReference>, Option<String>), String> {
        if let Some(msg) = self.message.as_ref() {
            let t = msg.trim();
            if t.is_empty() {
                return Err("`message` must not be empty if provided.".to_string());
            }
            if !self.references.is_empty() {
                return Err("Provide either `references` or `message`, not both.".to_string());
            }
            return Ok((Vec::new(), Some(t.to_string())));
        }
        if self.references.is_empty() {
            return Err(
                "`references` must not be empty unless `message` is provided.".to_string(),
            );
        }
        for (i, r) in self.references.iter().enumerate() {
            let f = r.field.trim().to_lowercase();
            let valid = matches!(
                f.as_str(),
                "definition" | "notes" | "etymology" | "rafsi" | "example" | "decomposition"
            );
            if !valid {
                return Err(format!(
                    "references[{}].field `{}` is not one of: definition, notes, etymology, rafsi, example, decomposition",
                    i, r.field
                ));
            }
            if f == "example" && r.exampleid.is_none() {
                return Err(format!(
                    "references[{}].exampleid is required when field is `example`",
                    i
                ));
            }
            let exact = r.exact_text.as_deref().unwrap_or("").trim();
            if exact.is_empty() {
                return Err(format!(
                    "references[{}].exact_text is required and must not be empty",
                    i
                ));
            }
        }
        Ok((self.references.clone(), None))
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
    let tag = locale.split(['-', '_']).next()?;
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
    Resolve {
        tool_call_id: Option<String>,
        name: Option<String>,
        refs: Vec<ResolveReference>,
        message: Option<String>,
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
        let mut response = run_db().await?;
        attach_examples_to_definitions(pool, &mut response.definitions).await?;
        return Ok(response);
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
            log::warn!(
                "Assistant semantic cache read failed ({}); running search",
                e
            );
        }
    }

    let mut response = run_db().await?;
    attach_examples_to_definitions(pool, &mut response.definitions).await?;

    if let Err(e) = redis
        .set(&cache_key, &response, Some(assistant_semantic_cache_ttl()))
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
                    let delay =
                        Duration::from_millis(TOOL_INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1));
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
        "definitionid": def.definitionid,
        "valsiid": def.valsiid,
        "valsi": def.valsiword,
        "type": def.type_name,
        "lang": def.langrealname,
        "score": def.score,
        "selmaho": def.selmaho.as_deref(),
        "rafsi": def.rafsi.as_deref(),
        "jargon": def.jargon.as_deref(),
        "definition": &def.definition,
        "notes": def.notes.as_deref(),
        "etymology": def.etymology.as_deref(),
        "decomposition": def.decomposition.as_ref(),
    })
}

/// Plain-text tool results for the LLM (avoids echoing JSON fragments in the final reply).
fn semantic_tool_results_plain_text_for_llm(
    query: &str,
    definitions: &[DefinitionDetail],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("Semantic search for: \"{}\"\n\n", query));
    if definitions.is_empty() {
        out.push_str("(No definitions returned.)");
        return out;
    }
    for (i, def) in definitions.iter().enumerate() {
        out.push_str(&format!("--- {} ---\n", i + 1));
        out.push_str(&format!("definitionid: {}\n", def.definitionid));
        out.push_str(&format!("valsiid: {}\n", def.valsiid));
        out.push_str(&format!("valsi: {}\n", def.valsiword));
        out.push_str(&format!("type: {}\n", def.type_name));
        out.push_str(&format!("language: {}\n", def.langrealname));
        if let Some(j) = def.jargon.as_ref() {
            if !j.trim().is_empty() {
                out.push_str(&format!("jargon: {}\n", j));
            }
        }
        if let Some(s) = def.selmaho.as_ref() {
            if !s.trim().is_empty() {
                out.push_str(&format!("selmaho: {}\n", s));
            }
        }
        if let Some(r) = def.rafsi.as_ref() {
            if !r.trim().is_empty() {
                out.push_str(&format!("rafsi: {}\n", r));
            }
        }
        if let Some(d) = def.decomposition.as_ref() {
            if !d.is_empty() {
                out.push_str(&format!("decomposition: {}\n", d.join(", ")));
            }
        }
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
        if let Some(et) = def.etymology.as_ref() {
            if !et.trim().is_empty() {
                out.push_str("etymology:\n");
                out.push_str(et);
                out.push('\n');
            }
        }
        if let Some(examples) = def.examples.as_ref() {
            if !examples.is_empty() {
                out.push_str("examples:\n");
                for ex in examples {
                    out.push_str(&format!("exampleid: {}\n", ex.exampleid));
                    out.push_str(&ex.content);
                    out.push('\n');
                }
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

/// Fetches examples in bulk for definitions returned by semantic search and attaches them.
async fn attach_examples_to_definitions(
    pool: &Pool,
    definitions: &mut [DefinitionDetail],
) -> Result<(), AppError> {
    if definitions.is_empty() {
        return Ok(());
    }
    let def_ids: Vec<i32> = definitions.iter().map(|d| d.definitionid).collect();
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::ExternalService(e.to_string()))?;
    let rows = client
        .query(
            "SELECT e.definitionid, e.exampleid, e.content, e.examplenum, e.time, u.username
             FROM example e
             JOIN users u ON e.userid = u.userid
             WHERE e.definitionid = ANY($1)
             ORDER BY e.definitionid, e.examplenum",
            &[&def_ids],
        )
        .await
        .map_err(|e| AppError::ExternalService(format!("Failed to load examples: {}", e)))?;
    let mut map: HashMap<i32, Vec<Example>> = HashMap::new();
    for row in rows {
        let did: i32 = row.get("definitionid");
        let ex = Example {
            exampleid: row.get("exampleid"),
            content: row.get("content"),
            examplenum: row.get("examplenum"),
            time: row.get("time"),
            username: row.get("username"),
        };
        map.entry(did).or_default().push(ex);
    }
    for def in definitions {
        if let Some(examples) = map.remove(&def.definitionid) {
            if !examples.is_empty() {
                def.examples = Some(examples);
            }
        }
    }
    Ok(())
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

async fn emit_sse_error(
    tx: &Option<mpsc::Sender<sse::Event>>,
    persist: &Option<Arc<ChatPersistState>>,
    e: &AppError,
) {
    if let Some(ref sender) = tx {
        let payload = sse_error_payload(e);
        if let Err(send_err) = emit_sse_user_visible(persist, sender, payload).await {
            log::warn!("Assistant: failed to emit SSE error event: {}", send_err);
        }
    }
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

fn assistant_request_analysis_enabled() -> bool {
    env::var("ASSISTANT_REQUEST_ANALYSIS")
        .ok()
        .map(|v| !matches!(v.to_lowercase().as_str(), "0" | "false" | "no" | "off"))
        .unwrap_or(false)
}

#[derive(Debug, Deserialize)]
struct RequestAnalysis {
    intent: String,
    #[serde(default)]
    on_topic: bool,
    #[serde(default = "default_true")]
    needs_search: bool,
    #[serde(default)]
    search_queries: Vec<String>,
    ambiguity_note: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for RequestAnalysis {
    fn default() -> Self {
        Self {
            intent: String::new(),
            on_topic: true,
            needs_search: true,
            search_queries: Vec::new(),
            ambiguity_note: None,
        }
    }
}

/// Ask a low-temperature classifier to scrutinize the user's request before the
/// main assistant loop. On parse/call failure it returns a permissive default so
/// the chat is not blocked.
async fn analyze_request(
    request: &ChatRequest,
    candidate: &ModelIdName,
    api_key: &str,
    base_url: &str,
) -> RequestAnalysis {
    let Some(last_user) = request.messages.iter().rev().find(|m| m.role == "user") else {
        return RequestAnalysis::default();
    };

    let classifier_prompt = "You are a request classifier for a Lojban dictionary assistant. \
The user message is the last user turn below. \
Respond ONLY with valid JSON, no markdown, no explanation. \
Use this schema: \
{\"intent\": \"<one-sentence restatement>\", \"on_topic\": <true|false>, \"needs_search\": <true|false>, \"search_queries\": [\"...\"], \"ambiguity_note\": \"<optional clarification note>\"} \
Rules: \
- on_topic = true only if the question is about Lojban language, jbovlaste entries, or your own capabilities. \
- needs_search = true if the answer requires jbovlaste evidence not already in this conversation. \
- search_queries = 1-6 short gloss-style strings in the user's language (use the user's exact words when possible). \
- ambiguity_note = brief clarification if the request is ambiguous; otherwise empty string.";

    let body = ChatCompletionRequest {
        model: candidate.0.clone(),
        messages: vec![
            ChatCompletionMessageRequest {
                role: "system".to_string(),
                content: classifier_prompt.to_string(),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
            ChatCompletionMessageRequest {
                role: "user".to_string(),
                content: last_user.content.clone(),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
        ],
        tools: None,
        tool_choice: None,
        parallel_tool_calls: None,
    };

    let client = reqwest::Client::new();
    let label = "request analysis";
    let api_fut = openrouter_chat_with_retry(label, {
        let client = client.clone();
        let base_url = base_url.to_string();
        let api_key = api_key.to_string();
        let body = body.clone();
        let label = label.to_string();
        move || {
            let client = client.clone();
            let base_url = base_url.clone();
            let api_key = api_key.clone();
            let body = body.clone();
            let label = label.clone();
            async move {
                let res = client
                    .post(format!("{}/chat/completions", base_url))
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await?;
                let ok = ensure_openrouter_status(res, &label).await?;
                parse_chat_response(ok, &label).await
            }
        }
    });

    let timeout = Duration::from_secs(20);
    match tokio::time::timeout(timeout, api_fut).await {
        Ok(Ok(resp)) => {
            let raw = resp
                .choices
                .into_iter()
                .next()
                .and_then(|c| c.message.content)
                .unwrap_or_default();
            match serde_json::from_str::<RequestAnalysis>(&raw) {
                Ok(a) => a,
                Err(e) => {
                    log::warn!(
                        "Assistant: failed to parse request analysis JSON: {} (raw: {})",
                        e,
                        raw
                    );
                    RequestAnalysis::default()
                }
            }
        }
        Ok(Err(e)) => {
            log::warn!("Assistant: request analysis call failed: {}", e);
            RequestAnalysis::default()
        }
        Err(_) => {
            log::warn!(
                "Assistant: request analysis timed out after {}s",
                timeout.as_secs()
            );
            RequestAnalysis::default()
        }
    }
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
        match load_or_fetch_openrouter_candidates(redis, &base_url, &api_key).await {
            Ok(v) => v,
            Err(e) => {
                emit_sse_error(&event_tx, &persist, &e).await;
                return Err(e);
            }
        };

    // Optional pre-loop request analysis: refuse off-topic questions early and
    // avoid wasting model iterations on out-of-scope requests.
    if assistant_request_analysis_enabled() && !candidates.is_empty() {
        let analysis = analyze_request(request, &candidates[0], &api_key, &base_url).await;
        log::debug!(
            "Assistant: request analysis intent='{}' on_topic={} needs_search={} search_queries={:?}",
            analysis.intent,
            analysis.on_topic,
            analysis.needs_search,
            analysis.search_queries
        );
        if !analysis.on_topic {
            let refusal = analysis.ambiguity_note.unwrap_or_else(|| {
                "I am a Lojban dictionary assistant and can only help with questions about the Lojban language, jbovlaste entries, or my own capabilities. Please ask a Lojban-related question.".to_string()
            });
            if let Some(ref tx) = event_tx {
                let payload = json!({
                    "type": "done",
                    "model": candidates[0].0,
                    "model_name": candidates[0].1,
                    "reply": refusal
                });
                if let Err(e) = emit_sse_user_visible(&persist, tx, payload).await {
                    log::warn!(
                        "Assistant: failed to emit SSE done for off-topic refusal: {}",
                        e
                    );
                }
            }
            return Ok((refusal, Vec::new()));
        }
    }

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
                candidates = match fetch_latest_openrouter_models(&base_url, &api_key).await {
                    Ok(v) => v,
                    Err(e2) => {
                        emit_sse_error(&event_tx, &persist, &e2).await;
                        return Err(e2);
                    }
                };
                from_redis_only = false;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn run_agent_loop_with_candidates(
    pool: &Pool,
    request: &ChatRequest,
    mut event_tx: Option<mpsc::Sender<sse::Event>>,
    candidates: &[ModelIdName],
    persist: Option<Arc<ChatPersistState>>,
    redis: Option<&RedisCache>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    if candidates.is_empty() {
        return Err(AppError::ExternalService(
            "Assistant: no OpenRouter model candidates".into(),
        ));
    }

    let is_streaming = event_tx.is_some();
    let run_parallel = ASSISTANT_PARALLEL_DUAL_MODEL && is_streaming && candidates.len() >= 2;
    let parallel_pair: Vec<ModelIdName> = candidates.iter().take(2).cloned().collect();

    if let Some(ref tx) = event_tx {
        sse_stream_debug_models_plan(tx, candidates, run_parallel, &parallel_pair).await;
        if run_parallel {
            emit_parallel_branches(tx, &persist, &parallel_pair).await?;
        }
    }

    if run_parallel {
        // Give each parallel branch its own interleaved fallback chain so the two branches
        // do not race for the same spare model. Each branch runs the full agent loop and
        // switches models internally if its current model fails.
        let tx = event_tx
            .as_ref()
            .expect("run_parallel implies streaming sender")
            .clone();
        let post_debug_tx = tx.clone();
        let final_error_tx = tx.clone();
        let even: Vec<ModelIdName> = candidates.iter().step_by(2).cloned().collect();
        let odd: Vec<ModelIdName> = candidates.iter().skip(1).step_by(2).cloned().collect();
        let pool1 = pool.clone();
        let pool2 = pool.clone();
        let req1 = request.clone();
        let req2 = request.clone();

        let (r1, r2) = tokio::join!(
            run_agent_loop_inner_health_checked(
                &pool1,
                &req1,
                &even,
                Some(tx.clone()),
                persist.clone(),
                redis,
            ),
            run_agent_loop_inner_health_checked(
                &pool2,
                &req2,
                &odd,
                Some(tx.clone()),
                persist.clone(),
                redis,
            ),
        );

        sse_stream_debug_parallel_branch_finished(&post_debug_tx, &even[0].0, &even[0].1, &r1)
            .await;
        sse_stream_debug_parallel_branch_finished(&post_debug_tx, &odd[0].0, &odd[0].1, &r2).await;
        drop(post_debug_tx);

        match (r1, r2) {
            (Ok((reply, _)), Ok(_)) => {
                drop(final_error_tx);
                Ok((reply, vec![]))
            }
            (Ok((reply, _)), Err(e)) | (Err(e), Ok((reply, _))) => {
                let payload = sse_error_payload(&e);
                if let Err(send_err) =
                    emit_sse_user_visible(&persist, &final_error_tx, payload).await
                {
                    drop(final_error_tx);
                    return Err(send_err);
                }
                drop(final_error_tx);
                Ok((reply, vec![]))
            }
            (Err(e1), Err(_)) => {
                let payload = sse_error_payload(&e1);
                if let Err(send_err) =
                    emit_sse_user_visible(&persist, &final_error_tx, payload).await
                {
                    drop(final_error_tx);
                    return Err(send_err);
                }
                drop(final_error_tx);
                Err(e1)
            }
        }
    } else {
        let result = run_agent_loop_inner_health_checked(
            pool,
            request,
            candidates,
            event_tx.clone(),
            persist.clone(),
            redis,
        )
        .await;

        if let Err(ref e) = result {
            if let Some(tx) = event_tx.take() {
                let payload = sse_error_payload(e);
                if let Err(send_err) = emit_sse_user_visible(&persist, &tx, payload).await {
                    drop(tx);
                    return Err(send_err);
                }
                drop(tx);
            }
        }
        result
    }
}

/// Test phrase used as the user message when probing a candidate model with the real assistant
/// chat path. The probe expects any non-empty trimmed reply within
/// [`ASSISTANT_MODEL_HEALTH_TIMEOUT`] for the model to be considered healthy.
pub const ASSISTANT_MODEL_PROBE_PHRASE: &str = "The big brown fox jumps over the lazy dog";

/// Hard ceiling for a single `run_agent_loop_inner` invocation in both probe and real-chat paths;
/// candidates exceeding this are treated as unhealthy and (in real chat) evicted from the Redis
/// assistant model cache.
pub const ASSISTANT_MODEL_HEALTH_TIMEOUT: Duration = Duration::from_secs(120);

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
    let candidate = (model_id.to_string(), model_name.to_string());
    let candidates = [candidate];
    let fut = run_agent_loop_inner(pool, &request, &candidates, None, None, redis);
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
    candidates: &[ModelIdName],
    event_tx: Option<mpsc::Sender<sse::Event>>,
    persist: Option<Arc<ChatPersistState>>,
    redis: Option<&RedisCache>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    let fut = run_agent_loop_inner(pool, request, candidates, event_tx, persist, redis);
    let outcome = tokio::time::timeout(ASSISTANT_MODEL_HEALTH_TIMEOUT, fut).await;
    match outcome {
        Err(_) => Err(AppError::ExternalService(format!(
            "Assistant: did not return a final reply within {}s",
            ASSISTANT_MODEL_HEALTH_TIMEOUT.as_secs()
        ))),
        Ok(Err(e)) => Err(e),
        Ok(Ok((reply, steps))) => {
            if reply.trim().is_empty() {
                Err(AppError::ExternalService(
                    "Assistant: returned empty reply (trimmed)".into(),
                ))
            } else {
                Ok((reply, steps))
            }
        }
    }
}

pub(crate) async fn run_agent_loop_inner(
    pool: &Pool,
    request: &ChatRequest,
    candidates: &[ModelIdName],
    event_tx: Option<mpsc::Sender<sse::Event>>,
    persist: Option<Arc<ChatPersistState>>,
    redis: Option<&RedisCache>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    let mut state = AgentState::new(pool, request, candidates).await?;

    // Destructure mutable references to the state fields we will update in the loop.
    // api_key/base_url/client/system_content/tools are cloned once to owned values
    // so they can be moved into async closures.
    let AgentState {
        ref mut messages,
        ref mut steps,
        ref mut query_seen_count,
        ref mut current_model_idx,
        ref mut model,
        ref mut model_name,
        ref mut aggressive_context_retry,
        ref mut client_round,
        ref system_content,
        ref context_budget,
        ref tools,
        ref candidates,
        ref client,
        ref api_key,
        ref base_url,
        ref iteration_timeout,
        ref max_iterations,
    } = state;

    let api_key = api_key.clone();
    let base_url = base_url.clone();
    let client = client.clone();
    let system_content = system_content.clone();
    let tools = tools.clone();

    // Per-query repetition counter: if the model calls the same search query too many
    // times in a row, inject a tool-level reminder instead of running it again.
    // This mirrors Roo-Code's ToolRepetitionDetector pattern.
    const MAX_QUERY_REPETITIONS: u32 = 2;
    const MAX_RESOLVE_ATTEMPTS: u32 = 3;
    let mut resolve_attempts = 0u32;

    // Agent loop: call LLM until it returns a final reply (no tool_calls).
    for iteration in 1..=*max_iterations {
        let label = format!("chat/completions iteration {}", iteration);
        let response = loop {
            let request_body = ChatCompletionRequest {
                model: model.clone(),
                messages: messages.clone(),
                tools: Some(tools.clone()),
                tool_choice: Some(json!("auto")),
                parallel_tool_calls: Some(true),
            };
            let api_fut = openrouter_chat_with_retry(&label, {
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
            });
            let err: AppError = match tokio::time::timeout(*iteration_timeout, api_fut).await {
                Ok(Ok(r)) => break r,
                Ok(Err(e)) => e,
                Err(_) => AppError::ExternalServiceRetryable {
                    message: format!(
                        "Assistant: LLM call timed out after {}s for model {}",
                        iteration_timeout.as_secs(),
                        model
                    ),
                    raw_response: String::new(),
                },
            };

            if iteration == 1 && !*aggressive_context_retry && error_indicates_context_limit(&err) {
                *aggressive_context_retry = true;
                log::warn!(
                    "Assistant: context limit error on first iteration; retrying with aggressive history compression"
                );
                *client_round = context_compress::compress_chat_history_aggressive(
                    &request.messages,
                    context_budget,
                );
                *messages = vec![ChatCompletionMessageRequest {
                    role: "system".to_string(),
                    content: system_content.clone(),
                    tool_call_id: None,
                    name: None,
                    tool_calls: None,
                }];
                messages.extend(map_chat_messages(&*client_round));
                continue;
            }

            // Evict the failing model from cache and, if there is another candidate,
            // continue the same iteration with the next model. This keeps the
            // conversation state (messages, steps) intact when a model fails.
            if let Some(r) = redis {
                if let Err(e) = evict_openrouter_assistant_model_from_cache(r, model.as_str()).await
                {
                    log::warn!(
                        "Assistant: failed to evict failing model `{}` from Redis cache: {}",
                        model,
                        e
                    );
                }
            }
            if let Some(ref tx) = event_tx {
                sse_stream_debug_model_attempt_failed(
                    tx,
                    model.as_str(),
                    model_name.as_str(),
                    &err,
                )
                .await;
            }
            if *current_model_idx + 1 < candidates.len() {
                *current_model_idx += 1;
                *model = candidates[*current_model_idx].0.clone();
                *model_name = candidates[*current_model_idx].1.clone();
                log::warn!(
                    "Assistant: switching to fallback model {} ({}) for iteration {} after error: {}",
                    model,
                    model_name,
                    iteration,
                    err
                );
                continue;
            }
            return Err(err);
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
                    "model": model.clone(),
                    "model_name": model_name.clone(),
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
                let tool_name = call.function.name.as_deref().unwrap_or("unknown");
                if tool_name != "jbovlaste_semantic_search" && tool_name != "jbovlaste_resolve_results" {
                    log::error!(
                        "Assistant: unexpected tool call '{}' — not in schema",
                        tool_name
                    );
                    prepared.push(PreparedToolSlot::Immediate {
                        tool_call_id: call.id.clone(),
                        name: call.function.name.clone(),
                        content: "Unknown tool. Use jbovlaste_semantic_search or jbovlaste_resolve_results.".to_string(),
                    });
                    continue;
                }

                if tool_name == "jbovlaste_resolve_results" {
                    let global_step_index = base_step_index + pending_search_ordinal;
                    pending_search_ordinal += 1;
                    let ar = content_str.trim();
                    let assistant_reasoning = if ar.is_empty() { None } else { Some(ar.to_string()) };
                    let args_json: &str = match call.function.arguments.as_deref() {
                        None | Some("") => "{}",
                        Some(s) => s,
                    };
                    let args: ResolveArgs = match serde_json::from_str(args_json) {
                        Ok(a) => a,
                        Err(e) => {
                            log::warn!(
                                "Resolve tool arguments JSON parse error: {}; raw arguments: {:?}",
                                e,
                                call.function.arguments
                            );
                            prepared.push(PreparedToolSlot::Immediate {
                                tool_call_id: call.id.clone(),
                                name: call.function.name.clone(),
                                content: format!("Tool error: jbovlaste_resolve_results: invalid JSON: {}", e),
                            });
                            continue;
                        }
                    };
                    match args.normalized() {
                        Ok((refs, None)) => {
                            let refs_count = refs.len();
                            prepared.push(PreparedToolSlot::Resolve {
                                tool_call_id: call.id.clone(),
                                name: call.function.name.clone(),
                                refs,
                                message: None,
                                assistant_reasoning,
                                global_step_index,
                                action_desc: format!("Resolve references: {} item(s)", refs_count),
                            });
                        }
                        Ok((_, Some(msg))) => {
                            prepared.push(PreparedToolSlot::Resolve {
                                tool_call_id: call.id.clone(),
                                name: call.function.name.clone(),
                                refs: Vec::new(),
                                message: Some(msg),
                                assistant_reasoning,
                                global_step_index,
                                action_desc: "Resolve: no results".to_string(),
                            });
                        }
                        Err(msg) => {
                            prepared.push(PreparedToolSlot::Immediate {
                                tool_call_id: call.id.clone(),
                                name: call.function.name.clone(),
                                content: format!("Tool error: jbovlaste_resolve_results: {}", msg),
                            });
                        }
                    }
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
                            search_query_for_llm, prior
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
                        }
                        | PreparedToolSlot::Resolve {
                            global_step_index,
                            action_desc,
                            tool_call_id,
                            assistant_reasoning,
                            ..
                        } = slot
                        {
                            let mut start_payload = json!({
                                "type": "step_start",
                                "model": model.clone(),
                                "model_name": model_name.clone(),
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

                        let outcomes = join_all(trimmed.iter().zip(embeddings).map(|(q, emb)| {
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
                        }))
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
                    PreparedToolSlot::Resolve {
                        tool_call_id,
                        name,
                        refs,
                        message,
                        assistant_reasoning,
                        global_step_index,
                        action_desc,
                    } => {
                        if let Some(msg) = message {
                            let result = msg.clone();
                            if let Some(ref tx) = event_tx {
                                let mut payload = json!({
                                    "type": "step",
                                    "model": model.clone(),
                                    "model_name": model_name.clone(),
                                    "index": global_step_index,
                                    "action": action_desc,
                                    "result": "No results",
                                    "tool_call_id": tool_call_id,
                                    "tool_content_plain": result,
                                });
                                if let Some(ref ar) = assistant_reasoning {
                                    payload["assistant_reasoning"] = serde_json::Value::String(ar.clone());
                                }
                                emit_sse_user_visible(&persist, tx, payload).await?;
                                let done_payload = json!({
                                    "type": "done",
                                    "model": model.clone(),
                                    "model_name": model_name.clone(),
                                    "reply": result
                                });
                                emit_sse_user_visible(&persist, tx, done_payload).await?;
                            }
                            return Ok((result, steps.clone()));
                        }

                        resolve_attempts += 1;
                        if resolve_attempts > MAX_RESOLVE_ATTEMPTS {
                            let err = "Too many attempts to resolve references. Please start a new search.".to_string();
                            if let Some(ref tx) = event_tx {
                                let payload = json!({
                                    "type": "done",
                                    "model": model.clone(),
                                    "model_name": model_name.clone(),
                                    "reply": err
                                });
                                emit_sse_user_visible(&persist, tx, payload).await?;
                            }
                            return Ok((err, steps.clone()));
                        }

                        match resolve_references(pool, &refs).await? {
                            ResolveOutcome::Valid(validated) => {
                                let markdown = build_printable_markdown(&validated);
                                let result_summary = format!("Resolved {} reference(s)", validated.len());
                                let tool_content_for_llm = "References validated; printable answer generated.".to_string();
                                let step = AssistantStep {
                                    action: action_desc,
                                    result: result_summary,
                                    tool_output: None,
                                    assistant_reasoning,
                                };
                                steps.push(step.clone());
                                if let Some(ref tx) = event_tx {
                                    let mut payload = json!({
                                        "type": "step",
                                        "model": model.clone(),
                                        "model_name": model_name.clone(),
                                        "index": global_step_index,
                                        "action": step.action,
                                        "result": step.result,
                                        "tool_call_id": tool_call_id,
                                        "tool_content_plain": tool_content_for_llm,
                                    });
                                    if let Some(ref ar) = step.assistant_reasoning {
                                        payload["assistant_reasoning"] = serde_json::Value::String(ar.clone());
                                    }
                                    emit_sse_user_visible(&persist, tx, payload).await?;
                                    let done_payload = json!({
                                        "type": "done",
                                        "model": model.clone(),
                                        "model_name": model_name.clone(),
                                        "reply": markdown
                                    });
                                    emit_sse_user_visible(&persist, tx, done_payload).await?;
                                }
                                return Ok((markdown, steps.clone()));
                            }
                            ResolveOutcome::Invalid(errors) => {
                                let mut err_lines = String::from("Some references are invalid. Fix them and call jbovlaste_resolve_results again:\n");
                                for e in &errors {
                                    err_lines.push_str(&format!(
                                        "- index {} (definitionid {}, field '{}', exact_text '{}'): {}\n",
                                        e.index, e.definitionid, e.field, e.exact_text, e.reason
                                    ));
                                }
                                let result_summary = format!("Invalid: {} reference(s) failed validation", errors.len());
                                let step = AssistantStep {
                                    action: action_desc,
                                    result: result_summary,
                                    tool_output: None,
                                    assistant_reasoning,
                                };
                                steps.push(step.clone());
                                if let Some(ref tx) = event_tx {
                                    let mut payload = json!({
                                        "type": "step",
                                        "model": model.clone(),
                                        "model_name": model_name.clone(),
                                        "index": global_step_index,
                                        "action": step.action,
                                        "result": step.result,
                                        "tool_call_id": tool_call_id,
                                        "tool_content_plain": err_lines.clone(),
                                    });
                                    if let Some(ref ar) = step.assistant_reasoning {
                                        payload["assistant_reasoning"] = serde_json::Value::String(ar.clone());
                                    }
                                    emit_sse_user_visible(&persist, tx, payload).await?;
                                }
                                messages.push(ChatCompletionMessageRequest {
                                    role: "tool".to_string(),
                                    content: err_lines,
                                    tool_call_id,
                                    name,
                                    tool_calls: None,
                                });
                            }
                        }
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

                        let tool_content_json = serde_json::to_string(&tool_payload_value)
                            .unwrap_or_else(|_| "{}".to_string());

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
                                "model": model.clone(),
                                "model_name": model_name.clone(),
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
            // No tool calls: nudge the model toward the required tools.
            let reply = strip_llm_corner_bracket_segments(
                &choice.message.content.clone().unwrap_or_else(String::new),
            );

            if iteration < *max_iterations {
                log::warn!(
                    "Assistant: no tool calls at iteration {}; injecting resolve nudge",
                    iteration
                );
                let nudge = if reply.trim().is_empty() {
                    "Use jbovlaste_semantic_search to find evidence, then finish by calling jbovlaste_resolve_results with exact references. Do not write prose."
                } else {
                    "Do not write prose. Submit your answer by calling jbovlaste_resolve_results with exact references from the search results."
                };
                messages.push(ChatCompletionMessageRequest {
                    role: "user".to_string(),
                    content: nudge.to_string(),
                    tool_call_id: None,
                    name: None,
                    tool_calls: None,
                });
                continue;
            }
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
            "model": model.clone(),
            "model_name": model_name.clone(),
            "reply": &last_content
        });
        emit_sse_user_visible(&persist, &tx, payload).await?;
        drop(tx);
    }
    Ok((last_content, steps.clone()))
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
        assert!(super::error_indicates_context_limit(
            &AppError::ExternalServiceWithRaw {
                message: "x".into(),
                raw_response: "prompt is too long".into(),
            }
        ));
        assert!(!super::error_indicates_context_limit(
            &AppError::BadRequest("nope".into())
        ));
    }
}

#[cfg(test)]
mod request_analysis_tests {
    use super::RequestAnalysis;

    #[test]
    fn parses_request_analysis_json() {
        let raw = r#"{"intent": "lookup fox", "on_topic": true, "needs_search": true, "search_queries": ["fox", "animal"], "ambiguity_note": ""}"#;
        let a: RequestAnalysis = serde_json::from_str(raw).unwrap();
        assert_eq!(a.intent, "lookup fox");
        assert!(a.on_topic);
        assert!(a.needs_search);
        assert_eq!(a.search_queries, vec!["fox", "animal"]);
        assert_eq!(a.ambiguity_note.as_deref(), Some(""));
    }

    #[test]
    fn default_request_analysis_is_permissive() {
        let a = RequestAnalysis::default();
        assert!(a.on_topic);
        assert!(a.needs_search);
        assert!(a.search_queries.is_empty());
    }

    #[test]
    fn request_analysis_parses_off_topic() {
        let raw = r#"{"intent": "weather", "on_topic": false, "needs_search": false}"#;
        let a: RequestAnalysis = serde_json::from_str(raw).unwrap();
        assert!(!a.on_topic);
        assert!(!a.needs_search);
    }
}
