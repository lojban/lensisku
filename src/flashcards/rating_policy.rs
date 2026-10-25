//! Server-side review rating policy (subtractive; see docs/ARCHITECTURE-PRINCIPLES.md).
//!
//! Card types that show the answer's material on screen (quiz options, word-bank
//! chips, listen + word bank) cannot earn the top rating: they lose 1 point.
//! A client-reported hint (`hint_used`) loses 1 more. At most 2 points are
//! subtracted, and the result never drops below 1 ("Again"). The flag can only
//! lower a rating. Self-assessed (direct/reverse/both), typed fill-in and
//! info cards keep the full 1–4 range. Unknown directions count as hinted (−1).

/// Lowest stored rating that counts as a correct answer for level progress (FSRS: only
/// 1 "Again" is a lapse). A correct quiz/word-bank answer submitted as 3 is stored as 2,
/// so level stats (`user_level_progress`, V201) use this, not 3.
pub const PASS_RATING: u32 = 2;
pub const MIN_RATING: u32 = 1;
pub const MAX_RATING: u32 = 4;
const MAX_PENALTY: u32 = 2;

/// Base penalty for a `flashcards.direction` value (as stored, lowercase text).
pub fn type_penalty(direction: Option<&str>) -> u32 {
    match direction.map(str::to_ascii_lowercase).as_deref() {
        Some(
            "direct" | "reverse" | "both" | "fillin" | "fillin_reverse" | "fillin_both"
            | "just_information",
        ) => 0,
        // quiz*, quiz_image*, wordbank*, listen_wordbank*, and anything unknown.
        _ => 1,
    }
}

/// The rating actually stored for a review. `submitted` must already be 1..=4.
pub fn effective_rating(direction: Option<&str>, submitted: u32, hint_used: bool) -> u32 {
    let submitted = submitted.clamp(MIN_RATING, MAX_RATING);
    let penalty = (type_penalty(direction) + u32::from(hint_used)).min(MAX_PENALTY);
    submitted.saturating_sub(penalty).max(MIN_RATING)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_hinted_answer_still_passes() {
        // Lingo submits 3 for a correct answer; quiz/word-bank types lose 1 point.
        for d in ["quiz_direct", "wordbank_reverse", "listen_wordbank"] {
            assert!(effective_rating(Some(d), 3, false) >= PASS_RATING, "{d}");
            assert!(effective_rating(Some(d), 1, false) < PASS_RATING, "{d}");
        }
    }

    const SELF: &[&str] = &[
        "direct",
        "reverse",
        "both",
        "fillin",
        "fillin_reverse",
        "fillin_both",
        "just_information",
    ];
    const HINTED: &[&str] = &[
        "quiz_direct",
        "quiz_reverse",
        "quiz_both",
        "quiz_image_direct",
        "wordbank",
        "wordbank_reverse",
        "wordbank_both",
        "listen_wordbank",
        "listen_wordbank_reverse",
        "listen_wordbank_both",
        "something_new",
    ];

    #[test]
    fn self_assessed_matrix() {
        for d in SELF {
            for r in 1..=4 {
                assert_eq!(effective_rating(Some(d), r, false), r, "{d} {r}");
                assert_eq!(
                    effective_rating(Some(d), r, true),
                    (r - 1).max(1),
                    "{d} {r} hint"
                );
            }
        }
    }

    #[test]
    fn hinted_types_matrix() {
        for d in HINTED {
            for r in 1..=4 {
                assert_eq!(
                    effective_rating(Some(d), r, false),
                    (r - 1).max(1),
                    "{d} {r}"
                );
                assert_eq!(
                    effective_rating(Some(d), r, true),
                    r.saturating_sub(2).max(1),
                    "{d} {r} hint"
                );
            }
        }
    }

    #[test]
    fn unknown_and_missing_are_capped() {
        assert_eq!(effective_rating(None, 4, false), 3);
        assert_eq!(effective_rating(Some("QUIZ_DIRECT"), 4, false), 3);
        assert_eq!(effective_rating(Some("Both"), 4, false), 4);
    }

    #[test]
    fn never_raises_never_below_min_and_max_two_points() {
        for d in SELF.iter().chain(HINTED) {
            for r in 1..=4 {
                for h in [false, true] {
                    let e = effective_rating(Some(d), r, h);
                    assert!(e <= r && e >= MIN_RATING && r - e <= 2);
                }
            }
        }
        // Wrong answers stay at the lowest rating.
        assert_eq!(effective_rating(Some("wordbank"), 1, true), 1);
    }
}
