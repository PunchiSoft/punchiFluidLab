// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Size tokens shared by every region.
//!
//! A widget asks the theme for a length instead of writing a number, so the design
//! follows the desktop's grid unit and font size in one place.

/// Lengths derived from the desktop grid unit and font size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Base grid step, 18 points in Breeze.
    pub grid_unit: f32,
    /// One line of controls.
    pub row_height: f32,
    /// Toolbar height: one row plus the air a native toolbar keeps around it.
    pub ribbon_height: f32,
    /// Side of a toolbar icon.
    pub toolbar_icon: f32,
    /// Size of a toolbar label.
    pub toolbar_label: f32,
    /// Space between two toolbar buttons.
    pub toolbar_gap: f32,
    /// Padding inside a toolbar button.
    pub toolbar_padding: f32,
    /// Height of a toolbar button.
    pub toolbar_button_height: f32,
    /// Smallest width of a menu in a menu bar.
    pub menu_min_width: f32,
    /// Width of the about window.
    pub about_width: f32,
    /// Line height of the about window's banner.
    pub about_banner: f32,
    /// Default sidebar width, `18 × grid_unit`.
    pub sidebar_width: f32,
    /// Smallest sidebar width while dragging.
    pub sidebar_min: f32,
    /// Status bar height, one row.
    pub status_height: f32,
    /// Standard outline width.
    pub stroke: f32,
    /// Corner radius of frames and buttons.
    pub radius: u8,
    /// Minor grid of the sheet.
    pub sheet_grid_minor: f32,
    /// Major grid of the sheet.
    pub sheet_grid_major: f32,
    /// Smallest window the shell is designed for.
    pub min_window: (f32, f32),
    /// Width below which the ribbon collapses to a menu.
    pub ribbon_collapse_width: f32,
    /// Width of the keyboard help window.
    pub help_width: f32,
    /// Side of a catalog thumbnail.
    pub thumbnail: f32,
}

impl Metrics {
    /// Derives every length from the interface font size in points.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn from_font(body: f32) -> Self {
        let scale = (body / 13.0).clamp(0.85, 1.3);
        let grid_unit = 18.0 * scale;
        let row_height = grid_unit * 1.5;
        Self {
            grid_unit,
            row_height,
            ribbon_height: row_height * 1.4,
            toolbar_icon: grid_unit * 1.25,
            toolbar_label: grid_unit * 0.72,
            toolbar_gap: grid_unit * 0.35,
            toolbar_padding: grid_unit * 0.55,
            toolbar_button_height: row_height * 1.25,
            menu_min_width: grid_unit * 12.0,
            about_width: grid_unit * 22.0,
            about_banner: grid_unit * 1.6,
            sidebar_width: grid_unit * 18.0,
            sidebar_min: grid_unit * 10.0,
            status_height: row_height,
            stroke: scale,
            radius: (grid_unit * 0.28) as u8,
            sheet_grid_minor: grid_unit * 0.55,
            sheet_grid_major: grid_unit * 2.75,
            min_window: (900.0, 600.0),
            ribbon_collapse_width: 1100.0,
            help_width: grid_unit * 26.0,
            thumbnail: grid_unit * 1.7,
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::from_font(13.0)
    }
}
