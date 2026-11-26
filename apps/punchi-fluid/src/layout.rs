// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shell's three layout modes and the room each one gives its regions.
//!
//! `docs/ui-ux.md` §2.2 fixes the numbers: the sheet keeps 480 px and the catalog 260 px,
//! while the catalog prefers 300 px and never takes more than 400. The mode is a consequence
//! of those measurements, never a list of screen sizes: [`LayoutMode::from_available`] takes
//! the room the shell has in device-independent points, so a scaled display changes the pixels
//! without changing the answer. The properties window is not a region: it floats over the
//! sheet and reserves no width, so only the catalog is measured here.

use eframe::egui;

/// The sheet's floor: the drawing is the region that never gives way.
pub const SHEET_MIN_WIDTH: f32 = 480.0;

/// The catalog's content floor, the width it prefers in the wide mode and its drag maximum.
pub const CATALOG_MIN_WIDTH: f32 = 260.0;
pub const CATALOG_PREFERRED_WIDTH: f32 = 300.0;
pub const CATALOG_MAX_WIDTH: f32 = 400.0;

/// The rail a folded catalog keeps. It is the declared floor; a larger font makes the panel
/// chrome wider than it, because the rail follows the spacing unit.
pub const RAIL_WIDTH: f32 = crate::panel::MIN_RAIL;

/// One of the three shells `docs/ui-ux.md` §2.2 describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// The sheet and the catalog fit at the catalog's preferred width.
    Wide,
    /// The preferred width no longer fits, but the catalog's minimum does.
    Medium,
    /// The minimum does not fit: the catalog folds to its rail and opens as a drawer.
    Compact,
}

/// The width a mode gives the catalog, in points. The sheet takes the rest; the mode
/// derivation guarantees that rest is never below [`SHEET_MIN_WIDTH`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Widths {
    pub catalog: f32,
}

impl LayoutMode {
    /// The mode the shell takes when it has `available` points of room.
    #[must_use]
    pub fn from_available(available: f32) -> Self {
        if available >= Self::wide_from() {
            Self::Wide
        } else if available >= Self::medium_from() {
            Self::Medium
        } else {
            Self::Compact
        }
    }

    /// The room the preferred catalog needs beside the sheet's floor: the wide threshold.
    #[must_use]
    pub const fn wide_from() -> f32 {
        SHEET_MIN_WIDTH + CATALOG_PREFERRED_WIDTH
    }

    /// The room the minimum catalog needs beside the sheet's floor: the medium threshold.
    #[must_use]
    pub const fn medium_from() -> f32 {
        SHEET_MIN_WIDTH + CATALOG_MIN_WIDTH
    }

    /// The width this mode gives the catalog for `available` points. In the wide mode the
    /// catalog keeps the width the reader gave it, narrowed if needed so the sheet's floor
    /// holds; the medium mode gives its minimum and compact is the rail.
    #[must_use]
    pub fn widths(self, available: f32, remembered_catalog: f32) -> Widths {
        let available = if available.is_finite() {
            available.max(0.0)
        } else {
            0.0
        };
        match self {
            Self::Wide => Widths {
                catalog: clamped(
                    remembered_catalog,
                    CATALOG_MIN_WIDTH,
                    (available - SHEET_MIN_WIDTH).min(CATALOG_MAX_WIDTH),
                ),
            },
            Self::Medium => Widths {
                catalog: CATALOG_MIN_WIDTH,
            },
            Self::Compact => Widths {
                catalog: RAIL_WIDTH,
            },
        }
    }

    /// The widest the catalog may open in this mode with `available` points: the room that
    /// leaves the sheet its floor. The compact mode docks no catalog — the drawer floats over
    /// the sheet — so the bound is the room the window has.
    #[must_use]
    pub fn max_widths(self, available: f32) -> Widths {
        let available = if available.is_finite() {
            available.max(0.0)
        } else {
            0.0
        };
        match self {
            Self::Wide | Self::Medium => Widths {
                catalog: (available - SHEET_MIN_WIDTH).clamp(CATALOG_MIN_WIDTH, CATALOG_MAX_WIDTH),
            },
            Self::Compact => Widths {
                catalog: available.max(RAIL_WIDTH),
            },
        }
    }

    /// The room the sheet keeps once the catalog took its width. The tests read it; the shell
    /// itself lets egui do the subtraction.
    #[cfg(test)]
    #[must_use]
    pub fn sheet_width(self, available: f32, remembered_catalog: f32) -> f32 {
        available - self.widths(available, remembered_catalog).catalog
    }
}

/// The room the floating view strip takes at the right edge of the drawing area: the strip's
/// width (the reference's `2·unit + 8`, written as `2.5·unit` so it follows the unit) plus the
/// margin that keeps the paper clear of it.
pub const VIEW_STRIP_GUTTER_UNITS: f32 = 3.5;

/// The gutter in points for a given spacing unit.
#[must_use]
pub fn view_strip_gutter(unit: f32) -> f32 {
    unit.max(0.0) * VIEW_STRIP_GUTTER_UNITS
}

/// The height the circuit strip takes at the foot of the drawing region: the reference's
/// `2·unit − 8`, with a 28 point floor so the tabs stay legible at the smallest unit.
#[must_use]
pub fn circuit_strip_height(unit: f32) -> f32 {
    (unit.max(0.0) * 2.0 - 8.0).max(28.0)
}

/// The strip's band at the foot of the drawing region, spanning the canvas up to the view
/// strip's gutter: the paper is fitted above it, so the strip never covers the sheet.
#[must_use]
pub fn circuit_strip_rect(canvas: egui::Rect, unit: f32) -> egui::Rect {
    let right = (canvas.right() - view_strip_gutter(unit)).max(canvas.left());
    let top = (canvas.bottom() - circuit_strip_height(unit)).max(canvas.top());
    egui::Rect::from_min_max(
        egui::pos2(canvas.left(), top),
        egui::pos2(right, canvas.bottom()),
    )
}

/// The drawing area: the canvas minus the view strip's gutter on the right and the circuit
/// strip's band at the foot. The paper is fitted and centred in this rectangle; both reserved
/// bands stay part of the canvas, so neither the floating strip nor the circuit strip covers
/// the sheet and the page never enters them.
#[must_use]
pub fn drawing_area(canvas: egui::Rect, unit: f32) -> egui::Rect {
    let right = (canvas.right() - view_strip_gutter(unit)).max(canvas.left());
    let bottom = (canvas.bottom() - circuit_strip_height(unit)).max(canvas.top());
    egui::Rect::from_min_max(canvas.min, egui::pos2(right, bottom))
}

/// A value kept between two bounds even when the room is smaller than the floor: the floor
/// wins, as it does for the sheet.
fn clamped(value: f32, low: f32, high: f32) -> f32 {
    value.max(low).min(high.max(low))
}

#[cfg(test)]
mod tests;
