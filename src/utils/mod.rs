pub mod embeddings;
pub mod kitten_tts;
pub mod kitten_tts_singleton;
pub mod lojban_ipa;
pub mod openrouter_models;
pub mod canonical;

use actix_web::{HttpResponse, ResponseError};
use ammonia::Builder as AmmoniaBuilder;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use once_cell::sync::Lazy;
use std::collections::HashSet;

use crate::collections::models::{ImageData, SoundData};

/// Maximum decoded size for a collection item image (multipart upload and JSON import).
pub const MAX_ITEM_IMAGE_BYTES: usize = 8 * 1024 * 1024;

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
    let decoded = BASE64
        .decode(&image.data)
        .map_err(|_| "Invalid base64 data".to_string())?;
    validate_item_image_bytes(image.mime_type.as_str(), &decoded)
}

/// Raw image bytes + MIME (e.g. from multipart or ZIP); same rules as [`validate_item_image`].
pub fn validate_item_image_bytes(mime_type: &str, data: &[u8]) -> Result<(), String> {
    if !["image/jpeg", "image/png", "image/gif", "image/webp"].contains(&mime_type) {
        return Err("Invalid image type. Supported types: JPEG, PNG, GIF, WebP".to_string());
    }

    if data.len() > MAX_ITEM_IMAGE_BYTES {
        return Err(format!(
            "Image size exceeds {}MB limit",
            MAX_ITEM_IMAGE_BYTES / (1024 * 1024)
        ));
    }

    Ok(())
}

/// Infer MIME from magic bytes, then filename extension; used for bulk uploads without a trusted client Content-Type.
pub fn detect_image_mime_from_content(data: &[u8], filename_hint: &str) -> Result<String, String> {
    if data.len() >= 12 && &data[0..4] == b"RIFF" && data.len() >= 12 && &data[8..12] == b"WEBP" {
        return Ok("image/webp".to_string());
    }
    if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Ok("image/jpeg".to_string());
    }
    if data.len() >= 8 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
        return Ok("image/gif".to_string());
    }
    if data.len() >= 8 && data[0] == 0x89 && &data[1..4] == b"PNG" && &data[4..8] == b"\r\n\x1a\n" {
        return Ok("image/png".to_string());
    }

    let lower = filename_hint.to_lowercase();
    let ext = std::path::Path::new(&lower)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "jpg" | "jpeg" => Ok("image/jpeg".to_string()),
        "png" => Ok("image/png".to_string()),
        "gif" => Ok("image/gif".to_string()),
        "webp" => Ok("image/webp".to_string()),
        _ => Err(
            "Could not detect image type (unsupported or corrupt file); use JPEG, PNG, GIF, or WebP"
                .to_string(),
        ),
    }
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
