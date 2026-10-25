//! Challenge-mix assigner: diversify study directions within a level.
//!
//! Rules (per ordered card index in the level):
//! - index % 3 == 0 → `quiz_direct` (caller seeds quiz options)
//! - index % 3 == 1 → `wordbank` / `wordbank_reverse` (reverse if Lojban front has 2+ whitespace tokens)
//! - index % 3 == 2 → `listen_wordbank` if sound present, else `both`

/// Inputs for one card when assigning a challenge direction.
#[derive(Debug, Clone)]
pub struct ChallengeMixCard<'a> {
    /// Lojban / front side text (used for wordbank reverse heuristic).
    pub front: &'a str,
    /// True when the item has playable audio (custom or valsi).
    pub has_sound: bool,
}

/// Assign directions for an ordered level of cards (same order as `item_positions` / level items).
pub fn assign_challenge_mix(cards: &[ChallengeMixCard<'_>]) -> Vec<&'static str> {
    cards
        .iter()
        .enumerate()
        .map(|(index, card)| assign_one(index, card))
        .collect()
}

fn assign_one(index: usize, card: &ChallengeMixCard<'_>) -> &'static str {
    match index % 3 {
        0 => "quiz_direct",
        1 => {
            if lojban_front_has_multiple_tokens(card.front) {
                "wordbank_reverse"
            } else {
                "wordbank"
            }
        }
        _ => {
            if card.has_sound {
                "listen_wordbank"
            } else {
                "both"
            }
        }
    }
}

/// True when the Lojban front splits into 2+ whitespace-separated tokens.
fn lojban_front_has_multiple_tokens(front: &str) -> bool {
    front.split_whitespace().count() >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_quiz_wordbank_listen() {
        let cards = [
            ChallengeMixCard {
                front: "mi",
                has_sound: true,
            },
            ChallengeMixCard {
                front: "do",
                has_sound: true,
            },
            ChallengeMixCard {
                front: "ti",
                has_sound: true,
            },
            ChallengeMixCard {
                front: "mi do",
                has_sound: false,
            },
        ];
        assert_eq!(
            assign_challenge_mix(&cards),
            vec!["quiz_direct", "wordbank", "listen_wordbank", "quiz_direct",]
        );
    }

    #[test]
    fn wordbank_reverse_for_multi_token_front() {
        let cards = [ChallengeMixCard {
            front: "mi nelci do",
            has_sound: true,
        }];
        // index 0 → quiz; use index 1 via two-card level
        let cards2 = [
            ChallengeMixCard {
                front: "x",
                has_sound: true,
            },
            ChallengeMixCard {
                front: "mi nelci do",
                has_sound: true,
            },
        ];
        assert_eq!(assign_challenge_mix(&cards2)[1], "wordbank_reverse");
        assert_eq!(assign_challenge_mix(&cards)[0], "quiz_direct");
    }

    #[test]
    fn no_sound_falls_back_to_both() {
        let cards = [
            ChallengeMixCard {
                front: "a",
                has_sound: false,
            },
            ChallengeMixCard {
                front: "b",
                has_sound: false,
            },
            ChallengeMixCard {
                front: "c",
                has_sound: false,
            },
        ];
        assert_eq!(assign_challenge_mix(&cards)[2], "both");
    }
}
