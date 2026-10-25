use super::error::FlashcardError;
use crate::jbovlaste::service::get_valsi_sound_urls_from_db;
use crate::AppError;
use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::Pool;
use deadpool_postgres::Transaction;
use fsrs::{
    current_retrievability, extract_simulator_config, optimal_retention, Card, FSRSItem,
    FSRSReview, ItemProgress, ItemState, MemoryState, NextStates, RevlogEntry, RevlogReviewKind,
    DEFAULT_PARAMETERS, FSRS,
};
use log::{debug, error, info};
use std::collections::{HashMap, HashSet};

use crate::auth_utils::{
    verify_collection_ownership, verify_collection_read_access, verify_flashcard_ownership,
};
use crate::utils::wordbank::{normalize_wordbank_phrase, WordbankLang};

use super::{
    dto::{
        self, AddCardsRequest, ChronoDateTime, CreateFlashcardRequest, CreateLevelRequest,
        DailyProgress, DirectAnswerRequest, DirectAnswerResponse, FillInAnswerRequest, Flashcard,
        FlashcardListQuery, FlashcardListResponse, FlashcardResponse, ImportFromCollectionResponse,
        LevelCardListResponse, LevelCardProgress, LevelCardResponse, LevelListResponse,
        LevelProgress, LevelResponse, MergeProgressRequest, MergeProgressResponse,
        PrerequisiteLevel, ReviewRequest, ReviewResponse, StreakResponse, UpdateLevelRequest,
    },
    models::*,
};

async fn get_flashcard(
    transaction: &Transaction<'_>,
    flashcard_id: i32,
) -> Result<Flashcard, FlashcardError> {
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
        wordbank_chips: None,
        wordbank_expected_normalized: None,
    })
}

pub async fn create_flashcard(
    pool: &Pool,
    collection_id: i32,
    user_id: i32,
    req: &CreateFlashcardRequest,
) -> Result<FlashcardResponse, FlashcardError> {
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
                    crate::utils::canonical::get_canonical_form(front)
                } else if let Some(def_id) = req.definition_id {
                    // Fetch word for dictionary items
                    let word_row = transaction.query_opt(
                        "SELECT v.word FROM definitions d JOIN valsi v ON d.valsiid = v.valsiid WHERE d.definitionid = $1",
                        &[&def_id]
                    ).await?;
                    word_row.and_then(|r| {
                        let word: String = r.get(0);
                        crate::utils::canonical::get_canonical_form(&word)
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
                .and_then(|front| crate::utils::canonical::get_canonical_form(front));

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
            wordbank_chips: None,
            wordbank_expected_normalized: None,
            sound_url: if row.get::<_, bool>("has_custom_sound") {
                Some(format!(
                    "/api/collections/{}/items/{}/sound",
                    collection_id, item_id
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
        FlashcardDirection::FillIn
        | FlashcardDirection::Wordbank
        | FlashcardDirection::ListenWordbank => {
            initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
        }
        FlashcardDirection::FillInReverse
        | FlashcardDirection::WordbankReverse
        | FlashcardDirection::ListenWordbankReverse => {
            initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
        }
        FlashcardDirection::FillInBoth
        | FlashcardDirection::WordbankBoth
        | FlashcardDirection::ListenWordbankBoth => {
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
        }
        FlashcardDirection::QuizImageDirect
        | FlashcardDirection::QuizImageReverse
        | FlashcardDirection::QuizImageBoth => {
            let correct_answer_text = match req.direction {
                FlashcardDirection::QuizImageDirect => format!("{}:front", item_id),
                FlashcardDirection::QuizImageReverse => format!("{}:back", item_id),
                FlashcardDirection::QuizImageBoth => item_id.to_string(),
                _ => unreachable!(),
            };
            transaction.execute(
                "INSERT INTO flashcard_quiz_options (flashcard_id, correct_answer_text) VALUES ($1, $2)",
                &[&flashcard_id, &correct_answer_text]
            ).await?;
            initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
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
) -> Result<String, FlashcardError> {
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

/// Make `side` active for the user: insert a new row, or un-archive the existing one.
///
/// `(user_id, flashcard_id, card_side)` is unique over archived rows too, so a plain
/// `DO NOTHING` left a side that had been archived without any active progress.
pub async fn initialize_progress(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    side: &str,
) -> Result<(), FlashcardError> {
    transaction
        .execute(
            "INSERT INTO user_flashcard_progress
 (user_id, flashcard_id, card_side, status, next_review_at)
 VALUES ($1, $2, $3, 'new', CURRENT_TIMESTAMP)
 ON CONFLICT (user_id, flashcard_id, card_side) DO UPDATE SET archived = false",
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
) -> Result<(), FlashcardError> {
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
) -> Result<(), FlashcardError> {
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
) -> Result<Vec<UserFlashcardProgress>, FlashcardError> {
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
) -> Result<(), FlashcardError> {
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
) -> Result<FlashcardQuizOptions, FlashcardError> {
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
            "SELECT f.direction, f.item_id, ci.definition_id, ci.free_content_front, ci.free_content_back
         FROM flashcards f
         JOIN collection_items ci ON f.item_id = ci.item_id
         WHERE f.id = $1",
            &[&flashcard_id],
        )
        .await?;

    let direction: FlashcardDirection = flashcard_details_row.get("direction");
    let item_id: i32 = flashcard_details_row.get("item_id");
    let definition_id: Option<i32> = flashcard_details_row.get("definition_id");
    let free_content_front: Option<String> = flashcard_details_row.get("free_content_front");
    let free_content_back: Option<String> = flashcard_details_row.get("free_content_back");

    // Ensure the flashcard is a quiz type (text or image options)
    if !matches!(
        direction,
        FlashcardDirection::QuizDirect
            | FlashcardDirection::QuizReverse
            | FlashcardDirection::QuizBoth
            | FlashcardDirection::QuizImageDirect
            | FlashcardDirection::QuizImageReverse
            | FlashcardDirection::QuizImageBoth
    ) {
        return Err("Flashcard is not a quiz type.".into());
    }

    let correct_answer_text = if matches!(
        direction,
        FlashcardDirection::QuizImageDirect
            | FlashcardDirection::QuizImageReverse
            | FlashcardDirection::QuizImageBoth
    ) {
        match direction {
            FlashcardDirection::QuizImageDirect => format!("{}:front", item_id),
            FlashcardDirection::QuizImageReverse => format!("{}:back", item_id),
            FlashcardDirection::QuizImageBoth => item_id.to_string(),
            _ => unreachable!(),
        }
    } else {
        determine_correct_answer_for_quiz(
            &transaction,
            &direction,
            definition_id,
            free_content_front.as_deref(),
            free_content_back.as_deref(),
        )
        .await?
    };

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

/// Build a Flashcard from a row that has the standard flashcard + collection_items + definition columns.
/// Used by both list_flashcards_public and list_flashcards.
fn build_flashcard_from_row(
    row: &tokio_postgres::Row,
    collection_id: i32,
    sound_url_from_valsi: Option<String>,
    question_text: Option<String>,
    quiz_options: Option<Vec<String>>,
) -> Flashcard {
    let item_id: i32 = row.get("item_id");
    let has_custom_sound: bool = row.get("has_custom_sound");
    let sound_url = if has_custom_sound {
        Some(format!(
            "/api/collections/{}/items/{}/sound",
            collection_id, item_id
        ))
    } else {
        sound_url_from_valsi
    };
    Flashcard {
        id: row.get("id"),
        collection_id,
        item_id,
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
        definition_language_id: row.get("definition_language_id"),
        sound_url,
        has_custom_sound,
        canonical_form: row.get("canonical_form"),
        use_canonical_comparison: row.get("use_canonical_comparison"),
        created_at: row.get("created_at"),
        question_text,
        quiz_options,
        wordbank_chips: None,
        wordbank_expected_normalized: None,
    }
}

/// Expected raw answer for fill-in / wordbank / listen-wordbank (preserves casing for wordbank).
fn expected_fillin_like_raw(fc: &Flashcard, card_side: &str) -> Option<String> {
    let pick_back = || {
        fc.definition
            .as_deref()
            .or(fc.free_content_back.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    let pick_front = || {
        fc.word
            .as_deref()
            .or(fc.free_content_front.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    match (&fc.direction, card_side) {
        (
            FlashcardDirection::FillIn
            | FlashcardDirection::FillInBoth
            | FlashcardDirection::Wordbank
            | FlashcardDirection::WordbankBoth,
            "direct",
        ) => pick_back(),
        (FlashcardDirection::ListenWordbank | FlashcardDirection::ListenWordbankBoth, "direct") => {
            pick_front()
        }
        (
            FlashcardDirection::FillInReverse
            | FlashcardDirection::FillInBoth
            | FlashcardDirection::WordbankReverse
            | FlashcardDirection::WordbankBoth,
            "reverse",
        ) => pick_front(),
        (
            FlashcardDirection::ListenWordbankReverse | FlashcardDirection::ListenWordbankBoth,
            "reverse",
        ) => pick_back(),
        _ => None,
    }
}

/// Distractor phrases from the same language side as the assemble target.
fn distractor_texts_for_lang(
    word: Option<&String>,
    definition: Option<&String>,
    free_front: Option<&String>,
    free_back: Option<&String>,
    lang: WordbankLang,
) -> Vec<String> {
    let texts: [Option<&String>; 2] = match lang {
        WordbankLang::Lojban => [word, free_front],
        WordbankLang::Translation => [definition, free_back],
    };
    texts
        .into_iter()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .collect()
}

fn flashcard_distractor_texts(fc: &Flashcard, lang: WordbankLang) -> Vec<String> {
    distractor_texts_for_lang(
        fc.word.as_ref(),
        fc.definition.as_ref(),
        fc.free_content_front.as_ref(),
        fc.free_content_back.as_ref(),
        lang,
    )
}

/// Lojban front vs translation back for chip / grade normalization.
fn wordbank_lang_for(direction: &FlashcardDirection, card_side: &str) -> Option<WordbankLang> {
    match (direction, card_side) {
        (FlashcardDirection::Wordbank | FlashcardDirection::WordbankBoth, "direct")
        | (
            FlashcardDirection::ListenWordbankReverse | FlashcardDirection::ListenWordbankBoth,
            "reverse",
        ) => Some(WordbankLang::Translation),
        (FlashcardDirection::WordbankReverse | FlashcardDirection::WordbankBoth, "reverse")
        | (FlashcardDirection::ListenWordbank | FlashcardDirection::ListenWordbankBoth, "direct") => {
            Some(WordbankLang::Lojban)
        }
        _ => None,
    }
}

/// Attach shuffled word-bank chips to study-queue cards (same idea as quiz_options).
fn attach_wordbank_chips(cards: &mut [FlashcardResponse]) {
    if cards.is_empty() {
        return;
    }

    for i in 0..cards.len() {
        if !is_wordbank_like_direction(&cards[i].flashcard.direction) {
            continue;
        }
        let side = cards[i]
            .progress
            .first()
            .map(|p| p.card_side.as_str())
            .unwrap_or("direct");
        let Some(lang) = wordbank_lang_for(&cards[i].flashcard.direction, side) else {
            continue;
        };
        let Some(expected) = expected_fillin_like_raw(&cards[i].flashcard, side) else {
            continue;
        };
        let id = cards[i].flashcard.id;
        let distractors: Vec<String> = cards
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .filter(|(_, c)| c.flashcard.id != id)
            .flat_map(|(_, c)| flashcard_distractor_texts(&c.flashcard, lang))
            .collect();
        let (chips, expected_normalized) =
            crate::utils::wordbank::build_wordbank_chips(&expected, &distractors, lang, None);
        if chips.is_empty() {
            continue;
        }
        cards[i].flashcard.wordbank_chips = Some(chips);
        cards[i].flashcard.wordbank_expected_normalized = Some(expected_normalized);
    }
}

fn attach_wordbank_chips_to_level_cards(cards: &mut [LevelCardResponse]) {
    if cards.is_empty() {
        return;
    }

    for i in 0..cards.len() {
        if !is_wordbank_like_direction(&cards[i].direction) {
            continue;
        }
        // Guest/level listing studies the primary side for this direction family.
        let side = match cards[i].direction {
            FlashcardDirection::WordbankReverse
            | FlashcardDirection::ListenWordbankReverse
            | FlashcardDirection::FillInReverse => "reverse",
            _ => "direct",
        };
        let Some(lang) = wordbank_lang_for(&cards[i].direction, side) else {
            continue;
        };
        let stub = Flashcard {
            id: cards[i].flashcard_id,
            collection_id: cards[i].collection_id,
            item_id: cards[i].item_id,
            definition_id: cards[i].definition_id,
            word: cards[i].word.clone(),
            definition: cards[i].definition.clone(),
            free_content_front: cards[i].free_content_front.clone(),
            free_content_back: cards[i].free_content_back.clone(),
            has_front_image: false,
            has_back_image: false,
            notes: None,
            position: cards[i].position,
            direction: cards[i].direction.clone(),
            definition_language_id: None,
            sound_url: None,
            has_custom_sound: false,
            canonical_form: None,
            use_canonical_comparison: true,
            created_at: DateTime::<Utc>::UNIX_EPOCH,
            question_text: None,
            quiz_options: None,
            wordbank_chips: None,
            wordbank_expected_normalized: None,
        };
        let Some(expected) = expected_fillin_like_raw(&stub, side) else {
            continue;
        };
        let id = cards[i].flashcard_id;
        let distractors: Vec<String> = cards
            .iter()
            .filter(|c| c.flashcard_id != id)
            .flat_map(|c| {
                distractor_texts_for_lang(
                    c.word.as_ref(),
                    c.definition.as_ref(),
                    c.free_content_front.as_ref(),
                    c.free_content_back.as_ref(),
                    lang,
                )
            })
            .collect();
        let (chips, expected_normalized) =
            crate::utils::wordbank::build_wordbank_chips(&expected, &distractors, lang, None);
        if chips.is_empty() {
            continue;
        }
        cards[i].wordbank_chips = Some(chips);
        cards[i].wordbank_expected_normalized = Some(expected_normalized);
    }
}

/// List flashcards for a public collection (no auth). Used by anonymous users when the collection has no levels.
pub async fn list_flashcards_public(
    pool: &Pool,
    collection_id: i32,
) -> Result<FlashcardListResponse, FlashcardError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_collection_read_access(&transaction, collection_id, None)
        .await
        .map_err(|e| match e {
            AppError::Auth(_) => FlashcardError::AccessDenied(e.to_string()),
            AppError::NotFound(_) => FlashcardError::NotFound(e.to_string()),
            other => FlashcardError::Other(other.to_string()),
        })?;

    let rows = transaction
        .query(
            "SELECT f.id, f.collection_id, f.item_id, f.position, f.direction, f.created_at,
                    ci.definition_id, ci.free_content_front, ci.free_content_back, ci.notes,
                    ci.canonical_form, f.use_canonical_comparison,
                    d.langid as definition_language_id, v.word, d.definition,
                    EXISTS(SELECT 1 FROM collection_item_images WHERE item_id = ci.item_id AND side = 'front') as has_front_image,
                    EXISTS(SELECT 1 FROM collection_item_images WHERE item_id = ci.item_id AND side = 'back') as has_back_image,
                    EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = ci.item_id) as has_custom_sound
             FROM flashcards f
             JOIN collection_items ci ON f.item_id = ci.item_id AND ci.collection_id = f.collection_id
             LEFT JOIN definitions d ON ci.definition_id = d.definitionid
             LEFT JOIN valsi v ON d.valsiid = v.valsiid
             WHERE f.collection_id = $1
             ORDER BY f.position",
            &[&collection_id],
        )
        .await?;

    let words_to_check: Vec<String> = rows
        .iter()
        .filter_map(|row| {
            let word: Option<String> = row.get("word");
            let free: Option<String> = row.get("free_content_front");
            word.or(free).filter(|w| !w.trim().is_empty())
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let sound_urls_map = if !words_to_check.is_empty() {
        get_valsi_sound_urls_from_db(pool, &words_to_check)
            .await
            .unwrap_or_else(|_| HashMap::new())
    } else {
        HashMap::new()
    };

    let mut flashcards = Vec::new();
    for row in &rows {
        let flashcard_id: i32 = row.get("id");
        let direction: FlashcardDirection = row.get("direction");
        let word: Option<String> = row.get("word");
        let free_content_front: Option<String> = row.get("free_content_front");
        let sound_key = word.as_ref().or(free_content_front.as_ref());
        let sound_url = sound_key.and_then(|k| sound_urls_map.get(k).cloned().flatten());

        let flashcard = build_flashcard_from_row(row, collection_id, sound_url, None, None);

        let synthetic_progress = |card_side: &str| UserFlashcardProgress {
            id: 0,
            user_id: 0,
            flashcard_id,
            card_side: card_side.to_string(),
            ease_factor: 0.0,
            stability: 0,
            difficulty: 0,
            interval: 0,
            review_count: 0,
            last_reviewed_at: None,
            next_review_at: None,
            status: FlashcardStatus::New,
        };

        match direction {
            FlashcardDirection::Both
            | FlashcardDirection::FillInBoth
            | FlashcardDirection::WordbankBoth
            | FlashcardDirection::ListenWordbankBoth => {
                // One response per side so wordbank chips match card_side.
                flashcards.push(FlashcardResponse {
                    flashcard: flashcard.clone(),
                    progress: vec![synthetic_progress("direct")],
                });
                flashcards.push(FlashcardResponse {
                    flashcard: flashcard.clone(),
                    progress: vec![synthetic_progress("reverse")],
                });
            }
            FlashcardDirection::Reverse
            | FlashcardDirection::FillInReverse
            | FlashcardDirection::WordbankReverse
            | FlashcardDirection::ListenWordbankReverse => {
                flashcards.push(FlashcardResponse {
                    flashcard: flashcard.clone(),
                    progress: vec![synthetic_progress("reverse")],
                });
            }
            _ => {
                flashcards.push(FlashcardResponse {
                    flashcard: flashcard.clone(),
                    progress: vec![synthetic_progress("direct")],
                });
            }
        }
    }

    attach_wordbank_chips(&mut flashcards);

    let total = flashcards.len() as i64;
    transaction.commit().await?;
    Ok(FlashcardListResponse {
        flashcards,
        total,
        page: 1,
        per_page: total.max(1),
        due_count: 0,
    })
}

pub async fn list_flashcards(
    pool: &Pool,
    user_id: i32,
    query: FlashcardListQuery,
) -> Result<FlashcardListResponse, FlashcardError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_collection_read_access(&transaction, query.collection_id, Some(user_id)).await?;

    // Level sessions must not load cards the user cannot review (review also enforces this).
    if let Some(level_id) = query.level_id {
        let unlocked: bool = transaction
            .query_one(
                "SELECT check_level_prerequisites($1, $2) AS unlocked",
                &[&user_id, &level_id],
            )
            .await?
            .get("unlocked");
        if !unlocked {
            return Err(format!(
                "access denied: level {} is locked. Complete prerequisites first.",
                level_id
            )
            .into());
        }
    }

    let fsrs = FSRS::new(&[])?;

    // Get base flashcards including free content.
    // Cards assigned only to locked levels are excluded so open due queues cannot
    // surface content that POST /review would reject.
    const LIST_FLASHCARDS_SQL: &str = "WITH ReviewHistory AS (
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
        AND ($6::int IS NULL OR f.id IN (SELECT flashcard_id FROM flashcard_level_items WHERE level_id = $6))
        AND (
            NOT EXISTS (
                SELECT 1 FROM flashcard_level_items fli_any WHERE fli_any.flashcard_id = f.id
            )
            OR EXISTS (
                SELECT 1 FROM flashcard_level_items fli_ok
                WHERE fli_ok.flashcard_id = f.id
                  AND check_level_prerequisites($1, fli_ok.level_id)
            )
        )
        ORDER BY
        CASE
            WHEN $4::boolean = true THEN EXTRACT(EPOCH FROM p.next_review_at)
            ELSE f.position::bigint
        END";

    let list_params: [&(dyn tokio_postgres::types::ToSql + Sync); 6] = [
        &user_id,
        &query.collection_id,
        &query.status,
        &query.due,
        &query.flashcard_id,
        &query.level_id,
    ];

    let mut rows = transaction.query(LIST_FLASHCARDS_SQL, &list_params).await?;

    // First pass: seed progress for cards the user has never seen. Those rows are
    // skipped in the response builder (`continue`), so without a re-query the first
    // due fetch for a new guest/user returns an empty queue ("All caught up").
    let mut seeded_any = false;
    for row in &rows {
        let flashcard_id: i32 = row.get("id");
        let direction: FlashcardDirection = row.get("direction");
        let card_side: Option<String> = row.get("card_side");
        if card_side.is_some() {
            continue;
        }
        seeded_any = true;
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
            FlashcardDirection::FillIn
            | FlashcardDirection::Wordbank
            | FlashcardDirection::ListenWordbank => {
                initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
            }
            FlashcardDirection::FillInReverse
            | FlashcardDirection::WordbankReverse
            | FlashcardDirection::ListenWordbankReverse => {
                initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
            }
            FlashcardDirection::FillInBoth
            | FlashcardDirection::WordbankBoth
            | FlashcardDirection::ListenWordbankBoth => {
                initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
                initialize_progress(&transaction, user_id, flashcard_id, "reverse").await?;
            }
            FlashcardDirection::JustInformation => {
                initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
                mark_progress_graduated(&transaction, user_id, flashcard_id, "direct").await?;
            }
            FlashcardDirection::QuizDirect
            | FlashcardDirection::QuizReverse
            | FlashcardDirection::QuizBoth
            | FlashcardDirection::QuizImageDirect
            | FlashcardDirection::QuizImageReverse
            | FlashcardDirection::QuizImageBoth => {
                initialize_progress(&transaction, user_id, flashcard_id, "direct").await?;
            }
        }
    }
    if seeded_any {
        rows = transaction.query(LIST_FLASHCARDS_SQL, &list_params).await?;
    }

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

        // Should only happen if seeding failed; skip rather than return half-built cards.
        if card_side.is_none() {
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
                .collect::<Result<Vec<_>, FlashcardError>>()?;

            let fsrs_item = FSRSItem {
                reviews: fsrs_reviews,
            };
            let memory_state = fsrs.memory_state(fsrs_item, None)?;

            let last_review: Option<DateTime<Utc>> = row.get("last_reviewed_at");
            if let Some(last) = last_review {
                let days_elapsed = (Utc::now() - last).num_days().max(0) as u32;
                current_retrievability(memory_state, days_elapsed as f32, -0.5)
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
                | FlashcardDirection::QuizImageDirect
                | FlashcardDirection::QuizImageReverse
                | FlashcardDirection::QuizImageBoth
        ) {
            let effective_quiz_direction = match direction {
                FlashcardDirection::QuizDirect | FlashcardDirection::QuizImageDirect => "direct",
                FlashcardDirection::QuizReverse | FlashcardDirection::QuizImageReverse => "reverse",
                FlashcardDirection::QuizBoth | FlashcardDirection::QuizImageBoth => {
                    if rand::random() {
                        "direct"
                    } else {
                        "reverse"
                    }
                }
                _ => "direct",
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
        if (direction == FlashcardDirection::Both
            || direction == FlashcardDirection::FillInBoth
            || direction == FlashcardDirection::WordbankBoth
            || direction == FlashcardDirection::ListenWordbankBoth)
            && !query.due.unwrap_or(false)
        {
            // For non-due listing, combine progresses into single card
            if !processed_ids.contains(&flashcard_id) {
                let word: Option<String> = row.get("word");
                let free_content_front: Option<String> = row.get("free_content_front");
                let sound_key = word.as_ref().or(free_content_front.as_ref());
                let sound_url = sound_key.and_then(|k| sound_urls_map.get(k).cloned().flatten());

                let mut card_response = FlashcardResponse {
                    flashcard: build_flashcard_from_row(
                        row,
                        query.collection_id,
                        sound_url,
                        question_text.clone(),
                        quiz_options.clone(),
                    ),
                    progress: vec![progress],
                };

                // Find and add the other direction's progress
                if let Some(other_row) = rows.iter().find(|r| {
                    let other_id: i32 = r.get("id");
                    let other_side: String = r.get("card_side");
                    other_id == flashcard_id
                        && card_side.as_deref().is_some_and(|cs| other_side != cs)
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
                flashcard: build_flashcard_from_row(
                    row,
                    query.collection_id,
                    sound_url,
                    question_text.clone(),
                    quiz_options.clone(),
                ),
                progress: vec![progress],
            };
            all_flashcards_with_retrievability.push((card_response, retrievability));
        }
    }

    // Calculate pagination
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let offset = ((page - 1) * per_page) as usize;

    let mut paginated_cards: Vec<FlashcardResponse> = all_flashcards_with_retrievability
        .iter()
        .skip(offset)
        .take(per_page as usize)
        .map(|(card, _)| card.clone())
        .collect();
    attach_wordbank_chips(&mut paginated_cards);

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
                query
                    .flashcard_id
                    .is_none_or(|fid| row.get::<_, i32>("id") == fid)
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

    // Warm retention cache while the learner reads the first card (not on the rating click)
    let refresh_retention = if query.due.unwrap_or(false) {
        get_optimal_retention(&transaction, user_id)
            .await
            .map(|(_, needs)| needs)
            .unwrap_or(false)
    } else {
        false
    };

    transaction.commit().await?;

    if refresh_retention {
        spawn_optimal_retention_refresh(pool.clone(), user_id);
    }

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
) -> Result<(String, Vec<String>), FlashcardError> {
    // 1. Fetch the flashcard details to determine question and correct answer source
    let card_details_row = transaction.query_one(
        "SELECT f.direction as actual_direction, f.collection_id, f.item_id, ci.definition_id, ci.free_content_front, ci.free_content_back,
                v.word, d.definition, fqo.correct_answer_text
         FROM flashcards f
         JOIN collection_items ci ON f.item_id = ci.item_id
         LEFT JOIN definitions d ON ci.definition_id = d.definitionid
         LEFT JOIN valsi v ON d.valsiid = v.valsiid
         LEFT JOIN flashcard_quiz_options fqo ON f.id = fqo.flashcard_id
         WHERE f.id = $1",
        &[&flashcard_id]
    ).await?;

    let actual_direction: FlashcardDirection = card_details_row.get("actual_direction");
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

    // Image quiz: options are "item_id:side"; correct is stored as "item_id:side" or "item_id" (for Both)
    if matches!(
        actual_direction,
        FlashcardDirection::QuizImageDirect
            | FlashcardDirection::QuizImageReverse
            | FlashcardDirection::QuizImageBoth
    ) {
        let collection_id: i32 = card_details_row.get("collection_id");
        let current_item_id: i32 = card_details_row.get("item_id");
        let side = if correct_answer_text.contains(':') {
            correct_answer_text
                .split(':')
                .nth(1)
                .unwrap_or("front")
                .to_string()
        } else if actual_direction == FlashcardDirection::QuizImageBoth {
            if effective_direction_str == "direct" {
                "front".to_string()
            } else {
                "back".to_string()
            }
        } else if actual_direction == FlashcardDirection::QuizImageDirect {
            "front".to_string()
        } else {
            "back".to_string()
        };
        let correct_option = if correct_answer_text.contains(':') {
            correct_answer_text.clone()
        } else {
            format!("{}:{}", correct_answer_text.trim(), side)
        };

        let distractor_rows = transaction
            .query(
                "SELECT f.item_id
                 FROM flashcards f
                 JOIN collection_item_images cii ON cii.item_id = f.item_id AND cii.side = $2
                 WHERE f.collection_id = $1 AND f.item_id != $3
                 ORDER BY RANDOM()
                 LIMIT 3",
                &[&collection_id, &side, &current_item_id],
            )
            .await?;
        let mut distractors: Vec<String> = distractor_rows
            .iter()
            .map(|r| format!("{}:{}", r.get::<_, i32>("item_id"), side))
            .collect();
        while distractors.len() < 3 {
            distractors.push(
                distractors
                    .get(distractors.len().wrapping_sub(1))
                    .cloned()
                    .unwrap_or_else(|| correct_option.clone()),
            );
        }
        distractors.truncate(3);
        let mut options = vec![correct_option.clone()];
        options.extend(distractors);
        use rand::seq::SliceRandom;
        options.shuffle(&mut rand::rng());
        return Ok((question_text, options));
    }

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
        // Free-content items have no definition/valsi row, so fall back to the
        // item's own text. DISTINCT and ORDER BY RANDOM() cannot share one
        // SELECT in Postgres, so dedupe in a subquery first.
        let distractor_source_query_part = if effective_direction_str == "direct" {
            // Correct answer is like a definition
            "COALESCE(d.definition, ci.free_content_back)"
        } else {
            // Correct answer is like a word (for reverse quiz)
            "COALESCE(v.word, ci.free_content_front)"
        };

        let mut existing_options_for_filter = distractors.clone();
        existing_options_for_filter.push(correct_answer_text.clone());

        let exploration_distractors_rows = transaction
            .query(
                &format!(
                    "SELECT distractor FROM (
                    SELECT DISTINCT {src} AS distractor
                    FROM flashcards f
                    JOIN collection_items ci ON f.item_id = ci.item_id
                    LEFT JOIN definitions d ON ci.definition_id = d.definitionid
                    LEFT JOIN valsi v ON d.valsiid = v.valsiid
                    WHERE f.collection_id = (SELECT collection_id FROM flashcards WHERE id = $1)
                 ) c
                 WHERE distractor IS NOT NULL
                   AND distractor <> ''
                   AND distractor != $2
                   AND distractor <> ALL($3)
                 ORDER BY RANDOM()
                 LIMIT $4",
                    src = distractor_source_query_part
                ),
                &[
                    &flashcard_id,
                    &correct_answer_text,
                    &existing_options_for_filter,
                    &(needed_distractors as i64),
                ],
            )
            .await?;

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
const DEFAULT_OPTIMAL_RETENTION: f32 = 0.9;
const OPTIMAL_RETENTION_CACHE_DAYS: i64 = 7;

pub async fn calculate_optimal_retention(
    transaction: &Transaction<'_>,
    user_id: i32,
) -> Result<f32, FlashcardError> {
    let default_retention = DEFAULT_OPTIMAL_RETENTION;

    // Check if user has enough review history to calculate optimal retention
    let review_count: i64 = transaction
        .query_one(
            "SELECT COUNT(*) FROM flashcard_review_history WHERE user_id = $1",
            &[&user_id],
        )
        .await?
        .get(0);

    // Persist default so the review hot path stops re-checking every click
    if review_count < 50 {
        cache_optimal_retention(transaction, user_id, default_retention).await?;
        return Ok(default_retention);
    }

    // Join progress once — avoid correlated subqueries per history row
    let rows = transaction
        .query(
            "SELECT
                fh.id,
                fh.flashcard_id::bigint AS cid,
                0 AS usn,
                fh.rating AS button_chosen,
                fh.scheduled_days AS interval,
                fh.elapsed_days AS last_interval,
                0 AS ease_factor,
                0 AS taken_millis,
                CASE
                    WHEN ufp.status = 'learning' THEN 0
                    WHEN ufp.status = 'review' THEN 1
                    ELSE 2
                END AS review_kind
             FROM flashcard_review_history fh
             LEFT JOIN user_flashcard_progress ufp
               ON ufp.user_id = fh.user_id
              AND ufp.flashcard_id = fh.flashcard_id
              AND ufp.card_side = fh.card_side
              AND NOT ufp.archived
             WHERE fh.user_id = $1
             ORDER BY fh.review_time",
            &[&user_id],
        )
        .await?;

    let mut revlogs = Vec::with_capacity(rows.len());
    for row in rows {
        revlogs.push(RevlogEntry {
            id: row.get::<_, i32>("id") as i64,
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
        });
    }

    if revlogs.is_empty() {
        cache_optimal_retention(transaction, user_id, default_retention).await?;
        return Ok(default_retention);
    }

    let day_cutoff = Utc::now().timestamp() - 86400;
    let config = extract_simulator_config(revlogs, day_cutoff, true);
    let _fsrs = FSRS::new(&[])?;
    let progress_callback = |_: ItemProgress| true;

    let retention = match optimal_retention(
        &config,
        DEFAULT_PARAMETERS.as_slice(),
        progress_callback,
        None::<Vec<Card>>,
        None,
    ) {
        Ok(value) => value,
        Err(_) => default_retention,
    };

    cache_optimal_retention(transaction, user_id, retention).await?;
    Ok(retention)
}

async fn cache_optimal_retention(
    transaction: &Transaction<'_>,
    user_id: i32,
    retention: f32,
) -> Result<(), FlashcardError> {
    transaction
        .execute(
            "INSERT INTO user_settings (user_id, optimal_retention, last_calculated)
             VALUES ($1, $2, CURRENT_TIMESTAMP)
             ON CONFLICT (user_id)
             DO UPDATE SET optimal_retention = $2, last_calculated = CURRENT_TIMESTAMP",
            &[&user_id, &(retention as f64)],
        )
        .await?;
    Ok(())
}

/// Cached/default retention for the review hot path. Never runs the FSRS solver.
/// Returns `(retention, needs_background_refresh)`.
pub async fn get_optimal_retention(
    transaction: &Transaction<'_>,
    user_id: i32,
) -> Result<(f32, bool), FlashcardError> {
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
        let retention = row.get::<_, f64>("optimal_retention") as f32;
        let stale = Utc::now().signed_duration_since(last_calculated).num_days()
            >= OPTIMAL_RETENTION_CACHE_DAYS;
        // Use stale cache immediately — refresh in the background after the review commits
        return Ok((retention, stale));
    }

    Ok((DEFAULT_OPTIMAL_RETENTION, true))
}

/// Recalculate and cache optimal retention off the review request path.
pub async fn refresh_optimal_retention(pool: &Pool, user_id: i32) -> Result<(), FlashcardError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Bail if another worker already refreshed (stampede guard)
    let fresh_after = Utc::now() - Duration::days(OPTIMAL_RETENTION_CACHE_DAYS);
    let fresh = transaction
        .query_opt(
            "SELECT 1 FROM user_settings
             WHERE user_id = $1 AND last_calculated >= $2",
            &[&user_id, &fresh_after],
        )
        .await?;
    if fresh.is_some() {
        return Ok(());
    }

    // Claim the refresh slot so concurrent reviews don't all solve FSRS
    transaction
        .execute(
            "INSERT INTO user_settings (user_id, optimal_retention, last_calculated)
             VALUES ($1, $2, CURRENT_TIMESTAMP)
             ON CONFLICT (user_id)
             DO UPDATE SET last_calculated = CURRENT_TIMESTAMP",
            &[&user_id, &(DEFAULT_OPTIMAL_RETENTION as f64)],
        )
        .await?;

    calculate_optimal_retention(&transaction, user_id).await?;
    transaction.commit().await?;
    Ok(())
}

fn spawn_optimal_retention_refresh(pool: Pool, user_id: i32) {
    tokio::spawn(async move {
        if let Err(e) = refresh_optimal_retention(&pool, user_id).await {
            error!(
                "Background optimal retention refresh failed for user {}: {}",
                user_id, e
            );
        }
    });
}

pub async fn review_flashcard(
    pool: &Pool,
    user_id: i32,
    req: &ReviewRequest,
) -> Result<ReviewResponse, FlashcardError> {
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

    let fsrs = FSRS::new(&[])?;
    let (desired_retention, refresh_retention) =
        get_optimal_retention(&transaction, user_id).await?;
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

    if refresh_retention {
        spawn_optimal_retention_refresh(pool.clone(), user_id);
    }

    Ok(ReviewResponse {
        success: true,
        card_side: req.card_side.clone(),
        message: format!(
            "Review recorded successfully for {} side. New interval: {} days",
            req.card_side, interval
        ),
        next_review: Some(next_review),
        answer_correct: None,
        suggested_rating: None,
    })
}

async fn get_current_status(
    transaction: &Transaction<'_>,
    flashcard_id: i32,
    user_id: i32,
    card_side: &String,
) -> Result<FlashcardStatus, FlashcardError> {
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
) -> Result<(), FlashcardError> {
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
) -> Result<(), FlashcardError> {
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

    // 2. Levelled cards: allow when at least one of the card's levels is unlocked.
    // (A card may sit on multiple levels; locking a later level must not block an
    // unlocked earlier level that still contains the same flashcard.)
    let level_row = transaction
        .query_one(
            "SELECT
                EXISTS(
                    SELECT 1 FROM flashcard_level_items WHERE flashcard_id = $2
                ) AS in_level,
                COALESCE(
                    (
                        SELECT bool_or(check_level_prerequisites($1, fli.level_id))
                        FROM flashcard_level_items fli
                        WHERE fli.flashcard_id = $2
                    ),
                    true
                ) AS unlocked,
                (
                    SELECT fli.level_id
                    FROM flashcard_level_items fli
                    WHERE fli.flashcard_id = $2
                      AND NOT check_level_prerequisites($1, fli.level_id)
                    ORDER BY fli.level_id
                    LIMIT 1
                ) AS sample_locked_level_id",
            &[&user_id, &flashcard_id],
        )
        .await?;

    let in_level: bool = level_row.get("in_level");
    let is_unlocked: bool = level_row.get("unlocked");
    if in_level && !is_unlocked {
        let level_id: i32 = level_row.get("sample_locked_level_id");
        return Err(format!(
            "access denied: level {} is locked. Complete prerequisites first.",
            level_id
        )
        .into());
    }

    Ok(())
}

pub async fn update_flashcard_position(
    pool: &Pool,
    user_id: i32,
    flashcard_id: i32,
    new_position: i32,
) -> Result<(), FlashcardError> {
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
) -> Result<ImportFromCollectionResponse, FlashcardError> {
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
) -> Result<DirectAnswerResponse, FlashcardError> {
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
            crate::utils::canonical::get_canonical_form(&expected),
            crate::utils::canonical::get_canonical_form(&provided),
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

fn is_wordbank_like_direction(direction: &FlashcardDirection) -> bool {
    matches!(
        direction,
        FlashcardDirection::Wordbank
            | FlashcardDirection::WordbankReverse
            | FlashcardDirection::WordbankBoth
            | FlashcardDirection::ListenWordbank
            | FlashcardDirection::ListenWordbankReverse
            | FlashcardDirection::ListenWordbankBoth
    )
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

    let mut previous: Vec<usize> = (0..=len2).collect();
    let mut current = vec![0; len2 + 1];
    for i in 1..=len1 {
        current[0] = i;
        for j in 1..=len2 {
            let cost = usize::from(s1_chars[i - 1] != s2_chars[j - 1]);
            current[j] = (previous[j] + 1)
                .min(current[j - 1] + 1)
                .min(previous[j - 1] + cost);
        }
        std::mem::swap(&mut current, &mut previous);
    }
    let distance = previous[len2];
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
) -> Result<ReviewResponse, FlashcardError> {
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

    let expected_raw = match expected_fillin_like_raw(&flashcard, req.card_side.as_str()) {
        Some(s) => s,
        None => {
            return Err(format!(
                "Invalid card side '{}' for fill-in direction '{:?}'",
                req.card_side, direction
            )
            .into());
        }
    };
    // Typed fill-in lowercases for comparison; wordbank keeps casing until normalize
    // (so English "I" survives). Direct/Reverse must not use this endpoint.
    if matches!(
        direction,
        FlashcardDirection::Direct | FlashcardDirection::Reverse | FlashcardDirection::Both
    ) {
        return Err(format!(
            "Fill-in endpoint called for non-fill-in direction '{:?}'",
            flashcard.direction
        )
        .into());
    }

    validate_fill_in_answers(&req.answer, &expected_raw)?;
    let wordbank_lang = wordbank_lang_for(&direction, req.card_side.as_str());
    let provided = if let Some(lang) = wordbank_lang {
        // Chip-assembled answers are already normalized; normalize is idempotent.
        normalize_wordbank_phrase(&req.answer, lang)
    } else {
        req.answer.trim().to_lowercase()
    };
    let expected_for_grade = if let Some(lang) = wordbank_lang {
        normalize_wordbank_phrase(&expected_raw, lang)
    } else {
        expected_raw.trim().to_lowercase()
    };

    // Calculate similarity, handling semicolon-separated answers for free content
    let similarity = if is_free_content && expected_raw.contains(';') {
        // 1. Split, trim, (wordbank-normalize or lowercase), and filter empty parts
        let possible_answers: Vec<String> = expected_raw
            .split(';')
            .map(|s| {
                if let Some(lang) = wordbank_lang {
                    normalize_wordbank_phrase(s, lang)
                } else {
                    s.trim().to_lowercase()
                }
            })
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
            calculate_similarity(&expected_for_grade, &provided)
        }
    } else {
        // Not free content or no semicolons, use standard similarity calculation
        calculate_similarity(&expected_for_grade, &provided)
    };

    // Similarity-based suggested rating (used for grade-only Check / answer_correct).
    let suggested_rating = if similarity >= 0.99 {
        4u32
    } else if similarity >= 0.9 {
        3
    } else if similarity >= 0.7 {
        2
    } else {
        1
    };
    let answer_correct = suggested_rating >= 3;

    // Duo flow: Check grades only; Continue / Too easy / Too difficult commit with `rating`.
    let Some(commit_rating) = req.rating else {
        return Ok(ReviewResponse {
            success: true,
            card_side: req.card_side.clone(),
            message: format!(
                "Graded (Similarity: {:.1}%, suggested: {})",
                similarity * 100.0,
                match suggested_rating {
                    4 => "Easy",
                    3 => "Good",
                    2 => "Hard",
                    _ => "Again",
                }
            ),
            next_review: None,
            answer_correct: Some(answer_correct),
            suggested_rating: Some(suggested_rating),
        });
    };

    if !(1..=4).contains(&commit_rating) {
        return Err("rating must be 1–4".into());
    }

    let review_req = ReviewRequest {
        flashcard_id: req.flashcard_id,
        rating: commit_rating,
        card_side: req.card_side.clone(),
    };
    let review_result = review_flashcard(pool, user_id, &review_req).await?;

    let message = format!(
        "{} (Similarity: {:.1}%, Rating: {})",
        review_result.message,
        similarity * 100.0,
        match commit_rating {
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
        answer_correct: Some(answer_correct),
        suggested_rating: None,
    })
}

async fn auto_progress_related_cards(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    word: &str,
    card_side: &str,
) -> Result<(), FlashcardError> {
    // Escape special regex characters in the word
    let escaped_word = fancy_regex::escape(word);
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
) -> Result<(), FlashcardError> {
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

    let fsrs = FSRS::new(&[])?;
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
) -> Result<Option<dto::QuizFlashcardQuestionDto>, FlashcardError> {
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
        AND f.direction IN ('quiz_direct', 'quiz_reverse', 'quiz_both', 'quiz_image_direct', 'quiz_image_reverse', 'quiz_image_both')
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

    // Determine effective direction (direct/reverse) for QuizBoth / QuizImageBoth
    let effective_direction = match direction {
        FlashcardDirection::QuizDirect | FlashcardDirection::QuizImageDirect => "direct",
        FlashcardDirection::QuizReverse | FlashcardDirection::QuizImageReverse => "reverse",
        FlashcardDirection::QuizBoth | FlashcardDirection::QuizImageBoth => {
            if rand::random() {
                "direct"
            } else {
                "reverse"
            }
        }
        _ => "direct",
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
) -> Result<dto::QuizAnswerResultDto, FlashcardError> {
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

    // Text quiz: plain string match. Image quiz may store "item_id" while the client
    // submits "item_id:front" / "item_id:back" (or the reverse when the DB has a side).
    let selected = answer_data.selected_answer_text.trim().to_lowercase();
    let correct = correct_answer_text.trim().to_lowercase();
    let is_correct = selected == correct
        || selected == format!("{}:front", correct)
        || selected == format!("{}:back", correct)
        || correct == format!("{}:front", selected)
        || correct == format!("{}:back", selected);

    let suggested_rating = if is_correct { 4u32 } else { 1 };
    let message = if is_correct {
        "Correct!".to_string()
    } else {
        format!("Incorrect. The correct answer was: {}", correct_answer_text)
    };

    // Duo Check: grade only (no history / FSRS until Continue commits a rating).
    let Some(commit_rating) = answer_data.rating else {
        transaction.commit().await?;
        return Ok(dto::QuizAnswerResultDto {
            correct: is_correct,
            message,
            next_review: None,
            suggested_rating: Some(suggested_rating),
        });
    };

    if !(1..=4).contains(&commit_rating) {
        return Err("rating must be 1–4".into());
    }

    transaction.execute(
        "INSERT INTO user_quiz_answer_history (user_id, flashcard_id, selected_option_text, is_correct_selection, presented_options, answered_at)
         VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)",
        &[&user_id, &flashcard_id, &answer_data.selected_answer_text, &is_correct, &serde_json::to_value(&answer_data.presented_options)?]
    ).await?;

    let review_req = ReviewRequest {
        flashcard_id,
        rating: commit_rating,
        card_side: answer_data.card_side.clone(),
    };
    transaction.commit().await?;

    let review_response = review_flashcard(pool, user_id, &review_req).await?;

    Ok(dto::QuizAnswerResultDto {
        correct: is_correct,
        message,
        next_review: review_response.next_review,
        suggested_rating: None,
    })
}

pub async fn get_streak(
    pool: &Pool,
    user_id: i32,
    days: i32,
) -> Result<StreakResponse, FlashcardError> {
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
) -> Result<i32, FlashcardError> {
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
) -> Result<LevelResponse, FlashcardError> {
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
    insert_level_prerequisites(
        &transaction,
        level_row.get::<_, i32>("level_id"),
        &req.prerequisite_ids,
    )
    .await?;

    let level = get_level_details(&transaction, level_row.get("level_id"), Some(user_id)).await?;

    transaction.commit().await?;
    Ok(level)
}

pub async fn update_level(
    pool: &Pool,
    level_id: i32,
    user_id: i32,
    req: &UpdateLevelRequest,
) -> Result<LevelResponse, FlashcardError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let collection_id = get_collection_id(&transaction, level_id).await?;
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // `min_success_rate` is DOUBLE PRECISION; an `Option<f32>` fails to serialize.
    let min_success_rate = req.min_success_rate.map(f64::from);
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
                &min_success_rate,
                &req.position,
                &level_id,
            ],
        )
        .await?;

    if update_result == 0 {
        return Err(FlashcardError::LevelNotFound);
    }

    // Update prerequisites if provided
    if let Some(prereq_ids) = &req.prerequisite_ids {
        transaction
            .execute(
                "DELETE FROM level_prerequisites WHERE level_id = $1",
                &[&level_id],
            )
            .await?;

        insert_level_prerequisites(&transaction, level_id, prereq_ids).await?;
    }

    let level = get_level_details(&transaction, level_id, Some(user_id)).await?;

    transaction.commit().await?;
    Ok(level)
}

/// Insert all prerequisite links for a level in one statement.
async fn insert_level_prerequisites(
    transaction: &Transaction<'_>,
    level_id: i32,
    prerequisite_ids: &[i32],
) -> Result<(), tokio_postgres::Error> {
    if prerequisite_ids.is_empty() {
        return Ok(());
    }
    transaction
        .execute(
            "INSERT INTO level_prerequisites (level_id, prerequisite_id)
             SELECT DISTINCT $1::int4, p FROM UNNEST($2::int4[]) AS p",
            &[&level_id, &prerequisite_ids],
        )
        .await?;
    Ok(())
}

pub async fn add_cards_to_level(
    pool: &Pool,
    level_id: i32,
    user_id: i32,
    req: &AddCardsRequest,
) -> Result<Vec<LevelCardResponse>, FlashcardError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let collection_id = get_collection_id(&transaction, level_id).await?;
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    // One query locks every requested card that belongs to this collection.
    let requested: HashSet<i32> = req.flashcard_ids.iter().copied().collect();
    let found = transaction
        .query(
            "SELECT id FROM flashcards WHERE id = ANY($1) AND collection_id = $2 FOR SHARE",
            &[&req.flashcard_ids, &collection_id],
        )
        .await?
        .len();
    if found != requested.len() {
        return Err("Flashcard does not belong to the level's collection".into());
    }

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
    // A repeated id keeps its last position, as the old row-by-row upsert did
    // (one statement may not upsert the same row twice).
    let mut position_by_id: HashMap<i32, i32> = HashMap::with_capacity(requested.len());
    for (idx, &flashcard_id) in req.flashcard_ids.iter().enumerate() {
        position_by_id.insert(flashcard_id, start_position + idx as i32);
    }
    let (ids, positions): (Vec<i32>, Vec<i32>) = position_by_id.into_iter().unzip();
    transaction
        .execute(
            "INSERT INTO flashcard_level_items (level_id, flashcard_id, position)
             SELECT $1, i.id, i.pos FROM UNNEST($2::int4[], $3::int4[]) AS i(id, pos)
             ON CONFLICT (level_id, flashcard_id) DO UPDATE
             SET position = EXCLUDED.position",
            &[&level_id, &ids, &positions],
        )
        .await?;

    let cards = get_level_cards(&transaction, level_id, Some(user_id)).await?;

    transaction.commit().await?;
    Ok(cards)
}

pub async fn get_level_details(
    transaction: &Transaction<'_>,
    level_id: i32,
    user_id: Option<i32>,
) -> Result<LevelResponse, FlashcardError> {
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

    // Guests: unlock when there are no prerequisites (client merges localStorage for gated levels).
    let is_locked = if let Some(uid) = user_id {
        !transaction
            .query_one(
                "SELECT check_level_prerequisites($1, $2) as unlocked",
                &[&uid, &level_id],
            )
            .await?
            .get::<_, bool>("unlocked")
    } else {
        !prerequisites.is_empty()
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
) -> Result<LevelListResponse, FlashcardError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_collection_read_access(&transaction, collection_id, user_id)
        .await
        .map_err(|e| match e {
            AppError::Auth(_) => FlashcardError::AccessDenied(e.to_string()),
            AppError::NotFound(_) => FlashcardError::NotFound(e.to_string()),
            other => FlashcardError::Other(other.to_string()),
        })?;

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

        // Guests have no server progress: unlock levels with no prerequisites; gated
        // levels stay locked until the client merges localStorage completions.
        let is_locked = if let Some(uid) = user_id {
            !transaction
                .query_one(
                    "SELECT check_level_prerequisites($1, $2) as unlocked",
                    &[&uid, &level_id],
                )
                .await?
                .get::<_, bool>("unlocked")
        } else {
            !prerequisites.is_empty()
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
) -> Result<Vec<PrerequisiteLevel>, FlashcardError> {
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
) -> Result<Option<LevelProgress>, FlashcardError> {
    if let Some(row) = transaction
        .query_opt(
            "SELECT *, check_level_prerequisites($1, $2) AS prereqs_met
         FROM user_level_progress
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

        // Source of truth is check_level_prerequisites (same as review/list gates).
        // Do not trust unlocked_at alone — it can be stale from before V133.
        let prereqs_met: bool = row.get("prereqs_met");

        Ok(Some(LevelProgress {
            cards_completed: row.get("cards_completed"),
            correct_answers: row.get("correct_answers"),
            total_answers: row.get("total_answers"),
            success_rate,
            is_unlocked: prereqs_met,
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
) -> Result<Vec<LevelCardResponse>, FlashcardError> {
    let collection_id = get_collection_id(transaction, level_id).await?;
    verify_collection_read_access(transaction, collection_id, user_id).await?;
    let mut cards = Vec::new();

    let rows = transaction
        .query(
            "SELECT f.id, f.collection_id, f.direction, ci.item_id, ci.definition_id, fli.position,
                v.word, v.valsiid, d.definition, d.definitionid, ci.notes as ci_notes,
                ci.free_content_front, ci.free_content_back, ci.canonical_form,
                EXISTS(SELECT 1 FROM collection_item_images cii WHERE cii.item_id = ci.item_id AND cii.side = 'front') as has_front_image,
                EXISTS(SELECT 1 FROM collection_item_images cii WHERE cii.item_id = ci.item_id AND cii.side = 'back') as has_back_image,
                EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = ci.item_id) as has_sound,
                COUNT(CASE WHEN frh.rating >= 3 THEN 1 END) as correct_answers,
                COUNT(frh.rating) as total_attempts,
                MAX(frh.review_time) as last_reviewed_at
         FROM flashcard_level_items fli
         JOIN flashcard_levels fl ON fl.level_id = fli.level_id
         JOIN flashcards f ON fli.flashcard_id = f.id AND f.collection_id = fl.collection_id
         JOIN collection_items ci ON f.item_id = ci.item_id
         LEFT JOIN definitions d ON ci.definition_id = d.definitionid
         LEFT JOIN valsi v ON d.valsiid = v.valsiid
         LEFT JOIN flashcard_review_history frh
             ON frh.flashcard_id = f.id
             AND frh.user_id = $2
         WHERE fli.level_id = $1
         GROUP BY f.id, f.collection_id, f.direction, ci.item_id, fli.position, v.word, d.definition, v.valsiid, d.definitionid,
                  ci.free_content_front, ci.free_content_back, ci.canonical_form, ci.notes, ci.definition_id
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
            direction: row.get("direction"),
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
            wordbank_chips: None,
            wordbank_expected_normalized: None,
        });
    }

    attach_wordbank_chips_to_level_cards(&mut cards);
    Ok(cards)
}

pub async fn delete_level(pool: &Pool, level_id: i32, user_id: i32) -> Result<(), FlashcardError> {
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
        return Err(FlashcardError::BadRequest(
            "Cannot delete level: it is a prerequisite for other levels.".to_string(),
        ));
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
        return Err(FlashcardError::LevelNotFound);
    }

    transaction.commit().await?;
    Ok(())
}

async fn get_collection_id(
    transaction: &Transaction<'_>,
    level_id: i32,
) -> Result<i32, FlashcardError> {
    transaction
        .query_opt(
            "SELECT collection_id FROM flashcard_levels WHERE level_id = $1",
            &[&level_id],
        )
        .await?
        .map(|row| row.get("collection_id"))
        .ok_or(FlashcardError::LevelNotFound)
}

pub async fn get_level_cards_paginated(
    pool: &Pool,
    level_id: i32,
    user_id: Option<i32>,
    page: i64,
    per_page: i64,
) -> Result<LevelCardListResponse, FlashcardError> {
    let client = pool.get().await?;
    let offset = (page - 1) * per_page;

    // Enforce collection read access: anonymous only public, logged-in: public or owner
    let collection_row = client
        .query_opt(
            "SELECT c.collection_id, c.user_id as owner_id, c.is_public
             FROM flashcard_levels l
             JOIN collections c ON c.collection_id = l.collection_id
             WHERE l.level_id = $1",
            &[&level_id],
        )
        .await?
        .ok_or(FlashcardError::LevelNotFound)?;
    let owner_id: i32 = collection_row.get("owner_id");
    let is_public: bool = collection_row.get("is_public");
    match user_id {
        None if !is_public => {
            return Err(FlashcardError::AccessDenied("Access denied".to_string()))
        }
        Some(uid) if !is_public && owner_id != uid => {
            return Err(FlashcardError::AccessDenied("Access denied".to_string()))
        }
        _ => {}
    }

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
            return Err(FlashcardError::Other(
                "Level is locked. Complete prerequisites first.".to_string(),
            ));
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
            "SELECT f.id, f.collection_id, f.direction, ci.item_id, ci.definition_id, fli.position,
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
            GROUP BY f.id, f.collection_id, f.direction, ci.item_id, fli.position, v.word, d.definition, v.valsiid, d.definitionid,
                    ci.free_content_front, ci.free_content_back, ci.canonical_form, ci.notes, ci.definition_id
            ORDER BY fli.position
            LIMIT $3 OFFSET $4",
            &[&user_id, &level_id, &per_page, &offset],
        )
        .await?;

    let mut cards: Vec<LevelCardResponse> = rows
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
                direction: row.get("direction"),
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
                wordbank_chips: None,
                wordbank_expected_normalized: None,
            }
        })
        .collect();

    attach_wordbank_chips_to_level_cards(&mut cards);

    Ok(LevelCardListResponse {
        cards,
        total,
        page,
        per_page,
    })
}

/// Move guest study progress onto `target_user_id`, then delete the guest user (CASCADE).
pub async fn claim_guest_progress(
    pool: &Pool,
    target_user_id: i32,
    guest_access_token: &str,
) -> Result<crate::flashcards::dto::ClaimGuestProgressResponse, FlashcardError> {
    let guest_claims = crate::auth::models::authenticate_token(pool, guest_access_token)
        .await
        .map_err(|e| format!("Invalid guest session: {e}"))?;
    if guest_claims.role.to_lowercase() != "guest" {
        return Err("Token is not a guest session".into());
    }
    let guest_id = guest_claims.sub;
    if guest_id == target_user_id {
        return Err("Cannot claim your own session".into());
    }

    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    // Confirm guest row still exists and is guest.
    let guest_ok: bool = transaction
        .query_one(
            "SELECT LOWER(role::text) = 'guest' FROM users WHERE userid = $1 AND NOT disabled",
            &[&guest_id],
        )
        .await?
        .get(0);
    if !guest_ok {
        return Err("Guest account is no longer available".into());
    }

    // Upsert card progress: prefer more recent last_reviewed_at when both exist.
    let moved_cards = transaction
        .execute(
            r#"
            INSERT INTO user_flashcard_progress AS t (
                user_id, flashcard_id, card_side, status, ease_factor, interval,
                review_count, next_review_at, last_reviewed_at,
                stability, difficulty, archived
            )
            SELECT
                $1, flashcard_id, card_side, status, ease_factor, interval,
                review_count, next_review_at, last_reviewed_at,
                stability, difficulty, archived
            FROM user_flashcard_progress
            WHERE user_id = $2 AND NOT archived
            ON CONFLICT (user_id, flashcard_id, card_side) WHERE NOT archived DO UPDATE SET
                status = CASE
                    WHEN EXCLUDED.last_reviewed_at IS NOT NULL
                         AND (t.last_reviewed_at IS NULL OR EXCLUDED.last_reviewed_at >= t.last_reviewed_at)
                    THEN EXCLUDED.status ELSE t.status END,
                ease_factor = CASE
                    WHEN EXCLUDED.last_reviewed_at IS NOT NULL
                         AND (t.last_reviewed_at IS NULL OR EXCLUDED.last_reviewed_at >= t.last_reviewed_at)
                    THEN EXCLUDED.ease_factor ELSE t.ease_factor END,
                interval = CASE
                    WHEN EXCLUDED.last_reviewed_at IS NOT NULL
                         AND (t.last_reviewed_at IS NULL OR EXCLUDED.last_reviewed_at >= t.last_reviewed_at)
                    THEN EXCLUDED.interval ELSE t.interval END,
                review_count = GREATEST(t.review_count, EXCLUDED.review_count),
                next_review_at = CASE
                    WHEN EXCLUDED.last_reviewed_at IS NOT NULL
                         AND (t.last_reviewed_at IS NULL OR EXCLUDED.last_reviewed_at >= t.last_reviewed_at)
                    THEN EXCLUDED.next_review_at ELSE t.next_review_at END,
                last_reviewed_at = CASE
                    WHEN EXCLUDED.last_reviewed_at IS NOT NULL
                         AND (t.last_reviewed_at IS NULL OR EXCLUDED.last_reviewed_at >= t.last_reviewed_at)
                    THEN EXCLUDED.last_reviewed_at ELSE t.last_reviewed_at END,
                stability = CASE
                    WHEN EXCLUDED.last_reviewed_at IS NOT NULL
                         AND (t.last_reviewed_at IS NULL OR EXCLUDED.last_reviewed_at >= t.last_reviewed_at)
                    THEN EXCLUDED.stability ELSE t.stability END,
                difficulty = CASE
                    WHEN EXCLUDED.last_reviewed_at IS NOT NULL
                         AND (t.last_reviewed_at IS NULL OR EXCLUDED.last_reviewed_at >= t.last_reviewed_at)
                    THEN EXCLUDED.difficulty ELSE t.difficulty END,
                archived = false
            "#,
            &[&target_user_id, &guest_id],
        )
        .await?;

    // Re-point review history after target progress rows exist.
    transaction
        .execute(
            r#"
            UPDATE flashcard_review_history h
            SET user_id = $1
            WHERE h.user_id = $2
              AND EXISTS (
                SELECT 1 FROM user_flashcard_progress p
                WHERE p.user_id = $1
                  AND p.flashcard_id = h.flashcard_id
                  AND p.card_side = h.card_side
              )
            "#,
            &[&target_user_id, &guest_id],
        )
        .await?;

    let moved_levels = transaction
        .execute(
            r#"
            INSERT INTO user_level_progress AS t (
                user_id, level_id, cards_completed, correct_answers, total_answers,
                last_activity_at, unlocked_at, completed_at
            )
            SELECT
                $1, level_id, cards_completed, correct_answers, total_answers,
                last_activity_at, unlocked_at, completed_at
            FROM user_level_progress
            WHERE user_id = $2
            ON CONFLICT (user_id, level_id) DO UPDATE SET
                cards_completed = GREATEST(t.cards_completed, EXCLUDED.cards_completed),
                correct_answers = GREATEST(t.correct_answers, EXCLUDED.correct_answers),
                total_answers = GREATEST(t.total_answers, EXCLUDED.total_answers),
                last_activity_at = GREATEST(t.last_activity_at, EXCLUDED.last_activity_at),
                unlocked_at = COALESCE(t.unlocked_at, EXCLUDED.unlocked_at),
                completed_at = COALESCE(t.completed_at, EXCLUDED.completed_at)
            "#,
            &[&target_user_id, &guest_id],
        )
        .await?;

    transaction
        .execute(
            r#"
            UPDATE user_level_progress ulp
            SET unlocked_at = CASE
              WHEN check_level_prerequisites(ulp.user_id, ulp.level_id) AND ulp.unlocked_at IS NULL
              THEN CURRENT_TIMESTAMP ELSE ulp.unlocked_at END
            WHERE ulp.user_id = $1
            "#,
            &[&target_user_id],
        )
        .await?;

    // Guest insert trigger creates user_balances (no ON DELETE CASCADE). Drop that
    // and any leftover review history that was not re-pointed before deleting the user.
    transaction
        .execute(
            "DELETE FROM flashcard_review_history WHERE user_id = $1",
            &[&guest_id],
        )
        .await?;
    transaction
        .execute("DELETE FROM user_balances WHERE user_id = $1", &[&guest_id])
        .await?;

    // Drop guest account (CASCADE progress/sessions leftover).
    transaction
        .execute(
            "DELETE FROM users WHERE userid = $1 AND LOWER(role::text) = 'guest'",
            &[&guest_id],
        )
        .await?;

    transaction.commit().await?;
    Ok(crate::flashcards::dto::ClaimGuestProgressResponse {
        guest_user_id: guest_id,
        moved_card_rows: moved_cards as i64,
        moved_level_rows: moved_levels as i64,
    })
}

pub async fn merge_anonymous_progress(
    pool: &Pool,
    user_id: i32,
    req: &MergeProgressRequest,
) -> Result<MergeProgressResponse, FlashcardError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    verify_collection_read_access(&transaction, req.collection_id, Some(user_id))
        .await
        .map_err(|e| match e {
            AppError::Auth(_) => FlashcardError::AccessDenied(e.to_string()),
            AppError::NotFound(_) => FlashcardError::NotFound(e.to_string()),
            other => FlashcardError::Other(other.to_string()),
        })?;

    let mut merged = 0_i64;
    for item in &req.level_progress {
        let level_collection_id: i32 = transaction
            .query_one(
                "SELECT collection_id FROM flashcard_levels WHERE level_id = $1",
                &[&item.level_id],
            )
            .await?
            .get("collection_id");
        if level_collection_id != req.collection_id {
            continue; // skip levels that don't belong to this collection
        }

        transaction
            .execute(
                r#"
                INSERT INTO user_level_progress
                  (user_id, level_id, cards_completed, correct_answers, total_answers, last_activity_at, completed_at)
                VALUES (
                  $1, $2, $3, $4, $5, CURRENT_TIMESTAMP,
                  -- Keep $4/$5 typed as integer (column type); only the rate expr uses float8.
                  CASE
                    WHEN (SELECT min_cards FROM flashcard_levels WHERE level_id = $2) <= $3
                     AND (SELECT min_success_rate FROM flashcard_levels WHERE level_id = $2)
                         <= (CASE
                               WHEN $5::integer > 0
                               THEN ($4::integer)::float8 / ($5::integer)::float8
                               ELSE 0::float8
                             END)
                    THEN CURRENT_TIMESTAMP
                    ELSE NULL
                  END
                )
                ON CONFLICT (user_id, level_id) DO UPDATE SET
                  cards_completed = GREATEST(user_level_progress.cards_completed, EXCLUDED.cards_completed),
                  correct_answers = GREATEST(user_level_progress.correct_answers, EXCLUDED.correct_answers),
                  total_answers = GREATEST(user_level_progress.total_answers, EXCLUDED.total_answers),
                  last_activity_at = CURRENT_TIMESTAMP,
                  -- Judge completion on the merged values (same rule as check_level_completion,
                  -- which would read the pre-update row here).
                  completed_at = COALESCE(
                    user_level_progress.completed_at,
                    CASE
                      WHEN (SELECT min_cards FROM flashcard_levels WHERE level_id = EXCLUDED.level_id)
                             <= GREATEST(user_level_progress.cards_completed, EXCLUDED.cards_completed)
                       AND (SELECT min_success_rate FROM flashcard_levels WHERE level_id = EXCLUDED.level_id)
                             <= (CASE
                                   WHEN GREATEST(user_level_progress.total_answers, EXCLUDED.total_answers) > 0
                                   THEN GREATEST(user_level_progress.correct_answers, EXCLUDED.correct_answers)::float8
                                        / GREATEST(user_level_progress.total_answers, EXCLUDED.total_answers)::float8
                                   ELSE 0::float8
                                 END)
                      THEN CURRENT_TIMESTAMP
                      ELSE NULL
                    END
                  )
                "#,
                &[&user_id, &item.level_id, &item.cards_completed, &item.correct_answers, &item.total_answers],
            )
            .await?;
        merged += 1;
    }

    // Set unlocked_at for levels where prerequisites are now met
    transaction
        .execute(
            r#"
            UPDATE user_level_progress ulp
            SET unlocked_at = CASE
              WHEN check_level_prerequisites(ulp.user_id, ulp.level_id) AND ulp.unlocked_at IS NULL
              THEN CURRENT_TIMESTAMP ELSE ulp.unlocked_at END
            WHERE ulp.user_id = $1
            "#,
            &[&user_id],
        )
        .await?;

    transaction.commit().await?;
    Ok(MergeProgressResponse {
        merged_levels: merged,
    })
}

pub async fn remove_card_from_level(
    pool: &Pool,
    level_id: i32,
    flashcard_id: i32,
    user_id: i32,
) -> Result<(), FlashcardError> {
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
        return Err(FlashcardError::NotFound(
            "Card not found in level".to_string(),
        ));
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

fn validate_fill_in_answers(answer: &str, expected: &str) -> Result<(), FlashcardError> {
    if answer.len() > 2048
        || expected.len() > 8192
        || expected.split(';').count() > 32
        || expected.split(';').any(|answer| answer.len() > 2048)
    {
        return Err("Fill-in answers exceed the supported grading limit".into());
    }
    Ok(())
}

fn direction_from_challenge_mix_str(s: &str) -> Result<FlashcardDirection, FlashcardError> {
    match s {
        "direct" => Ok(FlashcardDirection::Direct),
        "reverse" => Ok(FlashcardDirection::Reverse),
        "both" => Ok(FlashcardDirection::Both),
        "fillin" => Ok(FlashcardDirection::FillIn),
        "fillin_reverse" => Ok(FlashcardDirection::FillInReverse),
        "fillin_both" => Ok(FlashcardDirection::FillInBoth),
        "wordbank" => Ok(FlashcardDirection::Wordbank),
        "wordbank_reverse" => Ok(FlashcardDirection::WordbankReverse),
        "wordbank_both" => Ok(FlashcardDirection::WordbankBoth),
        "listen_wordbank" => Ok(FlashcardDirection::ListenWordbank),
        "listen_wordbank_reverse" => Ok(FlashcardDirection::ListenWordbankReverse),
        "listen_wordbank_both" => Ok(FlashcardDirection::ListenWordbankBoth),
        "just_information" => Ok(FlashcardDirection::JustInformation),
        "quiz_direct" => Ok(FlashcardDirection::QuizDirect),
        "quiz_reverse" => Ok(FlashcardDirection::QuizReverse),
        "quiz_both" => Ok(FlashcardDirection::QuizBoth),
        "quiz_image_direct" => Ok(FlashcardDirection::QuizImageDirect),
        "quiz_image_reverse" => Ok(FlashcardDirection::QuizImageReverse),
        "quiz_image_both" => Ok(FlashcardDirection::QuizImageBoth),
        _ => Err(format!("Unknown challenge-mix direction: {s}").into()),
    }
}

async fn reseed_owner_progress_for_direction(
    transaction: &Transaction<'_>,
    user_id: i32,
    flashcard_id: i32,
    direction: &FlashcardDirection,
) -> Result<(), FlashcardError> {
    let wanted: &[&str] = match direction {
        FlashcardDirection::Reverse
        | FlashcardDirection::FillInReverse
        | FlashcardDirection::WordbankReverse
        | FlashcardDirection::ListenWordbankReverse => &["reverse"],
        FlashcardDirection::Both
        | FlashcardDirection::FillInBoth
        | FlashcardDirection::WordbankBoth
        | FlashcardDirection::ListenWordbankBoth => &["direct", "reverse"],
        _ => &["direct"],
    };
    // Archive only the sides the new direction no longer uses; a kept side keeps its
    // FSRS state (for every learner), and an archived side that is wanted again is restored.
    transaction
        .execute(
            "UPDATE user_flashcard_progress
             SET archived = NOT (card_side = ANY($2))
             WHERE flashcard_id = $1",
            &[&flashcard_id, &wanted],
        )
        .await?;

    match direction {
        FlashcardDirection::Direct
        | FlashcardDirection::FillIn
        | FlashcardDirection::Wordbank
        | FlashcardDirection::ListenWordbank
        | FlashcardDirection::JustInformation
        | FlashcardDirection::QuizDirect
        | FlashcardDirection::QuizReverse
        | FlashcardDirection::QuizBoth
        | FlashcardDirection::QuizImageDirect
        | FlashcardDirection::QuizImageReverse
        | FlashcardDirection::QuizImageBoth => {
            initialize_progress(transaction, user_id, flashcard_id, "direct").await?;
        }
        FlashcardDirection::Reverse
        | FlashcardDirection::FillInReverse
        | FlashcardDirection::WordbankReverse
        | FlashcardDirection::ListenWordbankReverse => {
            initialize_progress(transaction, user_id, flashcard_id, "reverse").await?;
        }
        FlashcardDirection::Both
        | FlashcardDirection::FillInBoth
        | FlashcardDirection::WordbankBoth
        | FlashcardDirection::ListenWordbankBoth => {
            initialize_progress(transaction, user_id, flashcard_id, "direct").await?;
            initialize_progress(transaction, user_id, flashcard_id, "reverse").await?;
        }
    }
    Ok(())
}

/// Apply challenge-mix directions to every flashcard in a level (owner only).
pub async fn apply_challenge_mix_to_level(
    pool: &Pool,
    user_id: i32,
    level_id: i32,
) -> Result<dto::ApplyChallengeMixResponse, FlashcardError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let collection_id = get_collection_id(&transaction, level_id).await?;
    verify_collection_ownership(&transaction, collection_id, user_id).await?;

    let rows = transaction
        .query(
            "SELECT f.id, f.direction,
                    COALESCE(v.word, ci.free_content_front, '') as front,
                    EXISTS(SELECT 1 FROM collection_item_sounds WHERE item_id = ci.item_id) as has_custom_sound,
                    v.word as valsi_word
             FROM flashcard_level_items fli
             JOIN flashcards f ON f.id = fli.flashcard_id
             JOIN collection_items ci ON f.item_id = ci.item_id
             LEFT JOIN definitions d ON ci.definition_id = d.definitionid
             LEFT JOIN valsi v ON d.valsiid = v.valsiid
             WHERE fli.level_id = $1
             ORDER BY fli.position",
            &[&level_id],
        )
        .await?;

    let fronts: Vec<String> = rows.iter().map(|r| r.get::<_, String>("front")).collect();
    let valsi_words: Vec<Option<String>> = rows
        .iter()
        .map(|r| r.get::<_, Option<String>>("valsi_word"))
        .collect();
    let words_for_sound: Vec<String> = valsi_words.iter().filter_map(|w| w.clone()).collect();
    let sound_urls = if words_for_sound.is_empty() {
        HashMap::new()
    } else {
        get_valsi_sound_urls_from_db(pool, &words_for_sound)
            .await
            .unwrap_or_default()
    };

    let mix_cards: Vec<crate::utils::challenge_mix::ChallengeMixCard<'_>> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let has_custom: bool = row.get("has_custom_sound");
            let has_valsi = valsi_words[i]
                .as_ref()
                .and_then(|w| sound_urls.get(w))
                .and_then(|u| u.as_ref())
                .is_some();
            crate::utils::challenge_mix::ChallengeMixCard {
                front: fronts[i].as_str(),
                has_sound: has_custom || has_valsi,
            }
        })
        .collect();

    let directions = crate::utils::challenge_mix::assign_challenge_mix(&mix_cards);
    let mut updated_count = 0i32;
    let mut quiz_ids: Vec<i32> = Vec::new();

    for (row, dir_str) in rows.iter().zip(directions.iter()) {
        let flashcard_id: i32 = row.get("id");
        let existing: FlashcardDirection = row.get("direction");
        let new_dir = direction_from_challenge_mix_str(dir_str)?;
        if existing == new_dir {
            if matches!(
                new_dir,
                FlashcardDirection::QuizDirect
                    | FlashcardDirection::QuizReverse
                    | FlashcardDirection::QuizBoth
                    | FlashcardDirection::QuizImageDirect
                    | FlashcardDirection::QuizImageReverse
                    | FlashcardDirection::QuizImageBoth
            ) {
                quiz_ids.push(flashcard_id);
            }
            continue;
        }
        transaction
            .execute(
                "UPDATE flashcards SET direction = $1 WHERE id = $2",
                &[&new_dir, &flashcard_id],
            )
            .await?;
        reseed_owner_progress_for_direction(&transaction, user_id, flashcard_id, &new_dir).await?;
        updated_count += 1;
        if matches!(
            new_dir,
            FlashcardDirection::QuizDirect
                | FlashcardDirection::QuizReverse
                | FlashcardDirection::QuizBoth
                | FlashcardDirection::QuizImageDirect
                | FlashcardDirection::QuizImageReverse
                | FlashcardDirection::QuizImageBoth
        ) {
            quiz_ids.push(flashcard_id);
        }
    }

    transaction.commit().await?;

    let mut quiz_seeded_count = 0i32;
    for fid in quiz_ids {
        match generate_and_set_quiz_options(pool, user_id, fid).await {
            Ok(_) => quiz_seeded_count += 1,
            Err(e) => {
                error!(
                    "challenge-mix: failed to seed quiz options for flashcard {}: {}",
                    fid, e
                );
            }
        }
    }

    Ok(dto::ApplyChallengeMixResponse {
        updated_count,
        quiz_seeded_count,
    })
}

/// Apply challenge-mix to every level in a collection (owner only).
pub async fn apply_challenge_mix_to_collection(
    pool: &Pool,
    user_id: i32,
    collection_id: i32,
) -> Result<dto::ApplyChallengeMixResponse, FlashcardError> {
    let client = pool.get().await?;
    // Ownership check without holding a long transaction across levels.
    {
        let mut c = pool.get().await?;
        let tx = c.transaction().await?;
        verify_collection_ownership(&tx, collection_id, user_id).await?;
        tx.commit().await?;
    }
    drop(client);

    let level_ids: Vec<i32> = {
        let client = pool.get().await?;
        client
            .query(
                "SELECT level_id FROM flashcard_levels WHERE collection_id = $1 ORDER BY position",
                &[&collection_id],
            )
            .await?
            .iter()
            .map(|r| r.get("level_id"))
            .collect()
    };

    let mut updated_count = 0i32;
    let mut quiz_seeded_count = 0i32;
    for level_id in level_ids {
        let res = apply_challenge_mix_to_level(pool, user_id, level_id).await?;
        updated_count += res.updated_count;
        quiz_seeded_count += res.quiz_seeded_count;
    }

    Ok(dto::ApplyChallengeMixResponse {
        updated_count,
        quiz_seeded_count,
    })
}
#[cfg(test)]
mod security_grading_tests {
    #[test]
    fn grading_limits_both_sources_and_alternatives() {
        assert!(super::validate_fill_in_answers(&"x".repeat(100_000), "a").is_err());
        assert!(super::validate_fill_in_answers("a", &"x".repeat(100_000)).is_err());
        assert!(super::validate_fill_in_answers("a", &"a;".repeat(33)).is_err());
        assert!(super::validate_fill_in_answers("日本語", "日本語;にほんご").is_ok());
        assert_eq!(
            super::calculate_similarity("kitten", "sitting"),
            1.0 - 3.0 / 7.0
        );
        assert_eq!(super::calculate_similarity(" 日本語 ", "日本語"), 1.0);
        assert_eq!(super::calculate_similarity("", "a"), 0.0);
        use crate::utils::wordbank::WordbankLang;
        assert_eq!(
            crate::utils::wordbank::normalize_wordbank_phrase(
                "Hello, world! How are you?",
                WordbankLang::Translation
            ),
            "hello world how are you"
        );
        assert_eq!(
            crate::utils::wordbank::normalize_wordbank_phrase(
                "Hello, I love you!",
                WordbankLang::Translation
            ),
            "hello I love you"
        );
        assert_eq!(
            crate::utils::wordbank::normalize_wordbank_phrase("coi do'u", WordbankLang::Lojban),
            "coi do'u"
        );
        assert_eq!(
            super::calculate_similarity(
                &crate::utils::wordbank::normalize_wordbank_phrase(
                    "Hello, you!",
                    WordbankLang::Translation
                ),
                &crate::utils::wordbank::normalize_wordbank_phrase(
                    "hello you",
                    WordbankLang::Translation
                )
            ),
            1.0
        );
    }
}
