// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Replays the project's recording onto an `egui::Painter`.
//!
//! Every primitive of `punchi-fluid-draw::surface::Surface` becomes a shape in screen
//! points; the drawing stays in millimetres and the surface owns the one multiplication.

use eframe::egui;
use punchi_fluid_draw::surface::{Anchor, Colour, Pt, Rect, Stroke, Surface};

use crate::theme::Tokens;

pub struct EguiSurface<'a> {
    pub shapes: Vec<egui::Shape>,
    pub scale: f32,
    pub origin: egui::Pos2,
    context: &'a egui::Context,
    screen: Option<ScreenInk>,
}

impl std::fmt::Debug for EguiSurface<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EguiSurface")
            .field("shapes", &self.shapes.len())
            .field("scale", &self.scale)
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

impl<'a> EguiSurface<'a> {
    #[must_use]
    pub fn new(context: &'a egui::Context, origin: egui::Pos2, scale: f32) -> Self {
        Self {
            shapes: Vec::new(),
            scale,
            origin,
            context,
            screen: None,
        }
    }

    /// Maps the drawing's print palette onto the window's theme.
    ///
    /// The drawing is written for the page: black ink on white paper. On screen that is
    /// black on a dark canvas, which is how the sheet used to disappear. A surface that
    /// carries the theme's ink turns the same recording into the window's colours.
    #[must_use]
    pub fn with_screen_ink(mut self, ink: ScreenInk) -> Self {
        self.screen = Some(ink);
        self
    }

    fn at(&self, point: Pt) -> egui::Pos2 {
        self.origin + egui::vec2(point.x * self.scale, point.y * self.scale)
    }

    fn ink(&self, colour: Colour) -> egui::Color32 {
        self.screen.map_or_else(
            || egui::Color32::from_rgb(colour.r, colour.g, colour.b),
            |screen| screen.map(colour),
        )
    }

    fn stroke(&self, stroke: Stroke) -> egui::Stroke {
        egui::Stroke::new(
            (stroke.width * self.scale).max(0.5),
            self.ink(stroke.colour),
        )
    }
}

impl Surface for EguiSurface<'_> {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        let points: Vec<egui::Pos2> = points.iter().map(|point| self.at(*point)).collect();
        if points.len() > 1 {
            self.shapes
                .push(egui::Shape::line(points, self.stroke(stroke)));
        }
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        let points: Vec<egui::Pos2> = points.iter().map(|point| self.at(*point)).collect();
        if points.len() < 3 {
            return;
        }
        if let Some(fill) = fill {
            self.shapes.push(egui::Shape::convex_polygon(
                points.clone(),
                self.ink(fill),
                egui::Stroke::NONE,
            ));
        }
        if let Some(stroke) = stroke {
            let mut closed = points;
            if let Some(first) = closed.first().copied() {
                closed.push(first);
            }
            self.shapes
                .push(egui::Shape::line(closed, self.stroke(stroke)));
        }
    }

    fn rect(&mut self, rect: Rect, _corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        let points = [
            Pt::new(rect.min.x, rect.min.y),
            Pt::new(rect.max.x, rect.min.y),
            Pt::new(rect.max.x, rect.max.y),
            Pt::new(rect.min.x, rect.max.y),
        ];
        self.polygon(&points, fill, stroke);
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "the segment count is small and exact in f32"
    )]
    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        self.arc(
            centre,
            radius,
            0.0,
            360.0,
            stroke.unwrap_or(Stroke::new(0.3, Colour::BLACK)),
        );
        if let Some(fill) = fill {
            let points: Vec<egui::Pos2> = (0..32)
                .map(|index| {
                    let angle = index as f32 * std::f32::consts::TAU / 32.0;
                    self.at(Pt::new(
                        centre.x + radius * angle.cos(),
                        centre.y + radius * angle.sin(),
                    ))
                })
                .collect();
            self.shapes.push(egui::Shape::convex_polygon(
                points,
                self.ink(fill),
                egui::Stroke::NONE,
            ));
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "the segment count is small and exact in f32"
    )]
    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke) {
        let steps = 48;
        let points: Vec<egui::Pos2> = (0..=steps)
            .map(|index| {
                let angle = start + (end - start) * (index as f32 / steps as f32);
                let radians = angle.to_radians();
                self.at(Pt::new(
                    centre.x + radius * radians.cos(),
                    centre.y + radius * radians.sin(),
                ))
            })
            .collect();
        self.shapes
            .push(egui::Shape::line(points, self.stroke(stroke)));
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        let colour = self.ink(colour);
        let font = egui::FontId::proportional(size * self.scale);
        let galley = self
            .context
            .fonts(|fonts| fonts.layout_no_wrap(text.to_owned(), font, colour));
        // The drawing gives a baseline; `Shape::galley` takes the top-left of the run, so
        // lift it by the approximate ascent of the face and centre it on the anchor.
        let position = self.at(at) - egui::vec2(0.0, size * self.scale * 0.35);
        let extent = galley.size();
        let top_left = match anchor {
            Anchor::Start => position - egui::vec2(0.0, extent.y * 0.5),
            Anchor::Middle => position - extent * 0.5,
            Anchor::End => position - egui::vec2(extent.x, extent.y * 0.5),
        };
        self.shapes
            .push(egui::Shape::galley(top_left, galley, colour));
    }
}

/// The window's translation of the print palette.
///
/// The drawing names its ink as black, white and two greys because that is what a printed
/// diagram needs. The screen is a different device: it has a dark paper and a light ink,
/// and the same recording has to be legible on it.
#[derive(Clone, Copy, Debug)]
pub struct ScreenInk {
    /// Lines, frames and primary text.
    pub ink: egui::Color32,
    /// The paper behind the drawing.
    pub paper: egui::Color32,
    /// Secondary text, such as a field label.
    pub muted: egui::Color32,
    /// A line of an inactive position.
    pub dim: egui::Color32,
    /// The active square of a valve.
    pub accent: egui::Color32,
}

impl ScreenInk {
    /// The translation for a theme.
    #[must_use]
    pub fn from_tokens(tokens: &Tokens) -> Self {
        Self {
            ink: tokens.ink,
            paper: tokens.paper,
            muted: tokens.ink_muted,
            dim: tokens.ink_dim,
            accent: tokens.accent,
        }
    }

    /// The print colour a drawing names, in the window's ink. A colour the drawing does
    /// not name — the note red, for one — is kept: the drawing chose it for the page and
    /// it still reads on the paper.
    fn map(self, colour: Colour) -> egui::Color32 {
        match (colour.r, colour.g, colour.b) {
            (0, 0, 0) => self.ink,
            (255, 255, 255) => self.paper,
            (90, 90, 90) => self.muted,
            (130, 130, 130) => self.dim,
            (220, 226, 240) => self.accent,
            (red, green, blue) => egui::Color32::from_rgb(red, green, blue),
        }
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use super::ScreenInk;
    use punchi_fluid_draw::surface::Colour;

    fn screen() -> ScreenInk {
        ScreenInk {
            ink: egui::Color32::from_rgb(250, 250, 250),
            paper: egui::Color32::from_rgb(60, 60, 60),
            muted: egui::Color32::from_rgb(150, 150, 150),
            dim: egui::Color32::from_rgb(170, 170, 170),
            accent: egui::Color32::from_rgb(49, 91, 239),
        }
    }

    #[test]
    fn the_print_palette_becomes_the_window_ink() {
        let screen = screen();
        assert_eq!(screen.map(Colour::BLACK), screen.ink);
        assert_eq!(screen.map(Colour::WHITE), screen.paper);
        assert_eq!(screen.map(Colour::new(90, 90, 90)), screen.muted);
        assert_eq!(screen.map(Colour::new(130, 130, 130)), screen.dim);
        assert_eq!(screen.map(Colour::new(220, 226, 240)), screen.accent);
    }

    #[test]
    fn a_colour_the_drawing_names_keeps_its_own_ink() {
        let note = Colour::new(170, 40, 40);
        assert_eq!(
            screen().map(note),
            egui::Color32::from_rgb(note.r, note.g, note.b)
        );
    }
}
