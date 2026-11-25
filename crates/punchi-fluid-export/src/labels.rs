// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The words a printed page carries, in the language the caller chose.
//!
//! The exporter never translates: it receives a [`Labels`] set from the front end and prints
//! it. The window builds the set from its own catalog language; a headless caller builds the
//! English or the Spanish set directly (ADR 0045). A page is printed in one language, never a
//! mixture.

/// The language the labels follow, so a part name can pick its own field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelLanguage {
    /// English names and labels.
    English,
    /// Spanish names and labels.
    Spanish,
}

/// The words a page prints. `{count}` in the omitted notes is replaced by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Labels {
    /// Language of the labels and of the catalog names.
    pub language: LabelLanguage,
    /// Title-block field labels.
    pub owner: &'static str,
    pub project: &'static str,
    pub equipment: &'static str,
    pub title: &'static str,
    pub number: &'static str,
    pub revision: &'static str,
    pub date: &'static str,
    pub prepared: &'static str,
    pub checked: &'static str,
    pub approved: &'static str,
    pub scale: &'static str,
    pub norms: &'static str,
    pub sheet: &'static str,
    /// The parts-list heading and its cut notes.
    pub parts_heading: &'static str,
    pub parts_omitted: &'static str,
    pub closures_omitted: &'static str,
    pub tubes_omitted: &'static str,
    /// The tube row prefix and an unspecified diameter.
    pub tube: &'static str,
    pub unspecified: &'static str,
    /// Line classes in the tube rows.
    pub class_working: &'static str,
    pub class_pilot: &'static str,
    pub class_drain: &'static str,
    pub class_electrical: &'static str,
    pub class_boundary: &'static str,
    /// Port roles in the closure rows.
    pub port_supply: &'static str,
    pub port_working: &'static str,
    pub port_exhaust: &'static str,
    pub port_mechanical: &'static str,
    pub port_electrical: &'static str,
    /// The note under the tube rows and the cut-list floor.
    pub tube_note: &'static str,
    pub under_tenth: &'static str,
    /// The decimal separator the lengths and the tube sizes use.
    pub decimal_separator: char,
}

impl Default for Labels {
    /// No words: a page a caller prints without labels, never the window's path.
    fn default() -> Self {
        Self {
            language: LabelLanguage::English,
            owner: "",
            project: "",
            equipment: "",
            title: "",
            number: "",
            revision: "",
            date: "",
            prepared: "",
            checked: "",
            approved: "",
            scale: "",
            norms: "",
            sheet: "",
            parts_heading: "",
            parts_omitted: "",
            closures_omitted: "",
            tubes_omitted: "",
            tube: "",
            unspecified: "",
            class_working: "",
            class_pilot: "",
            class_drain: "",
            class_electrical: "",
            class_boundary: "",
            port_supply: "",
            port_working: "",
            port_exhaust: "",
            port_mechanical: "",
            port_electrical: "",
            tube_note: "",
            under_tenth: "",
            decimal_separator: '.',
        }
    }
}

impl Labels {
    /// The omitted note with its count filled in.
    #[must_use]
    pub fn with_count(template: &str, count: usize) -> String {
        template.replace("{count}", &count.to_string())
    }
}
