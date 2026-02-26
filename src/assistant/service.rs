use std::env;

use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize, Clone)]
struct ChatCompletionMessageResponse {
    role: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct ToolCallFunction {
    name: String,
    // OpenRouter/OpenAI send arguments as a JSON string
    arguments: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct ToolCall {
    id: String,
    r#type: String,
    function: ToolCallFunction,
}

fn system_prompt(locale: Option<&str>) -> String {
    let mut base = String::from(
        "You are a Lojban dictionary assistant.\n\
         The user describes concepts (often in English), and you help find or compose good Lojban expressions.\n\
         You have access to a tool named `jbovlaste_semantic_search` that searches jbovlaste definitions semantically.\n\
         Use that tool whenever it would help to find relevant existing valsi or glosses, then explain suggestions clearly.",
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
            description: "Semantic search over jbovlaste definitions stored in the database. Use this to find Lojban words or combinations related to a concept."
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

fn summarise_definition(def: &DefinitionDetail) -> serde_json::Value {
    json!({
        "valsi": def.valsiword,
        "definition": def.definition,
        "lang": def.langrealname,
        "score": def.score,
        "similarity": def.similarity,
        "selmaho": def.selmaho,
        "jargon": def.jargon,
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

    let first_response: ChatCompletionResponse = client
        .post(format!("{}/chat/completions", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&first_request)
        .send()
        .await?
        .error_for_status()?
        .json()
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
                let args: ToolArgs = serde_json::from_str(&first_call.function.arguments)?;

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
                    tool_calls: Some(tool_calls),
                });
                second_messages.push(tool_message);

                let second_request = ChatCompletionRequest {
                    model: "openrouter/free".to_string(),
                    messages: second_messages,
                    tools: Some(tools),
                    tool_choice: Some(json!("none")),
                };

                let second_response: ChatCompletionResponse = client
                    .post(format!("{}/chat/completions", base_url))
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&second_request)
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
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
