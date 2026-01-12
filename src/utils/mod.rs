use actix_web::{HttpResponse, ResponseError};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use once_cell::sync::Lazy;
use regex::{Error as RegexError, Regex};

use crate::collections::models::ImageData;

struct RegexSet {
    math: Regex,
    braces: Regex,
    quotes: Regex,
    see_also: Regex,
    cmavo_list: Regex,
    see: Regex,
    multi_space: Regex,
    multi_dots: Regex,
    trailing_punct: Regex,
    multi_comma: Regex,
    leading_punct: Regex,
    unk_dot_unk: Regex,
    dot_unk_dot: Regex,
    unk_comma_unk: Regex,
    trailing_dot: Regex,
    trailing_unk: Regex,
    modal_unk: Regex,
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
            multi_space: Regex::new(r" {2,}")?,
            multi_dots: Regex::new(r"[\.,] ?[\.,]")?,
            trailing_punct: Regex::new(r"[,\. ]+$")?,
            multi_comma: Regex::new(r"(, *,)+")?,
            leading_punct: Regex::new(r"^[,\. ]+")?,
            unk_dot_unk: Regex::new(r"\[UNK\]\. *\[UNK\]")?,
            dot_unk_dot: Regex::new(r"\. *\[UNK\]\.")?,
            unk_comma_unk: Regex::new(r"\[UNK\] *, *\[UNK\]")?,
            trailing_dot: Regex::new(r" *\.$")?,
            trailing_unk: Regex::new(r" *\[UNK\]$")?,
            modal_unk: Regex::new(r"\[UNK\] modal, ")?,
            ka_nu_duu: Regex::new(r"\((ka|nu|du'u)\)")?,
        })
    }
}

static REGEX_SET: Lazy<Result<RegexSet, RegexError>> = Lazy::new(RegexSet::new);

/// Preprocesses a definition string for embedding generation
///
/// Handles special formatting, removes certain patterns, and normalizes whitespace
pub fn preprocess_definition_for_vectors(def: &str) -> Result<String, RegexError> {
    let regex_set = REGEX_SET.as_ref().map_err(|e| e.clone())?;
    let mut processed = def.trim().to_string();

    // Replace special patterns with [UNK]
    processed = processed.replace('/', " / ");
    processed = regex_set.math.replace_all(&processed, "[UNK]").into_owned();
    processed = regex_set
        .braces
        .replace_all(&processed, "[UNK]")
        .into_owned();
    processed = regex_set
        .quotes
        .replace_all(&processed, "[UNK]")
        .into_owned();

    // Remove specific patterns
    processed = regex_set.see_also.replace_all(&processed, "").into_owned();
    processed = regex_set
        .cmavo_list
        .replace_all(&processed, "")
        .into_owned();
    processed = regex_set.see.replace_all(&processed, "").into_owned();

    // Normalize whitespace and punctuation
    processed = regex_set
        .multi_space
        .replace_all(&processed, " ")
        .into_owned();
    processed = regex_set
        .multi_dots
        .replace_all(&processed, ".")
        .into_owned();
    processed = regex_set
        .trailing_punct
        .replace_all(&processed, "")
        .into_owned();
    processed = regex_set
        .multi_comma
        .replace_all(&processed, ",")
        .into_owned();
    processed = regex_set
        .leading_punct
        .replace_all(&processed, "")
        .into_owned();

    // Handle [UNK] patterns
    processed = regex_set
        .unk_dot_unk
        .replace_all(&processed, ". [UNK] ")
        .into_owned();
    processed = regex_set
        .dot_unk_dot
        .replace_all(&processed, ".")
        .into_owned();
    processed = regex_set
        .unk_comma_unk
        .replace_all(&processed, "[UNK]")
        .into_owned();
    processed = regex_set
        .trailing_dot
        .replace_all(&processed, "")
        .into_owned();
    processed = regex_set
        .trailing_unk
        .replace_all(&processed, "")
        .into_owned();
    processed = regex_set.modal_unk.replace_all(&processed, "").into_owned();

    // Remove specific Lojban patterns
    processed = regex_set.ka_nu_duu.replace_all(&processed, "").into_owned();

    Ok(processed.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessing() -> Result<(), RegexError> {
        let input = r#"See also {broda}, $x_1$ is a "thing" / cmavo list, See foo.. bar,,  [UNK]. [UNK]. (ka)"#;
        let expected = "[UNK] is a [UNK] / , foo. bar. . [UNK] .";
        let actual = preprocess_definition_for_vectors(input)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}

pub fn remove_html_tags(html: &str) -> String {
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)<[/?[a-z][^>]*>"#).unwrap());
    TAG_RE.replace_all(html, "").into_owned()
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
