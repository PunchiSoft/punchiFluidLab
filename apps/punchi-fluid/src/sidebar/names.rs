// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The catalog tile's text: the principal name and the variant that follows the first comma.

use eframe::egui;

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

/// The name of a tile: at most two lines, elided, so a long name cannot grow over the next
/// tile. The full name stays in the tooltip and in the accessible name.
pub(crate) fn name_job(
    name: &str,
    width: f32,
    font: egui::FontId,
    colour: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob {
        wrap: egui::text::TextWrapping {
            max_width: width,
            max_rows: 2,
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
