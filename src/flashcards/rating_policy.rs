//! Server-side review rating policy (subtractive; see docs/ARCHITECTURE-PRINCIPLES.md).
//!
//! Fixed card types (quiz, word bank, listen, fill-in, self-assessed, ...) keep the
//! submitted rating; only a client-reported hint (`hint_used`) lowers it by 1.
//! `auto` cards lose 1 point when they were *presented* as quiz / word bank / listening
//! (the answer material is on screen); typing, reverse and flip presentations keep it.
//! For auto cards a hint loses 1 more. At most 2 points are subtracted, the result never
//! drops below 1 ("Again"), and the hint flag can only lower a rating. An auto review
//! without a known presented type counts as a penalized (on-screen) presentation.

/// Lowest stored rating that counts as a correct answer for level progress and for auto-card
/// status (FSRS: only 1 "Again" is a lapse). A correct auto quiz/word-bank answer submitted
/// as 3 is stored as 2.
pub const PASS_RATING: u32 = 2;
/// Fixed cards: lowest stored rating that moves card status (new → learning → ...).
pub const FIXED_STATUS_RATING: u32 = 3;
pub const MIN_RATING: u32 = 1;
pub const MAX_RATING: u32 = 4;
const MAX_PENALTY: u32 = 2;

/// Presentation shows the answer material on screen (quiz options / word-bank chips / audio).
pub fn on_screen_type(presented: Option<&str>) -> bool {
    let Some(p) = presented else { return true };
    let p = p.to_ascii_lowercase().replace('_', "");
    p.starts_with("quiz") || p.contains("wordbank")
}

fn is_auto(direction: Option<&str>) -> bool {
    direction.is_some_and(|d| d.eq_ignore_ascii_case("auto"))
}

/// The rating actually stored for a review. `submitted` is clamped to 1..=4.
/// `presented` is the concrete type an auto card was shown as (ignored for fixed cards).
pub fn effective_rating(
    direction: Option<&str>,
    presented: Option<&str>,
    submitted: u32,
    hint_used: bool,
) -> u32 {
    let submitted = submitted.clamp(MIN_RATING, MAX_RATING);
    let type_penalty = u32::from(is_auto(direction) && on_screen_type(presented));
    let penalty = (type_penalty + u32::from(hint_used)).min(MAX_PENALTY);
    submitted.saturating_sub(penalty).max(MIN_RATING)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXED: &[&str] = &[
        "direct",
        "reverse",
        "both",
        "fillin",
        "fillin_both",
        "just_information",
        "quiz_direct",
        "quiz_image_direct",
        "wordbank",
        "wordbank_reverse",
        "listen_wordbank",
        "listen_wordbank_both",
    ];

    #[test]
    fn fixed_cards_lose_points_only_for_hints() {
        for d in FIXED {
            for r in 1..=4 {
                assert_eq!(effective_rating(Some(d), None, r, false), r, "{d} {r}");
                assert_eq!(
                    effective_rating(Some(d), None, r, true),
                    (r - 1).max(1),
                    "{d} {r} hint"
                );
            }
        }
        // A correct fixed quiz answer (client 3) is stored as 3 and moves status.
        assert_eq!(effective_rating(Some("quiz_direct"), None, 3, false), 3);
        assert!(effective_rating(Some("quiz_direct"), None, 3, false) >= FIXED_STATUS_RATING);
    }

    #[test]
    fn auto_cards_lose_a_point_for_on_screen_presentations() {
        for p in [
            "quiz_direct",
            "quizdirect",
            "wordbank",
            "wordbankreverse",
            "listen_wordbank",
        ] {
            assert_eq!(effective_rating(Some("auto"), Some(p), 3, false), 2, "{p}");
            assert_eq!(
                effective_rating(Some("auto"), Some(p), 4, true),
                2,
                "{p} hint"
            );
            assert_eq!(
                effective_rating(Some("auto"), Some(p), 3, true),
                1,
                "{p} hint"
            );
        }
        for p in ["fillin", "reverse", "direct"] {
            assert_eq!(effective_rating(Some("auto"), Some(p), 3, false), 3, "{p}");
            assert_eq!(
                effective_rating(Some("auto"), Some(p), 3, true),
                2,
                "{p} hint"
            );
        }
        // Missing presented type: treated as on-screen.
        assert_eq!(effective_rating(Some("AUTO"), None, 4, false), 3);
    }

    #[test]
    fn correct_auto_answers_still_pass() {
        for p in ["quiz_direct", "wordbank", "listen_wordbank", "fillin"] {
            assert!(effective_rating(Some("auto"), Some(p), 3, false) >= PASS_RATING);
            assert!(effective_rating(Some("auto"), Some(p), 1, false) < PASS_RATING);
        }
    }

    #[test]
    fn never_raises_never_below_min_and_max_two_points() {
        for d in FIXED.iter().chain(&["auto", "unknown"]) {
            for p in [None, Some("quiz_direct"), Some("fillin")] {
                for r in 1..=4 {
                    for h in [false, true] {
                        let e = effective_rating(Some(d), p, r, h);
                        assert!(e <= r && e >= MIN_RATING && r - e <= 2);
                    }
                }
            }
        }
        assert_eq!(effective_rating(Some("wordbank"), None, 1, true), 1);
    }
}
