//! Build [`ChatRequest`](crate::assistant::dto::ChatRequest) from stored frontend-shaped JSON messages
//! (same rules as `buildPayloadMessages` / `pickPrimaryReply` in AssistantChat.vue).

use serde_json::Value;

use crate::assistant::dto::{ChatMessage, ToolCallDto};
use crate::error::AppError;

fn prune_api_segment(seg: &Value) -> Option<ChatMessage> {
    let role = seg.get("role")?.as_str()?.to_string();
    let mut m = ChatMessage {
        role,
        content: seg
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string(),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    };
    if let Some(tc) = seg.get("tool_calls") {
        let v: Vec<ToolCallDto> = serde_json::from_value(tc.clone()).ok()?;
        m.tool_calls = Some(v);
    }
    if let Some(id) = seg.get("tool_call_id").and_then(|x| x.as_str()) {
        m.tool_call_id = Some(id.to_string());
    }
    if let Some(n) = seg.get("name").and_then(|x| x.as_str()) {
        m.name = Some(n.to_string());
    }
    Some(m)
}

/// Picks the reply object to feed the API (same as `pickPrimaryReply` in AssistantChat.vue).
fn pick_primary_reply<'a>(msg: &'a Value, primary_model_id: Option<&str>) -> &'a Value {
    if let Some(replies) = msg.get("replies").and_then(|r| r.as_array()) {
        if !replies.is_empty() {
            if let Some(pm) = primary_model_id {
                for r in replies {
                    if r.get("model").and_then(|m| m.as_str()) == Some(pm) {
                        return r;
                    }
                }
            }
            return &replies[0];
        }
    }
    msg
}

/// Strips a trailing empty assistant placeholder (added before streaming) so the API request matches Vue.
pub fn strip_trailing_empty_assistant_stub(messages: &mut Vec<Value>) {
    if messages.len() < 2 {
        return;
    }
    let last = messages.last().cloned();
    let Some(last) = last else {
        return;
    };
    if last.get("role").and_then(|r| r.as_str()) != Some("assistant") {
        return;
    }
    let content = last
        .get("content")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .trim();
    if !content.is_empty() {
        return;
    }
    let has_replies = last
        .get("replies")
        .and_then(|r| r.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    if has_replies {
        return;
    }
    let has_steps = last
        .get("steps")
        .and_then(|s| s.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    if has_steps {
        return;
    }
    let has_api_trace = last
        .get("apiTrace")
        .and_then(|a| a.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    if has_api_trace {
        return;
    }
    messages.pop();
}

/// Converts stored `messages` JSON array into [`ChatMessage`] for the agent.
pub fn build_chat_messages_from_stored(
    messages_json: &[Value],
    primary_model_id: Option<&str>,
) -> Result<Vec<ChatMessage>, AppError> {
    let mut out = Vec::new();
    for m in messages_json {
        let role = m
            .get("role")
            .and_then(|r| r.as_str())
            .ok_or_else(|| AppError::BadRequest("assistant message missing role".into()))?;
        if role == "user" {
            out.push(ChatMessage {
                role: "user".into(),
                content: m
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
            continue;
        }
        if role == "assistant" {
            let reply = pick_primary_reply(m, primary_model_id);
            if let Some(trace) = reply.get("apiTrace").and_then(|a| a.as_array()) {
                if !trace.is_empty() {
                    for seg in trace {
                        if let Some(cm) = prune_api_segment(seg) {
                            out.push(cm);
                        }
                    }
                } else {
                    let text = reply
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("");
                    out.push(ChatMessage {
                        role: "assistant".into(),
                        content: text.to_string(),
                        tool_calls: None,
                        tool_call_id: None,
                        name: None,
                    });
                }
            } else {
                let text = reply
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                out.push(ChatMessage {
                    role: "assistant".into(),
                    content: text.to_string(),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                });
            }
            continue;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn strips_trailing_empty_assistant() {
        let mut arr: Vec<Value> = vec![
            json!({"role": "user", "content": "hi"}),
            json!({"role": "assistant", "content": "", "steps": [], "replies": []}),
        ];
        strip_trailing_empty_assistant_stub(&mut arr);
        assert_eq!(arr.len(), 1);
    }
}
