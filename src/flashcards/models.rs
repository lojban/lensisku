use bytes::BytesMut;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use tokio_postgres::types::{FromSql, IsNull, ToSql, Type};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserFlashcardProgress {
    pub id: i32,
    pub user_id: i32,
    pub flashcard_id: i32,
    pub card_side: String,
    pub ease_factor: f32,
    pub stability: i32,  // in minutes
    pub difficulty: i32, // in minutes
    pub interval: i32,   // in minutes
    pub review_count: i32,
    #[schema(value_type = String, format = DateTime)]
    pub last_reviewed_at: Option<DateTime<Utc>>,
    #[schema(value_type = String, format = DateTime)]
    pub next_review_at: Option<DateTime<Utc>>,
    pub status: FlashcardStatus,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FlashcardStatus {
    New,
    Learning,
    Review,
    Graduated,
}

impl FromStr for FlashcardStatus {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "new" => Ok(FlashcardStatus::New),
            "learning" => Ok(FlashcardStatus::Learning),
            "review" => Ok(FlashcardStatus::Review),
            "graduated" => Ok(FlashcardStatus::Graduated),
            _ => Err("Invalid flashcard status".into()),
        }
    }
}

impl<'a> FromSql<'a> for FlashcardStatus {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let s = String::from_utf8(raw.to_vec())?;
        FlashcardStatus::from_str(&s)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar" || ty.name() == "flashcard_status"
    }
}

impl ToSql for FlashcardStatus {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let s = match self {
            FlashcardStatus::New => "new",
            FlashcardStatus::Learning => "learning",
            FlashcardStatus::Review => "review",
            FlashcardStatus::Graduated => "graduated",
        };
        out.extend_from_slice(s.as_bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar" || ty.name() == "flashcard_status"
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct FlashcardQuizOptions {
    pub quiz_option_id: i32,
    pub flashcard_id: i32,
    pub correct_answer_text: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserQuizAnswerHistory {
    pub history_id: i32,
    pub user_id: i32,
    pub flashcard_id: i32,
    pub selected_option_text: String,
    pub is_correct_selection: bool,
    pub presented_options: serde_json::Value,
    #[schema(value_type = String, format = DateTime)]
    pub answered_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FlashcardDirection {
    Direct,
    Reverse,
    Both,
    FillIn,
    FillInReverse,
    FillInBoth,
    JustInformation,
    QuizDirect,
    QuizReverse,
    QuizBoth,
    QuizImageDirect,
    QuizImageReverse,
    QuizImageBoth,
}

impl<'a> FromSql<'a> for FlashcardDirection {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let s = String::from_utf8(raw.to_vec())?;
        match s.to_lowercase().as_str() {
            "direct" => Ok(FlashcardDirection::Direct),
            "reverse" => Ok(FlashcardDirection::Reverse),
            "both" => Ok(FlashcardDirection::Both),
            "fillin" => Ok(FlashcardDirection::FillIn),
            "fillin_reverse" => Ok(FlashcardDirection::FillInReverse),
            "fillin_both" => Ok(FlashcardDirection::FillInBoth),
            "just_information" => Ok(FlashcardDirection::JustInformation),
            "quiz_direct" => Ok(FlashcardDirection::QuizDirect),
            "quiz_reverse" => Ok(FlashcardDirection::QuizReverse),
            "quiz_both" => Ok(FlashcardDirection::QuizBoth),
            "quiz_image_direct" => Ok(FlashcardDirection::QuizImageDirect),
            "quiz_image_reverse" => Ok(FlashcardDirection::QuizImageReverse),
            "quiz_image_both" => Ok(FlashcardDirection::QuizImageBoth),
            _ => Err("Invalid flashcard direction".into()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar" || ty.name() == "flashcard_direction"
    }
}

impl ToSql for FlashcardDirection {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let s = match self {
            FlashcardDirection::Direct => "direct",
            FlashcardDirection::Reverse => "reverse",
            FlashcardDirection::Both => "both",
            FlashcardDirection::FillIn => "fillin",
            FlashcardDirection::FillInReverse => "fillin_reverse",
            FlashcardDirection::FillInBoth => "fillin_both",
            FlashcardDirection::JustInformation => "just_information",
            FlashcardDirection::QuizDirect => "quiz_direct",
            FlashcardDirection::QuizReverse => "quiz_reverse",
            FlashcardDirection::QuizBoth => "quiz_both",
            FlashcardDirection::QuizImageDirect => "quiz_image_direct",
            FlashcardDirection::QuizImageReverse => "quiz_image_reverse",
            FlashcardDirection::QuizImageBoth => "quiz_image_both",
        };
        out.extend_from_slice(s.as_bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "text" || ty.name() == "varchar" || ty.name() == "flashcard_direction"
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}
