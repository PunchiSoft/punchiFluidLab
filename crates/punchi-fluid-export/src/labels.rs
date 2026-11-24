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

impl Labels {
    /// The English set.
    #[must_use]
    pub const fn english() -> Self {
        Self {
            language: LabelLanguage::English,
            owner: "Company / owner",
            project: "Project / installation",
            equipment: "Equipment / system",
            title: "Title",
            number: "Drawing no.",
            revision: "Revision",
            date: "Date",
            prepared: "Drawn by",
            checked: "Checked by",
            approved: "Approved by",
            scale: "Scale",
            norms: "Standards applied",
            sheet: "Sheet",
            parts_heading: "parts list",
            parts_omitted: "… and {count} more parts",
            closures_omitted: "… and {count} more closures",
            tubes_omitted: "… and {count} more hoses",
            tube: "Hose",
            unspecified: "unspecified",
            class_working: "working",
            class_pilot: "pilot",
            class_drain: "drain",
            class_electrical: "electrical",
            port_supply: "supply",
            port_working: "working",
            port_exhaust: "exhaust",
            port_mechanical: "mechanical",
            port_electrical: "electrical",
            tube_note: "length = drawn route, no offsets, fittings or waste",
            under_tenth: "< 0.1 m",
            decimal_separator: '.',
        }
    }

    /// The Spanish set.
    #[must_use]
    pub const fn spanish() -> Self {
        Self {
            language: LabelLanguage::Spanish,
            owner: "Empresa / propietario",
            project: "Proyecto / instalación",
            equipment: "Equipo / sistema",
            title: "Título",
            number: "Nº de plano",
            revision: "Revisión",
            date: "Fecha",
            prepared: "Dibujado",
            checked: "Revisado",
            approved: "Aprobado",
            scale: "Escala",
            norms: "Normas aplicadas",
            sheet: "Hoja",
            parts_heading: "lista de piezas",
            parts_omitted: "… y {count} piezas más",
            closures_omitted: "… y {count} cierres más",
            tubes_omitted: "… y {count} tubos más",
            tube: "Tubo",
            unspecified: "sin especificar",
            class_working: "trabajo",
            class_pilot: "pilotaje",
            class_drain: "drenaje",
            class_electrical: "eléctrico",
            port_supply: "suministro",
            port_working: "trabajo",
            port_exhaust: "escape",
            port_mechanical: "mecánico",
            port_electrical: "eléctrico",
            tube_note: "longitud = recorrido dibujado, sin desvíos, racores ni merma",
            under_tenth: "< 0,1 m",
            decimal_separator: ',',
        }
    }

    /// The omitted note with its count filled in.
    #[must_use]
    pub fn with_count(template: &str, count: usize) -> String {
        template.replace("{count}", &count.to_string())
    }
}
