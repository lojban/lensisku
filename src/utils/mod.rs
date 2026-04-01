pub mod embeddings;
pub mod kitten_tts;
pub mod kitten_tts_singleton;
pub mod lojban_ipa;
pub mod openrouter_models;
pub mod tersmu;

use actix_web::{HttpResponse, ResponseError};
use ammonia::Builder as AmmoniaBuilder;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use once_cell::sync::Lazy;
use std::collections::HashSet;

use crate::collections::models::{ImageData, SoundData};

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

/// Decode HTML entities so that LaTeX/MathJax validation sees literal characters.
/// E.g. `&lt;` → `<`, so that `$&lt;$` validates as math mode less-than instead of
/// causing "Misplaced alignment tab character &".
pub fn decode_html_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'&' {
            let start = i;
            i += 1;
            let mut name = String::new();
            let mut numeric_hex = false;
            let mut numeric_val: u32 = 0;
            let mut is_numeric = false;
            while i < bytes.len() && bytes[i] != b';' {
                let b = bytes[i];
                if b == b'#' && name.is_empty() {
                    is_numeric = true;
                    i += 1;
                    if i < bytes.len() && (bytes[i] == b'x' || bytes[i] == b'X') {
                        numeric_hex = true;
                        i += 1;
                    }
                    continue;
                }
                if is_numeric {
                    if numeric_hex {
                        if b.is_ascii_hexdigit() {
                            numeric_val = numeric_val * 16 + (b as char).to_digit(16).unwrap_or(0);
                        }
                    } else if b.is_ascii_digit() {
                        numeric_val = numeric_val * 10 + (b - b'0') as u32;
                    }
                } else if b.is_ascii_alphanumeric() {
                    name.push(b as char);
                }
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b';' {
                i += 1;
                let replacement: Option<char> = if is_numeric {
                    char::from_u32(numeric_val)
                } else {
                    match name.as_str() {
                        "lt" => Some('<'),
                        "gt" => Some('>'),
                        "amp" => Some('&'),
                        "quot" => Some('"'),
                        "apos" => Some('\''),
                        _ => None,
                    }
                };
                if let Some(c) = replacement {
                    out.push(c);
                } else {
                    out.push_str(&text[start..i]);
                }
            } else {
                out.push('&');
                if start + 1 < i {
                    out.push_str(&text[start + 1..i]);
                }
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
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

const ALLOWED_AUDIO_MIME_PREFIXES: &[&str] =
    &["audio/mpeg", "audio/mp3", "audio/ogg", "audio/webm"];

pub fn validate_item_audio(sound: &SoundData) -> Result<(), String> {
    let ok = ALLOWED_AUDIO_MIME_PREFIXES
        .iter()
        .any(|prefix| sound.mime_type.as_str().starts_with(prefix));
    if !ok {
        return Err(
            "Invalid audio type. Supported types: MP3, OGG, WEBM (WAV is not supported)."
                .to_string(),
        );
    }

    let decoded_size = BASE64
        .decode(&sound.data)
        .map_err(|_| "Invalid base64 data".to_string())?
        .len();

    if decoded_size > 5 * 1024 * 1024 {
        return Err("Audio size exceeds 5MB limit".to_string());
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

#[cfg(test)]
mod tests {
    use super::decode_html_entities;

    #[test]
    fn decode_html_entities_less_than() {
        assert_eq!(decode_html_entities("$&lt;$"), "$<$");
    }

    #[test]
    fn decode_html_entities_common() {
        assert_eq!(decode_html_entities("&lt;&gt;&amp;&quot;&apos;"), "<>&\"'");
    }

    #[test]
    fn decode_html_entities_numeric() {
        assert_eq!(decode_html_entities("&#60;"), "<");
        assert_eq!(decode_html_entities("&#x3C;"), "<");
    }

    #[test]
    fn decode_html_entities_unchanged() {
        assert_eq!(decode_html_entities("a & b"), "a & b");
        assert_eq!(decode_html_entities("$<$"), "$<$");
    }
}
