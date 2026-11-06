// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shell regions that are not the ribbon or the sidebar: the sheet and the status
//! bar, plus the keyboard reference.
//!
//! Every size comes from [`punchi_fluid_theme::Metrics`] and every colour from a
//! token, so the three regions scale together when the desktop font changes.

use egui::{Rect, Stroke, Ui};
use punchi_fluid_theme::{Metrics, Theme, Token};

/// Which half of the workflow the window is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Select, move and connect; the model is frozen.
    Edit,
    /// The model runs and the sheet reacts; the document cannot change.
    #[default]
    Simulate,
}

impl Mode {
    /// Name shown in the status bar and on the mode switch.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Edit => "Editing",
            Self::Simulate => "Simulation",
        }
    }

    /// The other mode.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Edit => Self::Simulate,
            Self::Simulate => Self::Edit,
        }
    }
}

/// The values the status bar prints.
#[derive(Debug, Clone, Copy)]
pub struct StatusInfo<'a> {
    /// Simulated time, seconds.
    pub time: f64,
    /// Gauge pressure, bar.
    pub pressure_bar: f64,
    /// Piston position, millimetres.
    pub position_mm: f64,
    /// Piston velocity, metres per second.
    pub velocity: f64,
    /// Air that entered the circuit, grams.
    pub mass_in_g: f64,
    /// Definitions the catalog loaded.
    pub definitions: usize,
    /// Files the catalog skipped.
    pub skipped: usize,
    /// Editing or simulating.
    pub mode: Mode,
    /// Last message: saved, exported, skipped file, error.
    pub message: &'a str,
}

/// Paints the sheet background and, optionally, its two-level grid.
pub fn paint_sheet(ui: &Ui, theme: &Theme, grid: bool) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();
    painter.rect_filled(rect, 0, theme.colour(Token::Sheet));
    if !grid {
        return;
    }

    let minor = theme.metrics.sheet_grid_minor.max(2.0);
    let major = theme.metrics.sheet_grid_major.max(minor * 2.0);
    for (step, token) in [(minor, Token::GridMinor), (major, Token::GridMajor)] {
        let stroke = Stroke::new(theme.metrics.stroke * 0.5, theme.colour(token));
        let mut x = rect.left();
        while x <= rect.right() {
            painter.vline(x, rect.y_range(), stroke);
            x += step;
        }
        let mut y = rect.top();
        while y <= rect.bottom() {
            painter.hline(rect.x_range(), y, stroke);
            y += step;
        }
    }
}

/// Draws the status bar: measured values, the mode and the last message.
pub fn show_status(ui: &mut Ui, theme: &Theme, info: &StatusInfo<'_>) {
    let muted = theme.colour(Token::TextMuted);
    let text = theme.colour(Token::Text);
    ui.horizontal_wrapped(|ui| {
        let mut field = |label: String| {
            ui.label(egui::RichText::new(label).small().color(muted));
            ui.separator();
        };
        field(format!("t {:.3} s", info.time));
        field(format!("p {:.2} bar", info.pressure_bar));
        field(format!("x {:.1} mm", info.position_mm));
        field(format!("v {:.2} m/s", info.velocity));
        field(format!("air {:.3} g", info.mass_in_g));
        field(format!("{} definitions", info.definitions));

        ui.label(
            egui::RichText::new(info.mode.label())
                .small()
                .strong()
                .color(theme.colour(Token::Accent)),
        );
        ui.separator();

        if info.skipped > 0 {
            ui.label(
                egui::RichText::new(format!("{} file(s) skipped", info.skipped))
                    .small()
                    .color(theme.colour(Token::Warning)),
            );
            ui.separator();
        }
        ui.label(egui::RichText::new(info.message).small().color(text));
    });
}

/// Shortcut reference, opened with `F1`.
///
/// A window, not a modal: the sheet stays visible and usable behind it.
pub fn show_help(context: &egui::Context, theme: &Theme, open: &mut bool) {
    egui::Window::new("Keyboard shortcuts")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .default_width(theme.metrics.help_width)
        .show(context, |ui| {
            egui::ScrollArea::vertical()
                .max_height(theme.metrics.row_height * 18.0)
                .show(ui, |ui| {
                    for tab in crate::ribbon::Tab::ALL {
                        ui.label(egui::RichText::new(tab.label()).strong());
                        egui::Grid::new(("shortcut-grid", tab))
                            .num_columns(2)
                            .spacing(egui::vec2(
                                theme.metrics.grid_unit,
                                theme.metrics.grid_unit * 0.25,
                            ))
                            .show(ui, |ui| {
                                for action in crate::ribbon::Action::ALL
                                    .iter()
                                    .copied()
                                    .filter(|action| action.tab() == tab)
                                {
                                    ui.label(action.label());
                                    ui.label(
                                        egui::RichText::new(action.shortcut_text())
                                            .monospace()
                                            .color(theme.colour(Token::TextMuted)),
                                    );
                                    ui.end_row();
                                }
                            });
                        ui.add_space(theme.metrics.grid_unit * 0.5);
                    }
                });
        });
}

/// Makes the controls of this scope flat, as a native menu bar and toolbar are.
///
/// No frame, no fill and no border until the pointer is over the control; then the
/// raised surface marks it. Writing it once keeps the menu bar and the toolbar from
/// drifting apart, and keeps the colours in the theme where they belong.
pub fn flat_controls(ui: &mut Ui, theme: &Theme) {
    let widgets = &mut ui.style_mut().visuals.widgets;
    for state in [
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.open,
        &mut widgets.noninteractive,
    ] {
        state.weak_bg_fill = punchi_fluid_theme::NO_PAINT;
        state.bg_fill = punchi_fluid_theme::NO_PAINT;
        state.bg_stroke = egui::Stroke::NONE;
        state.expansion = 0.0;
    }
    let highlight = theme.colour(Token::SurfaceRaised);
    widgets.hovered.weak_bg_fill = highlight;
    widgets.open.weak_bg_fill = highlight;
    // A group rule is still a rule: `ui.separator()` takes its stroke from the
    // non-interactive state, so clearing every fill would have erased it.
    widgets.noninteractive.bg_stroke =
        egui::Stroke::new(theme.metrics.stroke * 0.5, theme.colour(Token::Outline));
}

/// Height the ribbon needs, including the margin egui adds around a panel.
#[must_use]
pub fn ribbon_height(metrics: &Metrics) -> f32 {
    metrics.ribbon_height + metrics.grid_unit * 0.6
}

/// Rect of the sheet inside a region, kept in one place so the grid and the canvas
/// never disagree.
#[must_use]
pub fn sheet_rect(ui: &Ui) -> Rect {
    ui.available_rect_before_wrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_modes_are_visibly_named() {
        assert_eq!(Mode::Edit.label(), "Editing");
        assert_eq!(Mode::Simulate.label(), "Simulation");
        assert_eq!(Mode::Edit.toggled(), Mode::Simulate);
        assert_eq!(Mode::Simulate.toggled(), Mode::Edit);
    }

    #[test]
    fn the_toolbar_has_room_for_one_row_of_buttons() {
        // A native toolbar is one row: taller than a status row, shorter than the
        // ribbon it replaced.
        let metrics = Metrics::default();
        let height = ribbon_height(&metrics);
        assert!(height > metrics.status_height);
        assert!(height < metrics.row_height * 2.5);
        assert!(height >= metrics.toolbar_button_height);
    }
}
