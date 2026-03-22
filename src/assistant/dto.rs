use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallFunctionDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallDto {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    pub function: ToolCallFunctionDto,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCallDto>>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub locale: Option<String>,
}

/// One step of the assistant's thought process (e.g. a tool call and its result).
#[derive(Debug, Clone, Serialize)]
pub struct AssistantStep {
    /// Human-readable description of the action (e.g. "Semantic search: \"word combinations\"").
    pub action: String,
    /// Short summary of the result for display in the chat.
    pub result: String,
    /// Raw tool output (e.g. JSON) for display in folded/collapsible UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: String,
    /// Optional list of steps (tool calls + results) to show as thought process in the UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<AssistantStep>>,
}
