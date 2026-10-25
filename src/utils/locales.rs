//! Bundled frontend locale catalogs (`locales/*.json`), parsed once on first use.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde_json::Value;

static BUNDLED: LazyLock<HashMap<&'static str, Value>> = LazyLock::new(|| {
    [
        ("en", include_str!("../../locales/en.json")),
        ("jbo", include_str!("../../locales/jbo.json")),
        ("ja", include_str!("../../locales/ja.json")),
        ("zh", include_str!("../../locales/zh.json")),
    ]
    .into_iter()
    .filter_map(|(lang, src)| match serde_json::from_str(src) {
        Ok(value) => Some((lang, value)),
        Err(e) => {
            log::error!("bundled locale {lang}.json is invalid: {e}");
            None
        }
    })
    .collect()
});

/// The parsed catalog for `lang`, if bundled and valid.
pub fn catalog(lang: &str) -> Option<&'static Value> {
    BUNDLED.get(lang)
}

/// String `key` from `lang`, falling back to English.
pub fn lookup(lang: &str, key: &str) -> Option<&'static str> {
    catalog(lang)
        .and_then(|c| c.get(key))
        .and_then(Value::as_str)
        .or_else(|| catalog("en")?.get(key)?.as_str())
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_bundled_catalogs_parse_and_fall_back_to_english() {
        for lang in ["en", "jbo", "ja", "zh"] {
            assert!(super::catalog(lang).is_some(), "{lang}");
        }
        assert_eq!(
            super::lookup("xx", "collection_default_likes"),
            super::lookup("en", "collection_default_likes")
        );
        assert!(super::lookup("en", "collection_default_likes").is_some());
        assert!(super::lookup("en", "no_such_key_zz").is_none());
    }
}
