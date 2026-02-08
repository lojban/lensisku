use actix_web::{HttpResponse, ResponseError};
use ammonia::Builder as AmmoniaBuilder;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use once_cell::sync::Lazy;
use regex::{Error as RegexError, Regex};
use std::collections::HashSet;

use crate::collections::models::ImageData;

struct RegexSet {
    math: Regex,
    braces: Regex,
    quotes: Regex,
    see_also: Regex,
    cmavo_list: Regex,
    see: Regex,
    /// Structural boilerplate: "is the language with ISO 639-3 code " -> " language " so the actual language name dominates
    lang_iso_prefix: Regex,
    country_prefix: Regex,
    currency_prefix: Regex,
    quantity_prefix: Regex,
    multi_space: Regex,
    multi_dots: Regex,
    trailing_punct: Regex,
    multi_comma: Regex,
    leading_punct: Regex,
    trailing_dot: Regex,
    ka_nu_duu: Regex,
}

impl RegexSet {
    fn new() -> Result<Self, RegexError> {
        Ok(Self {
            math: Regex::new(r"\$.*?\$")?,
            braces: Regex::new(r"\{.*?\}")?,
            quotes: Regex::new(r#"".*?""#)?,
            see_also: Regex::new(r"See also *:?\b *")?,
            cmavo_list: Regex::new(r"\bcmavo list\b")?,
            see: Regex::new(r"See\b *")?,
            lang_iso_prefix: Regex::new(r"(?i)\s*is\s+the\s+language\s+with\s+ISO\s*(?:639-3\s+)?code\s*")?,
            country_prefix: Regex::new(r"(?i)\s*is\s+the\s+country\s+with\s+the\s*")?,
            currency_prefix: Regex::new(r"(?i)\s*is\s+measured\s+in\s+currency\s*")?,
            quantity_prefix: Regex::new(r"(?i)\s*is\s+a\s+quantity\s+of\s*/\s*contain\s*")?,
            multi_space: Regex::new(r" {2,}")?,
            multi_dots: Regex::new(r"[\.,] ?[\.,]")?,
            trailing_punct: Regex::new(r"[,\. ]+$")?,
            multi_comma: Regex::new(r"(, *,)+")?,
            leading_punct: Regex::new(r"^[,\. ]+")?,
            trailing_dot: Regex::new(r" *\.$")?,
            ka_nu_duu: Regex::new(r"\((ka|nu|du'u)\)")?,
        })
    }
}

static REGEX_SET: Lazy<Result<RegexSet, RegexError>> = Lazy::new(RegexSet::new);

/// Preprocesses a definition string for embedding generation.
///
/// Removes/normalizes formatting and structural boilerplate so embeddings capture meaning
/// rather than shared placeholders. Uses space (not [UNK]) for removed content to avoid
/// clustering many definitions around the same token.
pub fn preprocess_definition_for_vectors(def: &str) -> Result<String, RegexError> {
    let regex_set = REGEX_SET.as_ref().map_err(|e| e.clone())?;
    let mut processed = def.trim().to_string();

    // Replace special patterns with space (empty string would glue words; [UNK] would cluster)
    processed = processed.replace('/', " / ");
    processed = regex_set.math.replace_all(&processed, " ").into_owned();
    processed = regex_set.braces.replace_all(&processed, " ").into_owned();
    processed = regex_set.quotes.replace_all(&processed, " ").into_owned();

    // Remove service phrases
    processed = regex_set.see_also.replace_all(&processed, "").into_owned();
    processed = regex_set.cmavo_list.replace_all(&processed, "").into_owned();
    processed = regex_set.see.replace_all(&processed, "").into_owned();

    // Normalize structural boilerplate so the distinguishing part dominates (report §8.4)
    processed = regex_set
        .lang_iso_prefix
        .replace_all(&processed, " language ")
        .into_owned();
    processed = regex_set
        .country_prefix
        .replace_all(&processed, " country ")
        .into_owned();
    processed = regex_set
        .currency_prefix
        .replace_all(&processed, " currency ")
        .into_owned();
    processed = regex_set
        .quantity_prefix
        .replace_all(&processed, " quantity ")
        .into_owned();

    // Normalize whitespace and punctuation
    processed = regex_set.multi_space.replace_all(&processed, " ").into_owned();
    processed = regex_set.multi_dots.replace_all(&processed, ".").into_owned();
    processed = regex_set.trailing_punct.replace_all(&processed, "").into_owned();
    processed = regex_set.multi_comma.replace_all(&processed, ",").into_owned();
    processed = regex_set.leading_punct.replace_all(&processed, "").into_owned();
    processed = regex_set.trailing_dot.replace_all(&processed, "").into_owned();

    // Remove Lojban placeholder patterns
    processed = regex_set.ka_nu_duu.replace_all(&processed, "").into_owned();

    Ok(processed.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessing() -> Result<(), RegexError> {
        let input = r#"See also {broda}, $x_1$ is a "thing" / cmavo list, See foo.. bar,,  [UNK]. [UNK]. (ka)"#;
        // Math/braces/quotes replaced with space; See also/cmavo list/See removed; (ka) removed; no [UNK] produced
        let actual = preprocess_definition_for_vectors(input)?;
        assert!(actual.contains("is a") && actual.contains("foo. bar") && !actual.contains("See also") && !actual.contains("(ka)"));
        Ok(())
    }

    #[test]
    fn test_lang_iso_prefix() -> Result<(), RegexError> {
        let input = "$x_1$ is the language with ISO 639-3 code en.";
        let actual = preprocess_definition_for_vectors(input)?;
        assert!(actual.contains("language") && actual.contains("en"), "actual: {:?}", actual);
        Ok(())
    }
}

pub fn remove_html_tags(html: &str) -> String {
    static AMMONIA: Lazy<AmmoniaBuilder<'static>> = Lazy::new(|| {
        let mut builder = AmmoniaBuilder::default();
        // Remove all HTML tags; MathJax/LaTeX markers remain as plain text.
        builder.tags(HashSet::new());
        builder.clean_content_tags(HashSet::new());
        builder
    });

    AMMONIA.clean(html).to_string()
}

pub fn validate_item_image(image: &ImageData) -> Result<(), String> {
    if !["image/jpeg", "image/png", "image/gif", "image/webp"].contains(&image.mime_type.as_str()) {
        return Err("Invalid image type. Supported types: JPEG, PNG, GIF, WebP".to_string());
    }

    let decoded_size = BASE64
        .decode(&image.data)
        .map_err(|_| "Invalid base64 data".to_string())?
        .len();

    if decoded_size > 5 * 1024 * 1024 {
        return Err("Image size exceeds 5MB limit".to_string());
    }

    Ok(())
}

pub fn handle_error(e: Box<dyn std::error::Error>, context: &str) -> HttpResponse {
    use crate::error::AppError;

    let msg = e.to_string();
    let app_error = if msg.contains("not found") {
        AppError::NotFound(msg)
    } else if msg.contains("access denied") || msg.contains("Forbidden") {
        AppError::Auth(msg)
    } else if msg.contains("Invalid") || msg.contains("Validation") {
        AppError::Validation(msg)
    } else {
        AppError::Internal(format!("{}: {}", context, msg))
    };

    app_error.error_response()
}

pub fn handle_import_error(e: Box<dyn std::error::Error>) -> HttpResponse {
    handle_error(e, "Import failed")
}
