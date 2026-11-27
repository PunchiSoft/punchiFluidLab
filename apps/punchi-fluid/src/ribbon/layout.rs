// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! How much room each group asks for, and which ones keep their row at a given width.

use eframe::egui;
use punchi_fluid_i18n::Language;

use super::model::{Group, Item};
use super::{Palette, split};

/// Which groups keep their row at the current width: the last ones collapse until the row
/// fits, each to its own title button (P-028), never to a global menu.
pub(super) fn fit_groups(
    ui: &egui::Ui,
    groups: &[Group],
    palette: &Palette,
    language: Language,
) -> Vec<bool> {
    let available = ui.available_width();
    let expanded: Vec<f32> = groups
        .iter()
        .map(|group| estimate(ui, group, palette, language))
        .collect();
    let collapsed: Vec<f32> = groups
        .iter()
        .map(|group| {
            (text_width(ui, language.text(group.name), palette.text_label) + 44.0).max(96.0)
        })
        .collect();
    let mut shown = vec![true; groups.len()];
    let total = |shown: &[bool]| {
        expanded
            .iter()
            .zip(collapsed.iter())
            .zip(shown.iter())
            .map(|((wide, narrow), shown)| if *shown { *wide } else { *narrow } + 8.0)
            .sum::<f32>()
    };
    for index in (0..groups.len()).rev() {
        if total(&shown) <= available {
            break;
        }
        shown[index] = false;
    }
    shown
}

pub(super) fn estimate(ui: &egui::Ui, group: &Group, palette: &Palette, language: Language) -> f32 {
    let mut width = 0.0_f32;
    for item in group.items {
        width += match item {
            Item::Large(..) => 64.0 + 6.0,
            Item::Split(..) => split::WIDTH + 6.0,
            Item::Small(_, label, _)
            | Item::Menu(_, label, _)
            | Item::Gallery(_, label, _)
            | Item::Recents(_, label) => {
                16.0 + 6.0 + text_width(ui, language.text(label), palette.text_body) + 22.0 + 6.0
            }
        };
    }
    width.max(96.0) + 12.0
}

pub(super) fn text_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(
                text.to_owned(),
                egui::FontId::proportional(size),
                egui::Color32::WHITE,
            )
            .size()
            .x
    })
}
