//! "Auto" cards: pick the presented challenge type for each review.
//!
//! Ladder (one stage per card side, `user_flashcard_progress.auto_stage`):
//! 0 quiz (recognition) → 1 wordbank → 2 listening wordbank → 3 reverse (wordbank reverse
//! for multi-word Lojban, else a reverse flip card) → 4 typing (fill-in).
//! A pass without hint moves one stage up, a lapse (stored 1) drops two stages.
//! Impossible types are skipped (no quiz options, no audio), the same type is never
//! presented twice in a row when an alternative exists, and the occasional easier type is
//! mixed in via a seeded pick on `hash(user, flashcard, review_count)` (reproducible).

use super::models::FlashcardDirection;

pub const MAX_STAGE: i16 = 4;

#[derive(Debug, Clone)]
pub struct AutoInputs {
    pub stage: i16,
    pub last_type: Option<FlashcardDirection>,
    pub has_quiz: bool,
    pub has_sound: bool,
    /// Lojban front has 2+ words: reverse stage assembles it from a word bank.
    pub multi_token_front: bool,
    pub seed: u64,
}

/// Seed for the variety pick (FNV-1a over the three ids; stable across builds).
pub fn seed(user_id: i32, flashcard_id: i32, review_count: i32) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [user_id, flashcard_id, review_count] {
        for b in v.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

fn type_for_stage(stage: i16, i: &AutoInputs) -> Option<FlashcardDirection> {
    match stage {
        0 if i.has_quiz => Some(FlashcardDirection::QuizDirect),
        1 => Some(FlashcardDirection::Wordbank),
        2 if i.has_sound => Some(FlashcardDirection::ListenWordbank),
        3 if i.multi_token_front => Some(FlashcardDirection::WordbankReverse),
        3 => Some(FlashcardDirection::Reverse),
        4 => Some(FlashcardDirection::FillIn),
        _ => None,
    }
}

/// Stages to try from `start`: itself, then easier ones, then harder ones.
fn search_order(start: i16) -> impl Iterator<Item = i16> {
    let down = (0..=start).rev();
    let up = (start + 1)..=MAX_STAGE;
    down.chain(up)
}

/// The presented type for this review.
pub fn choose(i: &AutoInputs) -> FlashcardDirection {
    let stage = i.stage.clamp(0, MAX_STAGE);
    // One review in four (seeded) revisits the next easier type for variety.
    let target = if stage > 0 && i.seed.is_multiple_of(4) {
        stage - 1
    } else {
        stage
    };
    let mut first = None;
    for s in search_order(target) {
        if let Some(t) = type_for_stage(s, i) {
            if first.is_none() {
                first = Some(t.clone());
            }
            if i.last_type.as_ref() != Some(&t) {
                return t;
            }
        }
    }
    first.unwrap_or(FlashcardDirection::Wordbank)
}

/// Stage after a review with stored rating `stored`.
pub fn next_stage(stage: i16, stored: u32, hint_used: bool) -> i16 {
    if stored <= 1 {
        (stage - 2).max(0)
    } else if hint_used {
        stage
    } else {
        (stage + 1).min(MAX_STAGE)
    }
}

/// Types an auto card can be presented as (validates the client's `presented_type`).
pub fn is_ladder_type(t: &FlashcardDirection) -> bool {
    matches!(
        t,
        FlashcardDirection::QuizDirect
            | FlashcardDirection::Wordbank
            | FlashcardDirection::ListenWordbank
            | FlashcardDirection::WordbankReverse
            | FlashcardDirection::Reverse
            | FlashcardDirection::FillIn
            // Fallback when a word bank cannot be built.
            | FlashcardDirection::Direct
    )
}

/// Production types: a correct answer here is needed before an auto card reaches
/// review / graduated (recognition alone must not graduate a card).
pub fn is_production_type(t: &FlashcardDirection) -> bool {
    matches!(
        t,
        FlashcardDirection::FillIn
            | FlashcardDirection::Reverse
            | FlashcardDirection::WordbankReverse
    )
}

/// Which side the presented type shows first (study UI); scheduling stays on `direct`.
pub fn display_side(t: &FlashcardDirection) -> &'static str {
    match t {
        FlashcardDirection::Reverse | FlashcardDirection::WordbankReverse => "reverse",
        _ => "direct",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use FlashcardDirection as D;

    fn inp(stage: i16, last: Option<D>) -> AutoInputs {
        AutoInputs {
            stage,
            last_type: last,
            has_quiz: true,
            has_sound: true,
            multi_token_front: false,
            seed: 1,
        }
    }

    #[test]
    fn ladder_per_stage() {
        assert_eq!(choose(&inp(0, None)), D::QuizDirect);
        assert_eq!(choose(&inp(1, None)), D::Wordbank);
        assert_eq!(choose(&inp(2, None)), D::ListenWordbank);
        assert_eq!(choose(&inp(3, None)), D::Reverse);
        assert_eq!(choose(&inp(4, None)), D::FillIn);
        let mut i = inp(3, None);
        i.multi_token_front = true;
        assert_eq!(choose(&i), D::WordbankReverse);
    }

    #[test]
    fn never_repeats_when_an_alternative_exists() {
        for stage in 0..=MAX_STAGE {
            for seed in 0..8 {
                let mut i = inp(stage, None);
                i.seed = seed;
                let t = choose(&i);
                i.last_type = Some(t.clone());
                assert_ne!(choose(&i), t, "stage {stage} seed {seed}");
            }
        }
    }

    #[test]
    fn skips_impossible_types() {
        let mut i = inp(0, None);
        i.has_quiz = false;
        assert_eq!(choose(&i), D::Wordbank);
        let mut i = inp(2, None);
        i.has_sound = false;
        assert_eq!(choose(&i), D::Wordbank);
    }

    #[test]
    fn seeded_variety_is_reproducible() {
        let mut i = inp(4, None);
        i.seed = 4; // % 4 == 0 → one stage easier
        assert_eq!(choose(&i), D::Reverse);
        assert_eq!(choose(&i), choose(&i.clone()));
        assert_eq!(seed(1, 2, 3), seed(1, 2, 3));
        assert_ne!(seed(1, 2, 3), seed(1, 2, 4));
    }

    #[test]
    fn stage_moves() {
        assert_eq!(next_stage(0, 2, false), 1);
        assert_eq!(next_stage(4, 3, false), 4);
        assert_eq!(next_stage(3, 1, false), 1);
        assert_eq!(next_stage(1, 1, false), 0);
        assert_eq!(next_stage(2, 2, true), 2);
    }
}
