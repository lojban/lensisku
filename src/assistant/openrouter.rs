//! OpenRouter chat/completions client and wire-format types.

use std::env;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::error::AppError;
use crate::middleware::cache::RedisCache;
use crate::utils::openrouter_models::{load_or_fetch_openrouter_candidates, ModelIdName};

use super::dto::{ChatMessage, ToolCallDto};

const MAX_ATTEMPTS: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 500;

#[derive(Debug, Clone, Serialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub r#type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionMessageRequest {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatCompletionMessageRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatCompletionChoice>,
}

impl ChatCompletionResponse {
    pub fn first_message_content(self) -> Option<String> {
        self.choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .filter(|content| !content.trim().is_empty())
    }
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChoice {
    pub message: ChatCompletionMessageResponse,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatCompletionMessageResponse {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ToolCallFunction {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ToolCall {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    pub function: ToolCallFunction,
}

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

#[derive(Clone)]
pub struct OpenRouterClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl OpenRouterClient {
    pub fn from_env() -> Result<Self, AppError> {
        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| {
            AppError::ExternalService("OPENROUTER_API_KEY is not set in the environment".into())
        })?;
        let base_url = env::var("OPENROUTER_API_BASE")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
            base_url,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub async fn chat_completion(
        &self,
        request: &ChatCompletionRequest,
        label: &str,
    ) -> Result<ChatCompletionResponse, AppError> {
        let res = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("OpenRouter {} request failed: {}", label, e)))?;
        let ok = ensure_status(res, label).await?;
        parse_response(ok, label).await
    }

    pub async fn chat_completion_with_retry(
        &self,
        request: &ChatCompletionRequest,
        label: &str,
    ) -> Result<ChatCompletionResponse, AppError> {
        chat_with_retry(label, || self.chat_completion(request, label)).await
    }
}

pub async fn load_primary_model(
    redis: Option<&RedisCache>,
    client: &OpenRouterClient,
) -> Result<Option<ModelIdName>, AppError> {
    let (candidates, _) =
        load_or_fetch_openrouter_candidates(redis, client.base_url(), &client.api_key).await?;
    Ok(candidates.into_iter().next())
}

/// One-shot system+user completion using the primary cached OpenRouter model.
pub async fn text_completion(
    redis: Option<&RedisCache>,
    system_prompt: &str,
    user_prompt: &str,
    label: &str,
    timeout_secs: u64,
) -> Option<String> {
    let client = OpenRouterClient::from_env().ok()?;
    let candidate = load_primary_model(redis, &client).await.ok()??;
    text_completion_with_model(
        &client,
        &candidate.0,
        system_prompt,
        user_prompt,
        label,
        timeout_secs,
    )
    .await
    .ok()
    .flatten()
}

pub async fn text_completion_with_model(
    client: &OpenRouterClient,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    label: &str,
    timeout_secs: u64,
) -> Result<Option<String>, AppError> {
    let request = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![
            ChatCompletionMessageRequest {
                role: "system".to_string(),
                content: system_prompt.to_string(),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
            ChatCompletionMessageRequest {
                role: "user".to_string(),
                content: user_prompt.to_string(),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
        ],
        tools: None,
        tool_choice: None,
        parallel_tool_calls: None,
    };

    let timeout = Duration::from_secs(timeout_secs);
    let response = match tokio::time::timeout(timeout, client.chat_completion_with_retry(&request, label)).await
    {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            log::warn!("OpenRouter {label} call failed: {e}");
            return Ok(None);
        }
        Err(_) => {
            log::warn!("OpenRouter {label} timed out after {timeout_secs}s");
            return Ok(None);
        }
    };

    Ok(response.first_message_content())
}

pub fn map_chat_messages(messages: &[ChatMessage]) -> Vec<ChatCompletionMessageRequest> {
    messages
        .iter()
        .map(|m| {
            let role = match m.role.as_str() {
                "user" | "assistant" | "system" | "tool" => m.role.clone(),
                other => {
                    log::warn!("Unknown chat role `{other}`, mapping to `user`");
                    "user".to_string()
                }
            };
            let tool_calls = m
                .tool_calls
                .as_ref()
                .map(|tc| tc.iter().map(tool_call_dto_to_internal).collect::<Vec<ToolCall>>());
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

pub fn tool_call_dto_to_internal(c: &ToolCallDto) -> ToolCall {
    ToolCall {
        id: c.id.clone(),
        r#type: c.r#type.clone(),
        function: ToolCallFunction {
            name: c.function.name.clone(),
            arguments: c.function.arguments.clone(),
        },
    }
}

async fn ensure_status(res: reqwest::Response, label: &str) -> Result<reqwest::Response, AppError> {
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

async fn parse_response(
    res: reqwest::Response,
    label: &str,
) -> Result<ChatCompletionResponse, AppError> {
    let status = res.status();
    let body = res.text().await.map_err(|e| {
        AppError::ExternalService(format!("Failed to read {label} response body: {e}"))
    })?;
    let body_trimmed = body.trim();
    match serde_json::from_str::<ChatCompletionResponse>(body_trimmed) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            let retryable = if let Ok(err_payload) =
                serde_json::from_str::<OpenRouterErrorPayload>(body_trimmed)
            {
                let code = err_payload.error.code;
                let msg = if err_payload.error.message.is_empty() {
                    format!("Invalid {label} response: {e}")
                } else {
                    format!("{label}: {}", err_payload.error.message)
                };
                let is_server_error = code.map(|c| c >= 500).unwrap_or(true);
                if is_server_error {
                    log::warn!(
                        "OpenRouter {label} returned error body (code {code:?}), will retry: {msg}"
                    );
                    Some((msg, body.clone()))
                } else {
                    None
                }
            } else {
                Some((format!("Invalid {label} response: {e}"), body.clone()))
            };
            if let Some((message, raw_response)) = retryable {
                log::debug!(
                    "OpenRouter {label} response (status {status}): {raw_response}"
                );
                return Err(AppError::ExternalServiceRetryable {
                    message,
                    raw_response,
                });
            }
            log::debug!("OpenRouter {label} response (status {status}): {body}");
            log::warn!("OpenRouter {label} parse error: {e} (see debug log for raw body)");
            Err(AppError::ExternalServiceWithRaw {
                message: format!("Invalid {label} response: {e}"),
                raw_response: body,
            })
        }
    }
}

async fn chat_with_retry<F, Fut>(
    label: &str,
    mut run: F,
) -> Result<ChatCompletionResponse, AppError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<ChatCompletionResponse, AppError>>,
{
    let mut last_err = None;
    for attempt in 1..=MAX_ATTEMPTS {
        match run().await {
            Ok(r) => return Ok(r),
            Err(e) => {
                if let AppError::ExternalServiceRetryable { .. } = &e {
                    last_err = Some(e);
                    if attempt < MAX_ATTEMPTS {
                        let delay = Duration::from_millis(INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1));
                        log::info!(
                            "OpenRouter {label} retry {attempt}/{MAX_ATTEMPTS} after {delay:?}"
                        );
                        sleep(delay).await;
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }
    match last_err {
        Some(e) => Err(e),
        None => Err(AppError::Internal(format!(
            "OpenRouter {label} retry loop exhausted without success"
        ))),
    }
}
