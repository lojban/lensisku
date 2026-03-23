//! Apply assistant SSE payloads to the frontend-shaped `messages` JSON (mirrors AssistantChat.vue `performRequest`).
#![allow(clippy::unwrap_used)]

use serde_json::{json, Value};

use crate::error::AppError;

fn get_or_create_reply<'a>(
    msg: &'a mut Value,
    model_id: &str,
    model_name: Option<&str>,
) -> Result<&'a mut Value, AppError> {
    let arr = msg
        .as_object_mut()
        .ok_or_else(|| AppError::BadRequest("assistant message not object".into()))?
        .entry("replies".to_string())
        .or_insert_with(|| json!([]));
    let arr = arr
        .as_array_mut()
        .ok_or_else(|| AppError::BadRequest("replies not array".into()))?;
    let idx = arr
        .iter()
        .position(|r| r.get("model").and_then(|m| m.as_str()) == Some(model_id));
    if let Some(i) = idx {
        if let Some(name) = model_name {
            if let Some(r) = arr[i].as_object_mut() {
                if r.get("modelName").and_then(|n| n.as_str()) != Some(name) {
                    r.insert("modelName".into(), json!(name));
                }
            }
        }
        return Ok(&mut arr[i]);
    }
    let r = if let Some(name) = model_name {
        json!({
            "model": model_id,
            "modelName": name,
            "steps": [],
            "content": "",
            "streamFinished": false
        })
    } else {
        json!({
            "model": model_id,
            "steps": [],
            "content": "",
            "streamFinished": false
        })
    };
    let new_len = arr.len() + 1;
    arr.push(r);
    arr.get_mut(new_len - 1)
        .ok_or_else(|| AppError::BadRequest("assistant reply push failed".into()))
}

fn steps_mut<'a>(
    msg: &'a mut Value,
    model_id: Option<&str>,
    model_name: Option<&str>,
) -> Result<&'a mut Vec<Value>, AppError> {
    if let Some(mid) = model_id {
        let r = get_or_create_reply(msg, mid, model_name)?;
        let o = r
            .as_object_mut()
            .ok_or_else(|| AppError::BadRequest("reply not object".into()))?;
        o.entry("steps".to_string())
            .or_insert_with(|| json!([]));
        if !o["steps"].is_array() {
            o.insert("steps".to_string(), json!([]));
        }
        return o["steps"]
            .as_array_mut()
            .ok_or_else(|| AppError::BadRequest("steps not array".into()));
    }
    let o = msg
        .as_object_mut()
        .ok_or_else(|| AppError::BadRequest("assistant not object".into()))?;
    o.entry("steps".to_string())
        .or_insert_with(|| json!([]));
    if !o["steps"].is_array() {
        o.insert("steps".to_string(), json!([]));
    }
    o["steps"]
        .as_array_mut()
        .ok_or_else(|| AppError::BadRequest("steps not array".into()))
}

fn set_reply_stream_finished(msg: &mut Value, model_id: &str, finished: bool) {
    if let Some(arr) = msg.get_mut("replies").and_then(|v| v.as_array_mut()) {
        for r in arr.iter_mut() {
            if r.get("model").and_then(|m| m.as_str()) == Some(model_id) {
                if let Some(o) = r.as_object_mut() {
                    o.insert("streamFinished".into(), json!(finished));
                }
                break;
            }
        }
    }
}

/// Applies one SSE event to `messages` (array). `assistant_index` is the last assistant bubble index.
pub fn apply_sse_event_to_messages(
    messages: &mut Value,
    assistant_index: usize,
    event: &Value,
) -> Result<(), AppError> {
    let ty = event
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("");
    if ty == "stream_debug" {
        return Ok(());
    }

    let arr = messages
        .as_array_mut()
        .ok_or_else(|| AppError::BadRequest("messages not array".into()))?;
    if assistant_index >= arr.len() {
        return Ok(());
    }
    let msg = &mut arr[assistant_index];
    if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") {
        return Ok(());
    }

    let model_id_s = event.get("model").and_then(|m| m.as_str());
    let model_name = event.get("model_name").and_then(|m| m.as_str());

    if model_id_s.is_some() && model_id_s.unwrap().is_empty() {
        return Ok(());
    }

    match ty {
        "parallel_branches" => {
            let models = event
                .get("models")
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default();
            let o = msg
                .as_object_mut()
                .ok_or_else(|| AppError::BadRequest("assistant not object".into()))?;
            let replies = o.entry("replies".to_string()).or_insert_with(|| json!([]));
            let arr = replies
                .as_array_mut()
                .ok_or_else(|| AppError::BadRequest("replies not array".into()))?;
            for m in models {
                let Some(id) = m.get("id").and_then(|x| x.as_str()) else {
                    continue;
                };
                if arr
                    .iter()
                    .any(|r| r.get("model").and_then(|x| x.as_str()) == Some(id))
                {
                    continue;
                }
                let name = m.get("name").and_then(|x| x.as_str());
                let r = if let Some(name) = name {
                    json!({
                        "model": id,
                        "modelName": name,
                        "steps": [],
                        "content": "",
                        "streamFinished": false
                    })
                } else {
                    json!({
                        "model": id,
                        "steps": [],
                        "content": "",
                        "streamFinished": false
                    })
                };
                arr.push(r);
            }
        }
        "assistant_tool_calls" => {
            let entry = json!({
                "role": "assistant",
                "content": event.get("content").cloned().unwrap_or(json!("")),
                "tool_calls": event.get("tool_calls").clone(),
            });
            if let Some(mid) = model_id_s {
                let r = get_or_create_reply(msg, mid, model_name)?;
                let trace = r
                    .as_object_mut()
                    .unwrap()
                    .entry("apiTrace".to_string())
                    .or_insert_with(|| json!([]));
                if let Some(a) = trace.as_array_mut() {
                    a.push(entry);
                }
            } else {
                let trace = msg
                    .as_object_mut()
                    .unwrap()
                    .entry("apiTrace".to_string())
                    .or_insert_with(|| json!([]));
                if let Some(a) = trace.as_array_mut() {
                    a.push(entry);
                }
            }
        }
        "step_start" => {
            let steps = steps_mut(msg, model_id_s, model_name)?;
            let idx = event
                .get("index")
                .and_then(|i| i.as_u64())
                .map(|u| u as usize)
                .unwrap_or(steps.len());
            while steps.len() < idx {
                steps.push(json!({
                    "action": "",
                    "result": "…",
                }));
            }
            if steps.len() == idx {
                let mut o = serde_json::Map::new();
                o.insert(
                    "action".into(),
                    event.get("action").cloned().unwrap_or(json!("")),
                );
                o.insert("result".into(), json!("…"));
                if let Some(ar) = event.get("assistant_reasoning") {
                    o.insert("assistant_reasoning".into(), ar.clone());
                }
                steps.push(Value::Object(o));
            } else if let Some(existing) = steps.get_mut(idx) {
                let eo = existing.as_object_mut().unwrap();
                eo.insert(
                    "action".into(),
                    event.get("action").cloned().unwrap_or_else(|| json!("")),
                );
                eo.insert("result".into(), json!("…"));
                if let Some(ar) = event.get("assistant_reasoning") {
                    eo.insert("assistant_reasoning".into(), ar.clone());
                }
            }
        }
        "step" => {
            let steps = steps_mut(msg, model_id_s, model_name)?;
            let idx = event
                .get("index")
                .and_then(|i| i.as_u64())
                .map(|u| u as usize)
                .unwrap_or_else(|| steps.len().saturating_sub(1));
            let step_payload = json!({
                "action": event.get("action").and_then(|a| a.as_str()).unwrap_or(""),
                "result": event.get("result").and_then(|r| r.as_str()).unwrap_or(""),
                "tool_output": event.get("tool_output").clone(),
                "tool_call_id": event.get("tool_call_id").clone(),
                "tool_content_plain": event.get("tool_content_plain").clone(),
                "assistant_reasoning": event.get("assistant_reasoning").clone(),
            });
            while steps.len() < idx {
                steps.push(json!({
                    "action": "",
                    "result": "…",
                }));
            }
            if steps.len() == idx {
                steps.push(step_payload);
            } else {
                steps[idx] = step_payload;
            }

            let plain = event
                .get("tool_content_plain")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            let tc_id = event.get("tool_call_id").and_then(|t| t.as_str());
            if !plain.is_empty() {
                let tool_seg = json!({
                    "role": "tool",
                    "content": plain,
                    "tool_call_id": tc_id,
                    "name": "jbovlaste_semantic_search",
                });
                let dup_check = |trace: &Value| {
                    trace.as_array().map(|a| {
                        a.iter().any(|s| {
                            s.get("role").and_then(|r| r.as_str()) == Some("tool")
                                && tc_id.is_some()
                                && s.get("tool_call_id").and_then(|x| x.as_str()) == tc_id
                        })
                    }) == Some(true)
                };
                if let Some(mid) = model_id_s {
                    let r = get_or_create_reply(msg, mid, model_name)?;
                    let trace = r
                        .as_object_mut()
                        .unwrap()
                        .entry("apiTrace".to_string())
                        .or_insert_with(|| json!([]));
                    let is_dup = dup_check(trace);
                    if let Some(a) = trace.as_array_mut() {
                        if !is_dup {
                            a.push(tool_seg);
                        }
                    }
                } else {
                    let trace = msg
                        .as_object_mut()
                        .unwrap()
                        .entry("apiTrace".to_string())
                        .or_insert_with(|| json!([]));
                    let is_dup = dup_check(trace);
                    if let Some(a) = trace.as_array_mut() {
                        if !is_dup {
                            a.push(tool_seg);
                        }
                    }
                }
            }
        }
        "done" => {
            let reply_text = event
                .get("reply")
                .and_then(|r| r.as_str())
                .unwrap_or("");
            let done_seg = json!({
                "role": "assistant",
                "content": reply_text,
            });
            if let Some(mid) = model_id_s {
                let r = get_or_create_reply(msg, mid, model_name)?;
                r.as_object_mut()
                    .unwrap()
                    .insert("content".into(), json!(reply_text));
                let trace = r
                    .as_object_mut()
                    .unwrap()
                    .entry("apiTrace".to_string())
                    .or_insert_with(|| json!([]));
                if let Some(a) = trace.as_array_mut() {
                    a.push(done_seg);
                }
                set_reply_stream_finished(msg, mid, true);
            } else {
                let o = msg.as_object_mut().unwrap();
                o.insert("content".into(), json!(reply_text));
                o.insert("streamFinished".into(), json!(true));
                let trace = o
                    .entry("apiTrace".to_string())
                    .or_insert_with(|| json!([]));
                if let Some(a) = trace.as_array_mut() {
                    a.push(done_seg);
                }
            }
        }
        "error" => {
            let err_content = event.get("error").and_then(|e| e.as_str()).unwrap_or("");
            let formatted = if err_content.is_empty() {
                "Error".to_string()
            } else {
                format!("_{}_", err_content)
            };
            if let Some(mid) = model_id_s {
                let r = get_or_create_reply(msg, mid, model_name)?;
                let o = r.as_object_mut().unwrap();
                o.insert("content".into(), json!(formatted));
                o.remove("apiTrace");
                set_reply_stream_finished(msg, mid, true);
            } else {
                let o = msg.as_object_mut().unwrap();
                o.insert("content".into(), json!(formatted));
                o.remove("apiTrace");
                o.insert("streamFinished".into(), json!(true));
            }
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod stream_finished_tests {
    use super::apply_sse_event_to_messages;
    use serde_json::json;

    #[test]
    fn done_without_model_sets_stream_finished() {
        let mut messages = json!([
            {"role": "user", "content": "hi"},
            {
                "role": "assistant",
                "content": "",
                "steps": [],
                "streamFinished": false
            }
        ]);
        let ev = json!({"type": "done", "reply": "hello"});
        apply_sse_event_to_messages(&mut messages, 1, &ev).unwrap();
        let m = &messages[1];
        assert_eq!(m["streamFinished"], true);
        assert_eq!(m["content"], "hello");
    }

    #[test]
    fn done_with_model_sets_reply_stream_finished() {
        let mut messages = json!([
            {"role": "user", "content": "hi"},
            {
                "role": "assistant",
                "content": "",
                "steps": [],
                "streamFinished": false,
                "replies": []
            }
        ]);
        let ev = json!({
            "type": "done",
            "reply": "a",
            "model": "openrouter/x",
            "model_name": "X"
        });
        apply_sse_event_to_messages(&mut messages, 1, &ev).unwrap();
        let replies = messages[1]["replies"].as_array().unwrap();
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0]["streamFinished"], true);
    }
}
