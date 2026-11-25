// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The catalog tile's text: the principal name and the variant that follows the first comma.

use eframe::egui;
use punchi_fluid_catalog::ComponentDefinition;
use punchi_fluid_i18n::Language;

/// A catalog name split into its principal name and its variant, at the first comma.
///
/// The definition files write the variant after a comma (`3/2 valve, spring return`); the
/// split keeps the principal name readable and leaves the full name for the tooltip and
/// the accessibility layer. No rule looks at a component id.
pub(crate) fn split_name(name: &str) -> (&str, Option<&str>) {
    let Some((principal, variant)) = name.split_once(',') else {
        return (name, None);
    };
    let principal = principal.trim();
    let variant = variant.trim();
    if principal.is_empty() || variant.is_empty() {
        (name, None)
    } else {
        (principal, Some(variant))
    }
}

/// The variant line of a tile: one line, elided, never a third line of the name.
pub(crate) fn variant_job(
    variant: &str,
    width: f32,
    font: egui::FontId,
    colour: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob {
        wrap: egui::text::TextWrapping {
            max_width: width,
            max_rows: 1,
            break_anywhere: false,
            overflow_character: Some('…'),
        },
        ..Default::default()
    };
    job.append(
        variant,
        0.0,
        egui::TextFormat {
            font_id: font,
            color: colour,
            ..Default::default()
        },
    );
    job
}

/// The name of a tile: at most `max_rows` lines, elided, so a long name cannot grow over the
/// next tile. A word wider than the tile is elided on one line instead of being broken
/// between two rows (the large-font case). The full name stays in the tooltip and in the
/// accessible name.
pub(crate) fn name_job(
    name: &str,
    width: f32,
    font: egui::FontId,
    colour: egui::Color32,
    max_rows: usize,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob {
        wrap: egui::text::TextWrapping {
            max_width: width,
            max_rows,
            break_anywhere: false,
            overflow_character: Some('…'),
        },
        ..Default::default()
    };
    job.append(
        name,
        0.0,
        egui::TextFormat {
            font_id: font,
            color: colour,
            ..Default::default()
        },
    );
    job
}

/// The rows a tile name may use: two when every word fits across the tile, one when a word
/// is wider than the tile, so the large-font case elides instead of breaking a word.
pub(crate) fn name_rows(
    ui: &egui::Ui,
    name: &str,
    width: f32,
    font: &egui::FontId,
    colour: egui::Color32,
) -> usize {
    let too_wide = name.split_whitespace().any(|word| {
        ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(word.to_owned(), font.clone(), colour)
                .size()
                .x
                > width
        })
    });
    if too_wide { 1 } else { 2 }
}

/// The width one tile may take in a group: two columns when every principal word fits across
/// the half-width, one column when a large font would otherwise elide or split a word.
pub(crate) fn tile_width_for<'a>(
    ui: &egui::Ui,
    available: f32,
    spacing: f32,
    names: impl Iterator<Item = &'a str>,
    font: &egui::FontId,
    colour: egui::Color32,
) -> f32 {
    let half = ((available - spacing) * 0.5).max(1.0);
    let every_word_fits = names.into_iter().all(|name| {
        let (principal, _) = split_name(name);
        name_rows(ui, principal, (half - 4.0).max(1.0), font, colour) == 2
    });
    if every_word_fits { half } else { available }
}

/// The name of a definition in the reader's language, falling back to the source name.
pub(crate) fn full_name(definition: &ComponentDefinition, language: Language) -> &str {
    match language {
        Language::Spanish => definition.name_es.as_deref().unwrap_or(&definition.name),
        Language::English => &definition.name,
    }
}
