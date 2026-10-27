//! Dialogue cards: parsed view, per-line TTS clips, and per-question answers.
use super::parser::{self, assign_voices, line_voice, Dialogue};
use crate::flashcards::dto::ReviewRequest;
use crate::flashcards::rating_policy::effective_rating;
use crate::utils::wordbank::{build_wordbank_chips, WordbankChip, WordbankLang};
use deadpool_postgres::{GenericClient, Pool};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use utoipa::ToSchema;

pub const TTS_SPEED: f32 = 0.8;

#[derive(Debug)]
pub enum DialogueError {
    NotFound,
    Forbidden,
    BadRequest(String),
    Internal(String),
}

impl std::fmt::Display for DialogueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => f.write_str("not found"),
            Self::Forbidden => f.write_str("access denied"),
            Self::BadRequest(m) | Self::Internal(m) => f.write_str(m),
        }
    }
}

impl From<tokio_postgres::Error> for DialogueError {
    fn from(e: tokio_postgres::Error) -> Self {
        Self::Internal(e.to_string())
    }
}
impl From<deadpool_postgres::PoolError> for DialogueError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        Self::Internal(e.to_string())
    }
}
impl From<crate::flashcards::FlashcardError> for DialogueError {
    fn from(e: crate::flashcards::FlashcardError) -> Self {
        let m = e.to_string();
        if m.contains("not found") || m.contains("access denied") || m.contains("Access denied") {
            Self::Forbidden
        } else {
            Self::Internal(m)
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LineAudioStatus {
    pub index: usize,
    pub voice: String,
    /// A clip exists for this exact text + voice.
    pub has_audio: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BankChips {
    pub question_index: usize,
    #[schema(value_type = Vec<Object>)]
    pub chips: Vec<WordbankChip>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DialogueView {
    pub item_id: i32,
    pub collection_id: i32,
    pub is_owner: bool,
    pub dialogue: Dialogue,
    /// Speaker → voice (saved map, extended for new speakers).
    pub voices: HashMap<String, String>,
    pub lines: Vec<LineAudioStatus>,
    pub bank_chips: Vec<BankChips>,
    /// Free-content back (translation / notes).
    pub notes: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DialoguePreview {
    pub dialogue: Dialogue,
    pub voices: HashMap<String, String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PreviewRequest {
    pub text: String,
    /// Optional item whose saved voice map should be honoured.
    pub item_id: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct GenerateAudioRequest {
    /// Only this line (0-based); default all lines.
    pub line: Option<usize>,
    /// Re-synthesize even when the cached clip matches.
    #[serde(default)]
    pub force: bool,
    /// Speaker → voice overrides to save first (editor voice dropdown).
    #[serde(default)]
    pub voices: HashMap<String, String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DialogueAnswerRequest {
    /// Client-generated id of one pass through the dialogue.
    pub attempt_id: uuid::Uuid,
    /// `None` = no question (finish a dialogue without questions).
    pub question_index: Option<usize>,
    #[serde(default)]
    pub answer: String,
    #[serde(default)]
    pub hint_used: bool,
    /// Last question: commit one FSRS review for the card (lowest question rating).
    #[serde(default)]
    pub finish: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DialogueCheckRequest {
    pub question_index: usize,
    pub answer: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DialogueAnswerResponse {
    pub correct: bool,
    /// Canonical answer (fill/bank) or the right option (choice).
    pub expected: String,
    /// Stored rating for this question (1–4).
    pub rating: u32,
    /// Set on `finish`: the card rating committed to FSRS.
    pub card_rating: Option<u32>,
}

struct ItemRow {
    collection_id: i32,
    owner_id: i32,
    front: String,
    back: String,
    voices: HashMap<String, String>,
}

async fn load_item(
    client: &impl GenericClient,
    item_id: i32,
    user_id: Option<i32>,
) -> Result<ItemRow, DialogueError> {
    let row = client
        .query_opt(
            "SELECT ci.collection_id, c.user_id, c.is_public, ci.free_content_front,
                    ci.free_content_back, ci.dialogue_voices
             FROM collection_items ci JOIN collections c USING (collection_id)
             WHERE ci.item_id = $1",
            &[&item_id],
        )
        .await?
        .ok_or(DialogueError::NotFound)?;
    let owner_id: i32 = row.get(1);
    let is_public: bool = row.get(2);
    if !is_public && user_id != Some(owner_id) {
        return Err(DialogueError::Forbidden);
    }
    let voices: Option<serde_json::Value> = row.get(5);
    let voices = voices
        .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v).ok())
        .unwrap_or_default();
    Ok(ItemRow {
        collection_id: row.get(0),
        owner_id,
        front: row.get::<_, Option<String>>(3).unwrap_or_default(),
        back: row.get::<_, Option<String>>(4).unwrap_or_default(),
        voices,
    })
}

pub fn clip_hash(text: &str, voice: &str) -> String {
    let mut h = Sha256::new();
    h.update(format!("{voice}|{TTS_SPEED}|{}", text.trim()).as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn bank_chips(d: &Dialogue) -> Vec<BankChips> {
    let others: Vec<String> = d.lines.iter().map(|l| l.text.clone()).collect();
    d.questions
        .iter()
        .filter(|q| q.kind == "bank")
        .map(|q| {
            // Author-listed `(chips: …)` win; otherwise distractors come from the dialogue.
            let explicit: Vec<String> = q.options.iter().map(|o| o.text.clone()).collect();
            let (sources, max_extra) = if explicit.is_empty() {
                (others.clone(), 4)
            } else {
                let n = explicit.len();
                (explicit, n)
            };
            let expected = q.answers.first().cloned().unwrap_or_default();
            let (chips, _) =
                build_wordbank_chips(&expected, &sources, WordbankLang::Lojban, Some(max_extra));
            BankChips {
                question_index: q.index,
                chips,
            }
        })
        .collect()
}

async fn build_view(
    client: &impl GenericClient,
    item_id: i32,
    item: ItemRow,
    user_id: Option<i32>,
) -> Result<DialogueView, DialogueError> {
    let dialogue = parser::parse(&item.front);
    let voices = assign_voices(&dialogue, &item.voices);
    let hashes: HashMap<i32, String> = client
        .query(
            "SELECT line_index, text_hash FROM dialogue_line_sounds WHERE item_id = $1",
            &[&item_id],
        )
        .await?
        .iter()
        .map(|r| (r.get(0), r.get(1)))
        .collect();
    let lines = dialogue
        .lines
        .iter()
        .map(|l| {
            let voice = line_voice(l, &voices);
            let has_audio = hashes.get(&(l.index as i32)) == Some(&clip_hash(&l.text, &voice));
            LineAudioStatus {
                index: l.index,
                voice,
                has_audio,
            }
        })
        .collect();
    Ok(DialogueView {
        item_id,
        collection_id: item.collection_id,
        is_owner: user_id == Some(item.owner_id),
        bank_chips: bank_chips(&dialogue),
        dialogue,
        voices,
        lines,
        notes: item.back,
    })
}

pub async fn get_view(
    pool: &Pool,
    item_id: i32,
    user_id: Option<i32>,
) -> Result<DialogueView, DialogueError> {
    let client = pool.get().await?;
    let item = load_item(&client, item_id, user_id).await?;
    build_view(&client, item_id, item, user_id).await
}

pub async fn preview(
    pool: &Pool,
    user_id: i32,
    req: &PreviewRequest,
) -> Result<DialoguePreview, DialogueError> {
    let saved = match req.item_id {
        Some(id) => {
            let client = pool.get().await?;
            load_item(&client, id, Some(user_id)).await?.voices
        }
        None => HashMap::new(),
    };
    let dialogue = parser::parse(&req.text);
    let voices = assign_voices(&dialogue, &saved);
    Ok(DialoguePreview { dialogue, voices })
}

/// Lines needing synthesis: (index, text, voice, hash).
pub fn lines_to_synthesize(
    d: &Dialogue,
    voices: &HashMap<String, String>,
    existing: &HashMap<i32, String>,
    only: Option<usize>,
    force: bool,
) -> Vec<(usize, String, String, String)> {
    d.lines
        .iter()
        .filter(|l| only.is_none_or(|o| o == l.index))
        .filter_map(|l| {
            let voice = line_voice(l, voices);
            let hash = clip_hash(&l.text, &voice);
            let fresh = existing.get(&(l.index as i32)) == Some(&hash);
            (force || !fresh).then(|| (l.index, l.text.clone(), voice, hash))
        })
        .collect()
}

/// Owner only: save the voice map, synthesize missing / changed / forced lines, drop clips of
/// removed lines. `synth` is injected so DB tests run without the TTS model.
pub async fn generate_audio<F>(
    pool: &Pool,
    item_id: i32,
    user_id: i32,
    req: &GenerateAudioRequest,
    synth: F,
) -> Result<DialogueView, DialogueError>
where
    F: Fn(
        String,
        String,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>,
{
    let client = pool.get().await?;
    let mut item = load_item(&client, item_id, Some(user_id)).await?;
    if item.owner_id != user_id {
        return Err(DialogueError::Forbidden);
    }
    let d = parser::parse(&item.front);
    if d.lines.is_empty() {
        return Err(DialogueError::BadRequest("no dialogue lines".into()));
    }
    for (speaker, voice) in &req.voices {
        let v = parser::canonical_voice(voice)
            .ok_or_else(|| DialogueError::BadRequest(format!("unknown voice {voice}")))?;
        item.voices.insert(speaker.clone(), v.to_string());
    }
    let voices = assign_voices(&d, &item.voices);
    let json = serde_json::to_value(&voices).map_err(|e| DialogueError::Internal(e.to_string()))?;
    client
        .execute(
            "UPDATE collection_items SET dialogue_voices = $2 WHERE item_id = $1",
            &[&item_id, &json],
        )
        .await?;
    let n_lines = d.lines.len() as i32;
    client
        .execute(
            "DELETE FROM dialogue_line_sounds WHERE item_id = $1 AND line_index >= $2",
            &[&item_id, &n_lines],
        )
        .await?;
    let existing: HashMap<i32, String> = client
        .query(
            "SELECT line_index, text_hash FROM dialogue_line_sounds WHERE item_id = $1",
            &[&item_id],
        )
        .await?
        .iter()
        .map(|r| (r.get(0), r.get(1)))
        .collect();
    for (idx, text, voice, hash) in lines_to_synthesize(&d, &voices, &existing, req.line, req.force)
    {
        let bytes = synth(text, voice.clone())
            .await
            .map_err(|e| DialogueError::Internal(format!("TTS failed on line {}: {e}", idx + 1)))?;
        let idx = idx as i32;
        client
            .execute(
                "INSERT INTO dialogue_line_sounds (item_id, line_index, text_hash, voice, sound_data, mime_type)
                 VALUES ($1, $2, $3, $4, $5, 'audio/ogg')
                 ON CONFLICT (item_id, line_index) DO UPDATE
                 SET text_hash = EXCLUDED.text_hash, voice = EXCLUDED.voice,
                     sound_data = EXCLUDED.sound_data, created_at = CURRENT_TIMESTAMP",
                &[&item_id, &idx, &hash, &voice, &bytes],
            )
            .await?;
    }
    item.voices = voices;
    build_view(&client, item_id, item, Some(user_id)).await
}

pub async fn line_audio(
    pool: &Pool,
    item_id: i32,
    line: i32,
    user_id: Option<i32>,
) -> Result<(Vec<u8>, String), DialogueError> {
    let client = pool.get().await?;
    load_item(&client, item_id, user_id).await?;
    let row = client
        .query_opt(
            "SELECT sound_data, mime_type FROM dialogue_line_sounds WHERE item_id = $1 AND line_index = $2",
            &[&item_id, &line],
        )
        .await?
        .ok_or(DialogueError::NotFound)?;
    Ok((row.get(0), row.get(1)))
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if matches!(c, '.' | ',' | '!' | '?' | '"') {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// (correct, expected) for one question.
pub fn grade(d: &Dialogue, index: usize, answer: &str) -> Result<(bool, String), DialogueError> {
    let q = d
        .questions
        .get(index)
        .ok_or_else(|| DialogueError::BadRequest("no such question".into()))?;
    let given = normalize(answer);
    Ok(match q.kind.as_str() {
        "choice" => {
            let right = q
                .options
                .iter()
                .find(|o| o.correct)
                .map(|o| o.text.clone())
                .unwrap_or_default();
            (!given.is_empty() && given == normalize(&right), right)
        }
        _ => {
            let ok = !given.is_empty() && q.answers.iter().any(|a| normalize(a) == given);
            (ok, q.answers.first().cloned().unwrap_or_default())
        }
    })
}

/// Grade only (guests): nothing is stored.
pub async fn check(
    pool: &Pool,
    item_id: i32,
    user_id: Option<i32>,
    req: &DialogueCheckRequest,
) -> Result<DialogueAnswerResponse, DialogueError> {
    let client = pool.get().await?;
    let item = load_item(&client, item_id, user_id).await?;
    let (correct, expected) = grade(&parser::parse(&item.front), req.question_index, &req.answer)?;
    Ok(DialogueAnswerResponse {
        correct,
        expected,
        rating: if correct { 3 } else { 1 },
        card_rating: None,
    })
}

/// Question rating: correct 3, wrong 1; a hint costs 1 (never below 1), like fixed cards.
pub fn question_rating(correct: bool, hint_used: bool) -> u32 {
    effective_rating(
        Some("dialogue"),
        None,
        if correct { 3 } else { 1 },
        hint_used,
    )
}

/// Record one question answer (own row, own score). On `finish`, commit one FSRS review for the
/// card with the lowest rating of this attempt (3 when the dialogue has no questions).
pub async fn answer(
    pool: &Pool,
    user_id: i32,
    flashcard_id: i32,
    req: &DialogueAnswerRequest,
) -> Result<DialogueAnswerResponse, DialogueError> {
    let mut client = pool.get().await?;
    let row = client
        .query_opt(
            "SELECT f.item_id, f.direction::text FROM flashcards f WHERE f.id = $1",
            &[&flashcard_id],
        )
        .await?
        .ok_or(DialogueError::NotFound)?;
    let item_id: i32 = row.get(0);
    let direction: String = row.get(1);
    if direction != "dialogue" {
        return Err(DialogueError::BadRequest("not a dialogue card".into()));
    }
    {
        let tx = client.transaction().await?;
        crate::flashcards::verify_study_access(&tx, flashcard_id, user_id).await?;
        tx.commit().await?;
    }
    let item = load_item(&client, item_id, Some(user_id)).await?;
    let d = parser::parse(&item.front);
    let (correct, expected, rating) = match req.question_index {
        Some(qi) => {
            let (correct, expected) = grade(&d, qi, &req.answer)?;
            let rating = question_rating(correct, req.hint_used);
            let qi32 = qi as i32;
            let kind = d.questions[qi].kind.clone();
            let r = rating as i32;
            client
                .execute(
                    "INSERT INTO dialogue_question_answers
                       (user_id, flashcard_id, attempt_id, question_index, question_kind, answer, correct, hint_used, rating)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                     ON CONFLICT (attempt_id, question_index) DO NOTHING",
                    &[&user_id, &flashcard_id, &req.attempt_id, &qi32, &kind, &req.answer, &correct, &req.hint_used, &r],
                )
                .await?;
            (correct, expected, rating)
        }
        // Finish-only call (after the last question) or a dialogue without questions.
        None if req.finish || d.questions.is_empty() => (true, String::new(), 3),
        None => return Err(DialogueError::BadRequest("question_index required".into())),
    };
    let card_rating = if req.finish {
        let min: Option<i32> = client
            .query_one(
                "SELECT MIN(rating) FROM dialogue_question_answers
                 WHERE attempt_id = $1 AND user_id = $2 AND flashcard_id = $3",
                &[&req.attempt_id, &user_id, &flashcard_id],
            )
            .await?
            .get(0);
        let card = min.map(|m| m as u32).unwrap_or(rating);
        crate::flashcards::review_flashcard(
            pool,
            user_id,
            &ReviewRequest {
                flashcard_id,
                rating: card,
                card_side: "direct".into(),
                hint_used: false,
                presented_type: None,
            },
        )
        .await?;
        Some(card)
    } else {
        None
    };
    Ok(DialogueAnswerResponse {
        correct,
        expected,
        rating,
        card_rating,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grading_normalizes_and_accepts_alternatives() {
        let d = parser::parse("M: coi\n---\n? fill: M: lo mi [ckiku|ckiku be mi] cu zvati ma\n? choice: Q  + on the table  - in the car\n");
        assert!(grade(&d, 0, " Ckiku. ").unwrap().0);
        assert!(grade(&d, 0, "ckiku be mi").unwrap().0);
        assert!(!grade(&d, 0, "karce").unwrap().0);
        assert!(!grade(&d, 0, "").unwrap().0);
        assert_eq!(
            grade(&d, 1, "on the table").unwrap(),
            (true, "on the table".into())
        );
        assert!(!grade(&d, 1, "in the car").unwrap().0);
        assert!(grade(&d, 2, "x").is_err());
    }

    #[test]
    fn question_ratings_follow_rating_policy() {
        assert_eq!(question_rating(true, false), 3);
        assert_eq!(question_rating(true, true), 2);
        assert_eq!(question_rating(false, false), 1);
        assert_eq!(question_rating(false, true), 1);
    }

    #[test]
    fn only_changed_lines_are_resynthesized() {
        let d = parser::parse("A(M): coi\nB(F): coi\n");
        let v = assign_voices(&d, &HashMap::new());
        let mut existing = HashMap::new();
        existing.insert(0, clip_hash("coi", "Martin"));
        existing.insert(1, clip_hash("old text", "Victoria"));
        let todo = lines_to_synthesize(&d, &v, &existing, None, false);
        assert_eq!(todo.iter().map(|t| t.0).collect::<Vec<_>>(), vec![1]);
        assert_eq!(
            lines_to_synthesize(&d, &v, &existing, Some(0), true).len(),
            1
        );
        assert!(lines_to_synthesize(&d, &v, &existing, Some(0), false).is_empty());
    }
}
