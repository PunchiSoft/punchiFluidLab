// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shell's three layout modes and the room each one gives its regions.
//!
//! `docs/ui-ux.md` §2.2 fixes the numbers: the sheet keeps 480 px, the catalog 220 px and the
//! right dock 300 px, while the catalog prefers 240 px and the dock 360 px. The mode is a
//! consequence of those measurements, never a list of screen sizes: [`LayoutMode::from_available`]
//! takes the room the shell has in device-independent points, so a scaled display changes
//! the pixels without changing the answer. The widths are interface constants, named here once and read
//! by both panels; the panel's own maximum is a drag bound, not a layout measurement.

/// The sheet's floor: the drawing is the region that never gives way.
pub const SHEET_MIN_WIDTH: f32 = 480.0;

/// The catalog's content floor and the width it prefers in the wide mode.
pub const CATALOG_MIN_WIDTH: f32 = 220.0;
pub const CATALOG_PREFERRED_WIDTH: f32 = 240.0;

/// The right dock's content floor and the width it prefers in the wide mode.
pub const DOCK_MIN_WIDTH: f32 = 300.0;
pub const DOCK_PREFERRED_WIDTH: f32 = 360.0;

/// The rail a folded panel keeps. It is the declared floor; a larger font makes the panel
/// chrome wider than it, because the rail follows the spacing unit.
pub const RAIL_WIDTH: f32 = crate::panel::MIN_RAIL;

/// One of the three shells `docs/ui-ux.md` §2.2 describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// The sheet, the catalog and the dock fit at their preferred widths.
    Wide,
    /// The preferred widths no longer fit, but the minimums do: both panels dock narrower.
    Medium,
    /// The minimums do not fit: both panels fold to their rails and open as a drawer.
    Compact,
}

/// The widths a mode gives the catalog and the right dock, in points. The sheet takes the
/// rest; the mode derivation guarantees that rest is never below [`SHEET_MIN_WIDTH`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Widths {
    pub catalog: f32,
    pub dock: f32,
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

    /// The room the preferred widths need beside the sheet's floor: the wide threshold.
    #[must_use]
    pub const fn wide_from() -> f32 {
        SHEET_MIN_WIDTH + CATALOG_PREFERRED_WIDTH + DOCK_PREFERRED_WIDTH
    }

    /// The room the minimums need beside the sheet's floor: the medium threshold.
    #[must_use]
    pub const fn medium_from() -> f32 {
        SHEET_MIN_WIDTH + CATALOG_MIN_WIDTH + DOCK_MIN_WIDTH
    }

    /// The widths this mode gives both panels for `available` points. In the wide mode the
    /// panels keep the width the reader gave them, narrowed together so the sheet's floor
    /// holds; the minimums are the floors a panel may not cross, and compact is the rail.
    #[must_use]
    pub fn widths(self, available: f32, remembered_catalog: f32, remembered_dock: f32) -> Widths {
        let available = if available.is_finite() {
            available.max(0.0)
        } else {
            0.0
        };
        match self {
            Self::Wide => {
                let catalog = clamped(
                    remembered_catalog,
                    CATALOG_MIN_WIDTH,
                    available - SHEET_MIN_WIDTH - DOCK_MIN_WIDTH,
                );
                let dock = clamped(
                    remembered_dock,
                    DOCK_MIN_WIDTH,
                    available - SHEET_MIN_WIDTH - catalog,
                );
                let catalog = clamped(
                    catalog,
                    CATALOG_MIN_WIDTH,
                    available - SHEET_MIN_WIDTH - dock,
                );
                Widths { catalog, dock }
            }
            Self::Medium => Widths {
                catalog: CATALOG_MIN_WIDTH,
                dock: DOCK_MIN_WIDTH,
            },
            Self::Compact => Widths {
                catalog: RAIL_WIDTH,
                dock: RAIL_WIDTH,
            },
        }
    }

    /// The room the sheet keeps once the panels took theirs. The tests read it; the shell
    /// itself lets egui do the subtraction.
    #[cfg(test)]
    #[must_use]
    pub fn sheet_width(self, available: f32, remembered_catalog: f32, remembered_dock: f32) -> f32 {
        let widths = self.widths(available, remembered_catalog, remembered_dock);
        available - widths.catalog - widths.dock
    }
}

/// A value kept between two bounds even when the room is smaller than the floor: the floor
/// wins, as it does for the sheet.
fn clamped(value: f32, low: f32, high: f32) -> f32 {
    value.max(low).min(high.max(low))
}

#[cfg(test)]
mod tests;
