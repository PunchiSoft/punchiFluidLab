// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The screen as a [`Surface`].
//!
//! The drawing itself lives in `punchi-fluid-draw` and knows nothing about egui; this
//! adapter is the whole of the toolkit's involvement. One millimetre of the drawing is
//! one point on screen, so the composition is the same as on paper and only the scale
//! changes.

use egui::{Align2, FontId, Painter, Pos2, Shape, Stroke, Vec2};
use punchi_fluid_draw::{Anchor, Colour, Pt, Rect, Surface};

use punchi_fluid_theme::colour;

/// Points per millimetre when a widget draws at natural size.
pub const SCALE: f32 = 1.0;

/// Logical drawing area the canvas lays the circuit out in, in millimetres.
///
/// The drawing is designed in real millimetres — a 0.35 mm line, a 3 mm numeral — so the
/// canvas must scale that drawing to the pixel rectangle instead of pretending a point
/// is a millimetre. A fixed logical area also keeps the composition identical at any
/// window size.
pub const LOGICAL: (f32, f32) = (200.0, 130.0);

/// Adapter that turns drawing calls into egui shapes.
pub struct EguiSurface<'a> {
    painter: &'a Painter,
    origin: Pos2,
    scale: f32,
}

impl std::fmt::Debug for EguiSurface<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EguiSurface")
            .field("origin", &self.origin)
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

impl<'a> EguiSurface<'a> {
    /// Draws in `painter`, with the drawing origin at `origin`.
    #[must_use]
    pub fn new(painter: &'a Painter, origin: Pos2) -> Self {
        Self {
            painter,
            origin,
            scale: SCALE,
        }
    }

    /// Same surface at another scale, for a canvas that zooms.
    #[must_use]
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.1);
        self
    }

    fn point(&self, point: Pt) -> Pos2 {
        Pos2::new(
            self.origin.x + point.x * self.scale,
            self.origin.y + point.y * self.scale,
        )
    }

    fn points(&self, points: &[Pt]) -> Vec<Pos2> {
        points.iter().map(|point| self.point(*point)).collect()
    }

    fn stroke(stroke: punchi_fluid_draw::Stroke, scale: f32) -> Stroke {
        Stroke::new(
            (stroke.width * scale).max(0.5),
            colour::rgb(stroke.colour.r, stroke.colour.g, stroke.colour.b),
        )
    }

    fn fill(fill: Colour) -> egui::Color32 {
        colour::rgb(fill.r, fill.g, fill.b)
    }
}

impl Surface for EguiSurface<'_> {
    fn line(&mut self, points: &[Pt], stroke: punchi_fluid_draw::Stroke) {
        if points.len() < 2 {
            return;
        }
        let shape = Shape::line(self.points(points), Self::stroke(stroke, self.scale));
        self.painter.add(shape);
    }

    fn polygon(
        &mut self,
        points: &[Pt],
        fill: Option<Colour>,
        stroke: Option<punchi_fluid_draw::Stroke>,
    ) {
        if points.len() < 3 {
            return;
        }
        let outline = self.points(points);
        let outline_stroke = stroke.map_or(Stroke::NONE, |stroke| Self::stroke(stroke, self.scale));
        if let Some(colour) = fill {
            self.painter.add(Shape::convex_polygon(
                outline,
                Self::fill(colour),
                outline_stroke,
            ));
        } else {
            let mut closed = outline;
            if let Some(first) = closed.first().copied() {
                closed.push(first);
            }
            self.painter.add(Shape::line(closed, outline_stroke));
        }
    }

    fn rect(
        &mut self,
        rect: Rect,
        corner: f32,
        fill: Option<Colour>,
        stroke: Option<punchi_fluid_draw::Stroke>,
    ) {
        let min = self.point(rect.min);
        let max = self.point(rect.max);
        let shape = egui::Rect::from_min_max(min, max);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let radius = (corner * self.scale).round().clamp(0.0, 255.0) as u8;
        if let Some(colour) = fill {
            self.painter.rect_filled(shape, radius, Self::fill(colour));
        }
        if let Some(stroke) = stroke {
            self.painter.rect_stroke(
                shape,
                radius,
                Self::stroke(stroke, self.scale),
                egui::StrokeKind::Middle,
            );
        }
    }

    fn circle(
        &mut self,
        centre: Pt,
        radius: f32,
        fill: Option<Colour>,
        stroke: Option<punchi_fluid_draw::Stroke>,
    ) {
        let centre = self.point(centre);
        let radius = radius * self.scale;
        if let Some(colour) = fill {
            self.painter
                .circle_filled(centre, radius, Self::fill(colour));
        }
        if let Some(stroke) = stroke {
            self.painter
                .circle_stroke(centre, radius, Self::stroke(stroke, self.scale));
        }
    }

    fn arc(
        &mut self,
        centre: Pt,
        radius: f32,
        start: f32,
        end: f32,
        stroke: punchi_fluid_draw::Stroke,
    ) {
        let samples = 24_i16;
        let mut points = Vec::with_capacity(25);
        for step in 0..=samples {
            let angle = start + (end - start) * (f32::from(step) / f32::from(samples));
            let radians = angle.to_radians();
            points.push(Pt::new(
                centre.x + radius * radians.cos(),
                centre.y + radius * radians.sin(),
            ));
        }
        self.line(&points, stroke);
    }

    fn device_scale(&self) -> f32 {
        self.scale
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, fill: Colour, anchor: Anchor) {
        if text.is_empty() {
            return;
        }
        let align = match anchor {
            Anchor::Start => Align2::LEFT_CENTER,
            Anchor::Middle => Align2::CENTER_CENTER,
            Anchor::End => Align2::RIGHT_CENTER,
        };
        // `size` is a cap height; a font size is the em, roughly 1.4 times bigger.
        let font = FontId::proportional((size * self.scale * 1.4).max(4.0));
        self.painter.text(
            self.point(at),
            align,
            text,
            font,
            colour::rgb(fill.r, fill.g, fill.b),
        );
    }
}

/// The logical drawing, fitted into a pixel rectangle: scale and origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    /// Points per millimetre of the drawing.
    pub scale: f32,
    /// Screen position of the drawing origin.
    pub origin: Pos2,
}

/// Fits the logical area into `pixel_rect`, leaving a small margin.
#[must_use]
pub fn fit(pixel_rect: egui::Rect) -> Fit {
    let scale = (pixel_rect.width() / LOGICAL.0)
        .min(pixel_rect.height() / LOGICAL.1)
        .max(0.1);
    let size = Vec2::new(LOGICAL.0 * scale, LOGICAL.1 * scale);
    Fit {
        scale,
        origin: pixel_rect.center() - size * 0.5,
    }
}

/// The logical drawing area, in millimetres.
#[must_use]
pub fn logical_area() -> Rect {
    Rect::from_min_size(Pt::new(0.0, 0.0), LOGICAL.0, LOGICAL.1)
}

/// Point of the drawing under a screen position.
#[must_use]
pub fn to_drawing(fit: Fit, position: Pos2) -> Pt {
    Pt::new(
        (position.x - fit.origin.x) / fit.scale,
        (position.y - fit.origin.y) / fit.scale,
    )
}

/// Screen rectangle of a drawing rectangle.
#[must_use]
pub fn to_screen(fit: Fit, rect: Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        Pos2::new(
            fit.origin.x + rect.min.x * fit.scale,
            fit.origin.y + rect.min.y * fit.scale,
        ),
        Pos2::new(
            fit.origin.x + rect.max.x * fit.scale,
            fit.origin.y + rect.max.y * fit.scale,
        ),
    )
}

/// Vector helper kept here so callers do not have to know the scale.
#[must_use]
pub fn pixels(millimetres: f32) -> Vec2 {
    Vec2::splat(millimetres * SCALE)
}

/// Drawing ink for a theme: the tokens, converted once.
///
/// A widget never picks a colour; it asks the palette and this function turns the
/// answer into the toolkit-free type the drawing crate understands.
#[must_use]
pub fn ink(theme: &punchi_fluid_theme::Theme) -> punchi_fluid_draw::Ink {
    use punchi_fluid_theme::Token;

    let of = |token: Token| {
        let colour = theme.colour(token);
        Colour::new(colour.r(), colour.g(), colour.b())
    };
    let active = theme.palette.blend(Token::Surface, Token::Accent, 0.22);
    punchi_fluid_draw::Ink {
        line: of(Token::SymbolLine),
        dim: of(Token::SymbolDim),
        muted: of(Token::TextMuted),
        text: of(Token::Text),
        supply: of(Token::SupplyLine),
        work: of(Token::WorkLine),
        accent: Colour::new(active.r(), active.g(), active.b()),
        background: of(Token::Sheet),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_logical_area_fits_inside_the_pixel_rect() {
        let screen = egui::Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(400.0, 300.0));
        let fitted = fit(screen);
        let back = to_screen(fitted, logical_area());
        assert!(back.width() <= screen.width() + 0.01);
        assert!(back.height() <= screen.height() + 0.01);
        assert!(screen.contains(back.center()));
    }

    #[test]
    fn a_pointer_position_maps_back_to_the_drawing() {
        let screen = egui::Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(400.0, 300.0));
        let fitted = fit(screen);
        let area = logical_area();
        let point = to_screen(fitted, area);
        let back = to_drawing(fitted, point.center());
        assert!((back.x - area.center().x).abs() < 0.01);
        assert!((back.y - area.center().y).abs() < 0.01);
    }
}
