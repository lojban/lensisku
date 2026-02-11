use actix_web::{HttpResponse, ResponseError};
use ammonia::Builder as AmmoniaBuilder;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use once_cell::sync::Lazy;
use std::collections::HashSet;

use crate::collections::models::ImageData;

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

/// L2-normalize an embedding so cosine distance (e.g. pgvector <=>) matches
/// semantic-search MCP (Xenova/all-MiniLM-L6-v2 with normalize: true).
/// In-place; returns the same slice for convenience.
pub fn l2_normalize_embedding(embedding: &mut [f32]) {
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in embedding.iter_mut() {
            *x /= norm;
        }
    }
}
