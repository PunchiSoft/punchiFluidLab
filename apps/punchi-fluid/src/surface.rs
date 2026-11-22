// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Replays the project's recording onto an `egui::Painter`.
//!
//! Every primitive of `punchi-fluid-draw::surface::Surface` becomes a shape in screen
//! points; the drawing stays in millimetres and the surface owns the one multiplication.

use eframe::egui;
use punchi_fluid_draw::surface::{Anchor, Colour, Pt, Rect, Stroke, Surface};

pub struct EguiSurface<'a> {
    pub shapes: Vec<egui::Shape>,
    pub scale: f32,
    pub origin: egui::Pos2,
    context: &'a egui::Context,
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
        }
    }

    fn at(&self, point: Pt) -> egui::Pos2 {
        self.origin + egui::vec2(point.x * self.scale, point.y * self.scale)
    }

    fn stroke(&self, stroke: Stroke) -> egui::Stroke {
        egui::Stroke::new((stroke.width * self.scale).max(0.5), ink(stroke.colour))
    }
}

fn ink(colour: Colour) -> egui::Color32 {
    egui::Color32::from_rgb(colour.r, colour.g, colour.b)
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
                ink(fill),
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
                ink(fill),
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
        let colour = ink(colour);
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
