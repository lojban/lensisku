use crate::jbovlaste::service::get_valsi_sound_urls_from_db;
use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::Pool;
use deadpool_postgres::Transaction;
use fsrs::{
    extract_simulator_config, FSRSItem, FSRSReview, ItemProgress, ItemState, MemoryState,
    NextStates, RevlogEntry, RevlogReviewKind, FSRS,
};
use log::{debug, error, info};
use std::collections::HashMap;

use crate::auth_utils::{verify_collection_ownership, verify_flashcard_ownership};

use super::{
    dto::{
        self, AddCardsRequest, ChronoDateTime, CreateFlashcardRequest, CreateLevelRequest,
        DailyProgress, DirectAnswerRequest, DirectAnswerResponse, FillInAnswerRequest, Flashcard,
        FlashcardListQuery, FlashcardListResponse, FlashcardResponse, ImportFromCollectionResponse,
        LevelCardListResponse, LevelCardProgress, LevelCardResponse, LevelListResponse,
        LevelProgress, LevelResponse, PrerequisiteLevel, ReviewRequest, ReviewResponse,
        StreakResponse, UpdateLevelRequest,
    },
    models::*,
};

async fn get_flashcard(
    transaction: &Transaction<'_>,
    flashcard_id: i32,
) -> Result<Flashcard, Box<dyn std::error::Error>> {
    let row = transaction
        .query_one(
            "SELECT f.*, ci.definition_id, ci.free_content_front, ci.free_content_back, ci.notes,
                    ci.canonical_form,
                    d.definition, d.langid as definition_language_id,
                    v.word,
                    EXISTS(SELECT 1 FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as has_front_image,
                    EXISTS(SELECT 1 FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as has_back_image,
                    EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = ci.item_id) as has_custom_sound
             FROM flashcards f
             JOIN collection_items ci ON f.item_id = ci.item_id
             LEFT JOIN definitions d ON ci.definition_id = d.definitionid
             LEFT JOIN valsi v ON d.valsiid = v.valsiid
             WHERE f.id = $1",
            &[&flashcard_id],
        )
        .await?;
    let word: Option<String> = row.get("word");

    Ok(Flashcard {
        id: row.get("id"),
        collection_id: row.get("collection_id"),
        item_id: row.get("item_id"),
        definition_id: row.get("definition_id"),
        word, // Use the variable declared above
        definition: row.get("definition"),
        free_content_front: row.get("free_content_front"),
        free_content_back: row.get("free_content_back"),
        has_front_image: row.get("has_front_image"),
        has_back_image: row.get("has_back_image"),
        notes: row.get("notes"),
        direction: row.get("direction"),
        position: row.get("position"),
        definition_language_id: row.get("definition_language_id"),
        sound_url: if row.get::<_, bool>("has_custom_sound") {
            Some(format!(
                "/api/collections/{}/items/{}/sound",
                row.get::<_, i32>("collection_id"),
                row.get::<_, i32>("item_id")
            ))
        } else {
            None
        },
        has_custom_sound: row.get("has_custom_sound"),
        canonical_form: row.get("canonical_form"),
        use_canonical_comparison: row.get("use_canonical_comparison"),
        created_at: row.get("created_at"),
        question_text: None, // Will be populated later if it's a quiz
        quiz_options: None,  // Will be populated later if it's a quiz
    })
}

pub async fn create_flashcard(
    pool: &Pool,
    collection_id: i32,
    user_id: i32,
    req: &CreateFlashcardRequest,
) -> Result<FlashcardResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Get or create collection item
    let item_id: i32 = match transaction
        .query_opt(
            "SELECT item_id FROM collection_items
             WHERE collection_id = $1 AND
             CASE
                WHEN $2::int IS NOT NULL THEN definition_id = $2
                ELSE free_content_front = $3 AND free_content_back = $4
             END",
            &[
                &collection_id,
                &req.definition_id,
                &req.free_content_front,
                &req.free_content_back,
            ],
        )
        .await?
    {
        Some(row) => {
            let item_id: i32 = row.get(0);
            // Ensure canonical_form is updated if missing (backfill)
            let existing_canon: Option<String> = transaction
                .query_one(
                    "SELECT canonical_form FROM collection_items WHERE item_id = $1",
                    &[&item_id],
                )
                .await?
                .get(0);

            if existing_canon.is_none() {
                let canon = if let Some(front) = &req.free_content_front {
                    crate::tersmu::get_canonical_form(front)
                } else if let Some(def_id) = req.definition_id {
                    // Fetch word for dictionary items
                    let word_row = transaction.query_opt(
                        "SELECT v.word FROM definitions d JOIN valsi v ON d.valsiid = v.valsiid WHERE d.definitionid = $1",
                        &[&def_id]
                    ).await?;
                    word_row.and_then(|r| {
                        let word: String = r.get(0);
                        crate::tersmu::get_canonical_form(&word)
                    })
                } else {
                    None
                };

                if let Some(canon_val) = canon {
                    transaction
                        .execute(
                            "UPDATE collection_items SET canonical_form = $1 WHERE item_id = $2",
                            &[&canon_val, &item_id],
                        )
                        .await?;
                }
            }
            item_id
        }
        None => {
            let max_position: i32 = transaction
                .query_one(
                    "SELECT COALESCE(MAX(position), -1) FROM collection_items WHERE collection_id = $1",
                    &[&collection_id],
                )
                .await?
                .get(0);

            let canonical_form = req
                .free_content_front
                .as_ref()
                .and_then(|front| crate::tersmu::get_canonical_form(front));

            transaction
                .query_one(
                    "INSERT INTO collection_items (
                        collection_id, definition_id,
                        free_content_front, free_content_back,
                        notes, position, canonical_form
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    RETURNING item_id",
                    &[
                        &collection_id,
                        &req.definition_id,
                        &req.free_content_front,
                        &req.free_content_back,
                        &req.notes,
                        &(max_position + 1),
                        &canonical_form,
                    ],
                )
                .await?
                .get(0)
        }
    };

    // Check if flashcard already exists
    let existing = transaction
        .query_opt(
            "SELECT f.id, f.created_at, ci.notes, f.direction,
                    f.position, ci.definition_id, ci.item_id,
                    ci.free_content_front, ci.free_content_back, ci.canonical_form,
                    v.word, d.definition,
                    EXISTS(SELECT 1 FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as has_front_image,
                    EXISTS(SELECT 1 FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as has_back_image,
                    EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = ci.item_id) as has_custom_sound
             FROM flashcards f
             JOIN collection_items ci ON f.item_id = ci.item_id
             LEFT JOIN definitions d ON ci.definition_id = d.definitionid
             LEFT JOIN valsi v ON d.valsiid = v.valsiid
             WHERE f.collection_id = $1 AND f.item_id = $2",
            &[&collection_id, &item_id],
        )
        .await?;

    if let Some(row) = existing {
        let flashcard = Flashcard {
            id: row.get("id"),
            collection_id,
            definition_id: row.get("definition_id"),
            word: row.get("word"),
            definition: row.get("definition"),
            free_content_front: row.get("free_content_front"),
            free_content_back: row.get("free_content_back"),
            has_front_image: row.get("has_front_image"),
            has_back_image: row.get("has_back_image"),
            notes: row.get("notes"),
            position: row.get("position"),
            direction: row.get("direction"),
            created_at: row.get("created_at"),
            definition_language_id: None,
            item_id,
            question_text: None,
            quiz_options: None,
            sound_url: if row.get::<_, bool>("has_custom_sound") {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id,
                    item_id
                ))
            } else {
                None
            },
            has_custom_sound: row.get("has_custom_sound"),
            canonical_form: row.get("canonical_form"),
            use_canonical_comparison: row.get("use_canonical_comparison"),
        };

        let progress = get_all_progress(&transaction, user_id, flashcard.id).await?;
        transaction.commit().await?;

        return Ok(FlashcardResponse {
            flashcard,
            progress,
        });
    }

    // Create new flashcard
    let max_position: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) FROM flashcards WHERE collection_id = $1",
            &[&collection_id],
        )
        .await?
        .get(0);

    let row = transaction
        .query_one(
            "INSERT INTO flashcards (
                collection_id,
                item_id, position, direction, use_canonical_comparison
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, created_at",
            &[
                &collection_id,
                &item_id,
                &(max_position + 1),
                &req.direction,
                &req.use_canonical_comparison,
            ],
        )
        .await?;

    let flashcard_id = row.get::<_, i32>("id");

    // Initialize progress based on direction
    match req.direction {
        FlashcardDirection::Direct => {
            initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
        }
        FlashcardDirection::Reverse => {
            initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
        }
        FlashcardDirection::Both => {
            initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
            initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
        }
        FlashcardDirection::FillIn => {
            initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
        }
        FlashcardDirection::FillInReverse => {
            initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
        }
        FlashcardDirection::FillInBoth => {
            initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
            initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
        }
        FlashcardDirection::JustInformation => {
            initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
            mark_progress_graduated(&transaction, user_id, flashcard_id, "direct").await?;
        }
        FlashcardDirection::QuizDirect
        | FlashcardDirection::QuizReverse
        | FlashcardDirection::QuizBoth => {
            // For quiz types, use provided correct answer or determine automatically
            let correct_answer_text = if let Some(text) = &req.correct_answer_text {
                text.clone()
            } else {
                determine_correct_answer_for_quiz(
                    &transaction,
                    &req.direction,
                    req.definition_id,
                    req.free_content_front.as_deref(),
                    req.free_content_back.as_deref(),
                )
                .await?
            };

            transaction.execute(
                "INSERT INTO flashcard_quiz_options (flashcard_id, correct_answer_text) VALUES ($1, $2)",
                &[&flashcard_id, &correct_answer_text]
            ).await?;
            initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
            // Quiz progress tracked on "direct" side for FSRS
        }
    }

    let flashcard = get_flashcard(&transaction, flashcard_id).await?;
    let progress = get_all_progress(&transaction, user_id, flashcard_id).await?;

    transaction.commit().await?;

    Ok(FlashcardResponse {
        flashcard,
        progress,
    })
}

async fn determine_correct_answer_for_quiz(
    transaction: &Transaction<'_>,
    direction: &FlashcardDirection,
    definition_id: Option<i32>,
    free_content_front: Option<&str>,
    free_content_back: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    match direction {
        FlashcardDirection::QuizDirect | FlashcardDirection::QuizBoth => {
            // For QuizBoth, store direct answer
            if let Some(def_id) = definition_id {
                Ok(transaction
                    .query_one(
                        "SELECT definition FROM definitions WHERE definitionid = $1",
                        &[&def_id],
                    )
                    .await?
                    .get(0))
            } else if let Some(back_text) = free_content_back {
                Ok(back_text.to_string())
            } else {
                Err("Quiz flashcard (direct) requires definition or free_content_back".into())
            }
        }
        FlashcardDirection::QuizReverse => {
            if let Some(def_id) = definition_id {
                Ok(transaction.query_one("SELECT v.word FROM definitions d JOIN valsi v ON d.valsiid = v.valsiid WHERE d.definitionid = $1", &[&def_id]).await?.get(0))
            } else if let Some(front_text) = free_content_front {
                Ok(front_text.to_string())
            } else {
                Err("Quiz flashcard (reverse) requires definition or free_content_front".into())
            }
        }
        _ => Err("Invalid direction for quiz option determination".into()),
    }
}

pub async fn initialize_progress(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    transaction
        .execute(
            "INSERT INTO user_flashcard_progress
 (user_id, flashcard_id, card_side, status, next_review_at)
 VALUES ($1, $2, $3, 'new', CURRENT_TIMESTAMP)
 ON CONFLICT (user_id, flashcard_id, card_side) WHERE NOT archived DO NOTHING",
            &[&user_id, &flashcard_id, &side],
        )
        .await?;
    Ok(())
}

async fn mark_progress_graduated(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    transaction
        .execute(
            "UPDATE user_flashcard_progress SET status = 'graduated', next_review_at = NULL
         WHERE user_id = $1 AND flashcard_id = $2 AND card_side = $3 AND NOT archived",
            &[&user_id, &flashcard_id, &side],
        )
        .await?;
    Ok(())
}

pub async fn snooze_flashcard(
    pool: &Pool,
    user_id: i32,
    flashcard_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_study_access(&transaction, flashcard_id, user_id).await?;

    let result = transaction
        .execute(
            "UPDATE user_flashcard_progress
             SET next_review_at = CURRENT_TIMESTAMP + INTERVAL '6 hours'
             WHERE user_id = $1 AND flashcard_id = $2 AND NOT archived",
            &[&user_id, &flashcard_id],
        )
        .await?;

    if result == 0 {
        // Check if progress exists at all, even if archived
        let exists = transaction
            .query_opt(
                "SELECT 1 FROM user_flashcard_progress WHERE user_id = $1 AND flashcard_id = $2",
                &[&user_id, &flashcard_id],
            )
            .await?
            .is_some();
        if !exists {
            return Err("Flashcard progress not found for this user".into());
        } else {
            // Progress exists but is archived or otherwise not updated
            // This might happen if the card direction changed and progress was archived
            // Or if there are multiple progress records (e.g., 'both' direction)
            // We can choose to either return an error or log a warning.
            // For now, let's return a specific error.
            return Err(
                "Could not snooze flashcard. Progress might be archived or inactive.".into(),
            );
        }
    }

    transaction.commit().await?;
    Ok(())
}

async fn get_all_progress(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
) -> Result<Vec<UserFlashcardProgress>, Box<dyn std::error::Error>> {
    let rows = transaction
        .query(
            "SELECT * FROM user_flashcard_progress
         WHERE user_id = $1 AND flashcard_id = $2 AND NOT archived",
            &[&user_id, &flashcard_id],
        )
        .await?;

    let mut progress = Vec::new();
    for row in rows {
        progress.push(UserFlashcardProgress {
            id: row.get::<_, i32>("id"),
            user_id,
            flashcard_id,
            card_side: row.get("card_side"),
            ease_factor: row.get::<_, f64>("ease_factor") as f32,
            stability: row.get::<_, f64>("stability") as i32,
            difficulty: row.get::<_, f64>("difficulty") as i32,
            interval: row.get("interval"),
            review_count: row.get("review_count"),
            last_reviewed_at: row.get("last_reviewed_at"),
            next_review_at: get_review_time(row.get("next_review_at")),
            status: row.get("status"),
        });
    }

    Ok(progress)
}

pub async fn delete_flashcard(
    pool: &Pool,
    user_id: i32,
    flashcard_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_flashcard_ownership(&transaction, flashcard_id, user_id).await?;

    // Delete dependent records first
    transaction
        .execute(
            "DELETE FROM flashcard_review_history WHERE flashcard_id = $1",
            &[&flashcard_id],
        )
        .await?;

    transaction
        .execute(
            "DELETE FROM user_flashcard_progress WHERE flashcard_id = $1",
            &[&flashcard_id],
        )
        .await?;

    transaction
        .execute("DELETE FROM flashcards WHERE id = $1", &[&flashcard_id])
        .await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn generate_and_set_quiz_options(
    pool: &Pool,
    user_id: i32,
    flashcard_id: i32,
) -> Result<FlashcardQuizOptions, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Verify ownership of the collection the flashcard belongs to
    let collection_id_row = transaction
        .query_one(
            "SELECT collection_id FROM flashcards WHERE id = $1",
            &[&flashcard_id],
        )
        .await?;
    let collection_id: i32 = collection_id_row.get("collection_id");
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Fetch flashcard and related item details
    let flashcard_details_row = transaction
        .query_one(
            "SELECT f.direction, ci.definition_id, ci.free_content_front, ci.free_content_back
         FROM flashcards f
         JOIN collection_items ci ON f.item_id = ci.item_id
         WHERE f.id = $1",
            &[&flashcard_id],
        )
        .await?;

    let direction: FlashcardDirection = flashcard_details_row.get("direction");
    let definition_id: Option<i32> = flashcard_details_row.get("definition_id");
    let free_content_front: Option<String> = flashcard_details_row.get("free_content_front");
    let free_content_back: Option<String> = flashcard_details_row.get("free_content_back");

    // Ensure the flashcard is a quiz type
    if !matches!(
        direction,
        FlashcardDirection::QuizDirect
            | FlashcardDirection::QuizReverse
            | FlashcardDirection::QuizBoth
    ) {
        return Err("Flashcard is not a quiz type.".into());
    }

    let correct_answer_text = determine_correct_answer_for_quiz(
        &transaction,
        &direction,
        definition_id,
        free_content_front.as_deref(),
        free_content_back.as_deref(),
    )
    .await?;

    // Insert or update the quiz option
    let quiz_option_row = transaction
        .query_one(
            "INSERT INTO flashcard_quiz_options (flashcard_id, correct_answer_text)
         VALUES ($1, $2)
         ON CONFLICT (flashcard_id) DO UPDATE SET correct_answer_text = EXCLUDED.correct_answer_text
         RETURNING quiz_option_id, flashcard_id, correct_answer_text, created_at",
            &[&flashcard_id, &correct_answer_text],
        )
        .await?;

    transaction.commit().await?;

    Ok(FlashcardQuizOptions {
        quiz_option_id: quiz_option_row.get("quiz_option_id"),
        flashcard_id: quiz_option_row.get("flashcard_id"),
        correct_answer_text: quiz_option_row.get("correct_answer_text"),
        created_at: quiz_option_row.get("created_at"),
    })
}

fn get_review_time(datetime: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    match datetime {
        Some(time) if time <= Utc::now() => None,
        Some(time) => Some(time),
        None => None,
    }
}

pub async fn list_flashcards(
    pool: &Pool,
    user_id: i32,
    query: FlashcardListQuery,
) -> Result<FlashcardListResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let fsrs = FSRS::new(Some(&[]))?;

    // Get base flashcards including free content
    let rows = transaction
    .query(
        "WITH ReviewHistory AS (
            SELECT fh.flashcard_id, fh.card_side,
                   array_agg(jsonb_build_object(
                       'rating', fh.rating,
                       'elapsed_days', fh.elapsed_days
                   ) ORDER BY fh.review_time) as reviews
            FROM flashcard_review_history fh
            WHERE fh.user_id = $1
            GROUP BY fh.flashcard_id, fh.card_side
        )
        SELECT f.*, p.*, ci.definition_id,
               ci.item_id, ci.free_content_front, ci.free_content_back, ci.notes,
               ci.canonical_form,
               d.langid as definition_language_id,
               v.word, d.definition,
               EXISTS(SELECT 1 FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as has_front_image,
               EXISTS(SELECT 1 FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as has_back_image,
               EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = ci.item_id) as has_custom_sound,
               rh.reviews,
               EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - p.last_reviewed_at))/86400 as days_since_review
        FROM flashcards f
        JOIN collection_items ci ON f.item_id = ci.item_id
        LEFT JOIN definitions d ON ci.definition_id = d.definitionid
        LEFT JOIN valsi v ON d.valsiid = v.valsiid
        LEFT JOIN user_flashcard_progress p
            ON f.id = p.flashcard_id
            AND p.user_id = $1
            AND NOT p.archived
        LEFT JOIN ReviewHistory rh
            ON f.id = rh.flashcard_id
            AND p.card_side = rh.card_side
        WHERE f.collection_id = $2
        AND ($3::flashcard_status IS NULL OR p.status = $3)
        AND ($4::boolean IS NULL
             OR ($4::boolean = true AND
                 (p.next_review_at IS NULL OR p.next_review_at <= CURRENT_TIMESTAMP)))
        AND ($5::int IS NULL OR f.id = $5)
        ORDER BY
        CASE
            WHEN $4::boolean = true THEN EXTRACT(EPOCH FROM p.next_review_at)
            ELSE f.position::bigint
        END",
        &[&user_id, &query.collection_id, &query.status, &query.due, &query.flashcard_id],
    )
    .await?;

    // Collect unique strings for sound URL lookup: valsi word when linked, otherwise free_content_front.
    let words_to_check: Vec<String> = rows
        .iter()
        .filter_map(|row| {
            let word: Option<String> = row.get("word");
            let free_content_front: Option<String> = row.get("free_content_front");
            word.or(free_content_front)
        })
        .filter(|w| !w.trim().is_empty())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Fetch sound URLs from DB (valsi_sounds) for valsi-backed and free-content items
    let sound_urls_map = if !words_to_check.is_empty() {
        get_valsi_sound_urls_from_db(pool, &words_to_check)
            .await
            .unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    };

    let mut all_flashcards_with_retrievability = Vec::new();
    let mut processed_ids = std::collections::HashSet::new();

    for row in &rows {
        let flashcard_id: i32 = row.get("id");
        let direction: FlashcardDirection = row.get("direction");
        let card_side: Option<String> = row.get("card_side");

        // Handle cards without progress records
        if card_side.is_none() {
            match direction {
                FlashcardDirection::Direct => {
                    initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
                }
                FlashcardDirection::Reverse => {
                    initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
                }
                FlashcardDirection::Both => {
                    initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
                    initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
                }
                FlashcardDirection::FillIn => {
                    initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
                }
                FlashcardDirection::FillInReverse => {
                    initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
                }
                FlashcardDirection::FillInBoth => {
                    initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
                    initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
                }
                FlashcardDirection::JustInformation => {
                    initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
                    mark_progress_graduated(&transaction, user_id, flashcard_id, "direct").await?;
                }
                FlashcardDirection::QuizDirect
                | FlashcardDirection::QuizReverse
                | FlashcardDirection::QuizBoth => {
                    // This case should ideally be handled by prior logic ensuring quiz_options exist.
                    // If progress is missing for a quiz, initialize it.
                    initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
                    // Quiz progress tracked on "direct" side for FSRS
                }
            }
            continue;
        }

        let reviews: Option<Vec<serde_json::Value>> = row.get("reviews");
        let retrievability = if let Some(review_array) = reviews {
            let fsrs_reviews = review_array
                .into_iter()
                .map(|review| {
                    Ok(FSRSReview {
                        rating: review["rating"].as_i64().ok_or_else(|| {
                            format!("Missing/invalid rating in review data: {:?}", review)
                        })? as u32,
                        delta_t: review["elapsed_days"].as_i64().ok_or_else(|| {
                            format!("Missing/invalid elapsed_days in review data: {:?}", review)
                        })? as u32,
                    })
                })
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

            let fsrs_item = FSRSItem {
                reviews: fsrs_reviews,
            };
            let memory_state = fsrs.memory_state(fsrs_item, None)?;

            let last_review: Option<DateTime<Utc>> = row.get("last_reviewed_at");
            if let Some(last) = last_review {
                let days_elapsed = (Utc::now() - last).num_days().max(0) as u32;
                fsrs.current_retrievability(memory_state, days_elapsed, -0.5)
            } else {
                1.0
            }
        } else {
            1.0
        };

        let progress = UserFlashcardProgress {
            id: row.get("id"),
            user_id,
            flashcard_id,
            card_side: row.get("card_side"),
            ease_factor: row.get::<_, f64>("ease_factor") as f32,
            stability: row.get::<_, f64>("stability") as i32,
            difficulty: row.get::<_, f64>("difficulty") as i32,
            interval: row.get("interval"),
            review_count: row.get("review_count"),
            last_reviewed_at: row.get("last_reviewed_at"),
            next_review_at: get_review_time(row.get("next_review_at")),
            status: row.get("status"),
        };

        let (question_text, quiz_options) = if matches!(
            direction,
            FlashcardDirection::QuizDirect
                | FlashcardDirection::QuizReverse
                | FlashcardDirection::QuizBoth
        ) {
            let effective_quiz_direction = match direction {
                FlashcardDirection::QuizDirect => "direct",
                FlashcardDirection::QuizReverse => "reverse",
                FlashcardDirection::QuizBoth => {
                    if rand::random() {
                        "direct"
                    } else {
                        "reverse"
                    }
                } // Randomly pick for QuizBoth
                _ => "direct", // Default, should not happen for quiz types
            };
            match get_quiz_options_for_listing(
                &transaction,
                flashcard_id,
                user_id,
                effective_quiz_direction,
            )
            .await
            {
                Ok((q, o)) => (Some(q), Some(o)),
                Err(e) => {
                    error!(
                        "Failed to get quiz options for flashcard {}: {}",
                        flashcard_id, e
                    );
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        // Check if this is a "both" direction card and how to handle it
        if (direction == FlashcardDirection::Both || direction == FlashcardDirection::FillInBoth)
            && !query.due.unwrap_or(false)
        {
            // For non-due listing, combine progresses into single card
            if !processed_ids.contains(&flashcard_id) {
                let word: Option<String> = row.get("word");
                let free_content_front: Option<String> = row.get("free_content_front");
                let sound_key = word.as_ref().or(free_content_front.as_ref());
                let sound_url = sound_key.and_then(|k| sound_urls_map.get(k).cloned().flatten());

                let mut card_response = FlashcardResponse {
                    flashcard: Flashcard {
                        id: flashcard_id,
                        collection_id: query.collection_id,
                        definition_id: row.get("definition_id"),
                        word: word.clone(),
                        definition: row.get("definition"),
                        direction,
                        notes: row.get("notes"),
                        position: row.get("position"),
                        created_at: row.get("created_at"),
                        definition_language_id: row.get("definition_language_id"),
                        free_content_front: row.get("free_content_front"),
                        free_content_back: row.get("free_content_back"),
                        has_front_image: row.get("has_front_image"),
                        has_back_image: row.get("has_back_image"),
                        item_id: row.get("item_id"),
                        sound_url: if row.get::<_, bool>("has_custom_sound") {
                            Some(format!(
                                "/api/collections/{}/items/{}/sound",
                                query.collection_id,
                                row.get::<_, i32>("item_id")
                            ))
                        } else {
                            sound_url
                        },
                        has_custom_sound: row.get("has_custom_sound"),
                        canonical_form: row.get("canonical_form"),
                        use_canonical_comparison: row.get("use_canonical_comparison"),
                        question_text: question_text.clone(), // Use cloned quiz data
                        quiz_options: quiz_options.clone(),
                    },
                    progress: vec![progress],
                };

                // Find and add the other direction's progress
                if let Some(other_row) = rows.iter().find(|r| {
                    let other_id: i32 = r.get("id");
                    let other_side: String = r.get("card_side");
                    other_id == flashcard_id
                        && card_side.as_deref().map_or(false, |cs| other_side != cs)
                }) {
                    let other_progress = UserFlashcardProgress {
                        id: other_row.get("id"),
                        user_id,
                        flashcard_id,
                        card_side: other_row.get("card_side"),
                        ease_factor: other_row.get::<_, f64>("ease_factor") as f32,
                        stability: other_row.get::<_, f64>("stability") as i32,
                        difficulty: other_row.get::<_, f64>("difficulty") as i32,
                        interval: other_row.get("interval"),
                        review_count: other_row.get("review_count"),
                        last_reviewed_at: other_row.get("last_reviewed_at"),
                        next_review_at: get_review_time(other_row.get("next_review_at")),
                        status: other_row.get("status"),
                    };
                    card_response.progress.push(other_progress);
                }

                all_flashcards_with_retrievability.push((card_response, retrievability));
                processed_ids.insert(flashcard_id);
            }
        } else {
            // For due cards or non-both direction, create separate response
            let word: Option<String> = row.get("word");
            let free_content_front: Option<String> = row.get("free_content_front");
            let sound_key = word.as_ref().or(free_content_front.as_ref());
            let sound_url = sound_key.and_then(|k| sound_urls_map.get(k).cloned().flatten());

            let card_response = FlashcardResponse {
                flashcard: Flashcard {
                    id: flashcard_id,
                    collection_id: query.collection_id,
                    definition_id: row.get("definition_id"),
                    word: word.clone(), // Use the cloned word
                    definition: row.get("definition"),
                    free_content_front: row.get("free_content_front"),
                    free_content_back: row.get("free_content_back"),
                    has_front_image: row.get("has_front_image"),
                    has_back_image: row.get("has_back_image"),
                    notes: row.get("notes"),
                    direction, // Use direction directly
                    position: row.get("position"),
                    created_at: row.get("created_at"),
                    definition_language_id: row.get("definition_language_id"),
                    item_id: row.get("item_id"),
                    sound_url: if row.get::<_, bool>("has_custom_sound") {
                        Some(format!(
                            "/api/collections/{}/items/{}/sound",
                            query.collection_id,
                            row.get::<_, i32>("item_id")
                        ))
                    } else {
                        sound_url
                    },
                    has_custom_sound: row.get("has_custom_sound"),
                    question_text: question_text.clone(),
                    quiz_options: quiz_options.clone(),
                    canonical_form: row.get("canonical_form"),
                    use_canonical_comparison: row.get("use_canonical_comparison"),
                },
                progress: vec![progress],
            };
            all_flashcards_with_retrievability.push((card_response, retrievability));
        }
    }

    // Calculate pagination
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let offset = ((page - 1) * per_page) as usize;

    let paginated_cards: Vec<FlashcardResponse> = all_flashcards_with_retrievability
        .iter()
        .skip(offset)
        .take(per_page as usize)
        .map(|(card, _)| card.clone())
        .collect();

    let due_count = if query.due.unwrap_or(false) {
        all_flashcards_with_retrievability.len() as i64
    } else {
        rows.iter()
            .filter(|row| {
                let next_review: Option<DateTime<Utc>> = row.get("next_review_at");
                next_review.is_none_or(|t| t <= Utc::now())
            })
            .filter(|row| {
                // Only count if the card ID matches the query ID, if provided
                query.flashcard_id.is_none()
                    || row.get::<_, i32>("id") == query.flashcard_id.unwrap()
            })
            .count() as i64
    };

    // If a specific flashcard_id was requested, the total is 1 if found, 0 otherwise
    let final_total = if query.flashcard_id.is_some() {
        if paginated_cards.is_empty() {
            0
        } else {
            1
        }
    } else {
        all_flashcards_with_retrievability.len() as i64
    };

    transaction.commit().await?;

    Ok(FlashcardListResponse {
        flashcards: paginated_cards,
        total: final_total,
        page,
        per_page,
        due_count,
    })
}

async fn get_quiz_options_for_listing(
    transaction: &Transaction<'_>,
    flashcard_id: i32,
    user_id: i32,
    effective_direction_str: &str, // "direct" or "reverse"
) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
    // 1. Fetch the flashcard details to determine question and correct answer source
    let card_details_row = transaction.query_one(
        "SELECT f.direction as actual_direction, ci.definition_id, ci.free_content_front, ci.free_content_back,
                v.word, d.definition, fqo.correct_answer_text
         FROM flashcards f
         JOIN collection_items ci ON f.item_id = ci.item_id
         LEFT JOIN definitions d ON ci.definition_id = d.definitionid
         LEFT JOIN valsi v ON d.valsiid = v.valsiid
         LEFT JOIN flashcard_quiz_options fqo ON f.id = fqo.flashcard_id
         WHERE f.id = $1",
        &[&flashcard_id]
    ).await?;

    let definition_id: Option<i32> = card_details_row.get("definition_id");
    let free_content_front: Option<String> = card_details_row.get("free_content_front");
    let free_content_back: Option<String> = card_details_row.get("free_content_back");
    let word: Option<String> = card_details_row.get("word");
    let definition_text: Option<String> = card_details_row.get("definition");
    let correct_answer_text: String = transaction
        .query_opt(
            "SELECT correct_answer_text FROM flashcard_quiz_options WHERE flashcard_id = $1",
            &[&flashcard_id],
        )
        .await?
        .map(|r| r.get(0))
        .ok_or_else(|| {
            format!(
                "Correct answer not found for quiz flashcard_id: {}",
                flashcard_id
            )
        })?;

    // 2. Determine Question Text
    let question_text = match effective_direction_str {
        "direct" => {
            if definition_id.is_some() {
                word.ok_or("Missing word for direct quiz")?
            } else {
                free_content_front.ok_or("Missing front content for direct quiz")?
            }
        }
        "reverse" => {
            if definition_id.is_some() {
                definition_text.ok_or("Missing definition for reverse quiz")?
            } else {
                free_content_back.ok_or("Missing back content for reverse quiz")?
            }
        }
        _ => return Err("Invalid effective_direction for quiz".into()),
    };

    // 3. Fetch distractors - Exploitation (user's past incorrect answers)
    let mut distractors: Vec<String> = transaction.query(
        "SELECT selected_option_text
         FROM user_quiz_answer_history
         WHERE user_id = $1 AND flashcard_id = $2 AND is_correct_selection = false AND selected_option_text != $3
         GROUP BY selected_option_text
         ORDER BY COUNT(*) DESC, MAX(answered_at) DESC
         LIMIT 2",
        &[&user_id, &flashcard_id, &correct_answer_text]
    ).await?
    .iter()
    .map(|row| row.get("selected_option_text"))
    .collect();

    // 4. Fetch remaining distractors - Exploration (plausible incorrect answers from the same collection)
    let needed_distractors = 3 - distractors.len();
    if needed_distractors > 0 {
        // Determine what kind of text to fetch for distractors based on what the correct answer is
        let distractor_source_query_part = if effective_direction_str == "direct" {
            // Correct answer is like a definition
            "d.definition"
        } else {
            // Correct answer is like a word (for reverse quiz)
            "v.word"
        };

        let mut existing_options_for_filter = distractors.clone();
        existing_options_for_filter.push(correct_answer_text.clone());

        let exploration_distractors_rows = transaction.query(
            &format!(
                "SELECT DISTINCT {} AS distractor
                 FROM flashcards f
                 JOIN collection_items ci ON f.item_id = ci.item_id
                 LEFT JOIN definitions d ON ci.definition_id = d.definitionid
                 LEFT JOIN valsi v ON d.valsiid = v.valsiid
                 WHERE f.collection_id = (SELECT collection_id FROM flashcards WHERE id = $1)
                   AND {} IS NOT NULL
                   AND {} != $2 -- Not the correct answer
                   AND ({} = '' OR {} <> ALL($3)) -- Not one of the already selected historical distractors, handle empty string case
                 ORDER BY RANDOM()
                 LIMIT $4",
                distractor_source_query_part, distractor_source_query_part,
                distractor_source_query_part, distractor_source_query_part, distractor_source_query_part
            ),
            &[&flashcard_id, &correct_answer_text, &existing_options_for_filter, &(needed_distractors as i64)]
        ).await?;

        for row in exploration_distractors_rows {
            distractors.push(row.get("distractor"));
        }
    }

    // Ensure we have exactly 3 distractors, pad with generic ones if necessary
    let mut fallback_idx = 1;
    while distractors.len() < 3 {
        distractors.push(format!("Fallback Option {}", fallback_idx));
        fallback_idx += 1;
    }
    distractors.truncate(3);

    let mut final_options = vec![correct_answer_text];
    final_options.extend(distractors);
    use rand::seq::SliceRandom;
    final_options.shuffle(&mut rand::rng());

    Ok((question_text, final_options))
}

// Calculate optimal retention for a user
pub async fn calculate_optimal_retention(
    transaction: &Transaction<'_>,
    user_id: i32,
) -> Result<f32, Box<dyn std::error::Error>> {
    // Default retention if we can't calculate an optimal one
    let default_retention = 0.9;

    // Check if user has enough review history to calculate optimal retention
    let review_count: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM flashcard_review_history WHERE user_id = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    // If user doesn't have enough review history, use default retention
    if review_count < 50 {
        return Ok(default_retention);
    }

    // Extract review history to build RevlogEntry objects
    let rows = transaction
        .query(
            "SELECT
                id,
                flashcard_id::bigint as cid,
                0 as usn,
                rating as button_chosen,
                scheduled_days as interval,
                elapsed_days as last_interval,
                0 as ease_factor,
                0 as taken_millis,
                CASE
                    WHEN (SELECT status FROM user_flashcard_progress
                          WHERE user_id = $1 AND flashcard_id = flashcard_review_history.flashcard_id
                          AND card_side = flashcard_review_history.card_side
                          LIMIT 1) = 'learning' THEN 0
                    WHEN (SELECT status FROM user_flashcard_progress
                          WHERE user_id = $1 AND flashcard_id = flashcard_review_history.flashcard_id
                          AND card_side = flashcard_review_history.card_side
                          LIMIT 1) = 'review' THEN 1
                    ELSE 2
                END as review_kind
             FROM flashcard_review_history
             WHERE user_id = $1
             ORDER BY review_time",
            &[&user_id],
        )
        .await?;

    // Convert rows to RevlogEntry objects
    let mut revlogs = Vec::new();
    for row in rows {
        let revlog = RevlogEntry {
            id: row.get::<_, i32>("id") as i64, // Store in i64 but read as i32 from Postgres
            cid: row.get("cid"),
            usn: row.get("usn"),
            button_chosen: row.get::<_, i32>("button_chosen") as u8,
            interval: row.get("interval"),
            last_interval: row.get("last_interval"),
            ease_factor: row.get::<_, i32>("ease_factor") as u32,
            taken_millis: row.get::<_, i32>("taken_millis") as u32,
            review_kind: match row.get::<_, i32>("review_kind") {
                0 => RevlogReviewKind::Learning,
                1 => RevlogReviewKind::Review,
                _ => RevlogReviewKind::Relearning,
            },
        };
        revlogs.push(revlog);
    }

    // If we don't have enough revlogs, use default retention
    if revlogs.is_empty() {
        return Ok(default_retention);
    }

    // Extract simulator config from review history
    let day_cutoff = Utc::now().timestamp() - 86400; // 24 hours ago
    let config = extract_simulator_config(revlogs, day_cutoff, true);

    // Calculate optimal retention
    let fsrs = fsrs::FSRS::new(Some(&[]))?;

    // Use a progress callback that always returns true to avoid interruption
    let progress_callback = |_: ItemProgress| true;

    match fsrs.optimal_retention(&config, &[], progress_callback) {
        Ok(optimal_retention) => {
            // Cache the optimal retention value in the database for this user
            transaction
                .execute(
                    "INSERT INTO user_settings (user_id, optimal_retention, last_calculated)
                     VALUES ($1, $2, CURRENT_TIMESTAMP)
                     ON CONFLICT (user_id)
                     DO UPDATE SET optimal_retention = $2, last_calculated = CURRENT_TIMESTAMP",
                    &[&user_id, &(optimal_retention as f64)],
                )
                .await?;

            Ok(optimal_retention)
        }
        Err(_) => Ok(default_retention),
    }
}

// Function to get the optimal retention for a user (with caching)
pub async fn get_optimal_retention(
    transaction: &Transaction<'_>,
    user_id: i32,
) -> Result<f32, Box<dyn std::error::Error>> {
    // Check if we have a cached value that's less than 7 days old
    let cached = transaction
        .query_opt(
            "SELECT optimal_retention, last_calculated
             FROM user_settings
             WHERE user_id = $1",
            &[&user_id],
        )
        .await?;

    if let Some(row) = cached {
        let last_calculated: DateTime<Utc> = row.get("last_calculated");
        let now = Utc::now();

        // If the cached value is less than 7 days old, use it
        if now.signed_duration_since(last_calculated).num_days() < 7 {
            return Ok(row.get::<_, f64>("optimal_retention") as f32);
        }
    }

    // Otherwise, calculate a new value
    calculate_optimal_retention(transaction, user_id).await
}

pub async fn review_flashcard(
    pool: &Pool,
    user_id: i32,
    req: &ReviewRequest,
) -> Result<ReviewResponse, Box<dyn std::error::Error>> {
    if !(1..=4).contains(&req.rating) {
        return Err("Rating must be between 1 and 4".into());
    }

    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_study_access(&transaction, req.flashcard_id, user_id).await?;

    // If the review is "good" or better (rating >= 3), check for related cards
    if req.rating >= 3 {
        // Get the word for the current flashcard
        // only applicable to flashcards linked to definitions, not to free_content cards
        if let Ok(row) = transaction
            .query_one(
                "SELECT v.word
                FROM flashcards f
                JOIN collection_items ci ON f.item_id = ci.item_id
                JOIN definitions d ON ci.definition_id = d.definitionid
                JOIN valsi v ON d.valsiid = v.valsiid
                WHERE f.id = $1",
                &[&req.flashcard_id],
            )
            .await
        {
            let word: String = row.get("word");
            auto_progress_related_cards(
                &transaction,
                user_id,
                req.flashcard_id,
                &word,
                &req.card_side,
            )
            .await?;
        }
    }

    // Get the current state for the specific side
    let current_state = transaction
        .query_opt(
            "SELECT stability, difficulty, status
             FROM user_flashcard_progress
             WHERE user_id = $1 AND flashcard_id = $2 AND card_side = $3 AND NOT archived",
            &[&user_id, &req.flashcard_id, &req.card_side],
        )
        .await?;

    let current_state = match current_state {
        Some(row) => {
            let stability = row.get::<_, f64>("stability") as f32;
            let difficulty = row.get::<_, f64>("difficulty") as f32;
            Some(MemoryState {
                stability,
                difficulty,
            })
        }
        None => None,
    };

    let current_state = match current_state {
        Some(row) => {
            let stability = row.stability;
            let difficulty = row.difficulty;
            debug!(
                "Current state - stability: {}, difficulty: {}",
                stability, difficulty
            );
            Some(MemoryState {
                stability,
                difficulty,
            })
        }
        None => {
            debug!("No current state found");
            None
        }
    };

    // Get and log last review time
    let last_review: Option<DateTime<Utc>> = transaction
        .query_opt(
            "SELECT last_reviewed_at
             FROM user_flashcard_progress
             WHERE user_id = $1 AND flashcard_id = $2 and card_side = $3 AND NOT archived",
            &[&user_id, &req.flashcard_id, &req.card_side],
        )
        .await?
        .and_then(|row| row.get("last_reviewed_at"));

    debug!("Last review: {:?}", last_review);

    let elapsed_days = if let Some(last) = last_review {
        let days = Utc::now().signed_duration_since(last).num_days().max(0) as u32;
        debug!("Elapsed days: {}", days);
        days
    } else {
        debug!("No previous review");
        0
    };

    let fsrs = FSRS::new(Some(&[]))?;
    //todo: uncomment to use hardcoded desired_retention
    // let desired_retention: f32 = 0.9;
    let desired_retention = get_optimal_retention(&transaction, user_id).await?;
    debug!("Using optimal retention: {}", desired_retention);

    let next_states = match fsrs.next_states(current_state, desired_retention, elapsed_days) {
        Ok(states) => states,
        Err(e) => {
            debug!("{:#?}", e);
            // Use default values if FSRS calculation fails
            let default_memory = MemoryState {
                stability: 1.0,
                difficulty: 5.0,
            };

            // Create default intervals for each rating
            let again = ItemState {
                memory: default_memory,
                interval: 1.0,
            };
            let hard = ItemState {
                memory: default_memory,
                interval: 3.0,
            };
            let good = ItemState {
                memory: default_memory,
                interval: 7.0,
            };
            let easy = ItemState {
                memory: default_memory,
                interval: 14.0,
            };

            NextStates {
                again,
                hard,
                good,
                easy,
            }
        }
    };

    // Calculate next interval based on rating
    let (interval, new_state) = match req.rating {
        1 => (next_states.again.interval as i32, next_states.again.memory),
        2 => (next_states.hard.interval as i32, next_states.hard.memory),
        3 => (next_states.good.interval as i32, next_states.good.memory),
        4 => (next_states.easy.interval as i32, next_states.easy.memory),
        _ => (next_states.again.interval as i32, next_states.again.memory),
    };

    let next_review = Utc::now() + Duration::days(interval as i64);

    // Record review history
    transaction
        .execute(
            "INSERT INTO flashcard_review_history
     (user_id, flashcard_id, card_side, rating, elapsed_days, scheduled_days, state)
     VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &user_id,
                &req.flashcard_id,
                &req.card_side,
                &(req.rating as i32),
                &(elapsed_days as i32),
                &interval,
                &serde_json::json!({
                    "stability": new_state.stability,
                    "difficulty": new_state.difficulty
                }),
            ],
        )
        .await?;

    // Update learning state
    let new_status = match (
        req.rating,
        get_current_status(&transaction, req.flashcard_id, user_id, &req.card_side).await?,
    ) {
        (1, _) => FlashcardStatus::Learning,
        (_, FlashcardStatus::New) if req.rating >= 3 => FlashcardStatus::Learning,
        (_, FlashcardStatus::Learning) if req.rating >= 3 => FlashcardStatus::Review,
        (_, FlashcardStatus::Review) if req.rating >= 3 => FlashcardStatus::Graduated,
        (_, current) => current, // Keep current status if none of the above conditions met
    };

    // Initialize values for new cards
    let (stability, difficulty) = if current_state.is_none() {
        // Get initial s0 stability based on first rating
        let initial_stability = match req.rating {
            1 => 0.4,  // DEFAULT_PARAMETERS[0]
            2 => 1.2,  // DEFAULT_PARAMETERS[1]
            3 => 3.2,  // DEFAULT_PARAMETERS[2]
            4 => 15.7, // DEFAULT_PARAMETERS[3]
            _ => 0.4,
        };

        // Initial difficulty should be calculated based on rating
        // Using w[4] - exp(w[5] * (rating - 1)) + 1.0
        let w4 = 7.1949; // DEFAULT_PARAMETERS[4]
        let w5 = 0.5345; // DEFAULT_PARAMETERS[5]
        let initial_difficulty =
            (w4 - (w5 * (req.rating as f64 - 1.0)).exp() + 1.0).clamp(1.0, 10.0);

        (initial_stability, initial_difficulty)
    } else {
        (new_state.stability as f64, new_state.difficulty as f64)
    };

    info!("Starting review update. Params: user={}, flashcard={}, side={}, stability={}, difficulty={}, interval={}, next_review={:?}, status={:?}",
    user_id, req.flashcard_id, req.card_side, stability, difficulty, interval, next_review, new_status);

    // Check existing record
    let existing = transaction
        .query_opt(
            "SELECT id FROM user_flashcard_progress
         WHERE user_id = $1 AND flashcard_id = $2 AND card_side = $3 AND NOT archived", // Ensure we target non-archived
            &[&user_id, &req.flashcard_id, &req.card_side],
        )
        .await?;
    info!("Existing record: {:?}", existing);

    // Update progress
    transaction
        .execute(
            "INSERT INTO user_flashcard_progress
     (user_id, flashcard_id, card_side, stability, difficulty, interval,
      next_review_at, last_reviewed_at, review_count, status)
     VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, 1, $8)
     ON CONFLICT (user_id, flashcard_id, card_side) WHERE NOT archived
     DO UPDATE SET
        card_side = $3,
        stability = $4,
        difficulty = $5,
        interval = $6,
        next_review_at = $7,
        last_reviewed_at = CURRENT_TIMESTAMP,
        review_count = user_flashcard_progress.review_count + 1,
        status = $8",
            &[
                &user_id,
                &req.flashcard_id,
                &req.card_side,
                &stability,
                &difficulty,
                &interval,
                &next_review,
                &new_status,
            ],
        )
        .await?;

    transaction.commit().await?;

    Ok(ReviewResponse {
        success: true,
        card_side: req.card_side.clone(),
        message: format!(
            "Review recorded successfully for {} side. New interval: {} days",
            req.card_side, interval
        ),
        next_review: Some(next_review),
    })
}

async fn get_current_status(
    transaction: &Transaction<'_>,
    flashcard_id: i32,
    user_id: i32,
    card_side: &String,
) -> Result<FlashcardStatus, Box<dyn std::error::Error>> {
    Ok(transaction
        .query_opt(
            "SELECT status FROM user_flashcard_progress
             WHERE flashcard_id = $1 AND user_id = $2 and card_side = $3 AND NOT archived",
            &[&flashcard_id, &user_id, card_side],
        )
        .await?
        .map(|row| row.get("status"))
        .unwrap_or(FlashcardStatus::New))
}

pub async fn reset_progress(
    pool: &Pool,
    user_id: i32,
    flashcard_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_study_access(&transaction, flashcard_id, user_id).await?;

    // Clear review history
    transaction
        .execute(
            "DELETE FROM flashcard_review_history
         WHERE user_id = $1 AND flashcard_id = $2",
            &[&user_id, &flashcard_id],
        )
        .await?;

    // Reset progress
    transaction
        .execute(
            "UPDATE user_flashcard_progress
         SET status = 'new',
             stability = 0.0,
             difficulty = 5.0,
             interval = 0,
             review_count = 0,
             last_reviewed_at = NULL,
             next_review_at = CURRENT_TIMESTAMP,
             ease_factor = 2.5
         WHERE user_id = $1 AND flashcard_id = $2 AND NOT archived",
            &[&user_id, &flashcard_id],
        )
        .await?;

    transaction.commit().await?;
    Ok(())
}

async fn verify_study_access(
    transaction: &Transaction<'_>,
    flashcard_id: i32,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Check collection ownership/public status
    let row = transaction
        .query_one(
            "SELECT c.user_id, c.is_public
             FROM flashcards f
             JOIN collections c ON f.collection_id = c.collection_id
             WHERE f.id = $1",
            &[&flashcard_id],
        )
        .await?;

    let owner_id: i32 = row.get("user_id");
    let is_public: bool = row.get("is_public");

    if !is_public && owner_id != user_id {
        return Err("access denied".into());
    }

    // 2. Check if the flashcard belongs to a level and if that level is unlocked
    let level_check = transaction
        .query_opt(
            "SELECT fli.level_id, check_level_prerequisites($1, fli.level_id) as unlocked
             FROM flashcard_level_items fli
             WHERE fli.flashcard_id = $2",
            &[&user_id, &flashcard_id],
        )
        .await?;

    if let Some(level_row) = level_check {
        let is_unlocked: bool = level_row.get("unlocked");
        if !is_unlocked {
            let level_id: i32 = level_row.get("level_id");
            return Err(format!(
                "access denied: level {} is locked. Complete prerequisites first.",
                level_id
            )
            .into());
        }
    }

    // If checks pass
    Ok(())
}

pub async fn update_flashcard_position(
    pool: &Pool,
    user_id: i32,
    flashcard_id: i32,
    new_position: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_flashcard_ownership(&transaction, flashcard_id, user_id).await?;

    // Get collection_id and current position
    let flashcard = transaction
        .query_one(
            "SELECT collection_id, position FROM flashcards WHERE id = $1",
            &[&flashcard_id],
        )
        .await?;

    let collection_id = flashcard.get::<_, i32>("collection_id");

    // Verify collection ownership
    let owner_id: i32 = transaction
        .query_one(
            "SELECT user_id FROM collections WHERE collection_id = $1",
            &[&collection_id],
        )
        .await?
        .get("user_id");

    if owner_id != user_id {
        return Err("access denied".into());
    }

    // First update all positions to temporary negative values to avoid unique constraint violations
    transaction
        .execute(
            "UPDATE flashcards SET position = -position - 1 WHERE collection_id = $1",
            &[&collection_id],
        )
        .await?;

    // Now perform the position update with the new desired positions
    transaction.execute(
            r#"
            WITH current_positions AS (
                SELECT id, position FROM flashcards
                WHERE collection_id = $1
                ORDER BY ABS(position)
            ),
            with_index AS (
                SELECT
                    id,
                    ROW_NUMBER() OVER () - 1 as desired_position
                FROM current_positions
            ),
            new_positions AS (
                SELECT
                    id,
                    CASE
                        WHEN id = $2 THEN $3::int4
                        WHEN desired_position >= $3::int4 AND desired_position < (SELECT desired_position FROM with_index WHERE id = $2)
                            THEN desired_position + 1
                        WHEN desired_position <= $3::int4 AND desired_position > (SELECT desired_position FROM with_index WHERE id = $2)
                            THEN desired_position - 1
                        ELSE desired_position
                    END as new_position
                FROM with_index
            )
            UPDATE flashcards f
            SET position = np.new_position
            FROM new_positions np
            WHERE f.id = np.id
            "#,
            &[&collection_id, &flashcard_id, &new_position]
        ).await?;

    // Finally, ensure positions form a contiguous sequence starting from 0
    transaction
        .execute(
            r#"
            WITH ordered AS (
                SELECT
                    id,
                    ROW_NUMBER() OVER (ORDER BY position) - 1 as desired_position
                FROM flashcards
                WHERE collection_id = $1
            )
            UPDATE flashcards f
            SET position = o.desired_position
            FROM ordered o
            WHERE f.id = o.id"#,
            &[&collection_id],
        )
        .await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn import_from_collection(
    pool: &Pool,
    user_id: i32,
    collection_id: i32,
) -> Result<ImportFromCollectionResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Check if collection exists and user has access to it
    let _collection = transaction
        .query_opt(
            "SELECT collection_id FROM collections
             WHERE collection_id = $1 AND (user_id = $2 OR is_public = true)",
            &[&collection_id, &user_id],
        )
        .await?
        .ok_or("Collection not found or access denied")?;

    // Get all items from the collection
    let items = transaction
        .query(
            "SELECT ci.*
             FROM collection_items ci
             WHERE ci.collection_id = $1
             ORDER BY ci.position",
            &[&collection_id],
        )
        .await?;

    let mut imported_count = 0;
    let mut skipped_count = 0;

    // Get current max position
    let max_position: i32 = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) FROM flashcards WHERE collection_id = $1",
            &[&collection_id],
        )
        .await?
        .get(0);

    // Create flashcards for each item if they don't already exist
    for (idx, item) in items.iter().enumerate() {
        let item_id: i32 = item.get("item_id");

        // Check if flashcard already exists
        let exists = transaction
            .query_one(
                "SELECT EXISTS(
                    SELECT 1 FROM flashcards f
                    WHERE f.collection_id = $1 and f.item_id = $2
                )",
                &[&collection_id, &item_id],
            )
            .await?
            .get::<_, bool>(0);

        if exists {
            skipped_count += 1;
            continue;
        }

        // Check if flashcard already exists for this collection/item
        let exists = transaction
            .query_one(
                "SELECT 1 FROM flashcards WHERE collection_id = $1 AND item_id = $2",
                &[&collection_id, &item_id],
            )
            .await
            .is_ok();

        if exists {
            continue;
        }

        // Insert new flashcard with unique position
        let flashcard_id: i32 = transaction
            .query_one(
                "INSERT INTO flashcards (collection_id, position, item_id, direction)
                 VALUES ($1, $2, $3, $4)
                 RETURNING id",
                &[
                    &collection_id,
                    &(max_position + 1 + idx as i32),
                    &item_id,
                    &FlashcardDirection::Both,
                ],
            )
            .await?
            .get(0);

        // Create initial progress record
        transaction
            .execute(
                "INSERT INTO user_flashcard_progress
                 (user_id, flashcard_id, next_review_at, card_side)
                 VALUES ($1, $2, CURRENT_TIMESTAMP, 'direct')
                 ON CONFLICT (user_id, flashcard_id, card_side) DO NOTHING",
                &[&user_id, &flashcard_id],
            )
            .await?;
        transaction
            .execute(
                "INSERT INTO user_flashcard_progress
                 (user_id, flashcard_id, next_review_at, card_side)
                 VALUES ($1, $2, CURRENT_TIMESTAMP, 'reverse')
                 ON CONFLICT (user_id, flashcard_id, card_side) DO NOTHING",
                &[&user_id, &flashcard_id],
            )
            .await?;

        imported_count += 1;
    }

    transaction.commit().await?;

    Ok(ImportFromCollectionResponse {
        imported_count,
        skipped_count,
    })
}

pub async fn check_answer(
    pool: &Pool,
    user_id: i32,
    req: &DirectAnswerRequest,
) -> Result<DirectAnswerResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_study_access(&transaction, req.flashcard_id, user_id).await?;

    // Prevent answers for "JustInformation" cards
    let direction: FlashcardDirection = transaction
        .query_one(
            "SELECT direction FROM flashcards WHERE id = $1",
            &[&req.flashcard_id],
        )
        .await?
        .get("direction");
    if direction == FlashcardDirection::JustInformation {
        return Err("Cannot answer 'Just Information' cards.".into());
    }

    // Get use_canonical_comparison flag
    let use_canonical: bool = transaction
        .query_one(
            "SELECT use_canonical_comparison FROM flashcards WHERE id = $1",
            &[&req.flashcard_id],
        )
        .await?
        .get(0);

    let flashcard = get_flashcard(&transaction, req.flashcard_id).await?;

    let (expected, provided, is_free_content) = match req.card_side.as_str() {
        "direct" => {
            if let Some(def) = flashcard.definition {
                (
                    def.trim().to_lowercase(),
                    req.answer.trim().to_lowercase(),
                    false,
                )
            } else if let Some(content) = flashcard.free_content_back {
                (
                    content.trim().to_lowercase(),
                    req.answer.trim().to_lowercase(),
                    true,
                )
            } else {
                return Err("Invalid flashcard content".into());
            }
        }
        "reverse" => {
            if let Some(word) = flashcard.word {
                (
                    word.trim().to_lowercase(),
                    req.answer.trim().to_lowercase(),
                    false,
                )
            } else if let Some(content) = flashcard.free_content_front {
                (
                    content.trim().to_lowercase(),
                    req.answer.trim().to_lowercase(),
                    true,
                )
            } else {
                return Err("Invalid flashcard content".into());
            }
        }
        _ => {
            return Err(format!(
                "Invalid card side '{}' for direction '{:?}'",
                req.card_side, flashcard.direction
            )
            .into())
        }
    };

    // Check if answer is correct, using canonical comparison if enabled
    let is_correct = if use_canonical && is_free_content {
        // Try canonical comparison for free content (Lojban phrases)
        if let (Some(expected_canon), Some(provided_canon)) = (
            crate::tersmu::get_canonical_form(&expected),
            crate::tersmu::get_canonical_form(&provided),
        ) {
            expected_canon == provided_canon
        } else {
            // Fall back to string comparison if canonicalization fails
            expected == provided
        }
    } else {
        // Use standard string comparison
        expected == provided
    };

    if is_correct {
        let review_req = ReviewRequest {
            flashcard_id: req.flashcard_id,
            rating: 4,
            card_side: req.card_side.clone(),
        };

        let review_result = review_flashcard(pool, user_id, &review_req).await?;

        Ok(DirectAnswerResponse {
            correct: true,
            expected: expected.clone(), // Clone here
            message: format!("Correct! {}", review_result.message),
            next_review: review_result.next_review,
            is_free_content,
        })
    } else {
        let answer_message = format!("Incorrect. The correct answer was: {}", expected);
        Ok(DirectAnswerResponse {
            correct: false,
            expected, // Move here is fine
            message: answer_message,
            next_review: None,
            is_free_content,
        })
    }
}

// Calculate similarity between two strings (0.0 to 1.0)
fn calculate_similarity(s1: &str, s2: &str) -> f32 {
    let s1 = s1.trim().to_lowercase();
    let s2 = s2.trim().to_lowercase();

    if s1 == s2 {
        return 1.0;
    }

    // Levenshtein distance calculation
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    if len1 == 0 {
        return if len2 == 0 { 1.0 } else { 0.0 };
    }
    if len2 == 0 {
        return 0.0;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }

    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    let distance = matrix[len1][len2];
    let max_len = std::cmp::max(len1, len2);

    if max_len == 0 {
        1.0
    } else {
        1.0 - (distance as f32 / max_len as f32)
    }
}

pub async fn review_flashcard_serverside(
    pool: &Pool,
    user_id: i32,
    req: &FillInAnswerRequest,
) -> Result<ReviewResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_study_access(&transaction, req.flashcard_id, user_id).await?;

    // Prevent answers for "JustInformation" cards
    let direction: FlashcardDirection = transaction
        .query_one(
            "SELECT direction FROM flashcards WHERE id = $1",
            &[&req.flashcard_id],
        )
        .await?
        .get("direction");
    if direction == FlashcardDirection::JustInformation {
        return Err("Cannot answer 'Just Information' cards.".into());
    }

    let flashcard = get_flashcard(&transaction, req.flashcard_id).await?;

    // Determine if it's free content
    let is_free_content = flashcard.definition_id.is_none();

    // Get the expected answer string (raw, without lowercasing yet)
    let expected_raw = match (&flashcard.direction, req.card_side.as_str()) {
        // FillIn or FillInBoth, direct side: Expect definition or back content
        (&FlashcardDirection::FillIn, "direct") | (&FlashcardDirection::FillInBoth, "direct") => {
            if let Some(def) = &flashcard.definition {
                def.trim().to_lowercase()
            } else if let Some(content) = &flashcard.free_content_back {
                content.trim().to_lowercase()
            } else {
                return Err("Invalid flashcard content for fill-in (direct)".into());
            }
        }
        // FillInReverse or FillInBoth, reverse side: Expect word or front content
        (&FlashcardDirection::FillInReverse, "reverse")
        | (&FlashcardDirection::FillInBoth, "reverse") => {
            if let Some(word) = &flashcard.word {
                word.trim().to_lowercase()
            } else if let Some(content) = &flashcard.free_content_front {
                content.trim().to_lowercase()
            } else {
                return Err("Invalid flashcard content for fill-in (reverse)".into());
            }
        }
        // Handle other directions (Direct, Reverse, Both) - they shouldn't use this endpoint
        (&FlashcardDirection::Direct, _)
        | (&FlashcardDirection::Reverse, _)
        | (&FlashcardDirection::Both, _) => {
            return Err(format!(
                "Fill-in endpoint called for non-fill-in direction '{:?}'",
                &flashcard.direction
            )
            .into());
        }
        // Catch any other invalid side combinations
        (direction, side) => {
            return Err(format!(
                "Invalid card side '{}' for fill-in direction '{:?}'",
                side, direction
            )
            .into());
        }
    };

    let provided = req.answer.trim().to_lowercase();

    // Calculate similarity, handling semicolon-separated answers for free content
    let similarity = if is_free_content && expected_raw.contains(';') {
        // 1. Split, trim, lowercase, and filter empty parts
        let possible_answers: Vec<String> = expected_raw
            .split(';')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty()) // Filter out empty strings after trimming
            .collect();

        // 2. Calculate maximum similarity if there are valid possible answers
        if !possible_answers.is_empty() {
            let mut max_similarity = 0.0f32;
            for possible_answer in possible_answers {
                let current_similarity = calculate_similarity(&provided, &possible_answer);
                max_similarity = max_similarity.max(current_similarity);
            }
            max_similarity // Use the maximum similarity found
        } else {
            // Fallback: If splitting results in no valid answers (e.g., ";;"),
            // calculate similarity against the original string.
            calculate_similarity(&expected_raw.trim().to_lowercase(), &provided)
        }
    } else {
        // Not free content or no semicolons, use standard similarity calculation
        calculate_similarity(&expected_raw.trim().to_lowercase(), &provided)
    };

    // Determine rating based on similarity
    let rating = if similarity >= 0.99 {
        4 // Perfect match - "Easy"
    } else if similarity >= 0.9 {
        3 // Good match - "Good"
    } else if similarity >= 0.7 {
        2 // Fair match - "Hard"
    } else {
        1 // Poor match - "Again"
    };

    // Create a review request with the determined rating
    let review_req = ReviewRequest {
        flashcard_id: req.flashcard_id,
        rating,
        card_side: req.card_side.clone(),
    };

    // Process the review
    let review_result = review_flashcard(pool, user_id, &review_req).await?;

    // Add similarity info to the response message
    let message = format!(
        "{} (Similarity: {:.1}%, Rating: {})",
        review_result.message,
        similarity * 100.0,
        match rating {
            4 => "Easy",
            3 => "Good",
            2 => "Hard",
            _ => "Again",
        }
    );

    Ok(ReviewResponse {
        success: true,
        card_side: req.card_side.clone(),
        message,
        next_review: review_result.next_review,
    })
}

async fn auto_progress_related_cards(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    word: &str,
    card_side: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Escape special regex characters in the word
    let escaped_word = regex::escape(word);
    let pattern = format!("(?:^|[^a-zA-Z']){}(?:[^a-zA-Z']|$)", escaped_word);

    let related_cards = transaction
        .query(
            "WITH current_card AS (
                SELECT f.collection_id, v.word
                FROM flashcards f
                JOIN collection_items ci1 ON f.item_id = ci1.item_id
                JOIN definitions d ON ci1.definition_id = d.definitionid
                JOIN valsi v ON d.valsiid = v.valsiid
                WHERE f.id = $1
            )
            SELECT DISTINCT f2.id, v2.word
            FROM flashcards f2
            JOIN collection_items ci ON f2.item_id = ci.item_id
            JOIN definitions d2 ON ci.definition_id = d2.definitionid
            JOIN valsi v2 ON d2.valsiid = v2.valsiid
            JOIN current_card cc ON f2.collection_id = cc.collection_id
            JOIN user_flashcard_progress ufp ON f2.id = ufp.flashcard_id
            WHERE f2.id != $1
            AND ci.auto_progress = true
            AND ufp.user_id = $2
            AND ufp.card_side = $3
            AND NOT ufp.archived
            AND ufp.status != 'graduated'
            AND v2.word ~ $4",
            &[&flashcard_id, &user_id, &card_side, &pattern],
        )
        .await?;

    for row in related_cards {
        let card_id: i32 = row.get("id");
        let review_req = ReviewRequest {
            flashcard_id: card_id,
            rating: 4,
            card_side: card_side.to_string(),
        };
        record_review(transaction, user_id, &review_req).await?;
    }

    Ok(())
}

async fn record_review(
    transaction: &Transaction<'_>,
    user_id: i32,
    req: &ReviewRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get current state
    let current_state = transaction
        .query_opt(
            "SELECT stability, difficulty, status
             FROM user_flashcard_progress
             WHERE user_id = $1 AND flashcard_id = $2 AND card_side = $3 AND NOT archived",
            &[&user_id, &req.flashcard_id, &req.card_side],
        )
        .await?;

    let current_state = current_state.map(|row| MemoryState {
        stability: row.get::<_, f64>("stability") as f32,
        difficulty: row.get::<_, f64>("difficulty") as f32,
    });

    let last_review: Option<DateTime<Utc>> = transaction
        .query_opt(
            "SELECT last_reviewed_at
             FROM user_flashcard_progress
             WHERE user_id = $1 AND flashcard_id = $2 and card_side = $3 AND NOT archived",
            &[&user_id, &req.flashcard_id, &req.card_side],
        )
        .await?
        .and_then(|row| row.get("last_reviewed_at"));

    let elapsed_days = if let Some(last) = last_review {
        Utc::now().signed_duration_since(last).num_days().max(0) as u32
    } else {
        0
    };

    let fsrs = FSRS::new(Some(&[]))?;
    let desired_retention = 0.9;

    let next_states = match fsrs.next_states(current_state, desired_retention, elapsed_days) {
        Ok(states) => states,
        Err(_) => {
            let default_memory = MemoryState {
                stability: 1.0,
                difficulty: 5.0,
            };
            NextStates {
                again: ItemState {
                    memory: default_memory,
                    interval: 1.0,
                },
                hard: ItemState {
                    memory: default_memory,
                    interval: 3.0,
                },
                good: ItemState {
                    memory: default_memory,
                    interval: 7.0,
                },
                easy: ItemState {
                    memory: default_memory,
                    interval: 14.0,
                },
            }
        }
    };

    let (interval, new_state) = match req.rating {
        1 => (next_states.again.interval as i32, next_states.again.memory),
        2 => (next_states.hard.interval as i32, next_states.hard.memory),
        3 => (next_states.good.interval as i32, next_states.good.memory),
        4 => (next_states.easy.interval as i32, next_states.easy.memory),
        _ => return Err("Invalid rating".into()),
    };

    let next_review = Utc::now() + Duration::days(interval as i64);

    // Record review history
    transaction
        .execute(
            "INSERT INTO flashcard_review_history
             (user_id, flashcard_id, rating, elapsed_days, scheduled_days,
              state, review_time, card_side)
             VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, $7)",
            &[
                &user_id,
                &req.flashcard_id,
                &(req.rating as i32),
                &(elapsed_days as i32),
                &interval,
                &serde_json::json!({
                    "stability": new_state.stability,
                    "difficulty": new_state.difficulty
                }),
                &req.card_side,
            ],
        )
        .await?;

    // Update progress
    transaction
        .execute(
            "INSERT INTO user_flashcard_progress
             (user_id, flashcard_id, card_side, stability, difficulty, interval,
              next_review_at, last_reviewed_at, review_count, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, 1, 'review')
             ON CONFLICT (user_id, flashcard_id, card_side) WHERE NOT archived
             DO UPDATE SET
                stability = $4,
                difficulty = $5,
                interval = $6,
                next_review_at = $7,
                last_reviewed_at = CURRENT_TIMESTAMP,
                review_count = user_flashcard_progress.review_count + 1,
                status = 'review'",
            &[
                &user_id,
                &req.flashcard_id,
                &req.card_side,
                &(new_state.stability as f64),
                &(new_state.difficulty as f64),
                &interval,
                &next_review,
            ],
        )
        .await?;

    Ok(())
}

pub async fn get_next_quiz_for_user(
    pool: &Pool,
    user_id: i32,
) -> Result<Option<dto::QuizFlashcardQuestionDto>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Find a due quiz flashcard for this user
    let flashcard_row = match transaction
        .query_opt(
            r#"
        SELECT f.id, f.direction
        FROM flashcards f
        JOIN user_flashcard_progress p ON f.id = p.flashcard_id
        WHERE p.user_id = $1
        AND p.next_review_at <= CURRENT_TIMESTAMP
        AND p.card_side = 'direct'  -- Quiz progress tracked on direct side
        AND f.direction IN ('QuizDirect', 'QuizReverse', 'QuizBoth')
        AND NOT p.archived
        ORDER BY p.next_review_at
        LIMIT 1
        "#,
            &[&user_id],
        )
        .await?
    {
        Some(row) => row,
        None => return Ok(None),
    };

    let flashcard_id: i32 = flashcard_row.get("id");
    let direction: FlashcardDirection = flashcard_row.get("direction");

    // Determine effective direction (direct/reverse) for QuizBoth
    let effective_direction = match direction {
        FlashcardDirection::QuizDirect => "direct",
        FlashcardDirection::QuizReverse => "reverse",
        FlashcardDirection::QuizBoth => {
            if rand::random() {
                "direct"
            } else {
                "reverse"
            }
        }
        _ => "direct", // Shouldn't happen since we filtered above
    };

    // Get question and options
    let (question_text, answer_options) =
        get_quiz_options_for_listing(&transaction, flashcard_id, user_id, effective_direction)
            .await?;

    transaction.commit().await?;

    Ok(Some(dto::QuizFlashcardQuestionDto {
        flashcard_id,
        question_text,
        answer_options,
    }))
}

pub async fn submit_quiz_answer(
    pool: &Pool,
    user_id: i32,
    answer_data: dto::SubmitQuizAnswerDto,
) -> Result<dto::QuizAnswerResultDto, Box<dyn std::error::Error>> {
    let flashcard_id = answer_data.flashcard_id;

    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_study_access(&transaction, flashcard_id, user_id).await?;

    // Fetch the correct answer
    let correct_answer_row = transaction
        .query_one(
            "SELECT correct_answer_text FROM flashcard_quiz_options WHERE flashcard_id = $1",
            &[&flashcard_id],
        )
        .await?;
    let correct_answer_text: String = correct_answer_row.get("correct_answer_text");

    let is_correct = answer_data.selected_answer_text.trim().to_lowercase()
        == correct_answer_text.trim().to_lowercase();

    // Log the attempt with all presented options
    transaction.execute(
        "INSERT INTO user_quiz_answer_history (user_id, flashcard_id, selected_option_text, is_correct_selection, presented_options, answered_at)
         VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)",
        &[&user_id, &flashcard_id, &answer_data.selected_answer_text, &is_correct, &serde_json::to_value(&answer_data.presented_options)?]
    ).await?;

    // Determine FSRS rating and call review_flashcard
    let rating = if is_correct { 4 } else { 1 }; // 4 for Easy (correct), 1 for Again (incorrect)
    let review_req = ReviewRequest {
        flashcard_id,
        rating,
        card_side: answer_data.card_side.clone(),
    };

    // Commit quiz history before calling review_flashcard which uses its own transaction.
    transaction.commit().await?;

    let review_response = review_flashcard(pool, user_id, &review_req).await?;

    Ok(dto::QuizAnswerResultDto {
        correct: is_correct,
        message: if is_correct {
            "Correct!".to_string()
        } else {
            format!("Incorrect. The correct answer was: {}", correct_answer_text)
        },
        next_review: review_response.next_review,
    })
}

pub async fn get_streak(
    pool: &Pool,
    user_id: i32,
    days: i32,
) -> Result<StreakResponse, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    // Get daily activity data
    let daily_rows = client
        .query(
            // Generate continuous date series and join with actual data
            r#"WITH RECURSIVE date_range AS (
                SELECT
                    CURRENT_DATE as date
                UNION ALL
                SELECT
                    date - 1
                FROM date_range
                WHERE date > CURRENT_DATE - ($2::int || ' days')::interval
            ),
            daily_points AS (
                SELECT
                    DATE(review_time) as review_date,
                    COUNT(*) as review_count,
                    SUM(
                        CASE rating
                            WHEN 1 THEN 1
                            WHEN 2 THEN 2
                            WHEN 3 THEN 3
                            WHEN 4 THEN 5
                            ELSE 0
                        END
                    ) as points
                FROM flashcard_review_history
                WHERE
                    user_id = $1 AND
                    review_time >= CURRENT_DATE - ($2::int || ' days')::interval
                GROUP BY DATE(review_time)
            )
            SELECT
                dr.date::timestamptz as date,
                COALESCE(dp.review_count, 0) as reviews_count,
                COALESCE(dp.points, 0) as points
            FROM date_range dr
            LEFT JOIN daily_points dp ON dr.date = dp.review_date
            ORDER BY dr.date DESC"#,
            &[&user_id, &days],
        )
        .await?;

    // Map to domain objects
    let daily_progress = daily_rows
        .iter()
        .map(|row| DailyProgress {
            date: row.get("date"),
            points: row.get::<_, i64>("points") as i32,
            reviews_count: row.get::<_, i64>("reviews_count") as i32,
        })
        .collect::<Vec<_>>();

    let total_points = daily_progress.iter().map(|d| d.points).sum();

    Ok(StreakResponse {
        current_streak: calculate_current_streak(&daily_progress),
        longest_streak: calculate_longest_streak(&client, user_id).await?,
        daily_progress,
        total_points,
    })
}

fn calculate_current_streak(daily_progress: &[DailyProgress]) -> i32 {
    let mut streak = 0;
    let mut prev_date: Option<DateTime<Utc>> = None;

    for progress in daily_progress {
        if progress.points > 0 {
            match prev_date {
                Some(prev) if (prev.signed_duration_since(progress.date)).num_days() == 1 => {
                    streak += 1
                }
                None => streak = 1,
                _ => break,
            }
            prev_date = Some(progress.date);
        } else if prev_date.is_none() {
            break;
        }
    }
    streak
}

async fn calculate_longest_streak(
    client: &deadpool_postgres::Client,
    user_id: i32,
) -> Result<i32, Box<dyn std::error::Error>> {
    let row = client
        .query_opt(
            r#"WITH daily_reviews AS (
                SELECT DISTINCT
                    DATE(review_time) as review_date
                FROM flashcard_review_history
                WHERE
                    user_id = $1 AND
                    rating > 0
            ),
            streaks AS (
                SELECT
                    review_date,
                    review_date - (ROW_NUMBER() OVER (ORDER BY review_date))::integer as streak_group
                FROM daily_reviews
            )
            SELECT COALESCE(MAX(streak_count), 0) as longest_streak
            FROM (
                SELECT COUNT(*) as streak_count
                FROM streaks
                GROUP BY streak_group
            ) t"#,
            &[&user_id],
        )
        .await?;

    Ok(row
        .map(|r| r.get::<_, i64>("longest_streak") as i32)
        .unwrap_or(0))
}

// levels:

pub async fn create_level(
    pool: &Pool,
    collection_id: i32,
    user_id: i32,
    req: &CreateLevelRequest,
) -> Result<LevelResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Get max position if not specified
    let position = match req.position {
        Some(pos) => pos,
        None => {
            let max_pos: i32 = transaction.query_one(
                "SELECT COALESCE(MAX(position), -1) FROM flashcard_levels WHERE collection_id = $1",
                &[&collection_id],
            ).await?.get(0);
            max_pos + 1
        }
    };

    // Create level
    let level_row = transaction.query_one(
        "INSERT INTO flashcard_levels (collection_id, name, description, min_cards, min_success_rate, position)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *",
        &[
            &collection_id,
            &req.name,
            &req.description,
            &req.min_cards.unwrap_or(5),
            &(req.min_success_rate.unwrap_or(0.8) as f64),
            &position,
        ],
    ).await?;

    // Add prerequisites
    for prereq_id in &req.prerequisite_ids {
        transaction
            .execute(
                "INSERT INTO level_prerequisites (level_id, prerequisite_id)
             VALUES ($1, $2)",
                &[&level_row.get::<_, i32>("level_id"), prereq_id],
            )
            .await?;
    }

    let level = get_level_details(&transaction, level_row.get("level_id"), Some(user_id)).await?;

    transaction.commit().await?;
    Ok(level)
}

pub async fn update_level(
    pool: &Pool,
    level_id: i32,
    user_id: i32,
    req: &UpdateLevelRequest,
) -> Result<LevelResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let collection_id = get_collection_id(&transaction, level_id).await?;
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Update level
    let update_result = transaction
        .execute(
            "UPDATE flashcard_levels
         SET name = COALESCE($1, name),
             description = COALESCE($2, description),
             min_cards = COALESCE($3, min_cards),
             min_success_rate = COALESCE($4, min_success_rate),
             position = COALESCE($5, position)
         WHERE level_id = $6",
            &[
                &req.name,
                &req.description,
                &req.min_cards,
                &req.min_success_rate,
                &req.position,
                &level_id,
            ],
        )
        .await?;

    if update_result == 0 {
        return Err("Level not found".into());
    }

    // Update prerequisites if provided
    if let Some(prereq_ids) = &req.prerequisite_ids {
        transaction
            .execute(
                "DELETE FROM level_prerequisites WHERE level_id = $1",
                &[&level_id],
            )
            .await?;

        for prereq_id in prereq_ids {
            transaction
                .execute(
                    "INSERT INTO level_prerequisites (level_id, prerequisite_id)
                 VALUES ($1, $2)",
                    &[&level_id, prereq_id],
                )
                .await?;
        }
    }

    let level = get_level_details(&transaction, level_id, Some(user_id)).await?;

    transaction.commit().await?;
    Ok(level)
}

pub async fn add_cards_to_level(
    pool: &Pool,
    level_id: i32,
    user_id: i32,
    req: &AddCardsRequest,
) -> Result<Vec<LevelCardResponse>, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let collection_id = get_collection_id(&transaction, level_id).await?;
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Get max position first
    let max_position = transaction
        .query_one(
            "SELECT COALESCE(MAX(position), -1) + 1
         FROM flashcard_level_items
         WHERE level_id = $1",
            &[&level_id],
        )
        .await?
        .get(0);

    let start_position = req.start_position.unwrap_or(max_position);

    // Add cards with sequential positions
    for (idx, flashcard_id) in req.flashcard_ids.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO flashcard_level_items (level_id, flashcard_id, position)
             VALUES ($1, $2, $3)
             ON CONFLICT (level_id, flashcard_id) DO UPDATE
             SET position = EXCLUDED.position",
                &[&level_id, flashcard_id, &(start_position + idx as i32)],
            )
            .await?;
    }

    let cards = get_level_cards(&transaction, level_id, Some(user_id)).await?;

    transaction.commit().await?;
    Ok(cards)
}

pub async fn get_level_details(
    transaction: &Transaction<'_>,
    level_id: i32,
    user_id: Option<i32>,
) -> Result<LevelResponse, Box<dyn std::error::Error>> {
    let level_row = transaction
        .query_one(
            "SELECT l.*, COUNT(fli.flashcard_id) as card_count
         FROM flashcard_levels l
         LEFT JOIN flashcard_level_items fli ON l.level_id = fli.level_id
         WHERE l.level_id = $1
         GROUP BY l.level_id",
            &[&level_id],
        )
        .await?;

    let prerequisites = get_prerequisites(transaction, level_id, user_id).await?;
    let progress = if let Some(uid) = user_id {
        get_level_progress(transaction, level_id, uid).await?
    } else {
        None
    };

    let is_locked = if let Some(uid) = user_id {
        !transaction
            .query_one(
                "SELECT check_level_prerequisites($1, $2) as unlocked",
                &[&uid, &level_id],
            )
            .await?
            .get::<_, bool>("unlocked")
    } else {
        true
    };

    let is_started = progress.as_ref().is_some_and(|p| p.total_answers > 0);

    Ok(LevelResponse {
        level_id,
        name: level_row.get("name"),
        description: level_row.get("description"),
        min_cards: level_row.get("min_cards"),
        min_success_rate: level_row.get::<_, f64>("min_success_rate") as f32,
        position: level_row.get("position"),
        card_count: level_row.get::<_, i64>("card_count") as i32,
        prerequisites,
        progress,
        is_locked,
        is_started,
        created_at: level_row.get("created_at"),
    })
}

pub async fn get_collection_levels(
    pool: &Pool,
    collection_id: i32,
    user_id: Option<i32>,
) -> Result<LevelListResponse, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let rows = transaction
        .query(
            "SELECT l.*, COUNT(fli.flashcard_id) as card_count
         FROM flashcard_levels l
         LEFT JOIN flashcard_level_items fli ON l.level_id = fli.level_id
         WHERE l.collection_id = $1
         GROUP BY l.level_id
         ORDER BY l.position",
            &[&collection_id],
        )
        .await?;

    let mut levels = Vec::new();
    for row in rows {
        let level_id = row.get("level_id");
        let prerequisites = get_prerequisites(&transaction, level_id, user_id).await?;
        let progress = if let Some(uid) = user_id {
            get_level_progress(&transaction, level_id, uid).await?
        } else {
            None
        };

        let is_locked = if let Some(uid) = user_id {
            !transaction
                .query_one(
                    "SELECT check_level_prerequisites($1, $2) as unlocked",
                    &[&uid, &level_id],
                )
                .await?
                .get::<_, bool>("unlocked")
        } else {
            true
        };

        let is_started = progress.as_ref().is_some_and(|p| p.total_answers > 0);

        levels.push(LevelResponse {
            level_id,
            name: row.get("name"),
            description: row.get("description"),
            min_cards: row.get("min_cards"),
            min_success_rate: row.get::<_, f64>("min_success_rate") as f32,
            position: row.get("position"),
            card_count: row.get::<_, i64>("card_count") as i32,
            prerequisites,
            progress,
            is_locked,
            is_started,
            created_at: ChronoDateTime(row.get("created_at")),
        });
    }

    transaction.commit().await?;

    Ok(LevelListResponse {
        total: levels.len() as i64,
        levels,
    })
}

async fn get_prerequisites(
    transaction: &Transaction<'_>,
    level_id: i32,
    user_id: Option<i32>,
) -> Result<Vec<PrerequisiteLevel>, Box<dyn std::error::Error>> {
    let mut prereqs = Vec::new();

    let rows = transaction
        .query(
            "SELECT l.level_id, l.name,
                CASE WHEN p.completed_at IS NOT NULL THEN true ELSE false END as is_completed
         FROM level_prerequisites lp
         JOIN flashcard_levels l ON lp.prerequisite_id = l.level_id
         LEFT JOIN user_level_progress p ON p.level_id = l.level_id
                                       AND p.user_id = $2
         WHERE lp.level_id = $1
         ORDER BY l.position",
            &[&level_id, &user_id],
        )
        .await?;

    for row in rows {
        prereqs.push(PrerequisiteLevel {
            level_id: row.get("level_id"),
            name: row.get("name"),
            is_completed: row.get("is_completed"),
        });
    }

    Ok(prereqs)
}

async fn get_level_progress(
    transaction: &Transaction<'_>,
    level_id: i32,
    user_id: i32,
) -> Result<Option<LevelProgress>, Box<dyn std::error::Error>> {
    if let Some(row) = transaction
        .query_opt(
            "SELECT * FROM user_level_progress
         WHERE user_id = $1 AND level_id = $2",
            &[&user_id, &level_id],
        )
        .await?
    {
        let success_rate = if row.get::<_, i32>("total_answers") > 0 {
            row.get::<_, i32>("correct_answers") as f32 / row.get::<_, i32>("total_answers") as f32
        } else {
            0.0
        };

        Ok(Some(LevelProgress {
            cards_completed: row.get("cards_completed"),
            correct_answers: row.get("correct_answers"),
            total_answers: row.get("total_answers"),
            success_rate,
            is_unlocked: row
                .get::<_, Option<ChronoDateTime>>("unlocked_at")
                .is_some(),
            is_completed: row
                .get::<_, Option<ChronoDateTime>>("completed_at")
                .is_some(),
            unlocked_at: row.get::<_, Option<ChronoDateTime>>("unlocked_at"),
            completed_at: row.get::<_, Option<ChronoDateTime>>("completed_at"),
            last_activity_at: row.get::<_, ChronoDateTime>("last_activity_at"),
        }))
    } else {
        Ok(None)
    }
}

async fn get_level_cards(
    transaction: &Transaction<'_>,
    level_id: i32,
    user_id: Option<i32>,
) -> Result<Vec<LevelCardResponse>, Box<dyn std::error::Error>> {
    let mut cards = Vec::new();

    let rows = transaction
        .query(
            "SELECT f.id, f.collection_id, ci.item_id, ci.definition_id, fli.position,
                v.word, v.valsiid, d.definition, d.definitionid, ci.notes as ci_notes,
                ci.free_content_front, ci.free_content_back, ci.canonical_form,
                EXISTS(SELECT 1 FROM collection_item_images cii WHERE cii.item_id = ci.item_id AND cii.side = 'front') as has_front_image,
                EXISTS(SELECT 1 FROM collection_item_images cii WHERE cii.item_id = ci.item_id AND cii.side = 'back') as has_back_image,
                EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = ci.item_id) as has_sound,
                COUNT(CASE WHEN frh.rating >= 3 THEN 1 END) as correct_answers,
                COUNT(frh.rating) as total_attempts,
                MAX(frh.review_time) as last_reviewed_at
         FROM flashcard_level_items fli
         JOIN flashcards f ON fli.flashcard_id = f.id
         JOIN collection_items ci ON f.item_id = ci.item_id
         LEFT JOIN definitions d ON ci.definition_id = d.definitionid
         LEFT JOIN valsi v ON d.valsiid = v.valsiid
         LEFT JOIN flashcard_review_history frh
             ON frh.flashcard_id = f.id
             AND frh.user_id = $2
         WHERE fli.level_id = $1
         GROUP BY f.id, f.collection_id, ci.item_id, fli.position, v.word, d.definition, v.valsiid, d.definitionid,
                  ci.free_content_front, ci.free_content_back, ci.canonical_form
         ORDER BY fli.position",
            &[&level_id, &user_id],
        )
        .await?;

    for row in rows {
        let success_rate = if row.get::<_, i64>("total_attempts") > 0 {
            row.get::<_, i64>("correct_answers") as f32 / row.get::<_, i64>("total_attempts") as f32
        } else {
            0.0
        };

        let progress = if user_id.is_some() {
            Some(LevelCardProgress {
                correct_answers: row.get::<_, i64>("correct_answers") as i32,
                total_attempts: row.get::<_, i64>("total_attempts") as i32,
                success_rate,
                last_reviewed_at: row.get("last_reviewed_at"),
            })
        } else {
            None
        };

        let has_sound: bool = row.get("has_sound");
        let collection_id: i32 = row.get("collection_id");
        let item_id: i32 = row.get("item_id");
        let sound_url = if has_sound {
            Some(format!(
                "/api/collections/{}/items/{}/sound",
                collection_id, item_id
            ))
        } else {
            None
        };

        cards.push(LevelCardResponse {
            flashcard_id: row.get("id"),
            position: row.get("position"),
            word: row.get("word"),
            definition: row.get("definition"),
            free_content_front: row.get("free_content_front"),
            free_content_back: row.get("free_content_back"),
            has_front_image: row.get("has_front_image"),
            has_back_image: row.get("has_back_image"),
            has_sound,
            sound_url,
            collection_id: row.get("collection_id"),
            item_id,
            definition_id: row.get("definitionid"),
            valsi_id: row.get("valsiid"),
            ci_notes: row.get("ci_notes"),
            canonical_form: row.get("canonical_form"),
            progress,
        });
    }

    Ok(cards)
}

pub async fn delete_level(
    pool: &Pool,
    level_id: i32,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let collection_id = get_collection_id(&transaction, level_id).await?;
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // Check if this level is a prerequisite for any other level
    let dependent_count: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM level_prerequisites WHERE prerequisite_id = $1",
            &[&level_id],
        )
        .await?
        .get(0);

    if dependent_count > 0 {
        return Err("Cannot delete level: it is a prerequisite for other levels.".into());
    }

    // Delete items associated with the level
    transaction
        .execute(
            "DELETE FROM flashcard_level_items WHERE level_id = $1",
            &[&level_id],
        )
        .await?;

    // Delete prerequisites where this level is the dependent level
    transaction
        .execute(
            "DELETE FROM level_prerequisites WHERE level_id = $1",
            &[&level_id],
        )
        .await?;

    // Delete the level itself
    let result = transaction
        .execute(
            "DELETE FROM flashcard_levels WHERE level_id = $1",
            &[&level_id],
        )
        .await?;

    if result == 0 {
        return Err("Level not found".into());
    }

    transaction.commit().await?;
    Ok(())
}

async fn get_collection_id(
    transaction: &Transaction<'_>,
    level_id: i32,
) -> Result<i32, Box<dyn std::error::Error>> {
    Ok(transaction
        .query_one(
            "SELECT collection_id FROM flashcard_levels WHERE level_id = $1",
            &[&level_id],
        )
        .await?
        .get("collection_id"))
}

pub async fn get_level_cards_paginated(
    pool: &Pool,
    level_id: i32,
    user_id: Option<i32>,
    page: i64,
    per_page: i64,
) -> Result<LevelCardListResponse, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let offset = (page - 1) * per_page;

    // Check if level is unlocked for authenticated users
    if let Some(uid) = user_id {
        let is_unlocked = client
            .query_one(
                "SELECT check_level_prerequisites($1, $2) as unlocked",
                &[&uid, &level_id],
            )
            .await?
            .get::<_, bool>("unlocked");

        if !is_unlocked {
            return Err("Level is locked. Complete prerequisites first.".into());
        }
    }

    let total = client
        .query_one(
            "SELECT COUNT(*) FROM flashcard_level_items WHERE level_id = $1",
            &[&level_id],
        )
        .await?
        .get::<_, i64>(0);

    let rows = client
        .query(
            "SELECT f.id, f.collection_id, ci.item_id, ci.definition_id, fli.position,
                v.word, v.valsiid, d.definition, d.definitionid, ci.notes as ci_notes,
                ci.free_content_front, ci.free_content_back, ci.canonical_form,
                EXISTS(SELECT 1 FROM collection_item_images cii WHERE cii.item_id = ci.item_id AND cii.side = 'front') as has_front_image,
                EXISTS(SELECT 1 FROM collection_item_images cii WHERE cii.item_id = ci.item_id AND cii.side = 'back') as has_back_image,
                EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = ci.item_id) as has_sound,
                COUNT(CASE WHEN frh.rating >= 3 THEN 1 END) as correct_answers,
                COUNT(frh.rating) as total_attempts,
                MAX(frh.review_time) as last_reviewed_at
            FROM flashcard_level_items fli
            JOIN flashcards f ON fli.flashcard_id = f.id
            JOIN collection_items ci ON f.item_id = ci.item_id
            LEFT JOIN definitions d ON ci.definition_id = d.definitionid
            LEFT JOIN valsi v ON d.valsiid = v.valsiid
            LEFT JOIN flashcard_review_history frh
                ON frh.flashcard_id = f.id
                AND frh.user_id = $1
            WHERE fli.level_id = $2
            GROUP BY f.id, f.collection_id, ci.item_id, fli.position, v.word, d.definition, v.valsiid, d.definitionid,
                    ci.free_content_front, ci.free_content_back, ci.canonical_form
            ORDER BY fli.position
            LIMIT $3 OFFSET $4",
            &[&user_id, &level_id, &per_page, &offset],
        )
        .await?;

    let cards = rows
        .iter()
        .map(|row| {
            let success_rate = if row.get::<_, i64>("total_attempts") > 0 {
                row.get::<_, i64>("correct_answers") as f32
                    / row.get::<_, i64>("total_attempts") as f32
            } else {
                0.0
            };

            let progress = if user_id.is_some() {
                Some(LevelCardProgress {
                    correct_answers: row.get::<_, i64>("correct_answers") as i32,
                    total_attempts: row.get::<_, i64>("total_attempts") as i32,
                    success_rate,
                    last_reviewed_at: row.get("last_reviewed_at"),
                })
            } else {
                None
            };

            let has_sound: bool = row.get("has_sound");
            let collection_id: i32 = row.get("collection_id");
            let item_id: i32 = row.get("item_id");
            let sound_url = if has_sound {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id, item_id
                ))
            } else {
                None
            };

            LevelCardResponse {
                flashcard_id: row.get("id"),
                position: row.get("position"),
                word: row.get("word"),
                definition: row.get("definition"),
                definition_id: row.get("definitionid"),
                valsi_id: row.get("valsiid"),
                free_content_front: row.get("free_content_front"),
                free_content_back: row.get("free_content_back"),
                has_front_image: row.get("has_front_image"),
                has_back_image: row.get("has_back_image"),
                has_sound,
                sound_url,
                collection_id: row.get("collection_id"),
                item_id,
                ci_notes: row.get("ci_notes"),
                canonical_form: row.get("canonical_form"),
                progress,
            }
        })
        .collect();

    Ok(LevelCardListResponse {
        cards,
        total,
        page,
        per_page,
    })
}

pub async fn remove_card_from_level(
    pool: &Pool,
    level_id: i32,
    flashcard_id: i32,
    user_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let collection_id = get_collection_id(&transaction, level_id).await?;
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    let result = transaction
        .execute(
            "DELETE FROM flashcard_level_items
             WHERE level_id = $1 AND flashcard_id = $2",
            &[&level_id, &flashcard_id],
        )
        .await?;

    if result == 0 {
        return Err("Card not found in level".into());
    }

    // Reorder remaining cards
    transaction
        .execute(
            "WITH ranked AS (
                SELECT flashcard_id, position,
                       ROW_NUMBER() OVER (ORDER BY position) - 1 as new_pos
                FROM flashcard_level_items
                WHERE level_id = $1
            )
            UPDATE flashcard_level_items fli
            SET position = r.new_pos
            FROM ranked r
            WHERE fli.level_id = $1
            AND fli.flashcard_id = r.flashcard_id",
            &[&level_id],
        )
        .await?;

    transaction.commit().await?;
    Ok(())
}
