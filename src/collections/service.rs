use super::dto::SkippedItemInfo;
use super::dto::*;
use crate::jbovlaste::service::get_valsi_sound_urls_from_db;
use crate::utils::remove_html_tags;
use crate::{
    auth_utils::verify_collection_ownership, export::models::CollectionExportItem,
    flashcards::models::FlashcardDirection, utils::validate_item_audio, utils::validate_item_image,
    AppError, AppResult,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use deadpool_postgres::{Pool, Transaction};
use std::collections::HashSet;

pub async fn create_collection(
    pool: &Pool,
    user_id: i32,
    req: &CreateCollectionRequest,
) -> AppResult<CollectionResponse> {
    let sanitized_name = sanitize_html(&req.name);
    let sanitized_description = req.description.as_ref().map(|d| sanitize_html(d));

    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let row = transaction
        .query_one(
            "INSERT INTO collections (user_id, name, description, is_public)
             VALUES ($1, $2, $3, $4)
             RETURNING collection_id, created_at, updated_at",
            &[
                &user_id,
                &sanitized_name,
                &sanitized_description,
                &req.is_public.unwrap_or(true),
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let username = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = CollectionResponse {
        collection_id: row.get("collection_id"),
        name: sanitized_name,
        description: sanitized_description,
        is_public: req.is_public.unwrap_or(true),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        item_count: 0,
        owner: CollectionOwner { user_id, username },
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(response)
}

pub fn sanitize_html(html: &str) -> String {
    remove_html_tags(html)
}

pub async fn list_collections(pool: &Pool, user_id: i32) -> AppResult<CollectionListResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let rows = client
        .query(
            "SELECT c.*, u.username, 
                    (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.collection_id) as item_count
             FROM collections c
             JOIN users u ON c.user_id = u.userid
             WHERE c.user_id = $1
             ORDER BY c.updated_at DESC",
            &[&user_id],
        )
        .await.map_err(|e| AppError::Database(e.to_string()))?;

    let collections = rows
        .iter()
        .map(|row| CollectionResponse {
            collection_id: row.get("collection_id"),
            name: row.get("name"),
            description: row.get("description"),
            is_public: row.get("is_public"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            item_count: row.get("item_count"),
            owner: CollectionOwner {
                user_id,
                username: row
                    .try_get("username")
                    .unwrap_or_else(|_| "unknown".to_string()),
            },
        })
        .collect();

    Ok(CollectionListResponse {
        collections,
        total: rows.len() as i64,
    })
}

pub async fn list_public_collections(pool: &Pool) -> AppResult<CollectionListResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let rows = client
        .query(
            "SELECT c.*, u.userid, u.username, 
                    (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.collection_id) as item_count
             FROM collections c
             JOIN users u ON c.user_id = u.userid
             WHERE c.is_public = true
             ORDER BY c.updated_at DESC",
            &[],
        )
        .await.map_err(|e| AppError::Database(e.to_string()))?;

    let collections = rows
        .iter()
        .map(|row| CollectionResponse {
            collection_id: row.get("collection_id"),
            name: row.get("name"),
            description: row.get("description"),
            is_public: row.get("is_public"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            item_count: row.get("item_count"),
            owner: CollectionOwner {
                user_id: row.get("userid"),
                username: row
                    .try_get("username")
                    .unwrap_or_else(|_| "unknown".to_string()),
            },
        })
        .collect();

    Ok(CollectionListResponse {
        collections,
        total: rows.len() as i64,
    })
}

pub async fn get_collection(
    pool: &Pool,
    collection_id: i32,
    user_id: Option<i32>,
) -> AppResult<CollectionResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get collection details
    let collection_row = client
    .query_one(
        "SELECT c.*, u.userid, u.username, 
        (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.collection_id) as item_count
        FROM collections c
        JOIN users u ON c.user_id = u.userid
             WHERE c.collection_id = $1",
            &[&collection_id],
        )
        .await.map_err(|e| AppError::Database(e.to_string()))?;

    let is_public: bool = collection_row.get("is_public");
    let owner_id: i32 = collection_row.get("user_id");

    // Check access
    if !is_public && Some(owner_id) != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    Ok(CollectionResponse {
        collection_id,
        name: collection_row.get("name"),
        description: collection_row.get("description"),
        is_public,
        created_at: collection_row.get("created_at"),
        updated_at: collection_row.get("updated_at"),
        item_count: collection_row.get("item_count"),
        owner: CollectionOwner {
            user_id: owner_id,
            username: collection_row
                .try_get("username")
                .map_err(|e| AppError::Database(e.to_string()))?,
        },
    })
}

pub async fn update_collection(
    pool: &Pool,
    collection_id: i32,
    user_id: i32,
    req: &UpdateCollectionRequest,
) -> AppResult<CollectionResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let sanitized_name = req.name.as_ref().map(|n| sanitize_html(n));
    let sanitized_description = req.description.as_ref().map(|d| sanitize_html(d));

    // Update collection
    let row = transaction
        .query_one(
            "UPDATE collections 
             SET name = COALESCE($1, name),
                 description = COALESCE($2, description),
                 is_public = COALESCE($3, is_public),
                 updated_at = $4
             WHERE collection_id = $5
             RETURNING *",
            &[
                &sanitized_name,
                &sanitized_description,
                &req.is_public,
                &Utc::now(),
                &collection_id,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let username = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item_count = transaction
        .query_one(
            "SELECT COUNT(*) FROM collection_items WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get::<_, i64>(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(CollectionResponse {
        collection_id,
        name: row.get("name"),
        description: row.get("description"),
        is_public: row.get("is_public"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        item_count,
        owner: CollectionOwner { user_id, username },
    })
}

pub async fn delete_collection(pool: &Pool, collection_id: i32, user_id: i32) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // Delete in dependency order: flashcards reference collection_items, so delete flashcard
    // data first, then levels, then items, then the collection.

    // 1. Review history references flashcards without CASCADE
    transaction
        .execute(
            "DELETE FROM flashcard_review_history
             WHERE flashcard_id IN (SELECT id FROM flashcards WHERE collection_id = $1)",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 2. Flashcards (CASCADEs: user_flashcard_progress, flashcard_level_items)
    transaction
        .execute(
            "DELETE FROM flashcards WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 3. Levels (CASCADEs: level_prerequisites, flashcard_level_items, user_level_progress)
    transaction
        .execute(
            "DELETE FROM flashcard_levels WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 4. Collection items (CASCADEs: collection_item_images, etc.)
    transaction
        .execute(
            "DELETE FROM collection_items WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // 5. Collection
    transaction
        .execute(
            "DELETE FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

async fn mark_progress_graduated(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> AppResult<()> {
    transaction
        .execute(
            "UPDATE user_flashcard_progress SET status = 'graduated', next_review_at = NULL
         WHERE user_id = $1 AND flashcard_id = $2 AND card_side = $3 AND NOT archived",
            &[&user_id, &flashcard_id, &side],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn import_json(
    pool: &Pool,
    user_id: i32,
    req: &ImportJsonRequest,
) -> AppResult<ImportJsonResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Create the collection
    let row = transaction
        .query_one(
            "INSERT INTO collections (user_id, name, description, is_public)
             VALUES ($1, $2, $3, $4)
             RETURNING collection_id, created_at, updated_at",
            &[
                &user_id,
                &req.name,
                &req.description,
                &req.is_public.unwrap_or(true),
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection_id: i32 = row.get("collection_id");
    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut warnings = Vec::new();

    // Process each item
    for item in &req.items {
        if let Some(def_id) = item.definition_id {
            // Verify definition exists
            let exists = transaction
                .query_one(
                    "SELECT EXISTS(SELECT 1 FROM definitions WHERE definitionid = $1)",
                    &[&def_id],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
                .try_get::<_, bool>(0)
                .map_err(|e| AppError::Database(e.to_string()))?;

            if !exists {
                warnings.push(format!(
                    "Definition ID {} not found for word '{}'",
                    def_id, item.word
                ));
                skipped_count += 1;
                continue;
            }

            // Get current max position
            let max_position: i32 = transaction
                .query_one(
                    "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
                    &[&collection_id],
                )
                .await.map_err(|e| AppError::Database(e.to_string()))?
                .try_get(0).map_err(|e| AppError::Database(e.to_string()))?;

            let canonical_form = crate::tersmu::get_canonical_form(&item.word);

            // Add item
            transaction
                .execute(
                    "INSERT INTO collection_items (collection_id, definition_id, notes, position, canonical_form)
                     VALUES ($1, $2, $3, $4, $5)",
                    &[
                        &collection_id,
                        &def_id,
                        &item.collection_note,
                        &(max_position + 1),
                        &canonical_form,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            imported_count += 1;
        } else {
            warnings.push(format!(
                "Skipping word '{}' - no definition ID provided",
                item.word
            ));
            skipped_count += 1;
        }
    }

    let username = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection = CollectionResponse {
        collection_id,
        name: req.name.clone(),
        description: req.description.clone(),
        is_public: req.is_public.unwrap_or(true),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        item_count: imported_count as i64,
        owner: CollectionOwner { user_id, username },
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(ImportJsonResponse {
        collection,
        imported_count,
        skipped_count,
        warnings,
    })
}

// Helper function to parse data URL and decode base64
fn decode_data_url(url: &str) -> AppResult<(String, Vec<u8>)> {
    if !url.starts_with("data:") {
        return Err(AppError::BadRequest("Invalid data URL format".to_string()));
    }
    let parts: Vec<&str> = url[5..].splitn(2, ';').collect();
    if parts.len() != 2 || parts[1].splitn(2, ',').count() != 2 {
        return Err(AppError::BadRequest("Invalid data URL format".to_string()));
    }
    let mime_type = parts[0].to_string();
    let data_part = parts[1].splitn(2, ',').nth(1).unwrap_or("");

    if !parts[1].starts_with("base64,") {
        return Err(AppError::BadRequest(
            "Only base64 encoded data URLs are supported".to_string(),
        ));
    }

    let decoded = BASE64
        .decode(data_part)
        .map_err(|e| AppError::BadRequest(format!("Invalid base64 data: {}", e)))?;

    Ok((mime_type, decoded))
}

pub async fn import_collection_from_json(
    pool: &Pool,
    target_collection_id: i32,
    user_id: i32,
    items: &[CollectionExportItem],
) -> AppResult<ImportCollectionJsonResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Verify ownership of the target collection
    verify_collection_ownership(&transaction, target_collection_id, user_id).await?;

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut skipped_items = Vec::new();

    // Get current max position in the target collection
    let mut current_max_position: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
            &[&target_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    for item in items {
        let mut skip_reason: Option<String> = None;
        let identifier: String;

        // Check for conflicts
        if let Some(def_id) = item.definition_id {
            identifier = format!("definition_id: {}", def_id);
            let exists = transaction
                .query_one(
                    "SELECT EXISTS(SELECT 1 FROM collection_items WHERE collection_id = $1 AND definition_id = $2)",
                    &[&target_collection_id, &def_id],
                )
                .await.map_err(|e| AppError::Database(e.to_string()))?
                .try_get::<_, bool>(0).map_err(|e| AppError::Database(e.to_string()))?;
            if exists {
                skip_reason = Some("Definition already exists in target collection".to_string());
            }
        } else if let (Some(front), Some(back)) =
            (&item.free_content_front, &item.free_content_back)
        {
            identifier = format!(
                "free_content_front: {}",
                front.chars().take(30).collect::<String>()
            );
            let exists = transaction
                .query_one(
                    "SELECT EXISTS(SELECT 1 FROM collection_items WHERE collection_id = $1 AND free_content_front = $2 AND free_content_back = $3)",
                    &[&target_collection_id, front, back],
                )
                .await.map_err(|e| AppError::Database(e.to_string()))?
                .try_get::<_, bool>(0).map_err(|e| AppError::Database(e.to_string()))?;
            if exists {
                skip_reason =
                    Some("Free content item already exists in target collection".to_string());
            }
        } else {
            // Invalid item format in export
            identifier = format!("item_id: {}", item.item_id); // Use item_id from export for identification
            skip_reason =
                Some("Invalid item format (missing definition_id or free content)".to_string());
        }

        if let Some(reason) = skip_reason {
            skipped_count += 1;
            skipped_items.push(SkippedItemInfo { identifier, reason });
            continue;
        }

        // Insert the item
        current_max_position += 1;
        let sanitized_front = item.free_content_front.as_ref().map(|f| sanitize_html(f));
        let sanitized_back = item.free_content_back.as_ref().map(|b| sanitize_html(b));
        let sanitized_note = item.collection_note.as_ref().map(|n| sanitize_html(n));

        let canonical_form = sanitized_front
            .as_ref()
            .and_then(|front| crate::tersmu::get_canonical_form(front))
            .or_else(|| {
                item.word
                    .as_ref()
                    .and_then(|w| crate::tersmu::get_canonical_form(w))
            });

        let new_item_id: i32 = transaction
            .query_one(
                "INSERT INTO collection_items (collection_id, definition_id, free_content_front, free_content_back, notes, position, canonical_form)
                 VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING item_id",
                &[
                    &target_collection_id,
                    &item.definition_id,
                    &sanitized_front,
                    &sanitized_back,
                    &sanitized_note,
                    &current_max_position,
                    &canonical_form,
                ],
            )
            .await.map_err(|e| AppError::Database(e.to_string()))?
            .try_get(0).map_err(|e| AppError::Database(e.to_string()))?;

        // Handle images
        for (side, url_option) in [
            ("front", &item.front_image_url),
            ("back", &item.back_image_url),
        ] {
            if let Some(url) = url_option {
                let (mime_type, image_data) = decode_data_url(url)?;
                // Basic validation (could reuse validate_item_image if needed)
                if image_data.len() > 5 * 1024 * 1024 {
                    return Err(AppError::BadRequest(format!(
                        "Image for item {} ({}) exceeds 5MB limit",
                        new_item_id, side
                    )));
                }
                transaction.execute(
                    "INSERT INTO collection_item_images (item_id, image_data, mime_type, side) VALUES ($1, $2, $3, $4)",
                    &[&new_item_id, &image_data, &mime_type, &side]
                ).await.map_err(|e| AppError::Database(e.to_string()))?;
            }
        }

        imported_count += 1;
    }

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(ImportCollectionJsonResponse {
        imported_count,
        skipped_count,
        skipped_items,
    })
}

/// Parse direction string from export; defaults to Both if missing or invalid.
fn parse_export_direction(s: Option<&String>) -> FlashcardDirection {
    let s = match s {
        Some(x) => x.to_lowercase(),
        None => return FlashcardDirection::Both,
    };
    match s.as_str() {
        "direct" => FlashcardDirection::Direct,
        "reverse" => FlashcardDirection::Reverse,
        "both" => FlashcardDirection::Both,
        "fillin" => FlashcardDirection::FillIn,
        "fillin_reverse" => FlashcardDirection::FillInReverse,
        "fillin_both" => FlashcardDirection::FillInBoth,
        "just_information" => FlashcardDirection::JustInformation,
        "quiz_direct" => FlashcardDirection::QuizDirect,
        "quiz_reverse" => FlashcardDirection::QuizReverse,
        "quiz_both" => FlashcardDirection::QuizBoth,
        _ => FlashcardDirection::Both,
    }
}

/// Full import: create collection, items, then (if levels present) flashcards and levels.
pub async fn import_full(
    pool: &Pool,
    user_id: i32,
    req: &ImportFullRequest,
) -> AppResult<ImportFullResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let sanitized_name = sanitize_html(&req.collection.name);
    let sanitized_description = req
        .collection
        .description
        .as_ref()
        .map(|d| sanitize_html(d));
    let is_public = req.collection.is_public.unwrap_or(true);

    let row = transaction
        .query_one(
            "INSERT INTO collections (user_id, name, description, is_public)
             VALUES ($1, $2, $3, $4)
             RETURNING collection_id, created_at, updated_at",
            &[
                &user_id,
                &sanitized_name,
                &sanitized_description,
                &is_public,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection_id: i32 = row.get("collection_id");
    let mut imported_count = 0i32;
    let mut skipped_count = 0i32;
    let mut warnings: Vec<String> = Vec::new();
    // For each export index, Some(item_id) if we inserted that item, None if skipped
    let mut inserted_item_ids_by_export_index: Vec<Option<i32>> =
        Vec::with_capacity(req.items.len());

    for (pos, item) in req.items.iter().enumerate() {
        let has_definition = item.definition_id.is_some();
        let has_free_content =
            item.free_content_front.is_some() && item.free_content_back.is_some();

        if !has_definition && !has_free_content {
            warnings.push(format!(
                "Item at position {}: missing definition_id and free content, skipped",
                pos
            ));
            skipped_count += 1;
            inserted_item_ids_by_export_index.push(None);
            continue;
        }

        if has_definition {
            let def_id = item.definition_id.unwrap();
            let exists = transaction
                .query_one(
                    "SELECT EXISTS(SELECT 1 FROM definitions WHERE definitionid = $1)",
                    &[&def_id],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
                .try_get::<_, bool>(0)
                .map_err(|e| AppError::Database(e.to_string()))?;
            if !exists {
                warnings.push(format!(
                    "Definition ID {} not found for word '{:?}', skipped",
                    def_id, item.word
                ));
                skipped_count += 1;
                inserted_item_ids_by_export_index.push(None);
                continue;
            }
        }

        let sanitized_front = item.free_content_front.as_ref().map(|f| sanitize_html(f));
        let sanitized_back = item.free_content_back.as_ref().map(|b| sanitize_html(b));
        let sanitized_note = item.collection_note.as_ref().map(|n| sanitize_html(n));
        let canonical_form = sanitized_front
            .as_ref()
            .and_then(|f| crate::tersmu::get_canonical_form(f))
            .or_else(|| {
                item.word
                    .as_ref()
                    .and_then(|w| crate::tersmu::get_canonical_form(w))
            });

        let new_item_id: i32 = transaction
            .query_one(
                "INSERT INTO collection_items (collection_id, definition_id, free_content_front, free_content_back, notes, position, canonical_form)
                 VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING item_id",
                &[
                    &collection_id,
                    &item.definition_id,
                    &sanitized_front,
                    &sanitized_back,
                    &sanitized_note,
                    &(pos as i32),
                    &canonical_form,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .try_get(0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        for (side, url_option) in [
            ("front", &item.front_image_url),
            ("back", &item.back_image_url),
        ] {
            if let Some(url) = url_option {
                if let Ok((mime_type, image_data)) = decode_data_url(url) {
                    if image_data.len() <= 5 * 1024 * 1024 {
                        let _ = transaction
                            .execute(
                                "INSERT INTO collection_item_images (item_id, image_data, mime_type, side) VALUES ($1, $2, $3, $4)",
                                &[&new_item_id, &image_data, &mime_type, &side],
                            )
                            .await;
                    }
                }
            }
        }

        inserted_item_ids_by_export_index.push(Some(new_item_id));
        imported_count += 1;
    }

    // When no explicit levels array is provided, infer levels from per-item level_index/position_in_level
    let levels_to_use: Vec<LevelExport> = if !req.levels.is_empty() {
        req.levels.clone()
    } else {
        let mut level_to_items: std::collections::BTreeMap<u32, Vec<(usize, i32)>> =
            std::collections::BTreeMap::new();
        for (item_idx, item) in req.items.iter().enumerate() {
            if let Some(li) = item.level_index {
                let pos = item.position_in_level.unwrap_or(i32::MAX);
                level_to_items.entry(li).or_default().push((item_idx, pos));
            }
        }
        level_to_items
            .into_iter()
            .enumerate()
            .map(|(pos_index, (level_index, mut items))| {
                items.sort_by_key(|&(idx, pos)| (pos, idx));
                // Chain prerequisites: level 0 has none, level 1 requires 0, level 2 requires 1, etc.
                let prerequisite_positions: Vec<usize> = if pos_index == 0 {
                    vec![]
                } else {
                    vec![pos_index - 1]
                };
                LevelExport {
                    name: format!("Level {}", level_index + 1),
                    description: None,
                    min_cards: 5,
                    min_success_rate: 0.8,
                    position: level_index as i32,
                    prerequisite_positions,
                    item_positions: items.into_iter().map(|(idx, _)| idx).collect(),
                }
            })
            .collect()
    };

    let mut levels_created = 0i32;
    if !levels_to_use.is_empty() {
        // For each export index, Some(flashcard_id) if we have a flashcard for that item, None if skipped
        let mut flashcard_id_by_export_index: Vec<Option<i32>> =
            Vec::with_capacity(inserted_item_ids_by_export_index.len());
        for (idx, item_id_opt) in inserted_item_ids_by_export_index.iter().enumerate() {
            let flashcard_id_opt = match *item_id_opt {
                Some(item_id) => {
                    let direction = parse_export_direction(
                        req.items.get(idx).and_then(|i| i.direction.as_ref()),
                    );
                    let flashcard_id: i32 = transaction
                        .query_one(
                            "INSERT INTO flashcards (collection_id, position, item_id, direction)
                     VALUES ($1, $2, $3, $4)
                     RETURNING id",
                            &[&collection_id, &(idx as i32), &item_id, &direction],
                        )
                        .await
                        .map_err(|e| AppError::Database(e.to_string()))?
                        .try_get(0)
                        .map_err(|e| AppError::Database(e.to_string()))?;
                    initialize_flashcard_progress(&transaction, user_id, flashcard_id, "direct")
                        .await?;
                    initialize_flashcard_progress(&transaction, user_id, flashcard_id, "reverse")
                        .await?;
                    Some(flashcard_id)
                }
                None => None,
            };
            flashcard_id_by_export_index.push(flashcard_id_opt);
        }

        let mut new_level_ids: Vec<i32> = Vec::with_capacity(levels_to_use.len());
        for level in &levels_to_use {
            let level_id: i32 = transaction
                .query_one(
                    "INSERT INTO flashcard_levels (collection_id, name, description, min_cards, min_success_rate, position)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     RETURNING level_id",
                    &[
                        &collection_id,
                        &level.name,
                        &level.description,
                        &level.min_cards,
                        &(level.min_success_rate as f64),
                        &level.position,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
                .try_get(0)
                .map_err(|e| AppError::Database(e.to_string()))?;
            new_level_ids.push(level_id);
            levels_created += 1;
        }

        for (level_idx, level) in levels_to_use.iter().enumerate() {
            let level_id = new_level_ids[level_idx];
            for &prereq_idx in &level.prerequisite_positions {
                if prereq_idx < new_level_ids.len() {
                    let prereq_id = new_level_ids[prereq_idx];
                    if level_id != prereq_id {
                        let _ = transaction
                            .execute(
                                "INSERT INTO level_prerequisites (level_id, prerequisite_id) VALUES ($1, $2)
                                 ON CONFLICT (level_id, prerequisite_id) DO NOTHING",
                                &[&level_id, &prereq_id],
                            )
                            .await;
                    }
                }
            }
            for (pos_in_level, &item_idx) in level.item_positions.iter().enumerate() {
                if let Some(Some(flashcard_id)) = flashcard_id_by_export_index.get(item_idx) {
                    let _ = transaction
                        .execute(
                            "INSERT INTO flashcard_level_items (level_id, flashcard_id, position)
                             VALUES ($1, $2, $3)
                             ON CONFLICT (level_id, flashcard_id) DO UPDATE SET position = EXCLUDED.position",
                            &[&level_id, &flashcard_id, &(pos_in_level as i32)],
                        )
                        .await;
                }
            }
        }
    }

    let username: String = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let collection_resp = CollectionResponse {
        collection_id,
        name: sanitized_name,
        description: sanitized_description,
        is_public,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        item_count: imported_count as i64,
        owner: CollectionOwner { user_id, username },
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(ImportFullResponse {
        collection: collection_resp,
        imported_count,
        skipped_count,
        levels_created,
        warnings,
    })
}

/// Full collection export: collection metadata, items (with flashcard direction when present), and levels.
pub async fn export_collection_full(
    pool: &Pool,
    collection_id: i32,
    user_id: Option<i32>,
) -> AppResult<CollectionFullExport> {
    let collection_resp = get_collection(pool, collection_id, user_id).await?;
    let collection_meta = CollectionExportMeta {
        name: collection_resp.name,
        description: collection_resp.description,
        is_public: Some(collection_resp.is_public),
    };

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item_rows = client
        .query(
            "SELECT
                ci.item_id, ci.definition_id, ci.notes as collection_note, ci.position,
                ci.free_content_front, ci.free_content_back,
                ci.langid as language_id, ci.owner_user_id, ci.license,
                v.word, d.definition, d.notes as definition_notes, d.jargon, t.descriptor as word_type,
                c.rafsi, c.selmaho,
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as front_image_mime,
                (SELECT image_data FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_data,
                (SELECT mime_type FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as back_image_mime,
                f.direction::text as flashcard_direction
            FROM collection_items ci
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN valsitypes t ON v.typeid = t.typeid
            LEFT JOIN convenientdefinitions c ON c.definitionid = d.definitionid
            LEFT JOIN flashcards f ON f.item_id = ci.item_id AND f.collection_id = ci.collection_id
            WHERE ci.collection_id = $1
            ORDER BY ci.position",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut item_id_to_index: std::collections::HashMap<i32, usize> =
        std::collections::HashMap::new();
    let items: Vec<CollectionExportItem> = item_rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let item_id: i32 = row.get("item_id");
            item_id_to_index.insert(item_id, idx);
            let front_image_url =
                row.get::<_, Option<Vec<u8>>>("front_image_data")
                    .and_then(|data| {
                        row.get::<_, Option<String>>("front_image_mime")
                            .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(&data)))
                    });
            let back_image_url =
                row.get::<_, Option<Vec<u8>>>("back_image_data")
                    .and_then(|data| {
                        row.get::<_, Option<String>>("back_image_mime")
                            .map(|mime| format!("data:{};base64,{}", mime, BASE64.encode(&data)))
                    });
            let direction: Option<String> = row.get("flashcard_direction");
            CollectionExportItem {
                item_id: row.get("item_id"),
                position: row.get("position"),
                collection_note: row.get("collection_note"),
                definition_id: row.get("definition_id"),
                language_id: row.get("language_id"),
                owner_user_id: row.get("owner_user_id"),
                license: row.get("license"),
                word: row.get("word"),
                word_type: row.get("word_type"),
                rafsi: row.get("rafsi"),
                selmaho: row.get("selmaho"),
                definition: row.get("definition"),
                definition_notes: row.get("definition_notes"),
                jargon: row.get("jargon"),
                free_content_front: row.get("free_content_front"),
                free_content_back: row.get("free_content_back"),
                front_image_url,
                back_image_url,
                direction,
                level_index: None,
                position_in_level: None,
            }
        })
        .collect();

    let level_rows = client
        .query(
            "SELECT level_id, name, description, min_cards, min_success_rate, position
             FROM flashcard_levels
             WHERE collection_id = $1
             ORDER BY position",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let level_ids: Vec<i32> = level_rows.iter().map(|r| r.get("level_id")).collect();
    let level_id_to_index: std::collections::HashMap<i32, usize> = level_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    let mut levels: Vec<LevelExport> = Vec::with_capacity(level_rows.len());
    for row in &level_rows {
        let level_id: i32 = row.get("level_id");
        let prerequisite_ids: Vec<i32> = client
            .query(
                "SELECT prerequisite_id FROM level_prerequisites WHERE level_id = $1",
                &[&level_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .iter()
            .map(|r| r.get::<_, i32>("prerequisite_id"))
            .collect();
        let prerequisite_positions: Vec<usize> = prerequisite_ids
            .iter()
            .filter_map(|&pid| level_id_to_index.get(&pid).copied())
            .collect();

        let fli_rows = client
            .query(
                "SELECT fli.flashcard_id, fli.position
                 FROM flashcard_level_items fli
                 WHERE fli.level_id = $1
                 ORDER BY fli.position",
                &[&level_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut item_positions_for_level: Vec<usize> = Vec::with_capacity(fli_rows.len());
        for fli_row in &fli_rows {
            let flashcard_id: i32 = fli_row.get("flashcard_id");
            let item_id: i32 = client
                .query_one(
                    "SELECT item_id FROM flashcards WHERE id = $1",
                    &[&flashcard_id],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
                .get("item_id");
            if let Some(&idx) = item_id_to_index.get(&item_id) {
                item_positions_for_level.push(idx);
            }
        }

        levels.push(LevelExport {
            name: row.get("name"),
            description: row.get("description"),
            min_cards: row.get("min_cards"),
            min_success_rate: row.get::<_, f64>("min_success_rate") as f32,
            position: row.get("position"),
            prerequisite_positions,
            item_positions: item_positions_for_level,
        });
    }

    Ok(CollectionFullExport {
        collection: collection_meta,
        items,
        levels,
    })
}

// Helper function to initialize flashcard progress
async fn initialize_flashcard_progress(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> AppResult<()> {
    transaction
        .execute(
            "INSERT INTO user_flashcard_progress
             (user_id, flashcard_id, card_side, status, next_review_at)
             VALUES ($1, $2, $3, 'new', CURRENT_TIMESTAMP)
             ON CONFLICT (user_id, flashcard_id, card_side) WHERE NOT archived DO NOTHING",
            &[&user_id, &flashcard_id, &side],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn upsert_item(
    pool: &Pool,
    collection_id: i32,
    user_id: i32,
    req: &AddItemRequest,
) -> AppResult<CollectionItemResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection ownership
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Validate images if present
    if let Some(img) = &req.front_image {
        validate_item_image(img).map_err(|e| AppError::BadRequest(e.to_string()))?;
    }
    if let Some(img) = &req.back_image {
        validate_item_image(img).map_err(|e| AppError::BadRequest(e.to_string()))?;
    }

    // Get highest current position
    let max_position: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let sanitized_notes = req.notes.as_ref().map(|n| sanitize_html(n));
    let sanitized_front = req.free_content_front.as_ref().map(|f| sanitize_html(f));
    let sanitized_back = req.free_content_back.as_ref().map(|b| sanitize_html(b));

    // Check if item exists either by specified ID or definition ID
    let existing_item = if let Some(item_id) = req.item_id {
        transaction
            .query_opt(
                "SELECT item_id, notes, added_at, position 
                 FROM collection_items 
                 WHERE collection_id = $1 AND item_id = $2",
                &[&collection_id, &item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else if let Some(def_id) = req.definition_id {
        transaction
            .query_opt(
                "SELECT item_id, notes, added_at, position 
                 FROM collection_items 
                 WHERE collection_id = $1 AND definition_id = $2",
                &[&collection_id, &def_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        None
    };

    // Use explicit position, or when updating existing item keep its current position; otherwise append
    let position = req
        .position
        .or_else(|| {
            existing_item
                .as_ref()
                .map(|row| row.get::<_, i32>("position"))
        })
        .unwrap_or(max_position + 1);

    // Validate item_id exists if provided
    if req.item_id.is_some() && existing_item.is_none() {
        return Err(AppError::NotFound("Item not found".to_string()));
    }

    // Create or update item
    let mut canonical_form = sanitized_front
        .as_ref()
        .and_then(|front| crate::tersmu::get_canonical_form(front));

    // For dictionary items without free content, try to use the word from the definition
    if canonical_form.is_none() {
        if let Some(def_id) = req.definition_id {
            if let Ok(row) = transaction.query_one(
                "SELECT v.word FROM definitions d JOIN valsi v ON d.valsiid = v.valsiid WHERE d.definitionid = $1",
                &[&def_id]
            ).await {
                let word: String = row.get(0);
                canonical_form = crate::tersmu::get_canonical_form(&word);
            }
        }
    }

    let (item_id, notes, added_at): (i32, Option<String>, DateTime<Utc>) =
        if let Some(row) = existing_item {
            let item_id: i32 = row.get("item_id");
            // Update existing item
            let old_position: i32 = row.get("position");

            if position != old_position {
                // Shift items if position changed
                let (start, end, shift) = if position > old_position {
                    (old_position + 1, position + 1, -1)
                } else {
                    (position, old_position, 1)
                };

                transaction
                    .execute(
                        "UPDATE collection_items 
                     SET position = position + $1
                     WHERE collection_id = $2 
                     AND position >= $3 AND position < $4
                     AND item_id != $5",
                        &[
                            &shift as &(dyn tokio_postgres::types::ToSql + Sync),
                            &collection_id,
                            &start,
                            &end,
                            &item_id,
                        ],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }

            transaction
                .execute(
                    "UPDATE collection_items 
                 SET notes = $1, position = $2, canonical_form = $3,
                     free_content_front = $4, free_content_back = $5
                 WHERE item_id = $6",
                    &[
                        &sanitized_notes,
                        &position,
                        &canonical_form,
                        &sanitized_front,
                        &sanitized_back,
                        &item_id,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            (
                item_id,
                sanitized_notes,
                row.get::<_, DateTime<Utc>>("added_at"),
            )
        } else {
            // Add new item
            let row = transaction
                .query_one(
                    "INSERT INTO collection_items (
                    collection_id, definition_id, 
                    free_content_front, free_content_back, 
                    langid, owner_user_id, license, script, is_original,
                    notes, position, auto_progress, canonical_form
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING item_id, added_at",
                    &[
                        &collection_id,
                        &req.definition_id,
                        &sanitized_front,
                        &sanitized_back,
                        &req.language_id,
                        &req.owner_user_id,
                        &req.license,
                        &req.script,
                        &req.is_original.unwrap_or(true),
                        &sanitized_notes,
                        &position,
                        &req.auto_progress.unwrap_or(true),
                        &canonical_form,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            (
                row.get::<_, i32>("item_id"),
                sanitized_notes,
                row.get::<_, DateTime<Utc>>("added_at"),
            )
        };

    // Handle front image
    if let Some(image) = &req.front_image {
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid front image base64: {}", e)))?;
        transaction
            .execute(
                "INSERT INTO collection_item_images (item_id, image_data, mime_type, side)
                 VALUES ($1, $2, $3, 'front')
                 ON CONFLICT (item_id, side) DO UPDATE SET
                   image_data = EXCLUDED.image_data,
                   mime_type = EXCLUDED.mime_type",
                &[&item_id, &image_data, &image.mime_type],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle back image
    if let Some(image) = &req.back_image {
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid back image base64: {}", e)))?;
        transaction
            .execute(
                "INSERT INTO collection_item_images (item_id, image_data, mime_type, side)
                 VALUES ($1, $2, $3, 'back')
                 ON CONFLICT (item_id, side) DO UPDATE SET
                   image_data = EXCLUDED.image_data,
                   mime_type = EXCLUDED.mime_type",
                &[&item_id, &image_data, &image.mime_type],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle sound: overwrite (delete then insert) or remove
    if req.remove_sound.unwrap_or(false) || req.sound.is_some() {
        transaction
            .execute(
                "DELETE FROM collection_item_sounds WHERE item_id = $1",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }
    if let Some(sound) = &req.sound {
        validate_item_audio(sound).map_err(|e| AppError::BadRequest(e.to_string()))?;
        let sound_data = BASE64
            .decode(&sound.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid sound base64: {}", e)))?;
        transaction
            .execute(
                "INSERT INTO collection_item_sounds (item_id, sound_data, mime_type)
                 VALUES ($1, $2, $3)",
                &[&item_id, &sound_data, &sound.mime_type],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle flashcard creation/update
    if req.direction.is_some() {
        // Get existing flashcard if it exists
        let existing_flashcard = transaction
            .query_opt(
                "SELECT id, direction FROM flashcards 
                 WHERE collection_id = $1 AND item_id = $2",
                &[&collection_id, &item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Parse requested direction
        let direction = if let Some(dir_str) = &req.direction {
            match dir_str.to_lowercase().as_str() {
                "direct" => FlashcardDirection::Direct,
                "reverse" => FlashcardDirection::Reverse,
                "fillin" => FlashcardDirection::FillIn,
                "fillin_reverse" => FlashcardDirection::FillInReverse,
                "fillin_both" => FlashcardDirection::FillInBoth,
                "just_information" => FlashcardDirection::JustInformation,
                _ => FlashcardDirection::Both, // Default to Both if unspecified or invalid
            }
        } else {
            // Default direction if not specified in request
            FlashcardDirection::Both
        };

        match existing_flashcard {
            Some(row) => {
                let existing_id: i32 = row.get("id");
                let existing_direction: FlashcardDirection = row.get("direction");

                // Handle direction change
                if existing_direction != direction {
                    // Archive existing progress
                    transaction
                        .execute(
                            "UPDATE user_flashcard_progress
                             SET archived = true
                             WHERE flashcard_id = $1 AND user_id = $2 AND NOT archived",
                            &[&existing_id, &user_id],
                        )
                        .await
                        .map_err(|e| AppError::Database(e.to_string()))?;

                    // Update flashcard direction
                    transaction
                        .execute(
                            "UPDATE flashcards SET direction = $1 WHERE id = $2",
                            &[&direction, &existing_id],
                        )
                        .await
                        .map_err(|e| AppError::Database(e.to_string()))?;

                    // Initialize new progress based on new direction
                    match direction {
                        FlashcardDirection::Direct => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?
                        }
                        FlashcardDirection::Reverse => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?
                        }
                        FlashcardDirection::Both => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?;
                        }
                        FlashcardDirection::JustInformation => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                            mark_progress_graduated(&transaction, user_id, existing_id, "direct")
                                .await?;
                        }
                        FlashcardDirection::FillIn => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                        }
                        FlashcardDirection::FillInReverse => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?;
                        }
                        FlashcardDirection::FillInBoth => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?;
                        }
                        FlashcardDirection::QuizDirect
                        | FlashcardDirection::QuizReverse
                        | FlashcardDirection::QuizBoth => {
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "direct",
                            )
                            .await?;
                            restore_or_initialize_progress(
                                &transaction,
                                user_id,
                                existing_id,
                                "reverse",
                            )
                            .await?;
                        }
                    }
                }
                existing_id
            }
            None => {
                // Create new flashcard
                let max_position: i32 = transaction
                    .query_one(
                        "SELECT COALESCE(MAX(position), -1) FROM flashcards WHERE collection_id = $1",
                        &[&collection_id],
                    )
                    .await.map_err(|e| AppError::Database(e.to_string()))?
                    .try_get(0).map_err(|e| AppError::Database(e.to_string()))?;

                let row = transaction
                    .query_one(
                        "INSERT INTO flashcards (
                            collection_id, item_id, position, direction
                        )
                        VALUES ($1, $2, $3, $4)
                        RETURNING id",
                        &[&collection_id, &item_id, &(max_position + 1), &direction],
                    )
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;

                let new_id: i32 = row.get("id");

                // Initialize progress based on direction
                match direction {
                    FlashcardDirection::Direct => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                    }
                    FlashcardDirection::Reverse => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                    FlashcardDirection::Both => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                    FlashcardDirection::FillIn => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                    }
                    FlashcardDirection::FillInReverse => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                    FlashcardDirection::FillInBoth => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                    FlashcardDirection::JustInformation => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                        mark_progress_graduated(&transaction, user_id, new_id, "direct").await?;
                    }
                    FlashcardDirection::QuizDirect
                    | FlashcardDirection::QuizReverse
                    | FlashcardDirection::QuizBoth => {
                        initialize_flashcard_progress(&transaction, user_id, new_id, "direct")
                            .await?;
                        initialize_flashcard_progress(&transaction, user_id, new_id, "reverse")
                            .await?;
                    }
                }
                new_id
            }
        };
    }

    // Resolve has_sound from DB after any sound updates
    let has_sound: bool = transaction
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = $1)",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .get(0);

    // Get item details
    let response = if let Some(def_id) = req.definition_id {
        // Get definition details
        let def_row = transaction
            .query_one(
                "SELECT d.*, v.word, v.valsiid, u.username
                 FROM definitions d
                 JOIN valsi v ON d.valsiid = v.valsiid
                 JOIN users u ON d.userid = u.userid
                 WHERE d.definitionid = $1",
                &[&def_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        CollectionItemResponse {
            item_id,
            definition_id: Some(def_id),
            word: Some(def_row.get("word")),
            definition: Some(def_row.get("definition")),
            username: Some(def_row.get("username")),
            valsi_id: Some(def_row.get("valsiid")),
            lang_id: Some(def_row.get("langid")),
            free_content_front: None,
            free_content_back: None,
            notes: def_row.get("notes"),
            language_id: req.language_id,
            owner_user_id: req.owner_user_id,
            license: req.license.clone(),
            script: req.script.clone(),
            is_original: req.is_original.unwrap_or(false),
            ci_notes: notes,
            position,
            auto_progress: req.auto_progress.unwrap_or(true),
            added_at,
            has_front_image: req.front_image.is_some(),
            has_back_image: req.back_image.is_some(),
            has_sound,
            sound_url: if has_sound {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id, item_id
                ))
            } else {
                None
            },
            canonical_form: canonical_form,
            flashcard: None,
        }
    } else {
        // Free content item
        CollectionItemResponse {
            item_id,
            definition_id: None,
            word: None,
            definition: None,
            username: None,
            valsi_id: None,
            lang_id: None,
            free_content_front: sanitized_front,
            free_content_back: sanitized_back,
            notes: None,
            language_id: req.language_id,
            owner_user_id: req.owner_user_id,
            license: req.license.clone(),
            script: req.script.clone(),
            is_original: req.is_original.unwrap_or(true),
            ci_notes: notes,
            position,
            auto_progress: req.auto_progress.unwrap_or(true),
            added_at,
            has_front_image: req.front_image.is_some(),
            has_back_image: req.back_image.is_some(),
            has_sound,
            sound_url: if has_sound {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id, item_id
                ))
            } else {
                None
            },
            canonical_form: canonical_form,
            flashcard: None,
        }
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(response)
}

async fn restore_or_initialize_progress(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> AppResult<()> {
    // Check for archived progress
    let archived_exists = transaction
        .query_opt(
            "SELECT 1 FROM user_flashcard_progress
             WHERE user_id = $1 AND flashcard_id = $2
             AND card_side = $3 AND archived = true",
            &[&user_id, &flashcard_id, &side],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .is_some();

    if archived_exists {
        // Unarchive existing progress
        transaction
            .execute(
                "UPDATE user_flashcard_progress
                 SET archived = false
                 WHERE user_id = $1 AND flashcard_id = $2
                 AND card_side = $3 AND archived = true",
                &[&user_id, &flashcard_id, &side],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    } else {
        // Initialize new progress
        initialize_flashcard_progress(transaction, user_id, flashcard_id, side).await?;
    }

    Ok(())
}

pub async fn update_item_position(
    pool: &Pool,
    collection_id: i32,
    item_id: i32,
    user_id: i32,
    new_position: i32,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // Get current item position
    let current_position: i32 = transaction
        .query_one(
            "SELECT position FROM collection_items 
             WHERE collection_id = $1 AND item_id = $2",
            &[&collection_id, &item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update positions
    let (start, end, shift) = if new_position > current_position {
        (current_position + 1, new_position + 1, -1)
    } else {
        (new_position, current_position, 1)
    };

    // Shift other items
    transaction
        .execute(
            "UPDATE collection_items 
             SET position = position + $1
             WHERE collection_id = $2 
             AND position >= $3 AND position < $4
             AND item_id != $5",
            &[
                &shift as &(dyn tokio_postgres::types::ToSql + Sync),
                &collection_id,
                &start,
                &end,
                &item_id,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update item position
    transaction
        .execute(
            "UPDATE collection_items 
             SET position = $1
             WHERE item_id = $2",
            &[&new_position, &item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update collection timestamp
    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn remove_item(
    pool: &Pool,
    collection_id: i32,
    item_id: i32,
    user_id: i32,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // First delete any associated flashcard history and progress
    transaction
        .execute(
            "DELETE FROM flashcard_review_history
             WHERE flashcard_id IN (SELECT id FROM flashcards WHERE item_id = $1)",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .execute(
            "DELETE FROM user_flashcard_progress
             WHERE flashcard_id IN (SELECT id FROM flashcards WHERE item_id = $1)",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Then delete the flashcards
    transaction
        .execute(
            "DELETE FROM flashcards 
             WHERE item_id = $1",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Then remove the item and verify it belonged to the correct collection
    let result = transaction
        .execute(
            "DELETE FROM collection_items 
             WHERE item_id = $1 AND collection_id = $2",
            &[&item_id, &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result == 0 {
        return Err(AppError::NotFound("Item not found".to_string()));
    }

    // Update collection's updated_at timestamp
    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn clone_collection(
    pool: &Pool,
    source_collection_id: i32,
    user_id: i32,
) -> AppResult<CollectionResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get source collection
    let source = transaction
        .query_one(
            "SELECT name, description, is_public 
             FROM collections 
             WHERE collection_id = $1",
            &[&source_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Create new collection
    let new_collection = transaction
        .query_one(
            "INSERT INTO collections (user_id, name, description, is_public)
             VALUES ($1, $2, $3, false)
             RETURNING collection_id, created_at, updated_at",
            &[
                &user_id,
                &format!("Copy of {}", source.get::<_, String>("name")),
                &source.get::<_, Option<String>>("description"),
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Copy items that have either definition_id or free content
    transaction
        .execute(
            "INSERT INTO collection_items (collection_id, definition_id, 
                free_content_front, free_content_back, 
                langid, owner_user_id, license, script, is_original, 
                notes, position, auto_progress, canonical_form)
            SELECT $1, definition_id, 
                   free_content_front, free_content_back, 
                   langid, owner_user_id, license, script, is_original, 
                   notes, position, auto_progress, canonical_form
            FROM collection_items 
            WHERE collection_id = $2
            AND (definition_id IS NOT NULL 
                 OR free_content_front IS NOT NULL 
                 OR free_content_back IS NOT NULL)",
            &[
                &new_collection.get::<_, i32>("collection_id"),
                &source_collection_id,
            ],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let username = transaction
        .query_one("SELECT username FROM users WHERE userid = $1", &[&user_id])
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("username")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let item_count = transaction
        .query_one(
            "SELECT COUNT(*) FROM collection_items WHERE collection_id = $1",
            &[&new_collection.get::<_, i32>("collection_id")],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get::<_, i64>(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(CollectionResponse {
        collection_id: new_collection.get("collection_id"),
        name: format!("Copy of {}", source.get::<_, String>("name")),
        description: source.get("description"),
        is_public: source.get("is_public"),
        created_at: new_collection.get("created_at"),
        updated_at: new_collection.get("updated_at"),
        item_count,
        owner: CollectionOwner { user_id, username },
    })
}

pub async fn merge_collections(
    pool: &Pool,
    user_id: i32,
    req: &MergeCollectionsRequest,
) -> AppResult<CollectionResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check ownership of source collections
    for collection_id in &[req.source_collection_id, req.target_collection_id] {
        let owner_id: i32 = transaction
            .query_one(
                "SELECT user_id FROM collections WHERE collection_id = $1",
                &[collection_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .try_get("user_id")
            .map_err(|e| AppError::Database(e.to_string()))?;

        if owner_id != user_id {
            return Err(AppError::Unauthorized("Access denied".to_string()));
        }
    }

    // Create new collection if name provided, otherwise use target
    let target_id = if let Some(name) = &req.new_collection_name {
        let new_collection = transaction
            .query_one(
                "INSERT INTO collections (user_id, name)
                VALUES ($1, $2)
                RETURNING collection_id",
                &[&user_id, name],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        new_collection.get("collection_id")
    } else {
        req.target_collection_id
    };

    // Merge items handling duplicates
    transaction
        .execute(
            "INSERT INTO collection_items (collection_id, definition_id, notes, canonical_form)
            SELECT $1, definition_id, notes, canonical_form 
            FROM collection_items 
            WHERE collection_id = $2
            ON CONFLICT (collection_id, definition_id) DO NOTHING",
            &[&target_id, &req.source_collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Update target collection's timestamp
    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &target_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get collection details
    let collection_row = transaction
        .query_one(
            "SELECT c.*, u.userid, u.username,
            (SELECT COUNT(*) FROM collection_items ci WHERE ci.collection_id = c.collection_id) as item_count
            FROM collections c
            JOIN users u ON c.user_id = u.userid
            WHERE c.collection_id = $1",
            &[&target_id],
        )
        .await.map_err(|e| AppError::Database(e.to_string()))?;

    let result = CollectionResponse {
        collection_id: collection_row.get("collection_id"),
        name: collection_row.get("name"),
        description: collection_row.get("description"),
        is_public: collection_row.get("is_public"),
        created_at: collection_row.get("created_at"),
        updated_at: collection_row.get("updated_at"),
        item_count: collection_row.get("item_count"),
        owner: CollectionOwner {
            user_id: collection_row.get("userid"),
            username: collection_row.get("username"),
        },
    };

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(result)
}

pub async fn list_collection_items(
    pool: &Pool,
    collection_id: i32,
    user_id: Option<i32>,
    page: i64,
    per_page: i64,
    search: Option<String>,
    item_id: Option<i32>,
    exclude_with_flashcards: Option<bool>,
) -> AppResult<CollectionItemListResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection access
    let collection = transaction
        .query_one(
            "SELECT user_id, is_public FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let is_public: bool = collection.get("is_public");
    let owner_id: i32 = collection.get("user_id");

    if !is_public && Some(owner_id) != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let offset = (page - 1) * per_page;

    // Build base query
    let mut query = String::from(
        "SELECT ci.item_id, ci.definition_id, ci.notes as ci_notes, ci.added_at, ci.auto_progress, 
                ci.free_content_front, ci.free_content_back, 
                ci.canonical_form,
                ci.langid, ci.owner_user_id, ci.license, ci.script, ci.is_original,
                d.langid as lang_id,
                coalesce(u.username,'') as username,
                d.definition, d.notes as notes, v.valsiid, v.word, ci.position,
                EXISTS(SELECT 1 FROM collection_item_images cii 
                       WHERE cii.item_id = ci.item_id AND cii.side = 'front') as has_front_image,
                EXISTS(SELECT 1 FROM collection_item_images cii 
                       WHERE cii.item_id = ci.item_id AND cii.side = 'back') as has_back_image,
                EXISTS(SELECT 1 FROM collection_item_sounds cis 
                       WHERE cis.item_id = ci.item_id) as has_sound,
                f.id as flashcard_id, f.direction::text as flashcard_direction, f.created_at as flashcard_created_at
         FROM collection_items ci
         LEFT JOIN definitions d ON ci.definition_id = d.definitionid
         LEFT JOIN valsi v ON d.valsiid = v.valsiid
         LEFT JOIN users u ON d.userid = u.userid
         LEFT JOIN flashcards f ON ci.item_id = f.item_id
         WHERE ci.collection_id = $1 
           AND ($2::int IS NULL OR ci.item_id = $2)
           AND ($3::boolean IS NULL OR ($3::boolean = true AND f.id IS NULL))",
    );

    // Create vectors to store parameters and the search pattern
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = vec![
        Box::new(collection_id),
        Box::new(item_id),
        Box::new(exclude_with_flashcards),
    ];
    let mut param_count = 4;

    // Store search pattern if search is provided
    let search_pattern = search.map(|s| format!("%{}%", s));

    // Add search condition if search term provided
    if let Some(pattern) = &search_pattern {
        query.push_str(&format!(
            " AND (
            ci.notes ILIKE ${} OR
            v.word ILIKE ${} OR
            d.definition ILIKE ${} OR
            d.notes ILIKE ${}
        )",
            param_count, param_count, param_count, param_count
        ));
        params.push(Box::new(pattern.clone()));
        param_count += 1;
    }

    // Add ordering and pagination
    query.push_str(" ORDER BY ci.position ASC, ci.added_at DESC");
    query.push_str(&format!(
        " LIMIT ${} OFFSET ${}",
        param_count,
        param_count + 1
    ));
    params.push(Box::new(per_page));
    params.push(Box::new(offset));

    // Execute query
    let rows = transaction
        .query(
            &query,
            &params.iter().map(|p| &**p as _).collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Map results: set sound_url to custom when has_sound, else None (fallback filled below)
    let mut items: Vec<CollectionItemResponse> = rows
        .iter()
        .map(|row| {
            let has_sound: bool = row.get("has_sound");
            let item_id: i32 = row.get("item_id");
            let sound_url = if has_sound {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id, item_id
                ))
            } else {
                None
            };
            CollectionItemResponse {
                lang_id: row.get("lang_id"),
                item_id,
                definition_id: row.get("definition_id"),
                valsi_id: row.get("valsiid"),
                word: row.get("word"),
                username: row.get("username"),
                definition: row.get("definition"),
                notes: row.get("notes"),
                ci_notes: row.get("ci_notes"),
                position: row.get("position"),
                auto_progress: row.get("auto_progress"),
                added_at: row.get("added_at"),
                free_content_front: row.get("free_content_front"),
                free_content_back: row.get("free_content_back"),
                has_front_image: exists_front_image(row),
                language_id: row.get("langid"),
                owner_user_id: row.get("owner_user_id"),
                license: row.get("license"),
                script: row.get("script"),
                is_original: row.get("is_original"),
                has_back_image: exists_back_image(row),
                has_sound,
                sound_url,
                canonical_form: row.get("canonical_form"),
                flashcard: if let Some(flashcard_id) = row.get::<_, Option<i32>>("flashcard_id") {
                    Some(FlashcardResponse {
                        id: flashcard_id,
                        direction: row.get("flashcard_direction"),
                        created_at: row.get("flashcard_created_at"),
                        canonical_form: row.get("canonical_form"),
                    })
                } else {
                    None
                },
            }
        })
        .collect();

    // Fallback sound_url from DB (valsi_sounds) for items without custom sound
    let words_to_check: Vec<String> = rows
        .iter()
        .filter(|r| !r.get::<_, bool>("has_sound"))
        .filter_map(|r| {
            let w: Option<String> = r.get("word");
            let f: Option<String> = r.get("free_content_front");
            w.or(f).filter(|s| !s.trim().is_empty())
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let sound_urls_map = if words_to_check.is_empty() {
        std::collections::HashMap::new()
    } else {
        get_valsi_sound_urls_from_db(pool, &words_to_check)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    };
    for (item, row) in items.iter_mut().zip(rows.iter()) {
        if !item.has_sound {
            let key: Option<String> = row
                .get::<_, Option<String>>("word")
                .or(row.get("free_content_front"));
            item.sound_url = key.and_then(|k| sound_urls_map.get(&k).cloned().flatten());
        }
    }

    // Build and execute count query
    let mut count_query = String::from(
        "SELECT COUNT(*) 
         FROM collection_items ci
         LEFT JOIN definitions d ON ci.definition_id = d.definitionid
         LEFT JOIN valsi v ON d.valsiid = v.valsiid
         WHERE ci.collection_id = $1 AND ($2::int IS NULL OR ci.item_id = $2)",
    );

    let mut count_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> =
        vec![Box::new(collection_id), Box::new(item_id)];

    if let Some(pattern) = &search_pattern {
        count_query.push_str(&format!(
            " AND (
            ci.notes ILIKE ${} OR
            v.word ILIKE ${} OR
            d.definition ILIKE ${} OR
            d.notes ILIKE ${}
        )
        AND ($5::boolean IS NULL OR ($5::boolean = true AND f.id IS NULL))",
            3, 3, 3, 3
        ));
        count_params.push(Box::new(pattern.clone()));
        count_params.push(Box::new(exclude_with_flashcards));
    }

    let total: i64 = transaction
        .query_one(
            &count_query,
            &count_params.iter().map(|p| &**p as _).collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(CollectionItemListResponse {
        items,
        total,
        page,
        per_page,
    })
}

pub async fn update_item_notes(
    pool: &Pool,
    collection_id: i32,
    item_id: i32,
    user_id: i32,
    req: &UpdateItemNotesRequest,
) -> AppResult<CollectionItemResponse> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check collection ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .try_get("user_id")
        .map_err(|e| AppError::Database(e.to_string()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // Update item notes and auto_progress flag
    let item = transaction
        .query_opt(
            "WITH updated AS (
                UPDATE collection_items 
                SET notes = $1,
                    auto_progress = COALESCE($4, auto_progress)
                WHERE collection_id = $2 AND item_id = $3
                RETURNING *
            )
            SELECT u.*, 
                   EXISTS(SELECT 1 FROM collection_item_images i 
                         WHERE i.item_id = u.item_id AND i.side = 'front') as has_front_image,
                   EXISTS(SELECT 1 FROM collection_item_images i 
                         WHERE i.item_id = u.item_id AND i.side = 'back') as has_back_image
            FROM updated u",
            &[&req.notes, &collection_id, &item_id, &req.auto_progress],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or(AppError::NotFound("Item not found".to_string()))?;

    // Get related definition info if this is a definition-based item
    let definition = if let Some(def_id) = item.get::<_, Option<i32>>("definition_id") {
        Some(transaction
            .query_one(
                "SELECT d.definition, d.notes, v.word, v.valsiid, u.username, d.langid as lang_id
                 FROM definitions d
                 JOIN valsi v ON d.valsiid = v.valsiid
                 JOIN users u ON d.userid = u.userid
                 WHERE d.definitionid = $1",
                &[&def_id],
            )
            .await.map_err(|e| AppError::Database(e.to_string()))?)
    } else {
        None
    };

    // Update collection's updated_at timestamp
    transaction
        .execute(
            "UPDATE collections SET updated_at = $1 WHERE collection_id = $2",
            &[&Utc::now(), &collection_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let has_sound: bool = transaction
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = $1)",
            &[&item_id],
        )
        .await
        .map(|r| r.get(0))
        .unwrap_or(false);

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(CollectionItemResponse {
        item_id,
        definition_id: item.get("definition_id"),
        valsi_id: definition.as_ref().map(|d| d.get("valsiid")),
        word: definition.as_ref().map(|d| d.get("word")),
        definition: definition.as_ref().map(|d| d.get("definition")),
        notes: definition.as_ref().map(|d| d.get("notes")),
        ci_notes: item.get("notes"),
        position: item.get("position"),
        auto_progress: item.get("auto_progress"),
        added_at: item.get("added_at"),
        lang_id: definition.as_ref().map(|d| d.get("lang_id")),
        username: definition.as_ref().map(|d| d.get("username")),
        free_content_front: item.get("free_content_front"),
        free_content_back: item.get("free_content_back"),
        has_front_image: item.get("has_front_image"),
        language_id: item.get("langid"),
        owner_user_id: item.get("owner_user_id"),
        license: item.get("license"),
        script: item.get("script"),
        is_original: item.get("is_original"),
        has_back_image: item.get("has_back_image"),
        has_sound,
        sound_url: if has_sound {
            Some(format!(
                "/api/collections/{}/items/{}/sound",
                collection_id, item_id
            ))
        } else {
            None
        },
        canonical_form: item.get("canonical_form"),
        flashcard: None,
    })
}

pub async fn get_item_image(
    pool: &Pool,
    item_id: i32,
    side: &str,
    user_id: Option<i32>,
) -> AppResult<Option<(Vec<u8>, String)>> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check access rights
    if let Some(uid) = user_id {
        let owner_id: i32 = client
            .query_one(
                "SELECT c.user_id FROM collections c 
                 JOIN collection_items ci ON c.collection_id = ci.collection_id 
                 WHERE ci.item_id = $1",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .try_get(0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        if owner_id != uid {
            return Err(AppError::Unauthorized("Access denied".to_string()));
        }
    }

    let result = client
        .query_opt(
            "SELECT image_data, mime_type 
             FROM collection_item_images 
             WHERE item_id = $1 AND side = $2",
            &[&item_id, &side],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(result.map(|row| (row.get("image_data"), row.get("mime_type"))))
}

pub async fn get_item_sound(
    pool: &Pool,
    item_id: i32,
    user_id: Option<i32>,
) -> AppResult<Option<(Vec<u8>, String)>> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check access rights
    if let Some(uid) = user_id {
        let owner_id: i32 = client
            .query_one(
                "SELECT c.user_id FROM collections c 
                 JOIN collection_items ci ON c.collection_id = ci.collection_id 
                 WHERE ci.item_id = $1",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .try_get(0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        if owner_id != uid {
            return Err(AppError::Unauthorized("Access denied".to_string()));
        }
    }

    let result = client
        .query_opt(
            "SELECT sound_data, mime_type 
             FROM collection_item_sounds 
             WHERE item_id = $1",
            &[&item_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(result.map(|row| (row.get("sound_data"), row.get("mime_type"))))
}

pub async fn update_item_images(
    pool: &Pool,
    collection_id: i32,
    item_id: i32,
    user_id: i32,
    req: &UpdateItemRequest,
) -> AppResult<()> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let transaction = client
        .transaction()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Update notes if provided
    if let Some(notes) = &req.notes {
        transaction
            .execute(
                "UPDATE collection_items SET notes = $1 WHERE item_id = $2",
                &[notes, &item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle front image
    if req.remove_front_image.unwrap_or(false) || req.front_image.is_some() {
        transaction
            .execute(
                "DELETE FROM collection_item_images WHERE item_id = $1 AND side = 'front'",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    if let Some(image) = &req.front_image {
        validate_item_image(image).map_err(|e| AppError::BadRequest(e.to_string()))?;
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid front image base64: {}", e)))?;
        transaction
            .execute(
                "INSERT INTO collection_item_images (item_id, image_data, mime_type, side)
             VALUES ($1, $2, $3, 'front')",
                &[&item_id, &image_data, &image.mime_type],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle back image
    if req.remove_back_image.unwrap_or(false) || req.back_image.is_some() {
        transaction
            .execute(
                "DELETE FROM collection_item_images WHERE item_id = $1 AND side = 'back'",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    if let Some(image) = &req.back_image {
        validate_item_image(image).map_err(|e| AppError::BadRequest(e.to_string()))?;
        let image_data = BASE64
            .decode(&image.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid back image base64: {}", e)))?;
        transaction
            .execute(
                "INSERT INTO collection_item_images (item_id, image_data, mime_type, side)
             VALUES ($1, $2, $3, 'back')",
                &[&item_id, &image_data, &image.mime_type],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    // Handle sound
    if req.remove_sound.unwrap_or(false) || req.sound.is_some() {
        transaction
            .execute(
                "DELETE FROM collection_item_sounds WHERE item_id = $1",
                &[&item_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    if let Some(sound) = &req.sound {
        validate_item_audio(sound).map_err(|e| AppError::BadRequest(e.to_string()))?;
        let sound_data = BASE64
            .decode(&sound.data)
            .map_err(|e| AppError::BadRequest(format!("Invalid sound base64: {}", e)))?;
        transaction
            .execute(
                "INSERT INTO collection_item_sounds (item_id, sound_data, mime_type)
             VALUES ($1, $2, $3)",
                &[&item_id, &sound_data, &sound.mime_type],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    transaction
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn exists_front_image(row: &tokio_postgres::Row) -> bool {
    row.get("has_front_image")
}

fn exists_back_image(row: &tokio_postgres::Row) -> bool {
    row.get("has_back_image")
}

pub async fn search_items(
    pool: &Pool,
    current_user_id: i32,
    query: &str,
    owner_id: Option<i32>,
) -> AppResult<SearchItemsResponse> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let search_pattern = format!("%{}%", query);
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> =
        vec![Box::new(&search_pattern)];
    let mut param_count = 2;

    let mut sql = String::from(
        "WITH accessible_collections AS (
            SELECT collection_id 
            FROM collections 
            WHERE is_public = true 
            OR user_id = $",
    );
    sql.push_str(&param_count.to_string());
    params.push(Box::new(current_user_id));
    param_count += 1;

    if let Some(uid) = owner_id {
        sql.push_str(" AND user_id = $");
        sql.push_str(&param_count.to_string());
        params.push(Box::new(uid));
    }

    sql.push_str(
        ")
        SELECT ci.item_id, ci.definition_id, ci.notes as ci_notes, 
               ci.added_at, ci.position, ci.auto_progress, 
               ci.langid, ci.owner_user_id, ci.license, ci.script, ci.is_original,
               ci.free_content_front, ci.free_content_back, ci.canonical_form,
               d.langid as lang_id, d.definition, d.notes,
               v.valsiid, v.word, u.username,
               c.collection_id,
               EXISTS(SELECT 1 FROM collection_item_images cii 
                      WHERE cii.item_id = ci.item_id AND cii.side = 'front') as has_front_image,
               EXISTS(SELECT 1 FROM collection_item_images cii 
                      WHERE cii.item_id = ci.item_id AND cii.side = 'back') as has_back_image,
               EXISTS(SELECT 1 FROM collection_item_sounds cis 
                      WHERE cis.item_id = ci.item_id) as has_sound
        FROM collection_items ci
        JOIN accessible_collections ac ON ci.collection_id = ac.collection_id
        JOIN collections c ON ci.collection_id = c.collection_id
        LEFT JOIN definitions d ON ci.definition_id = d.definitionid
        LEFT JOIN valsi v ON d.valsiid = v.valsiid
        LEFT JOIN users u ON d.userid = u.userid
        WHERE v.word ILIKE $1
           OR d.definition ILIKE $1
           OR d.notes ILIKE $1
           OR ci.notes ILIKE $1
           OR ci.free_content_front ILIKE $1
           OR ci.free_content_back ILIKE $1
        ORDER BY c.updated_at DESC, ci.position ASC",
    );

    let rows = client
        .query(&sql, &params.iter().map(|p| &**p as _).collect::<Vec<_>>())
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let items = rows
        .iter()
        .map(|row| {
            let has_sound: bool = row.get("has_sound");
            let item_id: i32 = row.get("item_id");
            let cid: i32 = row.get("collection_id");
            let sound_url = if has_sound {
                Some(format!("/api/collections/{}/items/{}/sound", cid, item_id))
            } else {
                None
            };
            CollectionItemResponse {
                item_id,
                definition_id: row.get("definition_id"),
                word: row.get("word"),
                username: row.get("username"),
                valsi_id: row.get("valsiid"),
                definition: row.get("definition"),
                notes: row.get("notes"),
                ci_notes: row.get("ci_notes"),
                position: row.get("position"),
                auto_progress: row.get("auto_progress"),
                added_at: row.get("added_at"),
                lang_id: row.get("lang_id"),
                free_content_front: row.get("free_content_front"),
                free_content_back: row.get("free_content_back"),
                has_front_image: row.get("has_front_image"),
                language_id: row.get("langid"),
                owner_user_id: row.get("owner_user_id"),
                license: row.get("license"),
                script: row.get("script"),
                is_original: row.get("is_original"),
                has_back_image: row.get("has_back_image"),
                has_sound,
                sound_url,
                canonical_form: row.get("canonical_form"),
                flashcard: None,
            }
        })
        .collect();

    Ok(SearchItemsResponse {
        items,
        total: rows.len() as i64,
    })
}
