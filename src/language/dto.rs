use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::models::LojbanToken;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LojbanParseRequest {
    pub text: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LojbanParseResponse {
    pub success: bool,
    pub tokens: Vec<LojbanToken>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalyzeWordRequest {
    pub word: String,
    /// Optional entry language ID (defaults to Lojban: 1)
    pub source_langid: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnalyzeWordResponse {
    pub success: bool,
    pub word_type: String,
    pub text: String,
    pub recommended: Option<String>,
    pub problems: Option<HashMap<String, Vec<String>>>,
    pub error: Option<String>,
    /// Source words (selrafsi) when `word_type` is `"lujvo"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decomposition: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MathJaxValidationRequest {
    pub text: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MathJaxValidationResponse {
    pub valid: bool,
    pub error: Option<String>,
}
