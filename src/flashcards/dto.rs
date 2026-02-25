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
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        match (&self.definition_id, &self.free_content_front, &self.free_content_back) {
            (Some(_), None, None) => Ok(()),
            (None, Some(front), Some(back)) if !front.trim().is_empty() && !back.trim().is_empty() => {
                Ok(())
            }
            _ => Err("Must provide either definition_id or both free_content_front and free_content_back"
                .into()),
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
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReviewResponse {
    pub success: bool,
    pub message: String,
    pub card_side: String,
    #[schema(value_type = String, format = DateTime)]
    pub next_review: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FlashcardListQuery {
    pub collection_id: i32,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<FlashcardStatus>,
    pub due: Option<bool>, // if true, only return cards due for review
    pub flashcard_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FlashcardListResponse {
    pub flashcards: Vec<FlashcardResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub due_count: i64,
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
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FillInAnswerRequest {
    pub flashcard_id: i32,
    pub card_side: String,
    pub answer: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SubmitQuizAnswerDto {
    pub flashcard_id: i32,
    pub selected_answer_text: String,
    pub card_side: String, // "direct" or "reverse" to match FSRS progress side
    pub presented_options: Vec<String>, // For logging what was shown
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
    // Add any other fields relevant to the quiz answer result, e.g., correct answer if different
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
