// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! What belongs in the palette and how its preview is inked: the entry rule, the search
//! match, the declared symbol box and the thumbnail colours. Kept out of `sidebar.rs` so the
//! panel's drawing and state stay inside the project's file budget.

use eframe::egui;
use punchi_fluid_catalog::{Behaviour, ComponentDefinition};
use punchi_fluid_draw::surface::Colour;
use punchi_fluid_draw::symbol::Ink;

use crate::theme::Tokens;

/// Whether a definition is a palette entry: a port closure is not.
pub(crate) fn is_entry(definition: &ComponentDefinition) -> bool {
    !matches!(
        definition.behaviour,
        Behaviour::Plug | Behaviour::Silencer { .. }
    )
}

/// Whether the definition matches the search: its name, its Spanish name and its id.
pub(crate) fn matches(definition: &ComponentDefinition, filter: &str) -> bool {
    if filter.trim().is_empty() {
        return true;
    }
    let filter = filter.to_lowercase();
    definition.id.to_lowercase().contains(&filter)
        || definition.name.to_lowercase().contains(&filter)
        || definition
            .name_es
            .as_deref()
            .is_some_and(|name| name.to_lowercase().contains(&filter))
}

/// The symbol box of a definition, in millimetres: the one its file declares, the same
/// the templates place and the drop uses.
pub(crate) fn box_of(definition: &ComponentDefinition) -> (f32, f32) {
    punchi_fluid_draw::symbol_box_mm(definition)
}

/// The ink of a thumbnail: the desktop's colours, not the printed sheet's.
pub(crate) fn preview_ink(tokens: &Tokens) -> Ink {
    let colour = |colour: egui::Color32| Colour {
        r: colour.r(),
        g: colour.g(),
        b: colour.b(),
    };
    Ink {
        line: colour(tokens.ink),
        dim: colour(tokens.ink_dim),
        muted: colour(tokens.ink_muted),
        text: colour(tokens.ink),
        supply: colour(tokens.supply),
        work: colour(tokens.work),
        accent: colour(tokens.accent),
        background: colour(tokens.panel),
    }
}
