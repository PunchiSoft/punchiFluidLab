// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The template thumbnails: each template's own document drawn by the sheet's preview path.
//!
//! The catalog panel draws its symbol previews with the same path and the same mapping of
//! the theme's ink, so a card shows the drawing the sheet will show and not a second
//! illustration. The document is composed in the authored area the proposal's 512 × 288
//! plate assumes and then fitted into the card.

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::surface::Colour;
use punchi_fluid_draw::symbol::Ink;
use punchi_fluid_draw::{Pt, Rect as SheetRect, bounds_placed, draw_placed_preview};

use super::document;
use crate::scene;
use crate::surface::EguiSurface;
use crate::theme::Tokens;

/// The authored area a thumbnail is composed in: 160 × 90 mm, the 16:9 of the proposal's
/// 512 × 288 plate.
pub(super) fn thumbnail_area() -> SheetRect {
    SheetRect::from_corners(Pt::new(0.0, 0.0), Pt::new(160.0, 90.0))
}

/// The document of a template drawn into its card's plate, fitted to the plate and in the
/// desktop's ink.
pub(super) fn thumbnail(
    ui: &egui::Ui,
    tokens: &Tokens,
    catalog: &Catalog,
    index: u32,
    plate: egui::Rect,
) {
    let Some(document) = document(index, catalog, thumbnail_area()) else {
        return;
    };
    let scene = scene::build(&document, catalog, document.active_circuit(), None);
    let Some(bounds) = bounds_placed(&scene.placed, &scene.pipes, catalog) else {
        return;
    };
    let (width, height) = (bounds.width(), bounds.height());
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let inner = plate.shrink(tokens.unit * 0.35);
    let scale = (inner.width() / width).min(inner.height() / height);
    let origin = inner.center() - egui::vec2(bounds.center().x * scale, bounds.center().y * scale);
    let mut drawing = EguiSurface::new(ui.ctx(), origin, scale);
    draw_placed_preview(
        &mut drawing,
        &scene.placed,
        &scene.pipes,
        catalog,
        &thumbnail_ink(tokens),
        &[],
    );
    ui.painter().extend(drawing.shapes);
}

/// The ink of a thumbnail: the desktop's colours, not the printed sheet's. The catalog panel
/// maps its symbol previews the same way, because both draw the same document.
pub(super) fn thumbnail_ink(tokens: &Tokens) -> Ink {
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
