use super::models::{FlashcardDirection, FlashcardStatus, UserFlashcardProgress};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::types::{FromSql, Type};
use utoipa::ToSchema;
use validator::Validate;
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFlashcardRequest {
    pub definition_id: Option<i32>,
    pub free_content_front: Option<String>,
    pub free_content_back: Option<String>,
    pub notes: Option<String>,
    pub direction: FlashcardDirection,
    #[serde(default)]
    pub correct_answer_text: Option<String>,
    #[serde(default = "default_use_canonical_comparison")]
    pub use_canonical_comparison: bool,
}

fn default_use_canonical_comparison() -> bool {
    true
}

impl CreateFlashcardRequest {
    pub fn validate(&self) -> Result<(), super::error::FlashcardError> {
        match (&self.definition_id, &self.free_content_front, &self.free_content_back) {
            (Some(_), None, None) => Ok(()),
            (None, Some(front), Some(back)) if !front.trim().is_empty() && !back.trim().is_empty() => {
                Ok(())
            }
            _ => Err(super::error::FlashcardError::BadRequest(
                "Must provide either definition_id or both free_content_front and free_content_back"
                    .to_string(),
            )),
        }
    }
}
#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct FlashcardResponse {
    pub flashcard: Flashcard,
    pub progress: Vec<UserFlashcardProgress>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewRequest {
    pub flashcard_id: i32,
    pub rating: u32,
    pub card_side: String,
    /// Client used a hint (gloss popover, audio replay, answer reveal) on this card.
    /// Only ever lowers the stored rating (see `rating_policy`). Optional; default false.
    #[serde(default)]
    pub hint_used: bool,
    /// `auto` cards: the presented type the client rendered (validated server-side).
    #[serde(default)]
    pub presented_type: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReviewResponse {
    pub success: bool,
    pub message: String,
    pub card_side: String,
    #[schema(value_type = String, format = DateTime)]
    pub next_review: Option<DateTime<Utc>>,
    /// When set (fill-in endpoint), true if the answer was correct (similarity rating >= 3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer_correct: Option<bool>,
    /// Similarity-based suggested FSRS rating (1–4) from grade-only Check; omit after commit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_rating: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FlashcardListQuery {
    pub collection_id: i32,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<FlashcardStatus>,
    pub due: Option<bool>, // if true, only return cards due for review
    pub flashcard_id: Option<i32>,
    pub level_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FlashcardListResponse {
    pub flashcards: Vec<FlashcardResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub due_count: i64,
    /// Level due queue only: nothing in the level was due and every card had been reviewed,
    /// so the level was marked complete (by schedule) and the next level unlocked.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub level_completed_by_schedule: bool,
    /// With `level_completed_by_schedule`: the next unlocked, incomplete level, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_level_id: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFlashcardPositionRequest {
    pub position: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportFromCollectionRequest {
    pub collection_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportFromCollectionResponse {
    pub imported_count: i32,
    pub skipped_count: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DirectAnswerRequest {
    pub flashcard_id: i32,
    pub card_side: String,
    pub answer: String,
    /// Client used a hint (gloss popover, audio replay, answer reveal) on this card.
    /// Only ever lowers the stored rating (see `rating_policy`). Optional; default false.
    #[serde(default)]
    pub hint_used: bool,
    /// `auto` cards: the presented type the client rendered (validated server-side).
    #[serde(default)]
    pub presented_type: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FillInAnswerRequest {
    pub flashcard_id: i32,
    pub card_side: String,
    pub answer: String,
    /// When set, commit FSRS with this rating (1–4). When omitted, grade only (Duo Check).
    #[serde(default)]
    pub rating: Option<u32>,
    /// Client used a hint (gloss popover, audio replay, answer reveal) on this card.
    /// Only ever lowers the stored rating (see `rating_policy`). Optional; default false.
    #[serde(default)]
    pub hint_used: bool,
    /// `auto` cards: the presented type the client rendered (validated server-side).
    #[serde(default)]
    pub presented_type: Option<String>,
    /// Word-bank: ordered chip texts the learner selected. When present the server grades
    /// token-by-token against the expected sequence it derives from the card (exact match
    /// against any `;` alternative), independent of `answer` text / `presented_type`.
    #[serde(default)]
    pub answer_tokens: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SubmitQuizAnswerDto {
    pub flashcard_id: i32,
    pub selected_answer_text: String,
    pub card_side: String, // "direct" or "reverse" to match FSRS progress side
    pub presented_options: Vec<String>, // For logging what was shown
    /// When set, commit FSRS with this rating (1–4). When omitted, grade only (Duo Check).
    #[serde(default)]
    pub rating: Option<u32>,
    /// Client used a hint (gloss popover, audio replay, answer reveal) on this card.
    /// Only ever lowers the stored rating (see `rating_policy`). Optional; default false.
    #[serde(default)]
    pub hint_used: bool,
    /// `auto` cards: the presented type the client rendered (validated server-side).
    #[serde(default)]
    pub presented_type: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct Flashcard {
    pub id: i32,
    pub collection_id: i32,
    pub item_id: i32,
    pub definition_id: Option<i32>,
    pub word: Option<String>,
    pub definition: Option<String>,
    pub free_content_front: Option<String>,
    pub free_content_back: Option<String>,
    pub has_front_image: bool,
    pub has_back_image: bool,
    pub notes: Option<String>,
    pub position: i32,
    pub direction: FlashcardDirection,
    pub definition_language_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound_url: Option<String>,
    pub has_custom_sound: bool,
    pub canonical_form: Option<String>,
    #[serde(default = "default_use_canonical_comparison")]
    pub use_canonical_comparison: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    // Fields for Quiz type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question_text: Option<String>, // Populated by service if it's a quiz
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiz_options: Option<Vec<String>>, // Populated by service if it's a quiz
    /// Word-bank / listen-wordbank: shuffled chips (server-normalized text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wordbank_chips: Option<Vec<crate::utils::wordbank::WordbankChip>>,
    /// Target phrase for chip assemble (same normalization as chip text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wordbank_expected_normalized: Option<String>,
    /// `auto` cards: the concrete challenge type to present for this review. Clients render
    /// `presented_type ?? direction` and send it back with the answer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presented_type: Option<FlashcardDirection>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DirectAnswerResponse {
    pub correct: bool,
    pub expected: String,
    pub message: String,
    #[schema(value_type = String, format = DateTime)]
    pub next_review: Option<DateTime<Utc>>,
    pub is_free_content: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuizAnswerResultDto {
    pub correct: bool,
    pub message: String,
    #[schema(value_type = String, format = DateTime)]
    pub next_review: Option<DateTime<Utc>>,
    /// Suggested FSRS rating from grade-only Check (4 correct / 1 wrong).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_rating: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DailyProgress {
    #[schema(value_type = String, format = Date)]
    pub date: DateTime<Utc>,
    pub points: i32,
    pub reviews_count: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StreakResponse {
    pub current_streak: i32,
    pub longest_streak: i32,
    pub daily_progress: Vec<DailyProgress>,
    pub total_points: i32,
}

// levels:

#[derive(Debug, Serialize, ToSchema)]
#[schema(value_type = String, format = DateTime)]
pub struct ChronoDateTime(pub DateTime<Utc>);

impl<'a> FromSql<'a> for ChronoDateTime {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        DateTime::<Utc>::from_sql(ty, raw).map(ChronoDateTime)
    }

    fn accepts(ty: &Type) -> bool {
        <DateTime<Utc> as FromSql>::accepts(ty)
    }
}

impl From<DateTime<Utc>> for ChronoDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        ChronoDateTime(dt)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateLevelRequest {
    pub name: String,
    pub description: Option<String>,
    pub min_cards: Option<i32>,
    pub min_success_rate: Option<f32>,
    pub position: Option<i32>,
    pub prerequisite_ids: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateLevelRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub min_cards: Option<i32>,
    pub min_success_rate: Option<f32>,
    pub position: Option<i32>,
    pub prerequisite_ids: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LevelResponse {
    pub level_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub min_cards: i32,
    pub min_success_rate: f32,
    pub position: i32,
    pub prerequisites: Vec<PrerequisiteLevel>,
    pub progress: Option<LevelProgress>,
    pub card_count: i32,
    pub is_locked: bool,
    pub is_started: bool,
    /// Memory strength 0–100 of the learner's reviewed (non-information) cards in this level;
    /// `None` for guests or before the first review. See `service::level_strengths`.
    pub strength: Option<i32>,
    /// Reviewed non-information cards whose `next_review_at` has passed.
    pub due_count: i32,
    /// Level should be practised again ("watering"): cards are due or strength fell below 70.
    pub needs_watering: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: ChronoDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PrerequisiteLevel {
    pub level_id: i32,
    pub name: String,
    pub is_completed: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LevelProgress {
    pub cards_completed: i32,
    pub correct_answers: i32,
    pub total_answers: i32,
    pub success_rate: f32,
    pub is_unlocked: bool,
    pub is_completed: bool,
    pub unlocked_at: Option<ChronoDateTime>,
    pub completed_at: Option<ChronoDateTime>,
    pub last_activity_at: ChronoDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddCardsRequest {
    pub flashcard_ids: Vec<i32>,
    pub start_position: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LevelCardResponse {
    pub flashcard_id: i32,
    pub position: i32,
    pub direction: FlashcardDirection,
    pub word: Option<String>,
    pub definition: Option<String>,
    pub free_content_front: Option<String>,
    pub free_content_back: Option<String>,
    pub has_front_image: bool,
    pub has_back_image: bool,
    pub has_sound: bool,
    /// Custom: /api/collections/{id}/items/{item_id}/sound; fallback not populated for level cards yet.
    pub sound_url: Option<String>,
    pub collection_id: i32,
    pub item_id: i32,
    pub definition_id: Option<i32>,
    pub valsi_id: Option<i32>,
    pub ci_notes: Option<String>,
    pub canonical_form: Option<String>,
    pub progress: Option<LevelCardProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wordbank_chips: Option<Vec<crate::utils::wordbank::WordbankChip>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wordbank_expected_normalized: Option<String>,
    /// `auto` cards: presented type for anonymous level study.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presented_type: Option<FlashcardDirection>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LevelCardProgress {
    pub correct_answers: i32,
    pub total_attempts: i32,
    pub success_rate: f32,
    pub last_reviewed_at: Option<ChronoDateTime>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LevelListResponse {
    pub levels: Vec<LevelResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LevelCardListResponse {
    pub cards: Vec<LevelCardResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuizFlashcardQuestionDto {
    pub flashcard_id: i32,
    pub question_text: String,
    pub answer_options: Vec<String>,
}

/// Single level progress item for merging anonymous progress into user account.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MergeProgressLevelItem {
    pub level_id: i32,
    pub cards_completed: i32,
    pub correct_answers: i32,
    pub total_answers: i32,
}

/// Request body for merging anonymous (localStorage) progress into the authenticated user.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MergeProgressRequest {
    pub collection_id: i32,
    pub level_progress: Vec<MergeProgressLevelItem>,
}

/// Response after merging progress.
#[derive(Debug, Serialize, ToSchema)]
pub struct MergeProgressResponse {
    pub merged_levels: i64,
}

/// Claim a guest study account's progress into the authenticated user, then delete the guest.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ClaimGuestProgressRequest {
    /// Access token of the guest session created via `POST /auth/guest`.
    pub guest_access_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClaimGuestProgressResponse {
    pub guest_user_id: i32,
    pub moved_card_rows: i64,
    pub moved_level_rows: i64,
}

/// Result of applying the challenge-mix assigner to a level or collection.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApplyChallengeMixResponse {
    pub updated_count: i32,
    pub quiz_seeded_count: i32,
}

/// Per-course (collection) study settings.
#[derive(Debug, Serialize, ToSchema)]
pub struct CourseSettingsResponse {
    /// When true, adding cards to a level re-applies the challenge mix to that level.
    pub auto_challenge_mix: bool,
    /// "off" | "fixed" (challenge mix of fixed types) | "auto" (cards mix types as you learn).
    pub challenge_mode: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCourseSettingsRequest {
    /// Legacy toggle: true = "fixed", false = "off" (ignored when `challenge_mode` is set).
    pub auto_challenge_mix: Option<bool>,
    pub challenge_mode: Option<String>,
}

/// Level membership of one collection item (via its flashcard).
#[derive(Debug, Serialize, ToSchema)]
pub struct ItemLevelEntry {
    pub item_id: i32,
    pub flashcard_id: i32,
    pub level_id: i32,
    pub level_name: String,
    pub level_position: i32,
}

/// One level's full, ordered card list in a course-order save.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LevelOrderEntry {
    pub level_id: i32,
    /// Flashcard ids in their new order (position = index). Replaces the level's list.
    pub flashcard_ids: Vec<i32>,
}

/// Batched course-order save: replace the card lists of the touched levels and,
/// optionally, reorder all levels. Applied in one transaction.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SaveCourseOrderRequest {
    /// Only the levels whose card list changed (source and target of a move).
    #[serde(default)]
    pub levels: Vec<LevelOrderEntry>,
    /// Every level id of the collection in the new order, when headings were reordered.
    pub level_order: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SaveCourseOrderResponse {
    pub updated_levels: usize,
    pub reordered_levels: bool,
}
