use camxes_rs::camxes::peg::parsing::ParseNode;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use tokio_postgres::Row;
use utoipa::ToSchema;

#[derive(Debug)]
pub enum MathJaxValidationError {
    Balance(String),
    Syntax(String),
    Tectonic(String),
}

impl std::fmt::Display for MathJaxValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MathJaxValidationError::Balance(msg) => write!(f, "Unbalanced delimiters: {}", msg),
            MathJaxValidationError::Syntax(msg) => write!(f, "Syntax error: {}", msg),
            MathJaxValidationError::Tectonic(msg) => write!(f, "LaTeX compilation error: {}", msg),
        }
    }
}
impl StdError for MathJaxValidationError {}

#[derive(Debug)]
pub struct MathJaxValidationOptions {
    pub use_tectonic: bool,
}

/// Dictionary-facing classification (vlazba morphology + lensisku type ids).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LujvoClassification {
    pub type_id: i16,
    pub type_name: &'static str,
    /// Score-optimal form; equal to `word` when type is canonical lujvo.
    pub canonical_word: String,
}

/// Owned rafsi maps so callers can build [`vlazba::jvozba::tools::RafsiOptions`] without lifetime issues.
#[derive(Debug, Default, Clone)]
pub struct OwnedRafsiMaps {
    pub cmavo: HashMap<String, Vec<String>>,
    pub cmavo_exp: HashMap<String, Vec<String>>,
    pub gismu: HashMap<String, Vec<String>>,
    pub gismu_exp: HashMap<String, Vec<String>>,
    pub se_words: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Language {
    pub id: i32,
    pub tag: String,
    pub english_name: String,
    pub lojban_name: String,
    pub real_name: String,
    pub for_lojban: Option<String>,
    pub url: Option<String>,
}

impl From<Row> for Language {
    fn from(row: Row) -> Self {
        Language {
            id: row.get("langid"),
            tag: row.get("tag"),
            english_name: row.get("englishname"),
            lojban_name: row.get("lojbanname"),
            real_name: row.get("realname"),
            for_lojban: row.get("forlojban"),
            url: row.get("url"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LojbanToken {
    pub kind: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
    #[schema(value_type = Vec<Object>)]
    pub children: Vec<LojbanToken>,
}

impl From<ParseNode> for LojbanToken {
    fn from(node: ParseNode) -> Self {
        match node {
            ParseNode::Terminal { span } => LojbanToken {
                kind: "terminal".to_string(),
                text: String::new(),
                start: span.0,
                end: span.1,
                children: vec![],
            },
            ParseNode::NonTerminal {
                name,
                span,
                children,
            } => LojbanToken {
                kind: format!("non_terminal_{}", name),
                text: String::new(),
                start: span.0,
                end: span.1,
                children: children.into_iter().map(LojbanToken::from).collect(),
            },
        }
    }
}
