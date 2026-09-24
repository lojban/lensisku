//! Pure helpers shared by dictionary / wiki / mail search paths.
//! Kept dependency-light so Criterion benches can `#[path]`-include this file.

/// Escape `%`, `_`, and `\` for PostgreSQL `ILIKE … ESCAPE '\'` patterns.
#[inline]
pub fn escape_like(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '%' => out.push_str("\\%"),
            '_' => out.push_str("\\_"),
            _ => out.push(c),
        }
    }
    out
}

/// Lojban orthography: treat ASCII `h` as `'` in search terms.
#[inline]
pub fn lojbanize_search_term(search_term: &str) -> String {
    search_term.replace('h', "'")
}

/// `%term%` ILIKE pattern (caller escapes if needed).
#[inline]
pub fn like_contains_pattern(term: &str) -> String {
    let mut out = String::with_capacity(term.len() + 2);
    out.push('%');
    out.push_str(term);
    out.push('%');
    out
}

/// PostgreSQL word-boundary regex (`\yterm\y`).
#[inline]
pub fn word_boundary_pattern(term: &str) -> String {
    let mut out = String::with_capacity(term.len() + 4);
    out.push_str(r"\y");
    out.push_str(term);
    out.push_str(r"\y");
    out
}

/// Patterns used by dictionary lexical / fast search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionarySearchPatterns {
    pub search_term: String,
    pub like_pattern: String,
    pub word_boundary_pattern: String,
    pub lojban_search_term: String,
    pub lojban_like_pattern: String,
    pub lojban_word_boundary_pattern: String,
}

impl DictionarySearchPatterns {
    pub fn from_search_term(search_term: &str) -> Self {
        let lojban_search_term = lojbanize_search_term(search_term);
        Self {
            search_term: search_term.to_string(),
            like_pattern: like_contains_pattern(search_term),
            word_boundary_pattern: word_boundary_pattern(search_term),
            lojban_like_pattern: like_contains_pattern(&lojban_search_term),
            lojban_word_boundary_pattern: word_boundary_pattern(&lojban_search_term),
            lojban_search_term,
        }
    }
}

/// Cheap gate before morphology: single Lojban-letter token, not an English phrase.
#[inline]
pub fn looks_like_possible_lujvo_query(search_term: &str) -> bool {
    let t = search_term.trim();
    if t.len() < 4 || t.contains(char::is_whitespace) {
        return false;
    }
    t.chars()
        .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '\'' | '.' | ','))
}

/// Wiki merge-sort relevance: exact title > title contains > body-only.
#[inline]
pub fn wiki_relevance_score(term_lower: &str, title_lower: &str) -> i32 {
    if title_lower == term_lower {
        3
    } else if title_lower.contains(term_lower) {
        2
    } else {
        1
    }
}

/// Pipe-delimited token membership without allocating a Vec (glosswords).
#[inline]
#[allow(dead_code)] // exercised in unit tests + Criterion benches
pub fn pipe_list_contains(haystack: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() || haystack.is_empty() {
        return false;
    }
    haystack.split('|').any(|part| part == needle_lower)
}

/// Space-delimited token membership without allocating owned Strings (rafsi).
#[inline]
#[allow(dead_code)] // exercised in unit tests + Criterion benches
pub fn space_list_contains(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() || haystack.is_empty() {
        return false;
    }
    haystack.split_whitespace().any(|part| part == needle)
}

/// Build per-word ILIKE patterns for multi-word mail/corpus search.
pub fn mail_word_like_patterns(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| like_contains_pattern(&escape_like(w)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_like_escapes_wildcards() {
        assert_eq!(escape_like(r"a%b_c\d"), r"a\%b\_c\\d");
    }

    #[test]
    fn lojbanize_replaces_h() {
        assert_eq!(lojbanize_search_term("coi_do'u"), "coi_do'u");
        assert_eq!(lojbanize_search_term("mlatu"), "mlatu");
        assert_eq!(lojbanize_search_term("brodahe"), "broda'e");
    }

    #[test]
    fn patterns_cover_h_variant() {
        let p = DictionarySearchPatterns::from_search_term("brodahe");
        assert_eq!(p.lojban_search_term, "broda'e");
        assert_eq!(p.like_pattern, "%brodahe%");
        assert_eq!(p.lojban_like_pattern, "%broda'e%");
        assert_eq!(p.word_boundary_pattern, r"\ybrodahe\y");
    }

    #[test]
    fn lujvo_gate() {
        assert!(looks_like_possible_lujvo_query("brivla"));
        assert!(!looks_like_possible_lujvo_query("hi there"));
        assert!(!looks_like_possible_lujvo_query("ab"));
    }

    #[test]
    fn list_contains() {
        assert!(pipe_list_contains("cat|dog|bird", "dog"));
        assert!(!pipe_list_contains("cat|dog|bird", "do"));
        assert!(space_list_contains("bro bya byu", "bya"));
        assert!(!space_list_contains("bro bya byu", "by"));
    }

    #[test]
    fn wiki_relevance() {
        assert_eq!(wiki_relevance_score("lojban", "lojban"), 3);
        assert_eq!(wiki_relevance_score("loj", "lojban"), 2);
        assert_eq!(wiki_relevance_score("xyz", "lojban"), 1);
    }
}
