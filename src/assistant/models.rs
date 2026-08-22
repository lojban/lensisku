//! Internal assistant agent models (tool args, search batches, reference resolution).

use serde::Deserialize;

pub const SEMANTIC_SEARCH_MAX_LIMIT: u32 = 15;
pub const SEMANTIC_SEARCH_MAX_QUERIES_PER_CALL: usize = 24;

#[derive(Debug, Clone)]
pub struct ResolvedSemanticFilters {
    pub languages_langids: Option<Vec<i32>>,
    pub source_langid: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct SemanticSearchCore {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolArgs {
    #[serde(default)]
    pub queries: Vec<String>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub source_language: Option<String>,
}

impl ToolArgs {
    pub fn normalized_queries(&self) -> Result<Vec<String>, String> {
        let mut v: Vec<String> = self
            .queries
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        if v.is_empty() {
            if let Some(ref q) = self.query {
                let t = q.trim();
                if !t.is_empty() {
                    v.push(t.to_string());
                }
            }
        }
        if v.is_empty() {
            return Err(
                "`queries` must be a non-empty array of search strings (non-empty after trimming)."
                    .to_string(),
            );
        }
        if v.len() > SEMANTIC_SEARCH_MAX_QUERIES_PER_CALL {
            return Err(format!(
                "At most {} queries per jbovlaste_semantic_search call.",
                SEMANTIC_SEARCH_MAX_QUERIES_PER_CALL
            ));
        }
        Ok(v)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResolveReference {
    pub definitionid: i32,
    pub field: String,
    #[serde(default)]
    pub exampleid: Option<i32>,
    #[serde(default)]
    pub exact_text: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResolveArgs {
    #[serde(default)]
    pub references: Vec<ResolveReference>,
    #[serde(default)]
    pub message: Option<String>,
}

impl ResolveArgs {
    pub fn normalized(&self) -> Result<(Vec<ResolveReference>, Option<String>), String> {
        if let Some(msg) = self.message.as_ref() {
            let t = msg.trim();
            if t.is_empty() {
                return Err("`message` must not be empty if provided.".to_string());
            }
            if !self.references.is_empty() {
                return Err("Provide either `references` or `message`, not both.".to_string());
            }
            return Ok((Vec::new(), Some(t.to_string())));
        }
        if self.references.is_empty() {
            return Err("`references` must not be empty unless `message` is provided.".to_string());
        }
        for (i, r) in self.references.iter().enumerate() {
            let f = r.field.trim().to_lowercase();
            let valid = matches!(
                f.as_str(),
                "definition" | "notes" | "etymology" | "rafsi" | "example" | "decomposition"
            );
            if !valid {
                return Err(format!(
                    "references[{i}].field `{}` is not one of: definition, notes, etymology, rafsi, example, decomposition",
                    r.field
                ));
            }
            if f == "example" && r.exampleid.is_none() {
                return Err(format!(
                    "references[{i}].exampleid is required when field is `example`"
                ));
            }
            let exact = r.exact_text.as_deref().unwrap_or("").trim();
            if exact.is_empty() {
                return Err(format!(
                    "references[{i}].exact_text is required and must not be empty"
                ));
            }
        }
        Ok((self.references.clone(), None))
    }
}

#[derive(Debug, Clone)]
pub struct SearchBatch {
    pub queries: Vec<String>,
    pub limit: Option<u32>,
    pub languages: Option<Vec<String>>,
    pub source_language: Option<String>,
}

fn language_tag_from_locale(locale: &str) -> Option<String> {
    let tag = locale.split(['-', '_']).next()?;
    let tag = tag.trim().to_lowercase();
    if tag.len() >= 2 {
        Some(tag)
    } else {
        None
    }
}

impl SearchBatch {
    pub fn from_tool_args(
        args: &ToolArgs,
        queries: Vec<String>,
        default_locale: Option<&str>,
    ) -> Self {
        let languages = args.languages.clone().or_else(|| {
            default_locale
                .and_then(language_tag_from_locale)
                .map(|tag| {
                    log::info!(
                        "Assistant: LLM omitted `languages`; defaulting to [\"{tag}\"] from locale"
                    );
                    vec![tag]
                })
        });
        Self {
            queries,
            limit: args.limit,
            languages,
            source_language: args.source_language.clone(),
        }
    }

    pub fn call_core(&self, query: &str) -> SemanticSearchCore {
        SemanticSearchCore {
            query: query.to_string(),
            limit: self.limit,
        }
    }
}

#[derive(Debug)]
pub enum PreparedToolSlot {
    Immediate {
        tool_call_id: Option<String>,
        name: Option<String>,
        content: String,
    },
    Search {
        tool_call_id: Option<String>,
        name: Option<String>,
        batch: SearchBatch,
        assistant_reasoning: Option<String>,
        global_step_index: usize,
        action_desc: String,
    },
    Resolve {
        tool_call_id: Option<String>,
        name: Option<String>,
        refs: Vec<ResolveReference>,
        message: Option<String>,
        assistant_reasoning: Option<String>,
        global_step_index: usize,
        action_desc: String,
    },
}

#[derive(Debug, Clone)]
pub struct ValidatedReference {
    pub definitionid: i32,
    pub valsiword: String,
    pub type_name: String,
    pub langid: i32,
    pub langrealname: String,
    pub selmaho: Option<String>,
    pub field: String,
    pub exampleid: Option<i32>,
    pub exact_text: String,
}

#[derive(Debug, Clone)]
pub struct ReferenceError {
    pub index: usize,
    pub definitionid: i32,
    pub field: String,
    pub exact_text: String,
    pub reason: String,
}

pub enum ResolveOutcome {
    Valid(Vec<ValidatedReference>),
    Invalid(Vec<ReferenceError>),
}

#[derive(Debug, Deserialize)]
pub struct RequestAnalysis {
    pub intent: String,
    #[serde(default)]
    pub on_topic: bool,
    #[serde(default = "default_true")]
    pub needs_search: bool,
    #[serde(default)]
    pub search_queries: Vec<String>,
    pub ambiguity_note: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for RequestAnalysis {
    fn default() -> Self {
        Self {
            intent: String::new(),
            on_topic: true,
            needs_search: true,
            search_queries: Vec::new(),
            ambiguity_note: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RequestAnalysis;

    #[test]
    fn parses_request_analysis_json() {
        let raw = r#"{"intent": "lookup fox", "on_topic": true, "needs_search": true, "search_queries": ["fox", "animal"], "ambiguity_note": ""}"#;
        let a: RequestAnalysis = serde_json::from_str(raw).unwrap();
        assert_eq!(a.intent, "lookup fox");
        assert!(a.on_topic);
        assert!(a.needs_search);
        assert_eq!(a.search_queries, vec!["fox", "animal"]);
        assert_eq!(a.ambiguity_note.as_deref(), Some(""));
    }

    #[test]
    fn default_request_analysis_is_permissive() {
        let a = RequestAnalysis::default();
        assert!(a.on_topic);
        assert!(a.needs_search);
        assert!(a.search_queries.is_empty());
    }

    #[test]
    fn request_analysis_parses_off_topic() {
        let raw = r#"{"intent": "weather", "on_topic": false, "needs_search": false}"#;
        let a: RequestAnalysis = serde_json::from_str(raw).unwrap();
        assert!(!a.on_topic);
        assert!(!a.needs_search);
    }
}
