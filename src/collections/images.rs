use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use deadpool_postgres::GenericClient;
use sha2::{Digest, Sha256};
use std::io::Cursor;

use crate::users::dto::ProfileImageRequest;
use crate::utils::{validate_item_image_bytes, MAX_ITEM_IMAGE_BYTES};
use crate::{AppError, AppResult};

/// Max decoded logo size (same order of magnitude as profile images).
pub(crate) const MAX_COLLECTION_LOGO_SIZE: usize = 5 * 1024 * 1024;

pub(crate) fn mime_base(mime: &str) -> &str {
    mime.split(';').next().unwrap_or(mime).trim()
}

/// Insert or reuse a row in `collection_images` (content-addressed by SHA-256 of stored bytes).
pub(crate) async fn get_or_insert_collection_image_id(
    client: &impl GenericClient,
    image_data: &[u8],
    mime_type: &str,
) -> AppResult<i32> {
    let (image_data, mime_type) = normalize_collection_image(image_data, mime_type)?;
    let image_data = image_data.as_slice();
    let mime_type = mime_type.as_str();
    let mut hasher = Sha256::new();
    hasher.update(image_data);
    let hash: Vec<u8> = hasher.finalize().to_vec();
    let row_opt = client
        .query_opt(
            "INSERT INTO collection_images (content_sha256, image_data, mime_type)
             VALUES ($1, $2, $3)
             ON CONFLICT (content_sha256) DO NOTHING
             RETURNING collection_image_id",
            &[&hash, &image_data, &mime_type],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    if let Some(row) = row_opt {
        return Ok(row.get("collection_image_id"));
    }
    let row = client
        .query_one(
            "SELECT collection_image_id FROM collection_images WHERE content_sha256 = $1",
            &[&hash],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(row.get("collection_image_id"))
}

/// Raster types go through WebP compression; SVG is stored as UTF-8 after light checks.
pub(crate) fn validate_collection_logo(req: &ProfileImageRequest) -> Result<Vec<u8>, String> {
    let decoded = BASE64
        .decode(&req.data)
        .map_err(|_| "Invalid base64 data".to_string())?;

    if decoded.len() > MAX_COLLECTION_LOGO_SIZE {
        return Err("Image size exceeds 5MB limit".to_string());
    }

    let mime = mime_base(&req.mime_type);
    if mime.eq_ignore_ascii_case("image/svg+xml") {
        validate_svg_logo(&decoded)?;
        return Ok(decoded);
    }

    if !["image/jpeg", "image/png", "image/webp"].contains(&mime) {
        return Err("Invalid image type. Supported types: JPEG, PNG, WebP, SVG".to_string());
    }

    Ok(decoded)
}

pub(crate) fn validate_svg_logo(data: &[u8]) -> Result<(), String> {
    use xml::reader::{EventReader, XmlEvent};
    let mut root_seen = false;
    for event in EventReader::new(data) {
        match event.map_err(|_| "Invalid SVG XML".to_string())? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if !root_seen {
                    if name.local_name != "svg"
                        || name.namespace.as_deref() != Some("http://www.w3.org/2000/svg")
                    {
                        return Err("Expected an SVG root".into());
                    }
                    root_seen = true;
                }
                if name.namespace.as_deref() != Some("http://www.w3.org/2000/svg")
                    || !matches!(
                        name.local_name.as_str(),
                        "svg"
                            | "g"
                            | "path"
                            | "rect"
                            | "circle"
                            | "ellipse"
                            | "line"
                            | "polyline"
                            | "polygon"
                            | "text"
                            | "tspan"
                            | "title"
                            | "desc"
                            | "defs"
                            | "linearGradient"
                            | "radialGradient"
                            | "stop"
                            | "clipPath"
                            | "mask"
                            | "use"
                            | "symbol"
                    )
                {
                    return Err("SVG contains unsupported active content".into());
                }
                for attr in attributes {
                    let key = attr.name.local_name.to_ascii_lowercase();
                    let value = attr.value.trim().to_ascii_lowercase();
                    if key.starts_with("on")
                        || key == "base"
                        || (key == "href" && !value.starts_with('#'))
                        || value.contains("javascript:")
                        || value.contains("data:")
                        || value.contains("http:")
                        || value.contains("https:")
                        || value.contains("@import")
                        || value.contains("\\")
                        || (value.contains("url(") && !value.contains("url(#"))
                    {
                        return Err("SVG contains external or executable references".into());
                    }
                }
            }
            XmlEvent::ProcessingInstruction { .. } => {
                return Err("SVG processing instructions are unsupported".into())
            }
            _ => {}
        }
    }
    if !root_seen {
        return Err("Expected an SVG document".into());
    }
    Ok(())
}

pub(crate) fn normalize_collection_image(data: &[u8], mime: &str) -> AppResult<(Vec<u8>, String)> {
    if data.len() > MAX_ITEM_IMAGE_BYTES {
        return Err(AppError::BadRequest("Image is too large".into()));
    }
    if mime_base(mime).eq_ignore_ascii_case("image/svg+xml") {
        validate_svg_logo(data).map_err(AppError::BadRequest)?;
        return Ok((data.to_vec(), "image/svg+xml".into()));
    }
    validate_item_image_bytes(mime_base(mime), data).map_err(AppError::BadRequest)?;
    let mut reader = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|_| AppError::BadRequest("Invalid image".into()))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| AppError::BadRequest("Invalid or oversized image".into()))?;
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::WebP)
        .map_err(|_| AppError::BadRequest("Image encoding failed".into()))?;
    let bytes = output.into_inner();
    if bytes.len() > MAX_ITEM_IMAGE_BYTES {
        return Err(AppError::BadRequest("Image is too large".into()));
    }
    Ok((bytes, "image/webp".into()))
}
