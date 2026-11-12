// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Fitting a drawing onto a page: the scale, the bounds and the surface that maps one
// rectangle of the drawing onto another.

use crate::surface::{Anchor, Colour, Pt, Rect, Stroke, Surface};

/// A surface that measures instead of drawing.
///
/// The export runs the real drawing through it once, so the bounds include every symbol,
/// caption, designation and readout the page will carry. Text is measured with a
/// conservative advance width, because the exact glyph metrics belong to the font and the
/// point of the margin is that a fit never crops a name.
#[derive(Debug, Default, Clone, Copy)]
pub struct Bounds {
    rect: Option<Rect>,
}

impl Bounds {
    /// The union of everything measured so far, or `None` when nothing was drawn.
    #[must_use]
    pub fn bounds(self) -> Option<Rect> {
        self.rect
    }

    fn add(&mut self, rect: Rect) {
        self.rect = Some(self.rect.map_or(rect, |current| {
            Rect::from_corners(
                Pt::new(current.min.x.min(rect.min.x), current.min.y.min(rect.min.y)),
                Pt::new(current.max.x.max(rect.max.x), current.max.y.max(rect.max.y)),
            )
        }));
    }
}

impl Surface for Bounds {
    fn line(&mut self, points: &[Pt], _stroke: Stroke) {
        for point in points {
            self.add(Rect {
                min: *point,
                max: *point,
            });
        }
    }

    fn polygon(&mut self, points: &[Pt], _fill: Option<Colour>, _stroke: Option<Stroke>) {
        self.line(points, Stroke::new(0.0, Colour::BLACK));
    }

    fn rect(&mut self, rect: Rect, _corner: f32, _fill: Option<Colour>, _stroke: Option<Stroke>) {
        self.add(rect);
    }

    fn circle(&mut self, centre: Pt, radius: f32, _fill: Option<Colour>, _stroke: Option<Stroke>) {
        self.add(Rect::from_center_size(centre, radius * 2.0, radius * 2.0));
    }

    fn arc(&mut self, centre: Pt, radius: f32, _start: f32, _end: f32, _stroke: Stroke) {
        self.add(Rect::from_center_size(centre, radius * 2.0, radius * 2.0));
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, _colour: Colour, anchor: Anchor) {
        // A little over half an em per character is the usual conservative advance for a
        // sans-serif face; the height covers the ascenders and descenders around the cap.
        #[allow(clippy::cast_precision_loss)]
        let width = text.chars().count() as f32 * size * 0.65;
        let min_x = match anchor {
            Anchor::Start => at.x,
            Anchor::Middle => at.x - width * 0.5,
            Anchor::End => at.x - width,
        };
        self.add(Rect::from_corners(
            Pt::new(min_x, at.y - size * 0.2),
            Pt::new(min_x + width, at.y + size * 1.2),
        ));
    }
}

/// Scale that fits `from` inside `to` without enlarging it.
///
/// A drawing already smaller than the page keeps its size and is centred by
/// [`Scaled`]; a larger one is reduced until it fits. `1.0` is returned for an empty
/// rectangle or a non-finite measurement, so a bad area never produces a zero scale.
#[must_use]
pub fn fit_scale(from: Rect, to: Rect) -> f32 {
    if from.is_empty() || to.is_empty() {
        return 1.0;
    }
    let scale = (to.width() / from.width())
        .min(to.height() / from.height())
        .min(1.0);
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

/// A surface that maps one rectangle of the drawing onto another.
///
/// The export computes the bounds of the sheet and hands the page a fitted surface, so
/// every primitive, stroke width and text size is scaled by the same factor. The screen
/// never uses it: a screen has a zoom of its own.
pub struct Scaled<'a> {
    inner: &'a mut dyn Surface,
    scale: f32,
    offset: Pt,
}

impl std::fmt::Debug for Scaled<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Scaled")
            .field("scale", &self.scale)
            .field("offset", &self.offset)
            .finish_non_exhaustive()
    }
}

impl<'a> Scaled<'a> {
    /// Fits `from` into `to`, centred, without enlarging it.
    #[must_use]
    pub fn fit(inner: &'a mut dyn Surface, from: Rect, to: Rect) -> Self {
        let scale = fit_scale(from, to);
        let offset = Pt::new(
            to.min.x + (to.width() - from.width() * scale) * 0.5 - from.min.x * scale,
            to.min.y + (to.height() - from.height() * scale) * 0.5 - from.min.y * scale,
        );
        Self {
            inner,
            scale,
            offset,
        }
    }

    /// The factor applied to the drawing.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    fn point(&self, point: Pt) -> Pt {
        Pt::new(
            point.x * self.scale + self.offset.x,
            point.y * self.scale + self.offset.y,
        )
    }

    fn points(&self, points: &[Pt]) -> Vec<Pt> {
        points.iter().map(|point| self.point(*point)).collect()
    }

    fn stroke(&self, stroke: Stroke) -> Stroke {
        Stroke {
            width: stroke.width * self.scale,
            ..stroke
        }
    }

    fn optional_stroke(&self, stroke: Option<Stroke>) -> Option<Stroke> {
        stroke.map(|stroke| self.stroke(stroke))
    }
}

impl Surface for Scaled<'_> {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        self.inner.line(&self.points(points), self.stroke(stroke));
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        self.inner
            .polygon(&self.points(points), fill, self.optional_stroke(stroke));
    }

    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        self.inner.rect(
            Rect::from_corners(self.point(rect.min), self.point(rect.max)),
            corner * self.scale,
            fill,
            self.optional_stroke(stroke),
        );
    }

    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        self.inner.circle(
            self.point(centre),
            radius * self.scale,
            fill,
            self.optional_stroke(stroke),
        );
    }

    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke) {
        self.inner.arc(
            self.point(centre),
            radius * self.scale,
            start,
            end,
            self.stroke(stroke),
        );
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        self.inner
            .text(self.point(at), text, size * self.scale, colour, anchor);
    }

    fn device_scale(&self) -> f32 {
        self.inner.device_scale() * self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{Call, Recording};

    #[test]
    fn fitting_never_enlarges_and_centres_the_source() {
        let source = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 50.0);
        let page = Rect::from_min_size(Pt::new(0.0, 0.0), 200.0, 200.0);
        assert!((fit_scale(source, page) - 1.0).abs() < f32::EPSILON);
        assert!((fit_scale(Rect::default(), page) - 1.0).abs() < f32::EPSILON);

        let large = Rect::from_min_size(Pt::new(0.0, 0.0), 400.0, 200.0);
        let page = Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 100.0);
        assert!((fit_scale(large, page) - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn a_scaled_surface_maps_points_strokes_and_text() {
        let mut recording = Recording::default();
        {
            let mut scaled = Scaled::fit(
                &mut recording,
                Rect::from_min_size(Pt::new(0.0, 0.0), 400.0, 200.0),
                Rect::from_min_size(Pt::new(0.0, 0.0), 100.0, 100.0),
            );
            assert!((scaled.scale() - 0.25).abs() < f32::EPSILON);
            scaled.line(
                &[Pt::new(0.0, 0.0), Pt::new(400.0, 200.0)],
                Stroke::new(4.0, Colour::BLACK),
            );
            scaled.text(Pt::new(0.0, 0.0), "x", 8.0, Colour::BLACK, Anchor::Start);
        }

        let calls = recording.calls();
        assert!(matches!(calls[0], Call::Line { .. }), "{:?}", calls[0]);
        if let Call::Line { points, stroke } = &calls[0] {
            assert!((points[0].x - 0.0).abs() < f32::EPSILON);
            assert!((points[0].y - 25.0).abs() < f32::EPSILON);
            assert!((points[1].x - 100.0).abs() < f32::EPSILON);
            assert!((points[1].y - 75.0).abs() < f32::EPSILON);
            assert!((stroke.width - 1.0).abs() < f32::EPSILON);
        }
        assert!(matches!(calls[1], Call::Text { .. }), "{:?}", calls[1]);
        if let Call::Text { at, size, .. } = &calls[1] {
            assert!((at.y - 25.0).abs() < f32::EPSILON);
            assert!((size - 2.0).abs() < f32::EPSILON);
        }
    }
}
