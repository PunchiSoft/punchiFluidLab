// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The raster device: the same drawing, on pixels.
//!
//! `tiny-skia` fills and strokes the paths; `ab_glyph` draws the text with a system
//! font. Nothing here decides what the drawing contains: it receives the calls of
//! [`punchi_fluid_draw::Surface`] and puts them on a pixmap.

use ab_glyph::{Font, FontVec, GlyphId, PxScale, ScaleFont};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Shader, Stroke as SkStroke, Transform};

use punchi_fluid_draw::{Anchor, Colour, Pt, Rect, Stroke, Surface};

use crate::ExportError;

/// Largest bitmap the raster device will allocate, in pixels.
pub const MAX_PIXELS: u64 = 80_000_000;
/// Highest resolution it accepts, in dots per inch.
pub const MAX_DPI: f32 = 600.0;
/// Ratio between the em of a font and the cap height the drawing speaks in.
const CAP_RATIO: f32 = 1.4;

/// A pixmap being painted with the drawing calls.
pub struct PngSurface {
    pixmap: Pixmap,
    page: Rect,
    /// Pixels per millimetre.
    scale: f32,
    font: Option<FontVec>,
}

impl std::fmt::Debug for PngSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PngSurface")
            .field("width", &self.pixmap.width())
            .field("height", &self.pixmap.height())
            .field("scale", &self.scale)
            .field("page", &self.page)
            .field("font", &self.font.is_some())
            .finish()
    }
}

impl PngSurface {
    /// Prepares a white page of `page` millimetres at `dpi`.
    ///
    /// # Errors
    ///
    /// Refuses a resolution above [`MAX_DPI`] or a bitmap above [`MAX_PIXELS`] before
    /// allocating anything.
    pub fn new(page: Rect, dpi: f32) -> Result<Self, ExportError> {
        if !dpi.is_finite() || dpi <= 0.0 || dpi > MAX_DPI {
            return Err(ExportError::Refused(format!(
                "{dpi:.0} dpi is outside the supported range 72-{MAX_DPI:.0}"
            )));
        }
        let scale = dpi / 25.4;
        let width = (page.width() * scale).round();
        let height = (page.height() * scale).round();
        if width < 1.0 || height < 1.0 {
            return Err(ExportError::EmptyContent("the page"));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (width, height) = (width as u32, height as u32);
        if u64::from(width) * u64::from(height) > MAX_PIXELS {
            return Err(ExportError::Refused(format!(
                "{width} × {height} px is above the {} megapixel budget",
                MAX_PIXELS / 1_000_000
            )));
        }
        let mut pixmap = Pixmap::new(width, height).ok_or_else(|| {
            ExportError::Refused(format!("cannot allocate {width} × {height} px"))
        })?;
        pixmap.fill(tiny_skia::Color::WHITE);
        Ok(Self {
            pixmap,
            page,
            scale,
            font: load_font(),
        })
    }

    /// The finished bitmap.
    #[must_use]
    pub fn pixmap(&self) -> &Pixmap {
        &self.pixmap
    }

    /// Pixel size of the bitmap.
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.pixmap.width(), self.pixmap.height())
    }

    /// Encodes the bitmap as a PNG.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::Refused`] when the encoder rejects the image.
    pub fn encode(self) -> Result<Vec<u8>, ExportError> {
        self.pixmap
            .encode_png()
            .map_err(|error| ExportError::Refused(error.to_string()))
    }

    fn x(&self, point: Pt) -> f32 {
        (point.x - self.page.min.x) * self.scale
    }

    fn y(&self, point: Pt) -> f32 {
        (point.y - self.page.min.y) * self.scale
    }

    fn paint(colour: Colour) -> Paint<'static> {
        Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(
                colour.r, colour.g, colour.b, 255,
            )),
            anti_alias: true,
            ..Paint::default()
        }
    }

    fn outline(&self, points: &[Pt], close: bool) -> Option<tiny_skia::Path> {
        let mut builder = PathBuilder::new();
        let first = points.first()?;
        builder.move_to(self.x(*first), self.y(*first));
        for point in &points[1..] {
            builder.line_to(self.x(*point), self.y(*point));
        }
        if close {
            builder.close();
        }
        builder.finish()
    }

    fn stroke_style(stroke: Stroke, scale: f32) -> SkStroke {
        SkStroke {
            width: (stroke.width * scale).max(0.4),
            line_join: tiny_skia::LineJoin::Miter,
            ..SkStroke::default()
        }
    }

    /// Blends one glyph's coverage into the pixmap.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn blit_glyph(&mut self, glyph: &ab_glyph::OutlinedGlyph, colour: Colour) {
        // `px_bounds` is already in positioned pixel space and `draw` yields offsets
        // inside that box, so the box corner is the destination and nothing else.
        let bounds = glyph.px_bounds();
        let left = bounds.min.x;
        let top = bounds.min.y;
        let (width, height) = (self.pixmap.width(), self.pixmap.height());
        let pixels = self.pixmap.pixels_mut();
        glyph.draw(|dx, dy, coverage| {
            if coverage <= 0.0 {
                return;
            }
            let px = left + dx as f32;
            let py = top + dy as f32;
            if px < 0.0 || py < 0.0 {
                return;
            }
            let (px, py) = (px as u32, py as u32);
            if px >= width || py >= height {
                return;
            }
            let index = (py * width + px) as usize;
            let Some(pixel) = pixels.get_mut(index) else {
                return;
            };
            let blend = |source: u8, destination: u8| {
                let a = coverage;
                (f32::from(source) * a + f32::from(destination) * (1.0 - a))
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            let current = pixel.demultiply();
            let result = tiny_skia::ColorU8::from_rgba(
                blend(colour.r, current.red()),
                blend(colour.g, current.green()),
                blend(colour.b, current.blue()),
                255,
            );
            *pixel = tiny_skia::PremultipliedColorU8::from_rgba(
                result.red(),
                result.green(),
                result.blue(),
                255,
            )
            .unwrap_or(*pixel);
        });
    }
}

impl Surface for PngSurface {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        if points.len() < 2 {
            return;
        }
        let Some(path) = self.outline(points, false) else {
            return;
        };
        let style = Self::stroke_style(stroke, self.scale);
        self.pixmap.stroke_path(
            &path,
            &Self::paint(stroke.colour),
            &style,
            Transform::identity(),
            None,
        );
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        if points.len() < 3 {
            return;
        }
        let Some(path) = self.outline(points, true) else {
            return;
        };
        if let Some(colour) = fill {
            self.pixmap.fill_path(
                &path,
                &Self::paint(colour),
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
        if let Some(stroke) = stroke {
            let style = Self::stroke_style(stroke, self.scale);
            self.pixmap.stroke_path(
                &path,
                &Self::paint(stroke.colour),
                &style,
                Transform::identity(),
                None,
            );
        }
    }

    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        if rect.is_empty() {
            return;
        }
        let _ = corner;
        let (x, y) = (self.x(rect.min), self.y(rect.min));
        let width = rect.width() * self.scale;
        let height = rect.height() * self.scale;
        let Some(shape) = tiny_skia::Rect::from_xywh(x, y, width, height) else {
            return;
        };
        let path = PathBuilder::from_rect(shape);
        if let Some(colour) = fill {
            self.pixmap.fill_path(
                &path,
                &Self::paint(colour),
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
        if let Some(stroke) = stroke {
            let style = Self::stroke_style(stroke, self.scale);
            self.pixmap.stroke_path(
                &path,
                &Self::paint(stroke.colour),
                &style,
                Transform::identity(),
                None,
            );
        }
    }

    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        if radius <= 0.0 {
            return;
        }
        let mut builder = PathBuilder::new();
        builder.push_circle(self.x(centre), self.y(centre), radius * self.scale);
        let Some(path) = builder.finish() else {
            return;
        };
        if let Some(colour) = fill {
            self.pixmap.fill_path(
                &path,
                &Self::paint(colour),
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
        if let Some(stroke) = stroke {
            let style = Self::stroke_style(stroke, self.scale);
            self.pixmap.stroke_path(
                &path,
                &Self::paint(stroke.colour),
                &style,
                Transform::identity(),
                None,
            );
        }
    }

    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke) {
        if radius <= 0.0 {
            return;
        }
        let samples = 32_i16;
        let mut points = Vec::with_capacity(33);
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

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        let Some(font) = self.font.as_ref() else {
            return;
        };
        let px = size * CAP_RATIO * self.scale;
        if px < 1.0 {
            return;
        }
        let scaled = font.as_scaled(PxScale::from(px));
        let width: f32 = text
            .chars()
            .map(|character| {
                let id = scaled.glyph_id(character);
                scaled.h_advance(id)
            })
            .sum();
        let x = self.x(at)
            + match anchor {
                Anchor::Start => 0.0,
                Anchor::Middle => -width * 0.5,
                Anchor::End => -width,
            };
        let baseline = self.y(at) + size * self.scale * 0.36;
        let mut caret = x;
        // Outlines are collected first: drawing them needs `&mut self`, and the font is
        // borrowed from `self`.
        let mut outlines = Vec::new();
        for character in text.chars() {
            let id: GlyphId = scaled.glyph_id(character);
            let glyph =
                id.with_scale_and_position(PxScale::from(px), ab_glyph::point(caret, baseline));
            if let Some(outlined) = font.outline_glyph(glyph) {
                outlines.push(outlined);
            }
            caret += scaled.h_advance(id);
        }
        for outlined in outlines {
            self.blit_glyph(&outlined, colour);
        }
    }
}

/// Loads a sans-serif face from the system font database.
///
/// A font is an input like any other: when the machine has none, text is skipped and the
/// diagram keeps its shapes, because an export without labels is still a diagram while a
/// crash is not.
fn load_font() -> Option<FontVec> {
    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    // `fontdb` does not read fontconfig, so a generic family may be unresolved on a
    // machine whose default is set elsewhere. Try the usual suspects and, as a last
    // resort, any face the database found.
    let named = [
        fontdb::Family::SansSerif,
        fontdb::Family::Name("DejaVu Sans"),
        fontdb::Family::Name("Noto Sans"),
        fontdb::Family::Name("Liberation Sans"),
    ];
    let id = database
        .query(&fontdb::Query {
            families: &named,
            weight: fontdb::Weight::NORMAL,
            style: fontdb::Style::Normal,
            stretch: fontdb::Stretch::Normal,
        })
        .or_else(|| database.faces().next().map(|face| face.id))?;
    let face = database.face(id)?;
    let data = match &face.source {
        fontdb::Source::Binary(binary) => binary.as_ref().as_ref().to_vec(),
        fontdb::Source::File(path) => std::fs::read(path).ok()?,
        fontdb::Source::SharedFile(_path, binary) => binary.as_ref().as_ref().to_vec(),
    };
    FontVec::try_from_vec_and_index(data, face.index).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Rect {
        Rect::from_min_size(Pt::new(0.0, 0.0), 210.0, 297.0)
    }

    #[test]
    fn a4_at_300_dpi_is_the_expected_bitmap() {
        let surface = PngSurface::new(page(), 300.0).expect("the page is allowed");
        assert_eq!(surface.size(), (2480, 3508));
    }

    #[test]
    fn a3_landscape_at_300_dpi_is_the_expected_bitmap() {
        let surface = PngSurface::new(Rect::from_min_size(Pt::new(0.0, 0.0), 420.0, 297.0), 300.0)
            .expect("the page is allowed");
        assert_eq!(surface.size(), (4961, 3508));
    }

    #[test]
    fn a_resolution_above_the_limit_is_refused_before_allocating() {
        assert!(matches!(
            PngSurface::new(page(), 1200.0),
            Err(ExportError::Refused(_))
        ));
        assert!(matches!(
            PngSurface::new(page(), 0.0),
            Err(ExportError::Refused(_))
        ));
    }

    #[test]
    fn a_bitmap_above_the_budget_is_refused_before_allocating() {
        // A2 at 600 dpi is 9921 × 14031, above the 80 megapixel budget.
        let error = PngSurface::new(Rect::from_min_size(Pt::new(0.0, 0.0), 420.0, 594.0), 600.0)
            .expect_err("too many pixels");
        assert!(matches!(error, ExportError::Refused(_)), "{error}");
    }

    #[test]
    fn a_line_marks_the_pixels_it_crosses() {
        let mut surface = PngSurface::new(page(), 72.0).expect("the page is allowed");
        surface.line(
            &[Pt::new(20.0, 20.0), Pt::new(80.0, 20.0)],
            Stroke::new(1.0, Colour::BLACK),
        );
        let pixmap = surface.pixmap();
        // 20 mm at 72 dpi is about 57 px.
        let pixel = pixmap.pixel(80, 57).expect("inside the page");
        assert!(pixel.demultiply().red() < 40, "the line must be dark");
    }

    #[test]
    fn the_same_calls_produce_the_same_bytes() {
        let render = || {
            let mut surface = PngSurface::new(page(), 72.0).expect("the page is allowed");
            surface.rect(
                Rect::from_min_size(Pt::new(10.0, 10.0), 50.0, 20.0),
                0.0,
                None,
                Some(Stroke::new(0.35, Colour::BLACK)),
            );
            surface.encode().expect("encodes")
        };
        assert_eq!(render(), render());
    }
}
