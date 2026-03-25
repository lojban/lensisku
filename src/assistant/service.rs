#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_web_lab::sse;
use futures::future::join_all;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tokio::time::sleep;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::utils::embeddings::get_embedding;
use crate::error::AppError;
use crate::jbovlaste::models::{DefinitionDetail, DefinitionResponse, SearchDefinitionsParams};
use crate::middleware::cache::RedisCache;
use crate::utils::openrouter_models::{
    fetch_latest_openrouter_models, load_or_fetch_openrouter_candidates, ModelIdName,
};
use crate::jbovlaste::service::semantic_search;
use std::borrow::Cow;

use super::context_compress;
use super::dto::{AssistantStep, ChatMessage, ChatRequest, ToolCallDto};
use super::persist::ChatPersistState;

/// When `true`, streaming runs two OpenRouter models in parallel when two candidates exist.
/// Temporarily set to `false` to use only one model per turn (sequential candidate fallback).
const ASSISTANT_PARALLEL_DUAL_MODEL: bool = false;

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
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::ExternalService(format!("Database pool error: {}", e)))?;
    let mut ids = Vec::with_capacity(norm.len());
    for tag in &norm {
        let row = client
            .query_opt(
                "SELECT langid FROM languages WHERE lower(tag) = lower($1)",
                &[tag],
            )
            .await
            .map_err(|e| AppError::ExternalService(format!("language tag lookup failed: {}", e)))?;
        match row {
            Some(r) => ids.push(r.get::<_, i32>(0)),
            None => {
                return Err(AppError::BadRequest(format!(
                    "Unknown language tag `{}`. Use jbovlaste tags such as en, ru, es, jbo.",
                    tag
                )));
            }
        }
    }
    Ok(Some(ids))
}

/// Resolves optional `source_language` tag to `langid` (valsi source language). `None` = caller default.
async fn resolve_optional_source_language_tag(
    pool: &Pool,
    tag: &Option<String>,
) -> Result<Option<i32>, AppError> {
    let Some(t) = tag else {
        return Ok(None);
    };
    let s = t.trim();
    if s.is_empty() {
        return Ok(None);
    }
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::ExternalService(format!("Database pool error: {}", e)))?;
    let row = client
        .query_opt(
            "SELECT langid FROM languages WHERE lower(tag) = lower($1)",
            &[&s],
        )
        .await
        .map_err(|e| AppError::ExternalService(format!("source_language lookup failed: {}", e)))?;
    match row {
        Some(r) => Ok(Some(r.get::<_, i32>(0))),
        None => Err(AppError::BadRequest(format!(
            "Unknown source_language tag `{}`. Example: jbo for Lojban head words.",
            s
        ))),
    }
}

fn system_prompt_base(locale: Option<&str>) -> String {
    let mut base = String::with_capacity(4096);
    let list_hint = "**core reference dictionary** (gismu, cmavo, learn-lojban, phrases) below";
    let trusted_block_1 = "1. The **\"Core reference dictionary\"** block in this system message (if present): \
         curated **gismu**, **cmavo**, **learn-lojban** notions/examples, and **English / Lojban phrase pairs**.\n\
         ";

    // ── Role & personality ───────────────────────────────────────────────
    base.push_str(
        "You are a friendly Lojban dictionary assistant powered by **jbovlaste** \
         (the community Lojban dictionary). You help users look up words, understand \
         definitions, explore morphology, and learn Lojban. Be welcoming to beginners \
         and precise for advanced learners. Encourage follow-up questions.\n\n",
    );

    // ── Objectives (multi-step reasoning model) ──────────────────────────
    base.push_str(
        "## Objectives\n\
         For each user message, follow this decision process:\n\
         1. **Decide & extract terms**: Does answering require new jbovlaste evidence \
         (unknown valsi, new topic, or no prior search for this material)? If yes, \
         **rewrite the user question into search terms** that match how dictionary \
         glosses are written (short, concrete; see *Query relevance* below), then call \
         `jbovlaste_semantic_search` with those terms—not with the user’s full sentence.\n\
         2. **Refine**: After reviewing results, if similarity is low or hits are off-topic, \
         call the tool again with a **different** query (synonym, shorter string, alternate \
         valsi spelling, or split into two searches). Do not repeat the same query.\n\
         3. **Answer**: Once you have enough grounded evidence, write your final response \
         **without calling any tool**.\n\n\
         If the user is asking a follow-up or clarification about a word you **already \
         searched** in this conversation, skip straight to step 3—do not search again.\n\n",
    );

    // ── Tool use (parallel calls) — adapted from Roo-Code tool-use guidelines ──
    base.push_str(&format!(
        "## Tool use\n\
         The chat API allows **multiple tool calls in a single assistant message** when \
         that saves turns (e.g. several independent lookups). Prefer native tool-calling; \
         the host runs those calls **in parallel** when they do not depend on each other.\n\
         - Decide what evidence you still need vs. what is already in this thread or the \
         {}.\n\
         - If several **independent** searches would help (different valsi or unrelated \
         concepts), issue them together in one step instead of one-by-one.\n\
         - **Translations and multi-word phrases:** First list every **valsi or gloss** you \
         still need from jbovlaste (each distinct word or meaning). Then call \
         `jbovlaste_semantic_search` **once per distinct lookup**, **all in the same assistant \
         message** (parallel tool calls) so the host returns every result together—then you \
         can build the full translation in the next turn without extra search round trips.\n\
         - If a later search should use wording informed by an earlier hit, run searches \
         **across separate turns** so each step is grounded in prior tool results. Do not \
         assume a search outcome before you have seen it.\n\n",
        list_hint
    ));

    // ── Trusted sources (hierarchy) ──────────────────────────────────────
    base.push_str(&format!(
        "## Trusted sources\n\
         Factual claims about Lojban (words, glosses, grammar, morphology, usage, examples) \
         may come **only** from these sources, in order of authority:\n\
         {}\
         2. `jbovlaste_semantic_search` tool results **in this thread**.\n\
         3. The user's messages and your earlier replies, only insofar as they quote or \
         clearly restate (1) or (2).\n\n\
         Treat your pretrained knowledge of Lojban as **untrusted**. \
         Do **not** state it as fact.\n\n",
        trusted_block_1
    ));

    // ── Guardrails ───────────────────────────────────────────────────────
    base.push_str(
        "## Guardrails\n\
         - **Mandatory uncertainty:** Any Lojban-related statement not directly supported \
         by the sources above must be prefixed with an explicit disclaimer such as \
         \"**Uncertain (not verified from the database):**\" or omitted entirely. \
         Prefer saying you cannot answer from sources over filling gaps from memory.\n\
         - **No hallucinated Lojban:** Never invent valsi, glosses, definitions, or place \
         structures. If you need a word that is not in the injected reference lists and not \
         in prior tool results, call `jbovlaste_semantic_search` before using it.\n\
         - **Examples:** Prefer quoting or lightly adapting Lojban from the system message \
         **reference block** (phrase pairs and/or bundled word lists) and from \
         `jbovlaste_semantic_search` results in this thread. When composing short examples, \
         use only **valsi** attested there; mirror common patterns (bridi structure, `.i`, \
         sumti, abstractions, connectives). If you are unsure, search first or state uncertainty.\n\n",
    );

    // ── Lojban domain context ────────────────────────────────────────────
    base.push_str(
        "## Lojban background (for interpreting tool output)\n\
         Word types you will encounter:\n\
         - **gismu** – root words (5 letters: CVCCV or CCVCV), each with a fixed place \
         structure using `$x_1$`, `$x_2$`, ... placeholders in definitions.\n\
         - **cmavo** – structure words (short, grammatical particles); grouped by \
         **selma'o** (grammatical class, shown as `selmaho` in results).\n\
         - **lujvo** – compound words built from gismu/cmavo via **rafsi** (affixes); \
         tool results include a `decomposition` field listing the source words.\n\
         - **fu'ivla** – loan words adapted into Lojban phonology.\n\
         - **cmevla** – proper names (always end in a consonant).\n\n\
         Other useful fields in tool output: `notes` (usage notes, cross-references), \
         `etymology`, `examples` (community-contributed example sentences), \
         `jargon` (domain tag), `rafsi` (assigned affix forms for compounding), \
         `relevance` (cosine similarity to your query).\n\n",
    );

    // ── Injected reference list (bundled `core_reference_dictionary.txt`) ─
    base.push_str(
        "## Core reference dictionary (single bundled source)\n\
         One block titled \"Core reference dictionary\" ships with the server: \
         **150 high-score gismu**, **60 cmavo** (PA1 digits 0–9 plus 50 more by score), \
         **learn-lojban** notions plus **book examples** (when the Markdown source is available \
         at build time), and **168** English/Lojban phrase lines: mostly grammar-diverse, plus a \
         **math- / measure- / logic-leaning** batch; each line `left ↔ right` within its subsection. \
         Treat it as your **primary** offline vocabulary and phrasing guide. For **any other valsi** \
         (remaining gismu/cmavo, lujvo, fu'ivla, full definitions, examples, notes), call \
         `jbovlaste_semantic_search`.\n\n",
    );

    // ── When to call the tool ────────────────────────────────────────────
    base.push_str(
        "## When to call `jbovlaste_semantic_search`\n\
         **Call** when the user asks about words, concepts, or meanings that require \
         **new** jbovlaste evidence: first question in a thread, a different valsi or \
         topic than already covered, or when nothing in the prior conversation gives \
         grounded definitions for what they asked.\n\n\
         **Do NOT call** when the user's message is a clarification, follow-up, or \
         rephrase about the **same** word or concept you already answered with \
         search-backed content. Reuse prior assistant turns for \"explain simpler\", \
         \"give an example from what you already quoted\", or short confirmations.\n\n\
         **Multi-step search:** After receiving tool results, if they are insufficient, \
         off-topic, or you need related terms, call the tool again in a later step with \
         a new or refined query. Do not give a final answer until search results \
         actually support it. You MAY issue several parallel calls in a single step \
         (e.g. multiple concepts) and search again in later steps when refining.\n\
         For **translation** tasks especially: prepare **all** needed word/meaning queries up \
         front and fire them **in parallel** in one step so you get the full evidence set \
         at once.\n\n\
         If the search returns no or few results, try different queries in further \
         steps, or say so and suggest rephrasing; do not make up answers.\n\n\
         **Finish rule:** Once the definitions you have retrieved are sufficient to \
         answer the question, reply directly in your next turn **without calling any \
         tool**. Calling the tool again after you already have the relevant results is \
         wasteful and confusing for the user.\n\n",
    );

    // ── Query relevance (critical) ───────────────────────────────────────
    base.push_str(
        "## Query relevance (critical)\n\
         The `query` is embedded and matched against **definition text** in jbovlaste. \
         Phrasing that matches gloss style yields better hits than conversational English.\n\
         - **Valsi-first:** If the user names or you can infer a Lojban word, search **only** \
         that valsi (`lorxu`, `.u`, `klama`). Do not add English around it.\n\
         - **Strip chat noise:** Remove question words (\"what\", \"how\", \"is there\"), \
         politeness, and phrases like \"in Lojban\", \"the word for\", \"tell me\" before \
         searching. Keep the **topic** (e.g. user asks \"what’s fox\" → query `fox`).\n\
         - **Concept queries:** Use **2–6 content words** max—nouns, verbs, adjectives, \
         standard grammar terms used in jbovlaste (`sumti`, `selbri`, `tanru`, \
         `logical connective`, `attitudinal`). Avoid long clauses and lists of unrelated \
         ideas; if the user asks two things, use **two** tool calls or two sequential steps.\n\
         - **Weak results:** If top hits look irrelevant or `similarity` scores are poor, \
         retry with: a shorter query; a synonym; the same concept in Lojban if you know a \
         related valsi from results; or a different facet (e.g. `animal` vs `mammal`).\n\
         - **Never** add meta-words like \"Lojban\", \"definition\", \"dictionary\", \
         \"jbovlaste\", \"meaning\", or \"word\"—they dilute embeddings.\n\
         - **limit:** Use ~8–12 for a specific valsi or tight concept; raise toward **30** \
         when exploring a broad English concept or when the first page lacks a good match.\n\
         - **languages:** Optional list of **language tags** (not numeric IDs), e.g. `en` for \
         English glosses, `ru` for Russian, `jbo` if you need definitions written in Lojban. \
         Omit to search across all natural-language definition rows the backend indexes. \
         Prefer `en` when the user reads English.\n\n",
    );

    // ── Response formatting ──────────────────────────────────────────────
    base.push_str(
        "## Response formatting\n\
         - Use valid, simple Markdown: **bold** for valsi and key terms, bullet lists \
         for definitions. **Never use markdown pipe tables** (lines starting with `|` and \
         `|---|` separators) or ASCII grid tables — they render poorly in chat; use bullet \
         lists, numbered lines, or plain paragraphs instead.\n\
         - When quoting definitions from jbovlaste, preserve inline `$...$` math \
         delimiters exactly as they appear in the tool output.\n\
         - When a definition has place structure (`$x_1$`, `$x_2$`, ...), present it \
         clearly so the user understands each argument slot.\n\
         - Mention the selma'o for cmavo, and rafsi for gismu/cmavo when available, \
         as these are useful for learners.\n\
         - If results include `notes`, `examples`, or `decomposition`, surface the \
         relevant parts to give a complete answer.\n\n",
    );

    // ── Fallback tool-call format ────────────────────────────────────────
    base.push_str(
        "## Fallback tool-call format\n\
         Prefer your platform's native tool-calling. If you cannot use it, emit \
         exactly: CALL>[{\"name\":\"jbovlaste_semantic_search\",\"arguments\":\
         {\"query\":\"lorxu\"}}]</TOOLCALL> or \
         {\"query\":\"logical connective\",\"limit\":16,\"languages\":[\"en\"]} — short \
         `query`; use `languages` tags like en, ru, jbo.",
    );

    // ── Locale preference ────────────────────────────────────────────────
    if let Some(loc) = locale {
        if !loc.is_empty() {
            base.push_str(&format!(
                "\n\nPrefer to explain things in locale `{}` where appropriate.",
                loc
            ));
        }
    }

    base
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

/// Bundled assistant dictionary (`archive/dict` snapshot; rebuild with `scripts/build_assistant_core_dictionary.py`).
fn assistant_bundled_core_dictionary_cow() -> Cow<'static, str> {
    const EMBEDDED: &str = include_str!("core_reference_dictionary.txt");
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
        t.push_str("\n\n[Dictionary truncated for context size; remaining entries omitted.]");
        Cow::Owned(t)
    }
}

async fn system_prompt_with_dictionary(_pool: &Pool, locale: Option<&str>) -> String {
    let base = system_prompt_base(locale);
    let dict = assistant_bundled_core_dictionary_cow();
    let dict_str = dict.as_ref();
    if dict_str.trim().is_empty() {
        return base;
    }
    format!(
        "{}\n\n## Core reference dictionary\n\
All data lines use the same column separator: spaced **↔** (U+2194): `left ↔ right`. \
Sections **gismu** and **cmavo**: `valsi ↔ English definition`. **learn-lojban tutorial**: \
English↔English notions and English↔Lojban examples from the lojban.pw course. **Phrases** (two \
subsections): `English ↔ Lojban` — corpus samples (general, then math- / logic-leaning). \
This bundled list is a **subset** of jbovlaste and full phrase corpora; \
use `jbovlaste_semantic_search` for any valsi or nuance not present here.\n\n{}",
        base, dict_str
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
            description: "Semantic search over jbovlaste definition text (embedding \
                          similarity). Your ONLY tool—call before stating facts about \
                          Lojban words or meanings, unless the system message **core reference dictionary** \
                          (bundled gismu, cmavo, learn-lojban, phrases) already answers. \
                          Emit **several parallel** jbovlaste_semantic_search calls in **one** \
                          message when lookups are independent (different valsi, glosses, or \
                          topics). For **translations**, list every word or meaning you still need \
                          first, then batch **all** those queries together so every result returns \
                          at once before you compose the full Lojban. \
                          Call when: new valsi/topic, or prior results were insufficient—\
                          then use a **different** `query` (no duplicate searches). \
                          Do NOT call when this thread already has search-backed \
                          definitions for the same question—answer from those. \
                          **`query`**: not a chat message. Prefer bare valsi (`lorxu`, \
                          `i`, `klama`). Else 2–6 gloss-style keywords (English or Lojban): \
                          e.g. `fox`, `logical connective`, `past tense`, `emotion`. \
                          Bad: \"what is the Lojban word for fox\", \"definition of klama\", \
                          \"jbovlaste search lorxu\". Strip questions and meta-words \
                          (\"Lojban\", \"definition\", \"dictionary\", \"meaning\", \
                          \"word\", \"jbovlaste\"). \
                          **`languages`**: optional BCP-47-style tags from jbovlaste \
                          (`languages.tag`), e.g. `en` (English glosses), `ru`, `es`, `jbo`. \
                          Omit to search across indexed natural-language definitions; use \
                          `[\"en\"]` when the user wants English glosses only."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Embedding input: bare valsi OR short keyword list \
                            matching dictionary gloss style. Strip question framing. \
                            Examples: `klama`, `fox`, `sumti`, `logical connective`. \
                            Avoid sentences and filler; synonyms are OK if first search \
                            was weak."
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 30,
                        "default": 12,
                        "description": "How many top matches to return. Use ~8–12 for a \
                            known valsi or narrow term; increase toward 30 for broad \
                            English concepts or when refining after weak results."
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
                "required": ["query"]
            }),
        },
    }
}

#[derive(Debug, Deserialize, Clone)]
struct ToolArgs {
    query: String,
    #[serde(default)]
    limit: Option<u32>,
    /// jbovlaste `languages.tag` values (e.g. en, ru, jbo), not numeric langids.
    #[serde(default)]
    languages: Option<Vec<String>>,
    #[serde(default)]
    source_language: Option<String>,
}

/// One assistant turn may include several tool calls; we prepare slots then run searches concurrently.
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
        args: ToolArgs,
        search_query: String,
        assistant_reasoning: Option<String>,
        global_step_index: usize,
        action_desc: String,
    },
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
    let query = args.query.trim().to_string();
    if query.is_empty() {
        return Err(AppError::BadRequest(
            "jbovlaste_semantic_search: query is empty after trimming".into(),
        ));
    }

    let limit = args
        .limit
        .unwrap_or(12)
        .clamp(1, SEMANTIC_SEARCH_MAX_LIMIT) as i64;

    let languages_langids = match args.languages.as_deref() {
        None | Some([]) => None,
        Some(tags) => resolve_jbovlaste_language_tags_to_langids(pool, tags).await?,
    };

    let source_langid = resolve_optional_source_language_tag(pool, &args.source_language).await?;

    let embedding = get_embedding(&query).await?;

    let params = SearchDefinitionsParams {
        page: 1,
        per_page: limit,
        search_term: query.clone(),
        include_comments: false,
        sort_by: "score".to_string(),
        sort_order: "desc".to_string(),
        languages: languages_langids,
        selmaho: None,
        username: None,
        word_type: None,
        source_langid,
        search_in_phrases: None,
        include_total_count: false,
    };

    let response = semantic_search(pool, params, embedding, None)
        .await
        .map_err(|e| {
            AppError::ExternalService(format!(
                "Semantic search failed for query \"{}\": {}",
                query, e
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
            run_agent_loop_inner(&pool1, &req1, &m1_id, &m1_name, Some(tx1), p1),
            run_agent_loop_inner(&pool2, &req2, &m2_id, &m2_name, Some(tx2), p2),
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
                let res = run_agent_loop_inner(
                    &pool1,
                    &req1,
                    &nid,
                    &nname,
                    Some(post_debug_tx.clone()),
                    persist.clone(),
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
                let res = run_agent_loop_inner(
                    &pool2,
                    &req2,
                    &nid,
                    &nname,
                    Some(post_debug_tx.clone()),
                    persist.clone(),
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
            match run_agent_loop_inner(pool, request, model_id, model_name, tx, persist.clone())
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

async fn run_agent_loop_inner(
    pool: &Pool,
    request: &ChatRequest,
    model: &str,
    model_name: &str,
    event_tx: Option<mpsc::Sender<sse::Event>>,
    persist: Option<Arc<ChatPersistState>>,
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

    // Agent loop: call LLM until it returns a final reply (no tool_calls).
    for iteration in 1..=AGENT_MAX_ITERATIONS {
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

                let query_trimmed = args.query.trim().to_string();
                if query_trimmed.is_empty() {
                    let err_msg =
                        "jbovlaste_semantic_search: query is empty after trimming".to_string();
                    prepared.push(PreparedToolSlot::Immediate {
                        tool_call_id: call.id.clone(),
                        name: call.function.name.clone(),
                        content: format!("Tool error: {}", err_msg),
                    });
                    continue;
                }

                let mut args = args;
                args.query = query_trimmed.clone();
                let search_query_for_llm = query_trimmed;

                let seen = query_seen_count
                    .entry(search_query_for_llm.clone())
                    .or_insert(0);
                *seen += 1;
                if *seen > MAX_QUERY_REPETITIONS {
                    log::warn!(
                        "Assistant: query '{}' repeated {} times; injecting loop-break tool result",
                        search_query_for_llm,
                        seen
                    );
                    prepared.push(PreparedToolSlot::Immediate {
                        tool_call_id: call.id.clone(),
                        name: call.function.name.clone(),
                        content: format!(
                            "You have already searched for \"{}\" {} time(s). \
                             The results are already in this conversation. \
                             Stop searching and formulate your answer now.",
                            search_query_for_llm,
                            *seen - 1
                        ),
                    });
                    continue;
                }

                pending_search_ordinal += 1;
                let action_desc = format!("Semantic search: \"{}\"", search_query_for_llm);
                prepared.push(PreparedToolSlot::Search {
                    tool_call_id: call.id.clone(),
                    name: call.function.name.clone(),
                    args,
                    search_query: search_query_for_llm,
                    assistant_reasoning,
                    global_step_index,
                    action_desc,
                });
            }

            let mut search_jobs: Vec<(usize, ToolArgs)> = Vec::new();
            for (slot_i, slot) in prepared.iter().enumerate() {
                if let PreparedToolSlot::Search { args, .. } = slot {
                    search_jobs.push((slot_i, args.clone()));
                }
            }

            let mut results_by_slot: HashMap<usize, Result<DefinitionResponse, AppError>> =
                HashMap::new();

            if !search_jobs.is_empty() {
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
                let outcomes = join_all(search_jobs.iter().map(|(_, args)| {
                    let pool = pool_clone.clone();
                    let args = args.clone();
                    async move { run_jbovlaste_semantic_search_with_retry(&pool, args).await }
                }))
                .await;

                for ((slot_i, _), outcome) in search_jobs.iter().zip(outcomes) {
                    results_by_slot.insert(*slot_i, outcome);
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
                        search_query,
                        assistant_reasoning,
                        global_step_index,
                        action_desc,
                        ..
                    } => {
                        let tool_result = results_by_slot
                            .remove(&slot_i)
                            .expect("search slot must have a result");

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
                                log::warn!(
                                    "Assistant semantic search failed after retries: {}",
                                    err_str
                                );
                                let summary = format!("Error after retries: {}", err_str);
                                let payload = json!({
                                    "error": err_str,
                                    "results": [],
                                });
                                (summary, payload)
                            }
                        };

                        let tool_content_json =
                            serde_json::to_string(&tool_payload_value).unwrap_or_else(|_| {
                                tool_payload_value["error"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string()
                            });

                        let tool_content_for_llm = match &tool_result {
                            Ok(results) => semantic_tool_results_plain_text_for_llm(
                                &search_query,
                                &results.definitions,
                            ),
                            Err(e) => format!("Semantic search error: {}", e),
                        };

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
            if reply.trim().is_empty() && iteration < AGENT_MAX_ITERATIONS && !steps.is_empty() {
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
