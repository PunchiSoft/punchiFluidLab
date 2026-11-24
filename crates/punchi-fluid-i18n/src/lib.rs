// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The language and its catalogs: English is the source and the default, Spanish is the
//! translation ([ADR 0045](../../../docs/adr/0045-the-interface-speaks-english-through-embedded-toml-catalogs.md)).
//!
//! The catalogs are embedded with `include_str!`, parsed once and never read from disk at
//! runtime. A key is a stable dotted path; a missing Spanish key falls back to English, and a
//! missing English key falls back to the key itself so a UI never disappears.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The languages the interface offers. English is the source and the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    /// Every language, in the order the selector offers them.
    pub const ALL: [Self; 2] = [Self::English, Self::Spanish];

    /// The code the settings file stores.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
        }
    }

    /// The language of a stored code, or `None` for an unknown one.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::English),
            "es" => Some(Self::Spanish),
            _ => None,
        }
    }

    /// The label that names this language in the current language.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::English => "language.english",
            Self::Spanish => "language.spanish",
        }
    }

    /// The text of `key` in this language; a missing key falls back to English and then to
    /// the key itself.
    #[must_use]
    pub fn text(self, key: &'static str) -> &'static str {
        catalog(self)
            .text(key)
            .or_else(|| catalog(Self::English).text(key))
            .unwrap_or(key)
    }

    /// A number formatted with the language's decimal separator: `.` in English, `,` in
    /// Spanish. The caller keeps the unit and the precision.
    #[must_use]
    pub fn decimal(self, value: f64, decimals: usize) -> String {
        let text = format!("{value:.decimals$}");
        match self {
            Self::English => text,
            Self::Spanish => text.replace('.', ","),
        }
    }

    /// The text of `key` with named `{parameter}` placeholders replaced.
    #[must_use]
    pub fn format(self, key: &'static str, parameters: &[(&str, &str)]) -> String {
        let mut text = self.text(key).to_owned();
        for (name, value) in parameters {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }
}

/// One parsed catalog: a flat map from a dotted key to its text.
#[derive(Debug, Default)]
pub struct Catalog {
    text: BTreeMap<String, String>,
}

impl Catalog {
    /// Parses a TOML catalog. Tables become dotted keys and a value that is not a string is
    /// ignored; a malformed catalog is the caller's error.
    ///
    /// # Errors
    ///
    /// Returns [`toml::de::Error`] when the source is not TOML.
    pub fn parse(source: &str) -> Result<Self, toml::de::Error> {
        let value: toml::Value = toml::from_str(source)?;
        let mut text = BTreeMap::new();
        if let toml::Value::Table(table) = value {
            flatten("", &table, &mut text);
        }
        Ok(Self { text })
    }

    /// The text of `key`, when the catalog has it.
    #[must_use]
    pub fn text(&self, key: &str) -> Option<&str> {
        self.text.get(key).map(String::as_str)
    }

    /// Every key in the catalog, for the completeness test.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.text.keys().map(String::as_str)
    }
}

fn flatten(prefix: &str, table: &toml::Table, out: &mut BTreeMap<String, String>) {
    for (key, value) in table {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match value {
            toml::Value::String(text) => {
                out.insert(path, text.clone());
            }
            toml::Value::Table(table) => flatten(&path, table, out),
            _ => {}
        }
    }
}

fn catalog(language: Language) -> &'static Catalog {
    static ENGLISH: OnceLock<Catalog> = OnceLock::new();
    static SPANISH: OnceLock<Catalog> = OnceLock::new();
    match language {
        Language::English => ENGLISH.get_or_init(|| {
            Catalog::parse(include_str!("../../../i18n/en.toml")).unwrap_or_default()
        }),
        Language::Spanish => SPANISH.get_or_init(|| {
            Catalog::parse(include_str!("../../../i18n/es.toml")).unwrap_or_default()
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{Catalog, Language};

    #[test]
    fn the_embedded_catalogs_parse() {
        Catalog::parse(include_str!("../../../i18n/en.toml")).expect("the English catalog");
        Catalog::parse(include_str!("../../../i18n/es.toml")).expect("the Spanish catalog");
    }

    #[test]
    fn the_catalogs_have_the_same_keys() {
        let english = Catalog::parse(include_str!("../../../i18n/en.toml")).expect("English");
        let spanish = Catalog::parse(include_str!("../../../i18n/es.toml")).expect("Spanish");
        let mut english_keys: Vec<&str> = english.keys().collect();
        let mut spanish_keys: Vec<&str> = spanish.keys().collect();
        english_keys.sort_unstable();
        spanish_keys.sort_unstable();
        assert_eq!(english_keys, spanish_keys);
    }

    #[test]
    fn a_key_is_translated_and_falls_back_to_english() {
        assert_eq!(Language::English.text("menu.move"), "Move");
        assert_eq!(Language::Spanish.text("menu.move"), "Mover");
        assert_eq!(Language::Spanish.text("does.not.exist"), "does.not.exist");
    }

    #[test]
    fn a_number_uses_the_language_decimal_separator() {
        assert_eq!(Language::English.decimal(4.5, 2), "4.50");
        assert_eq!(Language::Spanish.decimal(4.5, 2), "4,50");
        assert_eq!(Language::English.decimal(1200.0, 0), "1200");
        assert_eq!(Language::Spanish.decimal(0.25, 1), "0,2");
    }

    #[test]
    fn a_parameter_is_replaced() {
        assert_eq!(
            Language::English.format("notice.deleted", &[("count", "3")]),
            "deleted 3"
        );
        assert_eq!(
            Language::Spanish.format("notice.deleted", &[("count", "3")]),
            "borradas 3"
        );
    }

    #[test]
    fn the_language_code_round_trips() {
        for language in Language::ALL {
            assert_eq!(Language::from_code(language.code()), Some(language));
        }
        assert_eq!(Language::from_code("fr"), None);
    }
}
