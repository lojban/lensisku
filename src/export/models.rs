use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::jbovlaste::KeywordMapping;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Pdf,
    LaTeX,
    Xml,
    Json,
    Tsv,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExportFormat::Pdf => write!(f, "pdf"),
            ExportFormat::LaTeX => write!(f, "latex"),
            ExportFormat::Xml => write!(f, "xml"),
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Tsv => write!(f, "tsv"),
        }
    }
}

impl ExportFormat {
    pub fn content_type(&self) -> &str {
        match self {
            ExportFormat::Pdf => "application/pdf",
            ExportFormat::LaTeX => "application/x-latex",
            ExportFormat::Xml => "application/xml",
            ExportFormat::Json => "application/json",
            ExportFormat::Tsv => "application/zip",
        }
    }

    pub fn file_extension(&self) -> &str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::LaTeX => "tex",
            ExportFormat::Xml => "xml",
            ExportFormat::Json => "json",
            ExportFormat::Tsv => "zip",
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ExportOptions {
    pub format: Option<String>,
    pub positive_scores_only: Option<bool>,
    pub collection_id: Option<i32>,
}

#[derive(Serialize)]
pub struct User {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realname: Option<String>,
}

#[derive(Serialize)]
pub struct DictionaryEntry {
    pub word: String,
    pub word_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rafsi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selmaho: Option<String>,
    pub definition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etymology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jargon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_note: Option<String>,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gloss_keywords: Option<Vec<KeywordMapping>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place_keywords: Option<Vec<KeywordMapping>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CollectionExportItem {
    pub item_id: i32,
    pub position: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_note: Option<String>,
    // Fields for definition-based items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rafsi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selmaho: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jargon: Option<String>,
    // Fields for free-content items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_content_front: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_content_back: Option<String>,
    // Image data URLs (for JSON export)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub front_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back_image_url: Option<String>,
    // Flashcard direction when item has an associated flashcard (full export only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug)]
pub struct ValsiRow {
    pub word: String,
    pub rafsi: Option<String>,
    pub selmaho: Option<String>,
    pub definition: String,
    pub notes: Option<String>,
    pub collection_note: Option<String>,
    pub descriptor: String,
}

#[derive(Serialize)]
pub struct NaturalEntry {
    pub word: String,
    pub meaning: Option<String>,
    pub valsi: String,
    pub place: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_note: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CachedExport {
    pub language_tag: String,
    pub language_realname: String,
    pub format: String,
    pub filename: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}
