#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::env;
use std::time::Duration;

use actix_web_lab::sse;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::embeddings::get_embedding;
use crate::error::AppError;
use crate::jbovlaste::models::{DefinitionDetail, DefinitionResponse, SearchDefinitionsParams};
use crate::jbovlaste::service::{cmavo_gismu_english_dictionary_text, semantic_search};

use super::context_compress;
use super::dto::{AssistantStep, ChatMessage, ChatRequest, ToolCallDto};

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

/// OpenRouter /api/v1/models/user response: list of models (we use flexible parsing for pricing).
#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    created: Option<f64>,
    #[serde(default)]
    context_length: Option<f64>,
    #[serde(default)]
    architecture: Option<OpenRouterArchitecture>,
    /// e.g. `{ "prompt": "0", "completion": "0" }` — we require both to be zero for “free” models.
    #[serde(default)]
    pricing: Option<serde_json::Value>,
}

fn price_is_zero(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Number(n) => n.as_f64() == Some(0.0),
        serde_json::Value::String(s) => s == "0" || s == "0.0",
        serde_json::Value::Null => true,
        _ => false,
    }
}

#[derive(Debug, Deserialize)]
struct OpenRouterArchitecture {
    #[serde(default)]
    modality: Option<String>,
    #[serde(default)]
    input_modalities: Option<Vec<String>>,
    #[serde(default)]
    output_modalities: Option<Vec<String>>,
}

fn is_text_to_text(arch: &OpenRouterArchitecture) -> bool {
    let modality = arch.modality.as_deref().unwrap_or("");
    if modality == "text" {
        return true;
    }
    let inp = arch.input_modalities.as_deref().unwrap_or(&[]);
    let out = arch.output_modalities.as_deref().unwrap_or(&[]);
    inp.contains(&"text".to_string()) && out.contains(&"text".to_string())
}

/// (model_id, display_name) for use in UI. Display name is OpenRouter "name" or id as fallback.
pub type ModelIdName = (String, String);

/// OpenRouter aggregate / router endpoints — not concrete provider models (avoid for parallel runs).
fn is_placeholder_or_router_only_model(id: &str) -> bool {
    matches!(id, "openrouter/free" | "openrouter/auto")
}

async fn fetch_openrouter_models_list(
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<Vec<OpenRouterModel>, AppError> {
    let url = format!(
        "{}/models{}",
        base_url.trim_end_matches('/'),
        query
    );
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| AppError::ExternalService(format!("OpenRouter models request failed: {}", e)))?;

    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_else(|_| String::from("(no body)"));
        return Err(AppError::ExternalServiceWithRaw {
            message: format!("OpenRouter models returned {}", status),
            raw_response: body,
        });
    }

    let body: OpenRouterModelsResponse = res.json().await.map_err(|e| {
        AppError::ExternalService(format!("OpenRouter models JSON parse error: {}", e))
    })?;
    Ok(body.data)
}

/// Fetch the two **newest free** (`pricing.prompt` and `pricing.completion` both zero) text models from
/// OpenRouter's public [`GET /models`](https://openrouter.ai/docs/api/api-reference/models/get-models) catalog.
///
/// Uses the full model list (not `/models/user`). Excludes router placeholders (`openrouter/free`,
/// `openrouter/auto`) so parallel runs use real provider slugs when any exist. Prefers long context
/// (100k+), then falls back to 32k+. Sorted by `created` descending.
pub async fn fetch_latest_openrouter_models(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ModelIdName>, AppError> {
    const MIN_CONTEXT_PREFERRED: f64 = 100_000.0;
    const MIN_CONTEXT_FALLBACK: f64 = 32_000.0;

    let mut data = match fetch_openrouter_models_list(
        base_url,
        api_key,
        "?output_modalities=text&supported_parameters=tools",
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            log::debug!(
                "OpenRouter models (tools filter) failed or unavailable: {}; retrying without tools filter",
                e
            );
            fetch_openrouter_models_list(base_url, api_key, "?output_modalities=text").await?
        }
    };

    if data.is_empty() {
        data = fetch_openrouter_models_list(base_url, api_key, "?output_modalities=text").await?;
    }

    let pick = |min_ctx: f64| -> Vec<ModelIdName> {
        let mut eligible: Vec<(f64, String, String)> = data
            .iter()
            .filter_map(|m| {
                if is_placeholder_or_router_only_model(&m.id) {
                    return None;
                }
                let ctx = m.context_length.unwrap_or(0.0);
                if ctx <= min_ctx {
                    return None;
                }
                let arch = m.architecture.as_ref()?;
                if !is_text_to_text(arch) {
                    return None;
                }
                let pricing = m.pricing.as_ref()?;
                let prompt_zero = pricing.get("prompt").is_some_and(price_is_zero);
                let completion_zero = pricing.get("completion").is_some_and(price_is_zero);
                if !prompt_zero || !completion_zero {
                    return None;
                }
                let created = m.created.unwrap_or(0.0);
                let name = m
                    .name
                    .as_ref()
                    .filter(|s| !s.is_empty())
                    .map(String::to_owned)
                    .unwrap_or_else(|| m.id.clone());
                Some((created, m.id.clone(), name))
            })
            .collect();

        eligible.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        eligible
            .into_iter()
            .map(|(_, id, name)| (id, name))
            .take(2)
            .collect()
    };

    let mut top = pick(MIN_CONTEXT_PREFERRED);
    if top.is_empty() {
        top = pick(MIN_CONTEXT_FALLBACK);
    }

    if top.is_empty() {
        log::warn!(
            "OpenRouter: no free (zero-priced) text models matched filters; falling back to openrouter/free"
        );
        return Ok(vec![("openrouter/free".to_string(), "OpenRouter Free".to_string())]);
    }
    Ok(top)
}

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

fn system_prompt_base(locale: Option<&str>) -> String {
    let mut base = String::from(
        "You are a Lojban dictionary assistant. You must strictly rely on jbovlaste semantic search results whenever you need dictionary evidence.\n\
         - **Canonical cmavo/gismu list:** The message may include a block titled \"Official cmavo and gismu (English, jbovlaste)\". That list is the ground truth for whether a Lojban **cmavo** or **gismu** exists and for its English gloss in this database. Do **not** invent cmavo or gismu that are not in that list. For **lujvo**, **fu'ivla**, **cmavo compounds**, or anything not in that list, use `jbovlaste_semantic_search`.\n\
         - **When to call `jbovlaste_semantic_search`:** Call it when the user asks about Lojban words, concepts, or meanings that require **new** jbovlaste evidence: e.g. the first question in a thread, a **different** valsi or topic than already covered, or when nothing in the prior conversation gives you grounded definitions for what they asked.\n\
         - **When NOT to call (no tool):** If the user’s message is a **clarification, follow-up, or rephrase** about the **same** word or concept you already answered using prior search-backed content in this conversation, answer from that conversation only—**do not** run semantic search again. Same for “explain simpler”, “give an example from what you already quoted”, or short confirmations—reuse prior assistant turns.\n\
         - **Multi-step search within one answer:** After you receive tool results, read them carefully. If they are insufficient, off-topic, too vague, or you need related terms, alternate phrasings, or another sub-concept **for that same reply**, call `jbovlaste_semantic_search` again in a *later* step with a new or refined query. Do not give a final answer until search results actually support it.\n\
         - In a single step you MAY issue several `jbovlaste_semantic_search` calls in parallel (e.g. multiple concepts or phrasings). You MAY search again in later **steps of the same turn** when refining.\n\
         - Base any statement about valsi, glosses, or definitions on the definitions returned by the tool(s) in this thread **or** on prior assistant messages that were already grounded in those tools. Quote or paraphrase from those results; do not invent valsi, glosses, or definitions.\n\
         - Do not make up examples or Lojban text. Only use valsi, glosses, and sentences from semantic search results (or from your earlier replies that cited them).\n\
         - If the search returns no or few results, try different queries in further steps, or say so and suggest rephrasing; do not make up answers.\n\
         - You have access only to the tool `jbovlaste_semantic_search`. Use it with a natural-language query (e.g. in English) to find relevant jbovlaste definitions.\n\
         - Use the `limit` parameter when needed: default is fine for narrow queries; raise it (up to the allowed maximum) when you need a broader sample of candidates.\n\
         - Prefer using your platform's native tool-calling. If you cannot and must output a tool call as text, use exactly this format once per call: CALL>[{\"name\":\"jbovlaste_semantic_search\",\"arguments\":{\"query\":\"your search query\"}}]</TOOLCALL>\n\
         - Format your reply in valid, simple Markdown: use **bold**, lists. Use plain text or markdown lists instead of markdown tables.\n\
         - When quoting definitions from jbovlaste, preserve inline `$...$` math delimiters exactly as in the tool output (they are part of the definition text).",
    );

    if let Some(loc) = locale {
        if !loc.is_empty() {
            base.push_str(&format!(
                "\nPrefer to explain things in locale `{}` where appropriate.",
                loc
            ));
        }
    }

    base
}

static ASSISTANT_CMAVO_GISMU_DICT_CACHE: Lazy<RwLock<Option<String>>> =
    Lazy::new(|| RwLock::new(None));

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

/// Loads English cmavo+gismu lines from the DB (jbovlaste), cached for the process lifetime (official list is stable) with optional size cap.
async fn assistant_cmavo_gismu_dictionary_cached(pool: &Pool) -> String {
    let max_chars: usize = env::var("ASSISTANT_GISMU_CMAVO_DICT_MAX_CHARS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120_000);

    {
        let guard = ASSISTANT_CMAVO_GISMU_DICT_CACHE.read().await;
        if let Some(ref s) = *guard {
            return s.clone();
        }
    }

    let fetched = match cmavo_gismu_english_dictionary_text(pool).await {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Failed to load cmavo/gismu dictionary for assistant: {}", e);
            return String::new();
        }
    };

    let text = if fetched.len() > max_chars {
        let prefix = truncate_utf8_prefix(&fetched, max_chars);
        let cut = prefix.rfind('\n').unwrap_or(prefix.len());
        let mut t = prefix[..cut].to_string();
        t.push_str("\n\n[Dictionary truncated for context size; remaining entries omitted.]");
        t
    } else {
        fetched
    };

    if text.is_empty() {
        return text;
    }

    {
        let mut guard = ASSISTANT_CMAVO_GISMU_DICT_CACHE.write().await;
        if let Some(ref s) = *guard {
            return s.clone();
        }
        *guard = Some(text.clone());
    }
    text
}

async fn system_prompt_with_dictionary(pool: &Pool, locale: Option<&str>) -> String {
    let base = system_prompt_base(locale);
    let dict = assistant_cmavo_gismu_dictionary_cached(pool).await;
    if dict.is_empty() {
        return base;
    }
    format!(
        "{}\n\n## Official cmavo and gismu (English, jbovlaste)\n\
Each line is `word - definition` (best English definition per word). Use this as the authoritative list of **cmavo** and **gismu** in this database.\n\n{}",
        base, dict
    )
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
            description: "Semantic search over jbovlaste definitions. Call when you need jbovlaste evidence (new topic, new valsi, or no prior grounded answer in the conversation). Do not call for clarifying follow-ups about material already answered from search in this thread."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural-language concept to search for (typically in English)."
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 30,
                        "default": 12,
                        "description": "Number of top semantic matches to return (use more when you need a wider set of candidates)."
                    },
                    "languages": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Optional list of non-Lojban language IDs to restrict definitions to."
                    },
                    "source_langid": {
                        "type": "integer",
                        "description": "Optional source language ID of the valsi (defaults to 1 = Lojban)."
                    }
                },
                "required": ["query"]
            }),
        },
    }
}

#[derive(Debug, Deserialize)]
struct ToolArgs {
    query: String,
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    languages: Option<Vec<i32>>,
    #[serde(default)]
    source_langid: Option<i32>,
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
const SEMANTIC_SEARCH_MAX_LIMIT: u32 = 30;

async fn run_jbovlaste_semantic_search_once(
    pool: &Pool,
    args: &ToolArgs,
) -> Result<DefinitionResponse, AppError> {
    let limit = args
        .limit
        .unwrap_or(12)
        .clamp(1, SEMANTIC_SEARCH_MAX_LIMIT) as i64;

    let embedding = get_embedding(&args.query).await?;

    let params = SearchDefinitionsParams {
        page: 1,
        per_page: limit,
        search_term: args.query.clone(),
        include_comments: false,
        sort_by: "score".to_string(),
        sort_order: "desc".to_string(),
        languages: args.languages.clone(),
        selmaho: None,
        username: None,
        word_type: None,
        source_langid: args.source_langid,
        search_in_phrases: None,
        include_total_count: false,
    };

    let response = semantic_search(pool, params, embedding, None)
        .await
        .map_err(|e| {
            AppError::ExternalService(format!(
                "Semantic search failed for query \"{}\": {}",
                args.query, e
            ))
        })?;

    Ok(response)
}

/// Runs semantic search with retries on transient failure (embedding or DB/network).
async fn run_jbovlaste_semantic_search_with_retry(
    pool: &Pool,
    args: ToolArgs,
) -> Result<DefinitionResponse, AppError> {
    for attempt in 1..=TOOL_MAX_ATTEMPTS {
        match run_jbovlaste_semantic_search_once(pool, &args).await {
            Ok(r) => return Ok(r),
            Err(e) => {
                if attempt < TOOL_MAX_ATTEMPTS {
                    let delay = Duration::from_millis(
                        TOOL_INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1),
                    );
                    log::info!(
                        "Assistant semantic search retry {}/{} for query \"{}\" after {:?}",
                        attempt,
                        TOOL_MAX_ATTEMPTS,
                        args.query,
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
        "similarity": def.similarity,
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
        if let Some(sim) = def.similarity {
            out.push_str(&format!("relevance: {:.4}\n", sim));
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
        if let Some(s) = def.selmaho.as_ref() {
            if !s.trim().is_empty() {
                out.push_str(&format!("selmaho: {}\n", s));
            }
        }
        out.push('\n');
    }
    out
}

/// Maximum number of agent turns (LLM call + tool executions) per user message.
const AGENT_MAX_ITERATIONS: u32 = 15;

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
        let truncated = if raw_response.len() > ERROR_RAW_RESPONSE_MAX_LEN {
            format!(
                "{}... [truncated, total {} bytes]",
                &raw_response[..ERROR_RAW_RESPONSE_MAX_LEN],
                raw_response.len()
            )
        } else {
            raw_response.clone()
        };
        obj["raw_response"] = serde_json::Value::String(truncated);
    }
    obj
}

/// Runs the agent loop. If event_tx is Some, streams step/done/error events (with optional "model" key for parallel runs).
/// When event_tx is Some and we have 2 models, runs both in parallel and streams both; otherwise runs a single model.
pub async fn run_agent_loop(
    pool: &Pool,
    request: &ChatRequest,
    event_tx: Option<mpsc::Sender<sse::Event>>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
        AppError::ExternalService("OPENROUTER_API_KEY is not set in the environment".into())
    })?;
    let base_url = env::var("OPENROUTER_API_BASE")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let models = fetch_latest_openrouter_models(&base_url, &api_key).await?;
    let models: Vec<ModelIdName> = models.into_iter().take(2).collect();
    let is_streaming = event_tx.is_some();
    let run_parallel = is_streaming && models.len() == 2;

    if run_parallel {
        let tx1 = event_tx.clone().expect("run_parallel implies Some");
        let tx2 = event_tx.expect("run_parallel implies Some");
        let pool1 = pool.clone();
        let pool2 = pool.clone();
        let req1 = request.clone();
        let req2 = request.clone();
        let (m1_id, m1_name) = models[0].clone();
        let (m2_id, m2_name) = models[1].clone();
        let (r1, r2) = tokio::join!(
            run_agent_loop_inner(&pool1, &req1, &m1_id, &m1_name, Some(tx1)),
            run_agent_loop_inner(&pool2, &req2, &m2_id, &m2_name, Some(tx2)),
        );
        // Drop senders so the SSE stream ends. Return first successful reply for API compatibility.
        if let Ok((reply, _)) = r1 {
            Ok((reply, vec![]))
        } else if let Ok((reply, _)) = r2 {
            Ok((reply, vec![]))
        } else {
            Err(r1.unwrap_err())
        }
    } else {
        let (model_id, model_name) = models
            .first()
            .cloned()
            .unwrap_or_else(|| ("openrouter/free".to_string(), "OpenRouter Free".to_string()));
        let mut event_tx = event_tx;
        let result =
            run_agent_loop_inner(pool, request, &model_id, &model_name, event_tx.take()).await;
        if let Err(ref e) = result {
            if let Some(tx) = event_tx.take() {
                let payload = sse_error_payload(e);
                if let Ok(data) = serde_json::to_string(&payload) {
                    let _ = tx.send(sse::Data::new(data).into()).await;
                }
                drop(tx);
            }
        }
        result
    }
}

async fn run_agent_loop_inner(
    pool: &Pool,
    request: &ChatRequest,
    model: &str,
    model_name: &str,
    event_tx: Option<mpsc::Sender<sse::Event>>,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
        AppError::ExternalService("OPENROUTER_API_KEY is not set in the environment".into())
    })?;

    let base_url = env::var("OPENROUTER_API_BASE")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let client = reqwest::Client::new();

    let system_content = system_prompt_with_dictionary(pool, request.locale.as_deref()).await;
    let mut client_round = context_compress::compress_chat_history_for_request(&request.messages);
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

    // Agent loop: call LLM until it returns a final reply (no tool_calls).
    for iteration in 1..=AGENT_MAX_ITERATIONS {
        let label = format!("chat/completions iteration {}", iteration);
        let response = loop {
            let request_body = ChatCompletionRequest {
                model: model.to_string(),
                messages: messages.clone(),
                tools: Some(tools.clone()),
                tool_choice: Some(json!("auto")),
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
                        client_round =
                            context_compress::compress_chat_history_aggressive(&request.messages);
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
                if let Ok(data) = serde_json::to_string(&payload) {
                    let _ = tx.send(sse::Data::new(data).into()).await;
                }
            }
            // Execute each tool call and append tool results. Recover from parse/tool errors by
            // feeding an error message back to the LLM so it can try again.
            for call in calls.iter() {
                if call.function.name.as_deref() != Some("jbovlaste_semantic_search") {
                    continue;
                }

                // Global index across all agent iterations so SSE clients never overwrite earlier steps.
                let global_step_index = steps.len();

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
                        messages.push(ChatCompletionMessageRequest {
                            role: "tool".to_string(),
                            content: format!("Tool error: {}", err_msg),
                            tool_call_id: call.id.clone(),
                            name: call.function.name.clone(),
                            tool_calls: None,
                        });
                        continue;
                    }
                };

                let search_query_for_llm = args.query.clone();
                let action_desc = format!("Semantic search: \"{}\"", search_query_for_llm);

                // Emit step_start immediately so the UI shows this step before the tool runs.
                if let Some(ref tx) = event_tx {
                    let start_payload = json!({
                        "type": "step_start",
                        "model": model,
                        "model_name": model_name,
                        "index": global_step_index,
                        "action": action_desc,
                        "tool_call_id": call.id,
                    });
                    if let Ok(data) = serde_json::to_string(&start_payload) {
                        let _ = tx.send(sse::Data::new(data).into()).await;
                    }
                }

                let tool_result = run_jbovlaste_semantic_search_with_retry(pool, args).await;

                let (result_summary, tool_payload_value) = match &tool_result {
                    Ok(results) => {
                        let n = results.definitions.len();
                        let summary = format!("Returned {} definition(s).", n);
                        let compact_results: Vec<serde_json::Value> = results
                            .definitions
                            .iter()
                            .map(summarise_definition)
                            .collect();
                        let payload = json!({
                            "results": compact_results,
                        });
                        (summary, payload)
                    }
                    Err(e) => {
                        let err_str = format!("{}", e);
                        log::warn!("Assistant semantic search failed after retries: {}", err_str);
                        let summary = format!("Error after retries: {}", err_str);
                        let payload = json!({
                            "error": err_str,
                            "results": [],
                        });
                        (summary, payload)
                    }
                };

                let tool_content_json = serde_json::to_string(&tool_payload_value).unwrap_or_else(|_| {
                    tool_payload_value["error"]
                        .as_str()
                        .unwrap_or("")
                        .to_string()
                });

                let tool_content_for_llm = match &tool_result {
                    Ok(results) => semantic_tool_results_plain_text_for_llm(
                        &search_query_for_llm,
                        &results.definitions,
                    ),
                    Err(e) => format!("Semantic search error: {}", e),
                };

                let step = AssistantStep {
                    action: action_desc.clone(),
                    result: result_summary.clone(),
                    tool_output: Some(tool_content_json.clone()),
                };
                steps.push(step.clone());

                // Stream step event with result and optional tool_output so the UI can show it folded.
                if let Some(ref tx) = event_tx {
                    let mut payload = json!({
                        "type": "step",
                        "model": model,
                        "model_name": model_name,
                        "index": global_step_index,
                        "action": step.action,
                        "result": step.result,
                        "tool_call_id": call.id,
                        "tool_content_plain": tool_content_for_llm,
                    });
                    if let Some(ref out) = step.tool_output {
                        payload["tool_output"] = serde_json::Value::String(out.clone());
                    }
                    if let Ok(data) = serde_json::to_string(&payload) {
                        let _ = tx.send(sse::Data::new(data).into()).await;
                    }
                }

                messages.push(ChatCompletionMessageRequest {
                    role: "tool".to_string(),
                    content: tool_content_for_llm,
                    tool_call_id: call.id.clone(),
                    name: call.function.name.clone(),
                    tool_calls: None,
                });
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
            if let Some(tx) = event_tx {
                let payload = json!({
                    "type": "done",
                    "model": model,
                    "model_name": model_name,
                    "reply": reply
                });
                if let Ok(data) = serde_json::to_string(&payload) {
                    let _ = tx.send(sse::Data::new(data).into()).await;
                }
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
        if let Ok(data) = serde_json::to_string(&payload) {
            let _ = tx.send(sse::Data::new(data).into()).await;
        }
        drop(tx);
    }
    Ok((last_content, steps))
}

/// Non-streaming entry point: runs the agent and returns reply + steps.
pub async fn handle_chat(
    pool: &Pool,
    request: ChatRequest,
) -> Result<(String, Vec<AssistantStep>), AppError> {
    run_agent_loop(pool, &request, None).await
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
                        arguments: Some(r#"{"query":"test"}"#.into()),
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
