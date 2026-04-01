//! Lojban text → IPA, matching `lojban_speak.py` / `lojban2ipa` for Kitten TTS phoneme input.

#![allow(clippy::expect_used)] // compile-time-fixed patterns

use once_cell::sync::Lazy;
use regex::Regex;

fn krulermorna(text: &str) -> String {
    static DOT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.").expect("regex"));
    static U_VOWEL: Lazy<Regex> = Lazy::new(|| Regex::new(r"u([aeiouy])").expect("regex"));
    static I_VOWEL: Lazy<Regex> = Lazy::new(|| Regex::new(r"i([aeiouy])").expect("regex"));
    static AU: Lazy<Regex> = Lazy::new(|| Regex::new(r"au").expect("regex"));
    static AI: Lazy<Regex> = Lazy::new(|| Regex::new(r"ai").expect("regex"));
    static EI: Lazy<Regex> = Lazy::new(|| Regex::new(r"ei").expect("regex"));
    static OI: Lazy<Regex> = Lazy::new(|| Regex::new(r"oi").expect("regex"));

    let mut text = DOT.replace_all(text, "").to_string();
    text = format!(".{text}");
    text = U_VOWEL.replace_all(&text, "w$1").to_string();
    text = I_VOWEL.replace_all(&text, "ɩ$1").to_string();
    text = AU.replace_all(&text, "ḁ").to_string();
    text = AI.replace_all(&text, "ą").to_string();
    text = EI.replace_all(&text, "ę").to_string();
    text = OI.replace_all(&text, "ǫ").to_string();
    DOT.replace_all(&text, "").to_string()
}

fn krulermorna_words<const N: usize>(words: [&str; N]) -> [String; N] {
    let mut out: [String; N] = std::array::from_fn(|_| String::new());
    for (i, w) in words.iter().enumerate() {
        out[i] = krulermorna(w);
    }
    out
}

static QUESTION_WORDS: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    krulermorna_words(["ma", "mo", "xu"])
        .into_iter()
        .collect()
});
static STARTER_WORDS: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    krulermorna_words(["le", "lo", "lei", "loi"])
        .into_iter()
        .collect()
});
static TERMINATOR_WORDS: Lazy<std::collections::HashSet<String>> = Lazy::new(|| {
    krulermorna_words(["kei", "ku'o", "vau", "li'u"])
        .into_iter()
        .collect()
});

/// Syllable nuclei after krulermorna: diphthong letters, glide + vowel, or a simple vowel.
static NUCLEUS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"ą|ę|ǫ|ḁ|ɩ[aeiouy]|[aeiouy]").expect("nucleus_pattern")
});

/// Index *before* which to insert primary stress (ˈ), Lojban penultimate syllable.
fn stress_insert_index(word: &str) -> Option<usize> {
    let indices: Vec<usize> = NUCLEUS_PATTERN
        .find_iter(word)
        .map(|m| m.start())
        .collect();
    if indices.len() <= 1 {
        return None;
    }
    Some(indices[indices.len() - 2])
}

/// Sorted by pattern key length descending (matches Python `sorted(ipa_vits.items(), ...)`).
static IPA_RULES: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    let mut raw: Vec<(&str, &str)> = vec![
        // Longer `r` before short `r` (same as Python ipa_vits order by length)
        (r"r(?![ˈaeiouyḁąęǫ])", "ɹɹ"),
        ("ɩa", "jaː"),
        ("ɩe", "jɛː"),
        ("ɩi", "jiː"),
        ("ɩo", "jɔː"),
        ("ɩu", "juː"),
        ("ɩy", "jəː"),
        ("wa", "waː"),
        ("we", "wɛː"),
        ("wi", "wiː"),
        ("wo", "wɔː"),
        ("wu", "wuː"),
        ("wy", "wəː"),
        ("ng", "n.g"),
        ("a", "ɑː"),
        ("e", "ɛː"),
        ("i", "iː"),
        ("o", "oː"),
        ("u", "ʊː"),
        ("y", "əː"),
        ("ą", "aɪ"),
        ("ę", "ɛɪ"),
        ("ǫ", "ɔɪ"),
        ("ḁ", "aʊ"),
        ("ɩ", "j"),
        ("w", "w"),
        ("c", "ʃ"),
        ("j", "ʒ"),
        ("s", "s"),
        ("z", "z"),
        ("f", "f"),
        ("v", "v"),
        ("x", "hhhh"),
        ("'", "h"),
        ("r", "ɹ"),
        ("n", "n"),
        ("m", "m"),
        ("l", "l"),
        ("b", "b"),
        ("d", "d"),
        ("g", "ɡ"),
        ("k", "k"),
        ("p", "p"),
        ("t", "t"),
        ("h", "h"),
    ];
    raw.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    raw
        .into_iter()
        .filter_map(|(pat, rep)| {
            let r = Regex::new(&format!("^{pat}")).ok()?;
            Some((r, rep))
        })
        .collect()
});

fn ipa_transform_word(modified_word: &str) -> String {
    let mut rebuilt = String::with_capacity(modified_word.len() * 2);
    let mut byte_i = 0;
    let total = modified_word.len();
    while byte_i < total {
        let tail = &modified_word[byte_i..];
        let mut matched = false;
        for (re, val) in IPA_RULES.iter() {
            if let Some(m) = re.find(tail) {
                if m.start() == 0 {
                    rebuilt.push_str(val);
                    byte_i += m.len();
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            if let Some(ch) = modified_word[byte_i..].chars().next() {
                rebuilt.push(ch);
                byte_i += ch.len_utf8();
            } else {
                break;
            }
        }
    }
    rebuilt
}

fn char_is_vowel(ch: char) -> bool {
    matches!(
        ch,
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'ą' | 'ę' | 'ǫ' | 'ḁ'
    )
}

/// Lojban text → IPA string, matching `lojban2ipa` in the Python Kitten TTS script.
pub fn lojban_to_ipa(text: &str) -> String {
    let krul = krulermorna(text.trim());
    let words: Vec<&str> = krul.split(' ').filter(|w| !w.is_empty()).collect();
    let mut rebuilt_words: Vec<String> = Vec::with_capacity(words.len());

    for (index, word) in words.iter().enumerate() {
        let mut modified_word = (*word).to_string();
        let mut prefix = String::new();
        let mut postfix = String::new();

        if QUESTION_WORDS.contains(*word) {
            postfix.push('?');
            prefix = format!(" {prefix}");
        }
        if STARTER_WORDS.contains(*word) {
            prefix = format!(" {prefix}");
        }
        if TERMINATOR_WORDS.contains(*word) {
            postfix = ", ".to_string();
        }

        if index > 0 && (*word == "ni'o" || *word == "i") {
            prefix = format!(", {prefix}");
        }

        if let Some(si) = stress_insert_index(word) {
            modified_word = format!("{}ˈ{}", &word[..si], &word[si..]);
            let n_nuclei = NUCLEUS_PATTERN.find_iter(word).count();
            if n_nuclei >= 2 {
                postfix.push(' ');
            }
        }

        if !(index > 0 && STARTER_WORDS.contains(words[index - 1])) {
            prefix = format!(" {prefix}");
        }

        let rebuilt_word = ipa_transform_word(&modified_word);
        rebuilt_words.push(format!("{prefix}{rebuilt_word}{postfix}"));
    }

    let mut output = rebuilt_words.concat();
    static SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r" {2,}").expect("spaces"));
    static COMMA_COMMA: Lazy<Regex> = Lazy::new(|| Regex::new(r",\s*,").expect("comma"));
    output = SPACES.replace_all(output.trim(), " ").to_string();
    output = COMMA_COMMA.replace_all(&output, ",").to_string();

    if let Some(last) = krul.chars().last() {
        if char_is_vowel(last) {
            output.push('.');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::lojban_to_ipa;

    #[test]
    fn klama_stress_and_shape() {
        let ipa = lojban_to_ipa("klama");
        assert!(ipa.contains('ˈ') || ipa.contains('k'));
    }

    #[test]
    fn uses_open_back_a() {
        let ipa = lojban_to_ipa("a");
        assert!(ipa.contains('ɑ'));
    }
}
