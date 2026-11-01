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

    pub fn from_query(value: Option<&str>) -> Result<Self, &'static str> {
        match value.unwrap_or("pdf") {
            "pdf" => Ok(Self::Pdf),
            "latex" | "tex" => Ok(Self::LaTeX),
            "xml" => Ok(Self::Xml),
            "json" => Ok(Self::Json),
            "tsv" => Ok(Self::Tsv),
            _ => Err("Invalid format. Supported formats: pdf, latex, xml, json, tsv"),
        }
    }
}

pub use super::dto::{
    has_search_export_constraint, ExportOptions, SearchExportJson, SearchExportQuery,
};

/// Max collection + dictionary rows in one search export.
pub const SEARCH_EXPORT_ROW_CAP: i64 = 15_000;

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
    pub experimental_rafsi: Option<String>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CollectionExportItem {
    /// Omitted or zero when authoring an import file by hand (server assigns real ids).
    #[serde(default)]
    pub item_id: i32,
    /// Omitted or zero when authoring an import file; full import uses array order for positions.
    #[serde(default)]
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
    pub experimental_rafsi: Option<String>,
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
    /// Custom item audio as a data URL (`data:audio/mpeg;base64,…`); full export/import only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound_url: Option<String>,
    // Flashcard direction when item has an associated flashcard (full export only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// 0-based level index this item belongs to (for import when levels array is omitted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_index: Option<u32>,
    /// Order of this item within its level (for import when levels array is omitted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_in_level: Option<i32>,
    // ---- format_version >= 2 (all optional; older files simply omit them) ----
    /// `collection_items.auto_progress`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_progress: Option<bool>,
    /// ISO 15924 script code of the item content (`collection_items.script`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_code: Option<String>,
    /// `collection_items.is_original`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_original: Option<bool>,
    /// Stored canonical form; recomputed on import when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_form: Option<String>,
    /// `flashcards.use_canonical_comparison` (typed-answer grading).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_canonical_comparison: Option<bool>,
    /// Author-stored correct answer of a quiz card (`flashcard_quiz_options`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quiz_correct_answer: Option<String>,
    /// Dialogue card data beyond the script itself (script = `free_content_front`,
    /// card translation = `free_content_back`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dialogue: Option<DialogueExport>,
}

/// Dialogue extras. The script markup in `free_content_front` is the single source of truth
/// for lines, per-line glosses (`~`) and every separately scored question (`? fill:`,
/// `? bank:`, `? choice:`); `structure` is a parsed, read-only view of it for tools/readers
/// and is ignored on import.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DialogueExport {
    /// Stable speaker → voice map (`collection_items.dialogue_voices`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub voices: Option<serde_json::Value>,
    /// Generated per-line TTS clips (`dialogue_line_sounds`), reused instead of regenerated.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line_sounds: Vec<DialogueLineSoundExport>,
    /// Parsed script: `{title, lines:[{index,speaker,gender,voice_override,text,gloss}],
    /// questions:[{index,kind,prompt,before,after,answers,options:[{text,correct}]}]}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub structure: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DialogueLineSoundExport {
    pub line_index: i32,
    pub text_hash: String,
    pub voice: String,
    /// `data:audio/...;base64,...`
    pub sound_url: String,
}

#[derive(Debug)]
pub struct ValsiRow {
    pub definition_id: Option<i32>,
    pub word: String,
    pub rafsi: Option<String>,
    pub experimental_rafsi: Option<String>,
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
    pub source_language_tag: String,
    pub language_realname: String,
    pub format: String,
    pub positive_scores_only: bool,
    pub filename: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}
