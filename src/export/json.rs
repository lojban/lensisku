//! JSON serializers for user-facing exports.
//!
//! `serde_json` leaves U+2028 LINE SEPARATOR and U+2029 PARAGRAPH SEPARATOR
//! unescaped, which is valid JSON but trips JavaScript parsers and editors
//! that treat those characters as line terminators. These helpers first
//! rewrite the two characters with [`crate::utils::normalize_line_terminators`]
//! (U+2028 → `\n`, U+2029 → `\n\n`) and then emit them with the usual JSON
//! `\n` escape, not as a raw newline inside a string. Output is otherwise
//! byte-identical to `serde_json::to_string_pretty` / `to_vec_pretty`.

use serde::Serialize;
use serde_json::ser::{Formatter, PrettyFormatter, Serializer};
use std::io::{self, Write};

struct EscapeLineSeparatorFormatter<F> {
    inner: F,
}

impl<F> EscapeLineSeparatorFormatter<F> {
    fn new(inner: F) -> Self {
        Self { inner }
    }
}

fn write_normalized_fragment<F, W>(inner: &mut F, writer: &mut W, fragment: &str) -> io::Result<()>
where
    F: Formatter,
    W: ?Sized + Write,
{
    // write_string_fragment is only used for pieces that need no escaping.
    // After normalization, LS/PS have become `\n`, so those must go through
    // write_char_escape; the surrounding text can still use write_string_fragment.
    let normalized = crate::utils::normalize_line_terminators(fragment);
    let mut first = true;
    for piece in normalized.split('\n') {
        if !first {
            inner.write_char_escape(writer, serde_json::ser::CharEscape::LineFeed)?;
        }
        first = false;
        if !piece.is_empty() {
            inner.write_string_fragment(writer, piece)?;
        }
    }
    Ok(())
}

macro_rules! forward_formatter {
    ($($method:ident $(, $arg:ident : $ty:ty)* ;)+) => {
        $(
            #[inline]
            fn $method<W>(&mut self, writer: &mut W $(, $arg: $ty)*) -> io::Result<()>
            where
                W: ?Sized + Write,
            {
                self.inner.$method(writer $(, $arg)*)
            }
        )+
    };
}

impl<F: Formatter> Formatter for EscapeLineSeparatorFormatter<F> {
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        write_normalized_fragment(&mut self.inner, writer, fragment)
    }

    forward_formatter! {
        write_null;
        write_bool, value: bool;
        write_i8, value: i8;
        write_i16, value: i16;
        write_i32, value: i32;
        write_i64, value: i64;
        write_i128, value: i128;
        write_u8, value: u8;
        write_u16, value: u16;
        write_u32, value: u32;
        write_u64, value: u64;
        write_u128, value: u128;
        write_f32, value: f32;
        write_f64, value: f64;
        write_number_str, value: &str;
        begin_string;
        end_string;
        write_char_escape, char_escape: serde_json::ser::CharEscape;
        write_byte_array, value: &[u8];
        begin_array;
        end_array;
        begin_array_value, first: bool;
        end_array_value;
        begin_object;
        end_object;
        begin_object_key, first: bool;
        end_object_key;
        begin_object_value;
        end_object_value;
        write_raw_fragment, fragment: &str;
    }
}

/// Pretty-print `value` as JSON, rewriting U+2028/U+2029 as JSON `\n` escapes.
pub fn to_writer_pretty<W, T>(writer: W, value: &T) -> serde_json::Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::with_formatter(
        writer,
        EscapeLineSeparatorFormatter::new(PrettyFormatter::new()),
    );
    value.serialize(&mut ser)
}

/// Pretty-print `value` as JSON bytes, rewriting U+2028/U+2029 as JSON `\n` escapes.
pub fn to_vec_pretty<T>(value: &T) -> serde_json::Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer_pretty(&mut writer, value)?;
    Ok(writer)
}

/// Pretty-print `value` as a JSON string, rewriting U+2028/U+2029 as JSON `\n` escapes.
pub fn to_string_pretty<T>(value: &T) -> serde_json::Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = to_vec_pretty(value)?;
    String::from_utf8(vec).map_err(|error| serde::ser::Error::custom(error.utf8_error()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rewrites_line_and_paragraph_separators_as_json_newlines_and_round_trips() {
        let input = "lead\u{2028}mid\u{2029}trail";
        let pretty = to_string_pretty(input).unwrap();
        let normalized = crate::utils::normalize_line_terminators(input);

        assert!(
            !pretty.contains('\u{2028}'),
            "raw U+2028 must not appear in export JSON"
        );
        assert!(
            !pretty.contains('\u{2029}'),
            "raw U+2029 must not appear in export JSON"
        );
        assert!(
            !pretty.contains("\\u2028") && !pretty.contains("\\u2029"),
            "expected JSON \\n escapes, not \\u2028/\\u2029: {pretty}"
        );
        // Pretty-printing a lone string has no structural newlines, so a raw
        // `\n` here would be an unescaped newline inside the JSON string.
        assert!(
            !pretty.contains('\n'),
            "raw newline must not appear inside a JSON string: {pretty}"
        );
        assert_eq!(pretty, "\"lead\\nmid\\n\\ntrail\"");
        assert_eq!(pretty, serde_json::to_string_pretty(&normalized).unwrap());

        let back: String = serde_json::from_str(&pretty).unwrap();
        assert_eq!(back, normalized);
        assert_eq!(back, "lead\nmid\n\ntrail");
    }

    #[test]
    fn pretty_output_matches_serde_json_when_no_line_separators() {
        let value = json!({
            "word": "coi",
            "notes": "hello\n\t\"world\"\\",
            "score": 1.5,
            "tags": ["a", "b"],
            "empty": null,
            "flag": true
        });
        assert_eq!(
            to_string_pretty(&value).unwrap(),
            serde_json::to_string_pretty(&value).unwrap()
        );
        assert_eq!(
            to_vec_pretty(&value).unwrap(),
            serde_json::to_vec_pretty(&value).unwrap()
        );
    }
}
