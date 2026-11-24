// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view grid: a guide drawn over the paper, under the drawing, never printed.
//!
//! The step is a view setting shared by the ruling and, when the window has a snap, by the
//! snap; showing or hiding it changes no document field. The lines are clipped to the frame
//! and kept out of the title block, and the major line is the 50 mm reference field.

use eframe::egui;
use punchi_fluid_draw::{Colour, Pt, Stroke, Surface};
use punchi_fluid_export::page::Page;

use crate::theme::Tokens;

/// The steps the drawing benefits from, all divisors of the 50 mm reference field.
pub(crate) const STEPS: [u16; 4] = [5, 10, 20, 25];

/// The step a fresh profile gets, in millimetres.
pub(crate) const DEFAULT_STEP_MM: u16 = 5;

/// The grid's view state: whether it shows and its step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GridView {
    pub visible: bool,
    pub step_mm: u16,
}

impl Default for GridView {
    fn default() -> Self {
        Self {
            visible: false,
            step_mm: DEFAULT_STEP_MM,
        }
    }
}

impl GridView {
    /// Sets the step, falling back to the default for a value the panel does not offer.
    pub(crate) fn set_step(&mut self, step_mm: u16) {
        self.step_mm = if STEPS.contains(&step_mm) {
            step_mm
        } else {
            DEFAULT_STEP_MM
        };
    }

    /// The positions of the ruling on one axis, in sheet millimetres, with whether each one is
    /// a major line (the 50 mm reference field).
    pub(crate) fn positions(self, low: f32, high: f32) -> Vec<(f32, bool)> {
        let step = f32::from(self.step_mm);
        if step <= 0.0 || high <= low {
            return Vec::new();
        }
        let mut positions = Vec::new();
        let mut at = (low / step).ceil() * step;
        while at <= high + f32::EPSILON {
            positions.push((at, (at / 50.0 - (at / 50.0).round()).abs() < 1e-4));
            at += step;
        }
        positions
    }
}

/// Draws the grid over the paper and under the block and the drawing. The colours are the
/// theme's grid tokens, pre-blended over the paper because the drawing surface carries no
/// alpha.
pub(crate) fn draw(surface: &mut dyn Surface, page: &Page, view: GridView, tokens: &Tokens) {
    if !view.visible {
        return;
    }
    let content = page.content_rect();
    if content.width() <= 0.0 || content.height() <= 0.0 {
        return;
    }
    let block = page.title_rect();
    let minor = blended(tokens.ink_muted, tokens.paper, 56.0 / 255.0);
    let major = blended(tokens.ink_muted, tokens.paper, 87.0 / 255.0);
    let minor_stroke = Stroke::new(0.15, minor);
    let major_stroke = Stroke::new(0.25, major);
    for (x, is_major) in view.positions(content.min.x, content.max.x) {
        let stroke = if is_major { major_stroke } else { minor_stroke };
        if !block.is_empty() && x >= block.min.x && x <= block.max.x {
            surface.line(
                &[Pt::new(x, content.min.y), Pt::new(x, block.min.y)],
                stroke,
            );
            surface.line(
                &[Pt::new(x, block.max.y), Pt::new(x, content.max.y)],
                stroke,
            );
        } else {
            surface.line(
                &[Pt::new(x, content.min.y), Pt::new(x, content.max.y)],
                stroke,
            );
        }
    }
    for (y, is_major) in view.positions(content.min.y, content.max.y) {
        let stroke = if is_major { major_stroke } else { minor_stroke };
        if !block.is_empty() && y >= block.min.y && y <= block.max.y {
            surface.line(
                &[Pt::new(content.min.x, y), Pt::new(block.min.x, y)],
                stroke,
            );
            surface.line(
                &[Pt::new(block.max.x, y), Pt::new(content.max.x, y)],
                stroke,
            );
        } else {
            surface.line(
                &[Pt::new(content.min.x, y), Pt::new(content.max.x, y)],
                stroke,
            );
        }
    }
}

/// A theme colour over another, as an opaque print colour.
fn blended(over: egui::Color32, under: egui::Color32, alpha: f32) -> Colour {
    let mix = |a: u8, b: u8| {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the channels are 0-255 and the blend stays in that range"
        )]
        let value = (f32::from(a) * alpha + f32::from(b) * (1.0 - alpha)).round() as u8;
        value
    };
    Colour::new(
        mix(over.r(), under.r()),
        mix(over.g(), under.g()),
        mix(over.b(), under.b()),
    )
}

#[cfg(test)]
mod tests {
    use super::{GridView, STEPS};
    use punchi_fluid_draw::Recording;
    use punchi_fluid_export::page::{Page, TitleBlock};

    fn page() -> Page {
        Page::a4(TitleBlock::new("A circuit", "2026-11-06"))
    }

    #[test]
    fn the_ruling_reaches_the_frame_and_marks_the_reference_field() {
        let content = page().content_rect();
        let view = GridView {
            visible: true,
            step_mm: 5,
        };
        let lines = view.positions(content.min.x, content.max.x);
        assert!(!lines.is_empty());
        assert!(
            lines.iter().any(|(_, major)| *major),
            "the 50 mm line is major"
        );
        assert!(
            lines.iter().any(|(_, major)| !*major),
            "the step lines are minor"
        );
    }

    #[test]
    fn the_step_changes_the_ruling() {
        let content = page().content_rect();
        let mut view = GridView::default();
        let five = view.positions(content.min.x, content.max.x).len();
        view.set_step(10);
        let ten = view.positions(content.min.x, content.max.x).len();
        assert!(five > ten, "5 mm rules more lines than 10 mm");
        view.set_step(7);
        assert_eq!(
            view.step_mm,
            GridView::default().step_mm,
            "an unknown step falls back"
        );
        for step in STEPS {
            view.set_step(step);
            assert_eq!(view.step_mm, step);
        }
    }

    #[test]
    fn the_hidden_grid_draws_nothing() {
        let mut recording = Recording::default();
        let tokens = crate::theme::Theme::fallback().tokens;
        super::draw(&mut recording, &page(), GridView::default(), &tokens);
        assert!(
            recording
                .calls()
                .iter()
                .all(|call| !matches!(call, punchi_fluid_draw::Call::Line { .. }))
        );
    }
}
