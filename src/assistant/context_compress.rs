//! Compress client-provided chat history to reduce risk of context window overflow.

use super::dto::ChatMessage;

const TRUNCATE_MARKER: &str = "\n\n[Truncated for context size.]";
const OMIT_TURN: &str = "[Earlier discussion omitted.]";

/// Rough size estimate for budgeting (characters, not tokens).
pub fn estimate_messages_chars(messages: &[ChatMessage]) -> usize {
    messages.iter().fold(0, |acc, m| {
        acc + m.content.len()
            + m.tool_call_id.as_deref().map_or(0, str::len)
            + m.name.as_deref().map_or(0, str::len)
            + m
                .tool_calls
                .as_ref()
                .map_or(0, |tc| tc.iter().fold(0, |a, c| a + estimate_tool_call_chars(c)))
    })
}

fn estimate_tool_call_chars(c: &super::dto::ToolCallDto) -> usize {
    let mut n = 32;
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

fn default_max_input_chars() -> usize {
    std::env::var("ASSISTANT_MAX_INPUT_CHARS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(400_000)
}

fn default_tool_body_max_chars() -> usize {
    std::env::var("ASSISTANT_TOOL_BODY_MAX_CHARS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12_000)
}

/// Truncate oldest tool bodies first, then drop oldest turns (never removes the last message).
pub fn compress_chat_history(messages: &[ChatMessage], max_input_chars: usize) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return vec![];
    }
    let tool_cap = default_tool_body_max_chars();
    let mut out: Vec<ChatMessage> = messages.to_vec();

    let mut guard = 0u32;
    while estimate_messages_chars(&out) > max_input_chars && guard < 10_000 {
        guard += 1;
        if truncate_oldest_tool_body(&mut out, tool_cap) {
            continue;
        }
        if drop_oldest_turn(&mut out) {
            continue;
        }
        break;
    }
    out
}

/// Public entry using `ASSISTANT_MAX_INPUT_CHARS` (or override).
/// If the history is already under budget, returns a clone immediately—no truncation passes.
pub fn compress_chat_history_for_request(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    let max = default_max_input_chars();
    if messages.is_empty() {
        return vec![];
    }
    if estimate_messages_chars(messages) <= max {
        return messages.to_vec();
    }
    compress_chat_history(messages, max)
}

/// More aggressive compression (e.g. retry after provider context error).
pub fn compress_chat_history_aggressive(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    let budget = (default_max_input_chars() / 2).max(20_000);
    let tool_cap = (default_tool_body_max_chars() / 2).max(2_000);
    if messages.is_empty() {
        return vec![];
    }
    if estimate_messages_chars(messages) <= budget {
        return messages.to_vec();
    }
    let mut out: Vec<ChatMessage> = messages.to_vec();
    let mut guard = 0u32;
    while estimate_messages_chars(&out) > budget && guard < 10_000 {
        guard += 1;
        if truncate_oldest_tool_body(&mut out, tool_cap) {
            continue;
        }
        if drop_oldest_turn(&mut out) {
            continue;
        }
        break;
    }
    out
}

fn truncate_oldest_tool_body(msgs: &mut [ChatMessage], max_body: usize) -> bool {
    let idx = msgs.iter().position(|m| {
        m.role == "tool"
            && m.name.as_deref() == Some("jbovlaste_semantic_search")
            && m.content.len() > max_body
    });
    let Some(i) = idx else {
        return false;
    };
    let truncated = truncate_utf8(&msgs[i].content, max_body);
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

/// Drops the first complete "turn" before the final message (the final message is the active user query).
fn drop_oldest_turn(msgs: &mut Vec<ChatMessage>) -> bool {
    if msgs.len() < 2 {
        return false;
    }
    let rest = msgs.len() - 1;
    let Some(first_user) = msgs[..rest].iter().position(|m| m.role == "user") else {
        return false;
    };
    let after = &msgs[first_user + 1..rest];
    let second_user_rel = after.iter().position(|m| m.role == "user");
    let end = first_user
        + 1
        + second_user_rel.unwrap_or(after.len()); // exclusive end within 0..rest
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
        let before = estimate_messages_chars(&v);
        let out = compress_chat_history(&v, before / 2);
        assert!(
            estimate_messages_chars(&out) < before,
            "expected smaller estimate"
        );
        let tool = out.iter().find(|m| m.role == "tool").unwrap();
        assert!(tool.content.len() < 50_000);
        assert!(tool.content.contains(TRUNCATE_MARKER));
    }

    #[test]
    fn compress_drops_oldest_turn_when_tools_small() {
        let v = vec![user("old-turn-xxxxxxxx"), user("new-turn-yyyy")];
        let out = compress_chat_history(&v, 8);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].content, "new-turn-yyyy");
    }

    #[test]
    fn for_request_returns_clone_when_already_under_budget() {
        let v = vec![user("hello")];
        let out = super::compress_chat_history_for_request(&v);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].content, "hello");
    }

    #[test]
    fn aggressive_returns_clone_when_already_under_half_budget() {
        let v = vec![user(&"x".repeat(1000))];
        assert!(super::estimate_messages_chars(&v) < 20_000);
        let out = super::compress_chat_history_aggressive(&v);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].content.len(), 1000);
    }
}
