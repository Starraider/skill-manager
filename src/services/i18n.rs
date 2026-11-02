use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "de")]
    Deutsch,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Deutsch => "de",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Deutsch => "Deutsch",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code.trim().to_lowercase().as_str() {
            "de" | "deutsch" | "german" => Language::Deutsch,
            _ => Language::English,
        }
    }
}

const EN_JSON: &str = include_str!("../../locales/en.json");
const DE_JSON: &str = include_str!("../../locales/de.json");

static EN_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(EN_JSON).expect("Invalid en.json language file")
});

static DE_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(DE_JSON).expect("Invalid de.json language file")
});

/// Translates a given key into the requested language, falling back to English
/// and then to the raw key if no translation is found.
pub fn t(lang: Language, key: &str) -> String {
    let map = match lang {
        Language::English => &*EN_MAP,
        Language::Deutsch => &*DE_MAP,
    };

    if let Some(val) = map.get(key) {
        val.clone()
    } else if let Some(val) = EN_MAP.get(key) {
        val.clone()
    } else {
        key.to_string()
    }
}

/// Translates a given key and substitutes `{placeholder}` tokens with provided values.
pub fn tr(lang: Language, key: &str, args: &[(&str, &str)]) -> String {
    let mut text = t(lang, key);
    for (placeholder, value) in args {
        let token = format!("{{{}}}", placeholder);
        text = text.replace(&token, value);
    }
    text
}

/// Minimum width constraint for the left pane to prevent button truncation.
pub fn min_left_pane_width(lang: Language) -> f64 {
    match lang {
        Language::Deutsch => 450.0,
        Language::English => 340.0,
    }
}

/// Default recommended width for the left pane for a given language.
pub fn default_left_pane_width(lang: Language) -> f64 {
    match lang {
        Language::Deutsch => 450.0,
        Language::English => 350.0,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_json_parity() {
        let en_keys: Vec<_> = EN_MAP.keys().collect();
        let de_keys: Vec<_> = DE_MAP.keys().collect();

        for key in &en_keys {
            assert!(
                DE_MAP.contains_key(*key),
                "Key '{}' found in en.json but missing in de.json",
                key
            );
        }

        for key in &de_keys {
            assert!(
                EN_MAP.contains_key(*key),
                "Key '{}' found in de.json but missing in en.json",
                key
            );
        }
    }

    #[test]
    fn test_translations_and_placeholders() {
        assert_eq!(t(Language::English, "tabs.sources"), "1. Sources");
        assert_eq!(t(Language::Deutsch, "tabs.sources"), "1. Quellen");

        assert_eq!(
            tr(
                Language::English,
                "sources.skills_heading",
                &[("name", "TestRepo"), ("count", "3")]
            ),
            "Skills in TestRepo (3)"
        );
        assert_eq!(
            tr(
                Language::Deutsch,
                "sources.skills_heading",
                &[("name", "TestRepo"), ("count", "3")]
            ),
            "Skills in TestRepo (3)"
        );
    }
}

