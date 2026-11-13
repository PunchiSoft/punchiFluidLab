// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A small, deterministic SVG writer for one vector page.
//!
//! The same page the PDF receives becomes text a browser, an editor or a plotter opens
//! without losing the lines: an SVG sheet can be zoomed into a symbol, measured, or
//! imported into a document. The writer is hand-made for the same reasons as the PDF one —
//! no dependency, no producer string, no identifier, no date — so two exports of the same
//! circuit are byte for byte the same file.
//!
//! Units are millimetres throughout: the `viewBox` is the paper in millimetres, so a
//! recipient that opens the file in a CAD tool measures what the sheet says.

use std::fmt::Write as _;

use punchi_fluid_draw::{Anchor, Colour, Pt, Rect, Stroke, Surface};

/// A page being written as SVG.
#[derive(Debug)]
pub struct SvgSurface {
    page: Rect,
    body: String,
}

impl SvgSurface {
    /// Starts a page of `page` millimetres.
    #[must_use]
    pub fn new(page: Rect) -> Self {
        Self {
            page,
            body: String::new(),
        }
    }

    /// The body of the document, without the wrapping `<svg>` element.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Millimetre coordinates in the view box; the drawing already measures from the top.
    fn device(&self, point: Pt) -> (f32, f32) {
        (point.x - self.page.min.x, point.y - self.page.min.y)
    }

    /// `#rrggbb`, the form every reader understands.
    fn colour(colour: Colour) -> String {
        format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b)
    }

    /// The dash array of a class, or nothing for a solid line.
    fn dash(stroke: Stroke) -> String {
        stroke
            .class
            .pattern()
            .map_or_else(String::new, |(dash, gap)| {
                format!(" stroke-dasharray=\"{dash:.3} {gap:.3}\"")
            })
    }

    /// A `points` attribute from a run of points.
    fn points(&self, points: &[Pt]) -> String {
        let mut out = String::with_capacity(points.len() * 12);
        for point in points {
            let (x, y) = self.device(*point);
            let _ = write!(out, "{x:.3},{y:.3} ");
        }
        out.trim_end().to_owned()
    }

    /// Adds a shape element with the fill and stroke it asked for.
    fn paint(&mut self, element: &str, fill: Option<Colour>, stroke: Option<Stroke>) {
        let fill = fill.map_or_else(|| "none".to_owned(), Self::colour);
        let stroke_part = stroke.map_or_else(String::new, |stroke| {
            format!(
                " stroke=\"{}\" stroke-width=\"{:.3}\"{} stroke-linecap=\"round\" stroke-linejoin=\"round\"",
                Self::colour(stroke.colour),
                stroke.width,
                Self::dash(stroke)
            )
        });
        let _ = writeln!(self.body, "<{element} fill=\"{fill}\"{stroke_part}/>");
    }
}

impl Surface for SvgSurface {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        if points.len() < 2 {
            return;
        }
        let points = self.points(points);
        self.paint(&format!("polyline points=\"{points}\""), None, Some(stroke));
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        if points.len() < 3 {
            return;
        }
        let points = self.points(points);
        self.paint(&format!("polygon points=\"{points}\""), fill, stroke);
    }

    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        if rect.is_empty() {
            return;
        }
        let (x, y) = self.device(Pt::new(rect.min.x, rect.min.y));
        let radius = if corner > 0.0 {
            format!(" rx=\"{corner:.3}\"")
        } else {
            String::new()
        };
        self.paint(
            &format!(
                "rect x=\"{x:.3}\" y=\"{y:.3}\" width=\"{:.3}\" height=\"{:.3}\"{radius}",
                rect.width(),
                rect.height()
            ),
            fill,
            stroke,
        );
    }

    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        if radius <= 0.0 {
            return;
        }
        let (x, y) = self.device(centre);
        self.paint(
            &format!("circle cx=\"{x:.3}\" cy=\"{y:.3}\" r=\"{radius:.3}\""),
            fill,
            stroke,
        );
    }

    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke) {
        // Sampled like the PDF writer samples it, so the two devices never disagree about
        // where an arc ends: one geometry, two renderings.
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

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        if text.is_empty() {
            return;
        }
        let (x, y) = self.device(at);
        let anchor = match anchor {
            Anchor::Start => "start",
            Anchor::Middle => "middle",
            Anchor::End => "end",
        };
        let _ = writeln!(
            self.body,
            "<text x=\"{x:.3}\" y=\"{y:.3}\" font-family=\"Helvetica, Arial, sans-serif\" \
font-size=\"{size:.3}\" fill=\"{}\" text-anchor=\"{anchor}\" dominant-baseline=\"middle\">{}</text>",
            Self::colour(colour),
            escape(text)
        );
    }
}

/// Escapes the five characters XML reserves.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // Control characters are not allowed in XML 1.0 at all, not even escaped.
            control if control.is_control() => out.push(' '),
            other => out.push(other),
        }
    }
    out
}

/// Wraps the body in the document element, with the paper as its size.
#[must_use]
pub fn assemble(surface: &SvgSurface) -> String {
    let mut out = String::with_capacity(surface.body.len() + 256);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{:.1}mm\" height=\"{:.1}mm\" viewBox=\"0 0 {:.1} {:.1}\">",
        surface.page.width(),
        surface.page.height(),
        surface.page.width(),
        surface.page.height()
    );
    // The paper itself, so a viewer with a dark background shows a sheet and not a hole.
    let _ = writeln!(
        out,
        "<rect x=\"0\" y=\"0\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#ffffff\"/>",
        surface.page.width(),
        surface.page.height()
    );
    out.push_str(surface.body());
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_is_well_formed_and_measures_in_millimetres() {
        let page = Rect::from_min_size(Pt::new(0.0, 0.0), 210.0, 297.0);
        let mut surface = SvgSurface::new(page);
        surface.line(
            &[Pt::new(10.0, 10.0), Pt::new(60.0, 10.0)],
            Stroke::new(0.35, Colour::BLACK).with_class(punchi_fluid_draw::LineClass::Pilot),
        );
        surface.text(
            Pt::new(20.0, 20.0),
            "Válvula & <válvula>",
            3.0,
            Colour::BLACK,
            Anchor::Middle,
        );
        let svg = assemble(&surface);
        assert!(svg.starts_with("<?xml"));
        assert!(svg.contains("viewBox=\"0 0 210.0 297.0\""));
        assert!(svg.contains("length=\"mm\"") || svg.contains("mm\""));
        assert!(
            svg.contains("stroke-dasharray="),
            "the pilot class kept its dashes"
        );
        assert!(
            svg.contains("Válvula &amp; &lt;válvula&gt;"),
            "the text is escaped: {svg}"
        );
        assert!(svg.trim_end().ends_with("</svg>"));
    }

    #[test]
    fn the_same_calls_write_the_same_file() {
        let page = Rect::from_min_size(Pt::new(0.0, 0.0), 210.0, 297.0);
        let mut first = SvgSurface::new(page);
        let mut second = SvgSurface::new(page);
        for surface in [&mut first, &mut second] {
            surface.rect(
                Rect::from_min_size(Pt::new(5.0, 5.0), 200.0, 287.0),
                0.0,
                None,
                Some(Stroke::new(0.5, Colour::BLACK)),
            );
            surface.circle(
                Pt::new(100.0, 100.0),
                10.0,
                Some(Colour::new(10, 20, 30)),
                None,
            );
        }
        assert_eq!(assemble(&first), assemble(&second));
    }
}
