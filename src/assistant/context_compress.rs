//! Compress client-provided chat history when approaching the model context limit.
//!
//! Threshold logic matches [Roo-Code `manageContext`](../../Roo-Code/src/core/context-management/index.ts):
//! `allowed_tokens = context_window * (1 - TOKEN_BUFFER_PERCENTAGE) - reserved_output_tokens`,
//! then subtract the **system prompt** footprint so the **history** alone is budgeted.
//! No compression runs while estimated history fits in that allowance.

use std::collections::HashSet;

use super::dto::ChatMessage;

/// Same as Roo-Code `TOKEN_BUFFER_PERCENTAGE`: leave headroom before hitting the hard context limit.
pub const TOKEN_BUFFER_PERCENTAGE: f64 = 0.1;

const TRUNCATE_MARKER: &str = "\n\n[Truncated for context size.]";
const OMIT_TURN: &str = "[Earlier discussion omitted.]";

/// Number of most recent messages to always keep when compressing history.
/// The first user message is also preserved. Increase if you want more recent
/// tool results/turns kept intact; decrease if you need more aggressive savings.
const DEFAULT_PRESERVE_RECENT_TURNS: usize = 3;

fn preserve_recent_turns() -> usize {
    std::env::var("ASSISTANT_PRESERVE_RECENT_TURNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PRESERVE_RECENT_TURNS)
        .max(1)
}

/// Budget for deciding whether to compress client history (Roo-style).
#[derive(Debug, Clone, Copy)]
pub struct ContextBudget {
    /// Model context window in tokens (e.g. from provider docs or `ASSISTANT_CONTEXT_WINDOW_TOKENS`).
    pub context_window_tokens: u32,
    /// Tokens reserved for the model completion (`ASSISTANT_MAX_OUTPUT_TOKENS`, default 8192 like Roo’s Anthropic default).
    pub max_output_tokens: u32,
    /// Byte length of the assembled system prompt (dictionary + instructions) for this request.
    pub system_prompt_byte_len: usize,
    /// Bytes per token heuristic for mapping token budgets ↔ `compress_chat_history` char budgets (default 4).
    pub bytes_per_token_estimate: u32,
    /// Headroom before the hard context limit (`ASSISTANT_TOKEN_BUFFER_PERCENTAGE`, default [`TOKEN_BUFFER_PERCENTAGE`]).
    pub token_buffer_percentage: f64,
}

impl ContextBudget {
    /// Build from environment and the real system prompt size (must match what is sent to the API).
    pub fn from_env_and_system_prompt(system_prompt_byte_len: usize) -> Self {
        let context_window_tokens = std::env::var("ASSISTANT_CONTEXT_WINDOW_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(128_000);

        let max_output_tokens = std::env::var("ASSISTANT_MAX_OUTPUT_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8192_u32);

        let bytes_per_token_estimate = std::env::var("ASSISTANT_BYTES_PER_TOKEN_ESTIMATE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4_u32)
            .max(1);

        let token_buffer_percentage = std::env::var("ASSISTANT_TOKEN_BUFFER_PERCENTAGE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(TOKEN_BUFFER_PERCENTAGE)
            .clamp(0.01, 0.5);

        Self {
            context_window_tokens,
            max_output_tokens,
            system_prompt_byte_len,
            bytes_per_token_estimate,
            token_buffer_percentage,
        }
    }

    #[cfg(test)]
    fn huge_for_tests(system_prompt_byte_len: usize) -> Self {
        Self {
            context_window_tokens: 1_000_000,
            max_output_tokens: 8192,
            system_prompt_byte_len,
            bytes_per_token_estimate: 4,
            token_buffer_percentage: TOKEN_BUFFER_PERCENTAGE,
        }
    }
}

/// Rough token estimate from UTF-8 byte length (provider-agnostic; same order as Roo’s API counts when unavailable).
#[inline]
pub fn estimate_tokens_from_byte_len(byte_len: usize) -> u32 {
    ((byte_len as u64).div_ceil(4)).min(u32::MAX as u64) as u32
}

/// Rough size estimate for budgeting (bytes in message strings and tool payloads, not provider tokens).
pub fn estimate_messages_bytes(messages: &[ChatMessage]) -> usize {
    messages.iter().fold(0, |acc, m| {
        acc + m.content.len()
            + m.tool_call_id.as_deref().map_or(0, str::len)
            + m.name.as_deref().map_or(0, str::len)
            + m.tool_calls.as_ref().map_or(0, |tc| {
                tc.iter().fold(0, |a, c| a + estimate_tool_call_bytes(c))
            })
    })
}

fn estimate_tool_call_bytes(c: &super::dto::ToolCallDto) -> usize {
    let mut n = 32_usize;
    if let Some(ref id) = c.id {
        n += id.len();
    }
    if let Some(ref t) = c.r#type {
        n += t.len();
    }
    n += c.function.name.as_deref().map_or(0, str::len);
    n += c.function.arguments.as_deref().map_or(0, str::len);
    n
}

/// Tokens available for **chat history** after buffer, completion reserve, and system prompt (Roo `manageContext` formula).
pub fn allowed_history_tokens(budget: &ContextBudget) -> u32 {
    let cw = budget.context_window_tokens as f64;
    let after_buffer = cw * (1.0 - budget.token_buffer_percentage);
    let reserved_out = budget.max_output_tokens as f64;
    let system_tok = estimate_tokens_from_byte_len(budget.system_prompt_byte_len) as f64;
    let raw = after_buffer - reserved_out - system_tok;
    if raw <= 0.0 {
        log::warn!(
            "Assistant: history budget is non-positive (context_window={}, system_prompt_bytes≈{}). Using floor.",
            budget.context_window_tokens,
            budget.system_prompt_byte_len
        );
        return 4096;
    }
    raw.min(u32::MAX as f64) as u32
}

/// `true` if we would compress for this budget (same predicate as the early exit in [`compress_chat_history_for_budget`]).
pub fn should_compress_history(messages: &[ChatMessage], budget: &ContextBudget) -> bool {
    if messages.is_empty() {
        return false;
    }
    let allowed = allowed_history_tokens(budget);
    let cap = effective_max_history_bytes(allowed, budget);
    estimate_messages_bytes(messages) > cap
}

fn effective_max_history_bytes(allowed_history_tokens: u32, budget: &ContextBudget) -> usize {
    let base =
        (allowed_history_tokens as usize).saturating_mul(budget.bytes_per_token_estimate as usize);
    let capped = match std::env::var("ASSISTANT_MAX_INPUT_CHARS")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(limit) => base.min(limit),
        None => base,
    };
    capped.max(4096)
}

fn default_tool_body_max_bytes() -> usize {
    std::env::var("ASSISTANT_TOOL_BODY_MAX_CHARS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12_000)
}

/// Indices that must not be dropped or truncated during compression:
/// the first user message and the last `preserve_last_n` messages.
fn anchor_indices(msgs: &[ChatMessage], preserve_last_n: usize) -> HashSet<usize> {
    let mut anchors = HashSet::new();
    if let Some(i) = msgs.iter().position(|m| m.role == "user") {
        anchors.insert(i);
    }
    let n = preserve_last_n.min(msgs.len());
    for i in (msgs.len() - n)..msgs.len() {
        anchors.insert(i);
    }
    anchors
}

/// Truncate oldest tool bodies first, then drop oldest turns (never removes anchored messages).
pub fn compress_chat_history(
    messages: &[ChatMessage],
    max_input_bytes: usize,
    preserve_last_n: usize,
) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return vec![];
    }
    let tool_cap = default_tool_body_max_bytes();
    let mut out: Vec<ChatMessage> = messages.to_vec();

    let mut guard = 0u32;
    while estimate_messages_bytes(&out) > max_input_bytes && guard < 10_000 {
        guard += 1;
        let anchors = anchor_indices(&out, preserve_last_n);
        if truncate_oldest_tool_body(&mut out, tool_cap, &anchors) {
            continue;
        }
        if drop_oldest_turn(&mut out, &anchors, preserve_last_n) {
            continue;
        }
        break;
    }
    out
}

/// Roo-aligned entry: compress only when estimated history exceeds the allowed slice of the context window.
pub fn compress_chat_history_for_budget(
    messages: &[ChatMessage],
    budget: &ContextBudget,
) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return vec![];
    }
    if !should_compress_history(messages, budget) {
        return messages.to_vec();
    }
    let allowed = allowed_history_tokens(budget);
    let max_bytes = effective_max_history_bytes(allowed, budget);
    compress_chat_history(messages, max_bytes, preserve_recent_turns())
}

/// Backwards-compatible alias: prefer [`compress_chat_history_for_budget`] with a real [`ContextBudget`].
pub fn compress_chat_history_for_request(
    messages: &[ChatMessage],
    budget: &ContextBudget,
) -> Vec<ChatMessage> {
    compress_chat_history_for_budget(messages, budget)
}

/// Aggressive compression after the provider reports a context overflow (Roo: forced reduction / sliding window).
pub fn compress_chat_history_aggressive(
    messages: &[ChatMessage],
    budget: &ContextBudget,
) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return vec![];
    }
    let allowed = allowed_history_tokens(budget);
    let half_tokens = (allowed / 2).max(8000);
    let max_bytes = (half_tokens as usize).saturating_mul(budget.bytes_per_token_estimate as usize);
    let tool_cap = (default_tool_body_max_bytes() / 2).max(2000);
    if estimate_messages_bytes(messages) <= max_bytes {
        return messages.to_vec();
    }
    let mut out: Vec<ChatMessage> = messages.to_vec();
    let mut guard = 0u32;
    while estimate_messages_bytes(&out) > max_bytes && guard < 10_000 {
        guard += 1;
        let anchors = anchor_indices(&out, preserve_recent_turns());
        if truncate_oldest_tool_body(&mut out, tool_cap, &anchors) {
            continue;
        }
        if drop_oldest_turn(&mut out, &anchors, preserve_recent_turns()) {
            continue;
        }
        break;
    }
    out
}

fn truncate_oldest_tool_body(
    msgs: &mut [ChatMessage],
    max_body: usize,
    anchors: &HashSet<usize>,
) -> bool {
    let idx = msgs.iter().enumerate().position(|(i, m)| {
        m.role == "tool"
            && m.name.as_deref() == Some("jbovlaste_semantic_search")
            && m.content.len() > max_body
            && !anchors.contains(&i)
    });
    let Some(i) = idx else {
        return false;
    };
    // Account for the marker so the final content fits within the tool cap.
    let effective_max = max_body.saturating_sub(TRUNCATE_MARKER.len());
    let truncated = truncate_utf8(&msgs[i].content, effective_max);
    msgs[i].content = format!("{}{}", truncated, TRUNCATE_MARKER);
    true
}

fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Finds the earliest complete turn whose indices are not anchored and whose end is before the final message.
fn find_oldest_droppable_turn(
    msgs: &[ChatMessage],
    anchors: &HashSet<usize>,
) -> Option<(usize, usize)> {
    if msgs.len() < 2 {
        return None;
    }
    let rest = msgs.len() - 1; // preserve final message
    let mut start: Option<usize> = None;
    for (i, m) in msgs[..rest].iter().enumerate() {
        if m.role == "user" {
            if let Some(s) = start {
                let end = i;
                if !(s..end).any(|idx| anchors.contains(&idx)) {
                    return Some((s, end));
                }
            }
            start = Some(i);
        }
    }
    if let Some(s) = start {
        if s < rest && !(s..rest).any(|idx| anchors.contains(&idx)) {
            return Some((s, rest));
        }
    }
    None
}

fn recent_only_anchor_indices(msgs: &[ChatMessage], preserve_last_n: usize) -> HashSet<usize> {
    let mut anchors = HashSet::new();
    let n = preserve_last_n.min(msgs.len());
    for i in (msgs.len() - n)..msgs.len() {
        anchors.insert(i);
    }
    anchors
}

/// Drops the first complete unanchored "turn" before the final message.
/// If no non-anchored turn exists, falls back to dropping the oldest turn
/// while still preserving the most recent `preserve_last_n` messages.
fn drop_oldest_turn(
    msgs: &mut Vec<ChatMessage>,
    anchors: &HashSet<usize>,
    preserve_last_n: usize,
) -> bool {
    let mut turn = find_oldest_droppable_turn(msgs, anchors);
    if turn.is_none() {
        // Last resort: allow dropping the oldest (first user) turn as long as the
        // suffix of recent messages is preserved.
        let recent = recent_only_anchor_indices(msgs, preserve_last_n);
        turn = find_oldest_droppable_turn(msgs, &recent);
    }
    let Some((first_user, end)) = turn else {
        return false;
    };
    if end <= first_user {
        return false;
    }
    msgs.drain(first_user..end);
    if !msgs.is_empty() && msgs[0].role != "user" {
        msgs.insert(
            0,
            ChatMessage {
                role: "user".to_string(),
                content: OMIT_TURN.to_string(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        );
    }
    true
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::super::dto::ChatMessage;
    use super::*;

    fn user(s: &str) -> ChatMessage {
        ChatMessage {
            role: "user".into(),
            content: s.into(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    fn tool_body(n: usize) -> ChatMessage {
        let content = "x".repeat(n);
        ChatMessage {
            role: "tool".into(),
            content,
            tool_calls: None,
            tool_call_id: Some("call-1".into()),
            name: Some("jbovlaste_semantic_search".into()),
        }
    }

    #[test]
    fn compress_truncates_tool_before_dropping_turns() {
        let v = vec![
            user("hi"),
            ChatMessage {
                role: "assistant".into(),
                content: "a".into(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            tool_body(50_000),
            user("next"),
        ];
        let before = estimate_messages_bytes(&v);
        let out = compress_chat_history(&v, before / 2, 0);
        assert!(
            estimate_messages_bytes(&out) < before,
            "expected smaller estimate"
        );
        let tool = out
            .iter()
            .find(|m| m.role == "tool")
            .expect("tool message after compression");
        assert!(tool.content.len() < 50_000);
        assert!(tool.content.contains(TRUNCATE_MARKER));
    }

    #[test]
    fn compress_drops_oldest_turn_when_tools_small() {
        let v = vec![user("old-turn-xxxxxxxx"), user("new-turn-yyyy")];
        let out = compress_chat_history(&v, 8, 0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].content, "new-turn-yyyy");
    }

    #[test]
    fn for_budget_returns_clone_when_under_roo_threshold() {
        let v = vec![user("hello")];
        let budget = ContextBudget::huge_for_tests(10_000);
        let out = compress_chat_history_for_budget(&v, &budget);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].content, "hello");
    }

    #[test]
    fn aggressive_returns_clone_when_already_under_half_budget() {
        let v = vec![user(&"x".repeat(1000))];
        let budget = ContextBudget::huge_for_tests(500_000);
        assert!(estimate_messages_bytes(&v) < 20_000);
        let out = compress_chat_history_aggressive(&v, &budget);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].content.len(), 1000);
    }

    #[test]
    fn no_compression_when_short_history_and_large_window() {
        let msgs = vec![user("hi")];
        let budget = ContextBudget::huge_for_tests(1_000);
        assert!(!should_compress_history(&msgs, &budget));
    }

    #[test]
    fn compression_preserves_first_user_and_recent_turns() {
        // history: u1, a1, t1, u2, a2, t2, u3 (current query)
        let v = vec![
            user("first-question-xxxxxxxxxxxx"),
            ChatMessage {
                role: "assistant".into(),
                content: "a1".into(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            tool_body(15_000),
            user("second-question"),
            ChatMessage {
                role: "assistant".into(),
                content: "a2".into(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            tool_body(15_000),
            user("current-question"),
        ];
        let out = compress_chat_history(&v, 14_000, 1);
        // first user and final user survive; the middle turn is dropped
        assert!(out
            .iter()
            .any(|m| m.content == "first-question-xxxxxxxxxxxx"));
        assert!(out.iter().any(|m| m.content == "current-question"));
        assert!(!out.iter().any(|m| m.content == "second-question"));
        assert!(estimate_messages_bytes(&out) <= 14_000);
    }
}
