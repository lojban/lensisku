use std::env;
use std::time::Duration;

use deadpool_postgres::Pool;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::sleep;

use crate::embeddings::get_embedding;
use crate::error::AppError;
use crate::jbovlaste::models::{DefinitionDetail, DefinitionResponse, SearchDefinitionsParams};
use crate::jbovlaste::service::semantic_search;

use super::dto::{ChatMessage, ChatRequest};

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
    let body = res.text().await.unwrap_or_else(|_| String::from("(failed to read body)"));
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
            let retryable = if let Ok(err_payload) = serde_json::from_str::<OpenRouterErrorPayload>(body_trimmed) {
                let code = err_payload.error.code;
                let msg = if err_payload.error.message.is_empty() {
                    format!("Invalid {} response: {}", label, e)
                } else {
                    format!("{}: {}", label, err_payload.error.message)
                };
                let is_server_error = code.map(|c| c >= 500).unwrap_or(true);
                if is_server_error {
                    log::warn!("OpenRouter {} returned error body (code {:?}), will retry: {}", label, code, msg);
                    Some((msg, body.clone()))
                } else {
                    None
                }
            } else {
                // Unrecognized shape; treat parse failure as retryable (transient malformed response).
                Some((
                    format!("Invalid {} response: {}", label, e),
                    body.clone(),
                ))
            };
            if let Some((message, raw_response)) = retryable {
                log::debug!("OpenRouter {} response (status {}): {}", label, status, raw_response);
                return Err(AppError::ExternalServiceRetryable {
                    message,
                    raw_response,
                });
            }
            log::debug!("OpenRouter {} response (status {}): {}", label, status, body);
            log::warn!("OpenRouter {} parse error: {} (see debug log for raw body)", label, e);
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
async fn openrouter_chat_with_retry<F, Fut>(label: &str, mut run: F) -> Result<ChatCompletionResponse, AppError>
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
                        let delay = Duration::from_millis(OPENROUTER_INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1));
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
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct ToolCallFunction {
    #[serde(default)]
    name: String,
    // OpenRouter/OpenAI send arguments as a JSON string; some providers (e.g. Arcee) send null
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct ToolCall {
    #[serde(default)]
    id: String,
    #[serde(default)]
    r#type: String,
    function: ToolCallFunction,
}

fn system_prompt(locale: Option<&str>) -> String {
    let mut base = String::from(
        "You are a Lojban dictionary assistant. You must strictly rely on jbovlaste semantic search results.\n\
         - For every user query about Lojban words, concepts, or meanings: you MUST call the tool `jbovlaste_semantic_search` first. Do not answer from general knowledge.\n\
         - Base your reply ONLY on the definitions returned by the tool. Quote or paraphrase from those results; do not invent valsi, glosses, or definitions.\n\
         - Do not make up examples or Lojban text. Only use valsi, glosses, and sentences from the semantic search results.\n\
         - If the search returns no or few results, say so and suggest rephrasing the query; do not make up answers.\n\
         - You have access only to the tool `jbovlaste_semantic_search`. Use it with a natural-language query (e.g. in English) to find relevant jbovlaste definitions.\n\
         - Format your reply in valid, simple Markdown: use **bold**, lists. use plain text or markdown lists instead of markdown tables.",
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

fn to_openrouter_role(role: &str) -> String {
    match role {
        "user" | "assistant" | "system" => role.to_string(),
        other => {
            log::warn!("Unknown chat role `{}`, mapping to `user`", other);
            "user".to_string()
        }
    }
}

fn map_chat_messages(messages: &[ChatMessage]) -> Vec<ChatCompletionMessageRequest> {
    messages
        .iter()
        .map(|m| ChatCompletionMessageRequest {
            role: to_openrouter_role(&m.role),
            content: m.content.clone(),
            tool_call_id: None,
            name: None,
            tool_calls: None,
        })
        .collect()
}

fn jbovlaste_tool_schema() -> Tool {
    Tool {
        r#type: "function".to_string(),
        function: ToolFunction {
            name: "jbovlaste_semantic_search".to_string(),
            description: "Semantic search over jbovlaste definitions. Call this for every user question about Lojban words or concepts; results are the only source you may use for valsi and definitions."
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
                        "maximum": 50,
                        "default": 10,
                        "description": "Maximum number of results to return."
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

async fn run_jbovlaste_semantic_search(
    pool: &Pool,
    args: ToolArgs,
) -> Result<DefinitionResponse, AppError> {
    let limit = args.limit.unwrap_or(10).clamp(1, 50) as i64;

    let embedding = get_embedding(&args.query).await?;

    let params = SearchDefinitionsParams {
        page: 1,
        per_page: limit,
        search_term: args.query.clone(),
        include_comments: false,
        sort_by: "score".to_string(),
        sort_order: "desc".to_string(),
        languages: args.languages,
        selmaho: None,
        username: None,
        word_type: None,
        source_langid: args.source_langid,
    };

    let response = semantic_search(pool, params, embedding)
        .await
        .map_err(|e| AppError::ExternalService(format!("Semantic search failed: {}", e)))?;

    Ok(response)
}

/// Removes MathJax wrapping `$` and `$$` so the LLM receives plain text definitions/notes.
fn strip_mathjax_for_llm(s: &str) -> String {
    // Display math: $$...$$ → inner content (non-greedy, allow newlines)
    let re_display = Regex::new(r"\$\$([\s\S]*?)\$\$").expect("display math regex");
    let s = re_display.replace_all(s, "$1");
    // Inline math: $...$ → inner content (non-empty, no unescaped $ inside)
    let re_inline = Regex::new(r"\$([^$]+)\$").expect("inline math regex");
    re_inline.replace_all(&s, "$1").trim().to_string()
}

fn summarise_definition(def: &DefinitionDetail) -> serde_json::Value {
    json!({
        "valsi": def.valsiword,
        "definition": strip_mathjax_for_llm(&def.definition),
        "notes": def.notes.as_ref().map(|s| strip_mathjax_for_llm(s)),
        "lang": def.langrealname,
        "score": def.score,
        "similarity": def.similarity,
        "selmaho": def.selmaho.as_ref().map(|s| strip_mathjax_for_llm(s)),
        "jargon": def.jargon.as_ref().map(|s| strip_mathjax_for_llm(s)),
    })
}

pub async fn handle_chat(pool: &Pool, request: ChatRequest) -> Result<String, AppError> {
    let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
        AppError::ExternalService("OPENROUTER_API_KEY is not set in the environment".into())
    })?;

    let base_url = env::var("OPENROUTER_API_BASE")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let client = reqwest::Client::new();

    let mut messages = Vec::new();
    messages.push(ChatCompletionMessageRequest {
        role: "system".to_string(),
        content: system_prompt(request.locale.as_deref()),
        tool_call_id: None,
        name: None,
        tool_calls: None,
    });

    messages.extend(map_chat_messages(&request.messages));

    let tools = vec![jbovlaste_tool_schema()];

    let first_request = ChatCompletionRequest {
        model: "openrouter/free".to_string(),
        messages: messages.clone(),
        tools: Some(tools.clone()),
        tool_choice: Some(json!("auto")),
    };

    let first_response = openrouter_chat_with_retry("first chat/completions", || {
        let client = client.clone();
        let base_url = base_url.clone();
        let api_key = api_key.clone();
        let first_request = first_request.clone();
        async move {
            let res = client
                .post(format!("{}/chat/completions", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&first_request)
                .send()
                .await?;
            let ok = ensure_openrouter_status(res, "first chat/completions").await?;
            parse_chat_response(ok, "first chat/completions").await
        }
    })
    .await?;

    let choice = first_response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| AppError::ExternalService("No choices returned from OpenRouter".into()))?;

    if let Some(tool_calls) = choice.message.tool_calls.clone() {
        // Handle only the first semantic search tool call for now
        if let Some(first_call) = tool_calls.first() {
            if first_call.function.name == "jbovlaste_semantic_search" {
                let args_json: &str = match first_call.function.arguments.as_deref() {
                    None | Some("") => "{}",
                    Some(s) => s,
                };
                let args: ToolArgs = serde_json::from_str(args_json).map_err(|e| {
                    log::warn!(
                        "Tool call arguments JSON parse error: {}; raw arguments: {:?}",
                        e,
                        first_call.function.arguments
                    );
                    AppError::ExternalServiceWithRaw {
                        message: "Assistant produced invalid tool arguments; please try again.".into(),
                        raw_response: args_json.to_string(),
                    }
                })?;

                let results = run_jbovlaste_semantic_search(pool, args).await?;

                let compact_results: Vec<serde_json::Value> =
                    results.definitions.iter().map(summarise_definition).collect();

                let tool_payload = json!({
                    "results": compact_results,
                    "total": results.total,
                });

                let tool_message = ChatCompletionMessageRequest {
                    role: "tool".to_string(),
                    content: serde_json::to_string(&tool_payload)?,
                    tool_call_id: Some(first_call.id.clone()),
                    name: Some(first_call.function.name.clone()),
                    tool_calls: None,
                };

                let mut second_messages = messages;
                second_messages.push(ChatCompletionMessageRequest {
                    role: choice.message.role.clone(),
                    content: choice.message.content.clone(),
                    tool_call_id: None,
                    name: None,
                    tool_calls: Some(tool_calls.clone()),
                });
                second_messages.push(tool_message);

                let second_request = ChatCompletionRequest {
                    model: "openrouter/free".to_string(),
                    messages: second_messages.clone(),
                    tools: Some(tools.clone()),
                    tool_choice: Some(json!("none")),
                };

                let second_response = openrouter_chat_with_retry("second chat/completions", || {
                    let client = client.clone();
                    let base_url = base_url.clone();
                    let api_key = api_key.clone();
                    let second_request = second_request.clone();
                    async move {
                        let res = client
                            .post(format!("{}/chat/completions", base_url))
                            .header("Authorization", format!("Bearer {}", api_key))
                            .header("Content-Type", "application/json")
                            .json(&second_request)
                            .send()
                            .await?;
                        let ok =
                            ensure_openrouter_status(res, "second chat/completions").await?;
                        parse_chat_response(ok, "second chat/completions").await
                    }
                })
                .await?;

                let final_choice = second_response
                    .choices
                    .into_iter()
                    .next()
                    .ok_or_else(|| {
                        AppError::ExternalService("No choices in second OpenRouter response".into())
                    })?;

                return Ok(final_choice.message.content);
            }
        }
    }

    // No tool calls – just return the first message content
    Ok(choice.message.content)
}
