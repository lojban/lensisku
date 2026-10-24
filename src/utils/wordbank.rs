//! Word-bank chip tokenization (Duolingo-style assemble challenges).
//!
//! Lives **only on the backend** so the SPA does not reimplement casing/punct rules.
//!
//! - **[`WordbankLang::Translation`]** (English / L1 gloss): strip sentence punct, lowercase,
//!   keep English **`I`** / `I'm` / … (AP title-case does not treat `i` as a stopword).
//! - **[`WordbankLang::Lojban`]**: whitespace-split only — do **not** rewrite Lojban orthography.

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Which side of a Lojban↔translation card the chips assemble.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WordbankLang {
    /// Back / definition — apply English chip normalization.
    Translation,
    /// Front / word — space-split only.
    Lojban,
}

/// Prefer the first `;`-separated alternative for chip banks / grading.
pub fn primary_expected_answer(expected: &str) -> &str {
    if !expected.contains(';') {
        return expected.trim();
    }
    expected
        .split(';')
        .map(str::trim)
        .find(|s| !s.is_empty())
        .unwrap_or(expected.trim())
}

/// Normalize curly apostrophe to ASCII for stable English chip text / grading.
fn ascii_apostrophe(s: &str) -> String {
    s.replace('\u{2019}', "'")
}

/// English personal-pronoun forms (AP would capitalize; `i` is not an AP stopword).
/// Exact capital `I` only for the bare pronoun — never invent caps from lowercase `i`.
fn restore_english_i_form(stripped: &str) -> Option<String> {
    if stripped == "I" {
        return Some("I".to_string());
    }
    let lower = ascii_apostrophe(stripped).to_lowercase();
    match lower.as_str() {
        "i'm" => Some("I'm".to_string()),
        "i'll" => Some("I'll".to_string()),
        "i've" => Some("I've".to_string()),
        "i'd" => Some("I'd".to_string()),
        _ => None,
    }
}

/// English/translation token: strip sentence punct; lowercase except English **`I`** / `I'm` / ….
pub fn normalize_translation_token(token: &str) -> String {
    let stripped: String = token
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '\'' || *c == '\u{2019}' || *c == '-')
        .collect();
    if stripped.is_empty() {
        return String::new();
    }
    if let Some(form) = restore_english_i_form(&stripped) {
        return form;
    }
    ascii_apostrophe(&stripped).to_lowercase()
}

/// Lojban token: keep as written (after trim). No casing / punct rewrite.
pub fn normalize_lojban_token(token: &str) -> String {
    token.trim().to_string()
}

/// Strip / case according to [`WordbankLang`].
pub fn normalize_wordbank_token(token: &str, lang: WordbankLang) -> String {
    match lang {
        WordbankLang::Translation => normalize_translation_token(token),
        WordbankLang::Lojban => normalize_lojban_token(token),
    }
}

/// Whitespace-split → tokens for `lang` (empty dropped).
pub fn tokenize_wordbank_phrase(text: &str, lang: WordbankLang) -> Vec<String> {
    primary_expected_answer(text)
        .split_whitespace()
        .map(|t| normalize_wordbank_token(t, lang))
        .filter(|t| !t.is_empty())
        .collect()
}

/// Space-joined tokens (grading / expected phrase for the client).
pub fn normalize_wordbank_phrase(text: &str, lang: WordbankLang) -> String {
    tokenize_wordbank_phrase(text, lang).join(" ")
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WordbankChip {
    pub id: String,
    pub text: String,
}

/// Build a shuffled chip bank: correct tokens + distractors from other phrases (same `lang`).
pub fn build_wordbank_chips(
    expected: &str,
    distractor_sources: &[String],
    lang: WordbankLang,
    max_extra: Option<usize>,
) -> (Vec<WordbankChip>, String) {
    let correct_tokens = tokenize_wordbank_phrase(expected, lang);
    let expected_normalized = correct_tokens.join(" ");
    if correct_tokens.is_empty() {
        return (Vec::new(), expected_normalized);
    }

    let correct_set: std::collections::HashSet<String> =
        correct_tokens.iter().map(|t| t.to_lowercase()).collect();

    let mut extras: Vec<String> = Vec::new();
    for src in distractor_sources {
        for tok in tokenize_wordbank_phrase(src, lang) {
            let key = tok.to_lowercase();
            if correct_set.contains(&key) {
                continue;
            }
            if extras.iter().any(|e| e.to_lowercase() == key) {
                continue;
            }
            extras.push(tok);
        }
    }

    let mut rng = rand::rng();
    extras.shuffle(&mut rng);
    let max_extra = max_extra.unwrap_or_else(|| (correct_tokens.len() + 2).clamp(3, 6));
    let picked: Vec<String> = extras.into_iter().take(max_extra).collect();

    let mut chips: Vec<WordbankChip> = Vec::new();
    let mut i = 0usize;
    for text in &correct_tokens {
        chips.push(WordbankChip {
            id: format!("wb-c-{i}-{text}"),
            text: text.clone(),
        });
        i += 1;
    }
    for text in &picked {
        chips.push(WordbankChip {
            id: format!("wb-d-{i}-{text}"),
            text: text.clone(),
        });
        i += 1;
    }
    chips.shuffle(&mut rng);
    (chips, expected_normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_preserves_english_i_strips_punct() {
        assert_eq!(
            normalize_wordbank_phrase("Hello, I love you!", WordbankLang::Translation),
            "hello I love you"
        );
        assert_eq!(
            normalize_wordbank_token("I,", WordbankLang::Translation),
            "I"
        );
        assert_eq!(
            normalize_wordbank_token("I'm,", WordbankLang::Translation),
            "I'm"
        );
    }

    #[test]
    fn lojban_is_space_split_only() {
        assert_eq!(
            normalize_wordbank_phrase("mi prami do i do prami mi", WordbankLang::Lojban),
            "mi prami do i do prami mi"
        );
        // Must not lowercase or strip Lojban orthography:
        assert_eq!(
            normalize_wordbank_phrase("coi do'u .i mi klama", WordbankLang::Lojban),
            "coi do'u .i mi klama"
        );
        assert_eq!(
            tokenize_wordbank_phrase("  mi   PRAMI  ", WordbankLang::Lojban),
            vec!["mi", "PRAMI"]
        );
    }

    #[test]
    fn translation_does_not_invent_i_caps() {
        assert_eq!(
            normalize_wordbank_phrase("hello i love you", WordbankLang::Translation),
            "hello i love you"
        );
    }
}
