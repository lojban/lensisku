//! Canonical form generation using camxes-rs 1.0 native Rust semantic engine.
//!
//! This module replaces the previous tersmu.wasm WASM-based implementation with
//! a pure Rust implementation using camxes-rs 1.0's integrated semantic analysis.

use camxes_rs::eval_show::eval_text_to_outputs_with_options;
use camxes_rs::parse_lojban::parse_text;
use std::sync::OnceLock;

/// Marker to ensure parser initialization happens once.
static PARSER_INITIALIZED: OnceLock<()> = OnceLock::new();

/// Parse Lojban text and extract its canonical form.
///
/// Returns `Some(canonical)` if the text parses successfully and has a canonical form,
/// or `None` if parsing fails or the text is empty/invalid.
///
/// This function uses camxes-rs 1.0's native Rust semantic engine, which is much
/// faster than the previous WASM-based tersmu implementation.
pub fn get_canonical_form(text: &str) -> Option<String> {
    // Ensure parser is initialized (no-op after first call)
    PARSER_INITIALIZED.get_or_init(|| ());

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Add the %%%END%%% marker that the parser expects
    let with_end = format!("{} %%%END%%%", trimmed);

    match parse_text(&with_end) {
        Ok(parsed) => {
            // eval_text_to_outputs_with_options returns (logical, canonical, graph)
            let (_, canonical, _) = eval_text_to_outputs_with_options(&parsed, true);

            if canonical.is_empty() {
                None
            } else {
                Some(canonical)
            }
        }
        Err(_) => None,
    }
}

/// Parse a batch of Lojban strings and extract their canonical forms.
///
/// This is more efficient than calling `get_canonical_form` repeatedly because
/// the parser initialization overhead is amortized across all inputs.
///
/// With camxes-rs 1.0's native Rust implementation, this is significantly faster
/// than the previous WASM-based approach which required expensive Haskell RTS
/// initialization for each parser instance.
///
/// # Arguments
///
/// * `texts` - A slice of optional strings to parse. `None` values are passed through as `None`.
///
/// # Returns
///
/// A vector of the same length as `texts`, where each element is:
/// - `Some(canonical)` if the corresponding input parsed successfully
/// - `None` if the input was `None`, empty, or failed to parse
///
/// # Note
///
/// This function is blocking and should be called from `tokio::task::spawn_blocking`
/// when used in async contexts.
pub fn get_canonical_forms_batch(texts: &[Option<String>]) -> Vec<Option<String>> {
    if texts.is_empty() {
        return Vec::new();
    }

    // Ensure parser is initialized
    PARSER_INITIALIZED.get_or_init(|| ());

    texts
        .iter()
        .map(|opt| opt.as_deref().and_then(get_canonical_form))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_canonical_form_simple() {
        let result = get_canonical_form("mi prami do");
        assert!(result.is_some(), "Should parse simple Lojban sentence");
    }

    #[test]
    fn test_get_canonical_form_empty() {
        assert_eq!(get_canonical_form(""), None);
        assert_eq!(get_canonical_form("   "), None);
    }

    #[test]
    fn test_get_canonical_forms_batch() {
        let inputs = vec![
            Some("mi".to_string()),
            None,
            Some("do".to_string()),
            Some("".to_string()),
        ];

        let results = get_canonical_forms_batch(&inputs);
        assert_eq!(results.len(), 4);
        assert!(results[0].is_some());
        assert_eq!(results[1], None);
        assert!(results[2].is_some());
        assert_eq!(results[3], None);
    }

    #[test]
    fn test_get_canonical_forms_batch_empty() {
        let results = get_canonical_forms_batch(&[]);
        assert_eq!(results.len(), 0);
    }
}
