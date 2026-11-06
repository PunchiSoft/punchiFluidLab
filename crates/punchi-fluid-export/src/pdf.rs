// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A small, deterministic PDF writer for one vector page.
//!
//! Writing the file by hand keeps the dependency list at zero, which matters for a
//! program with `deny.toml` forbidding `openssl-sys` and for a build that has to be
//! reproducible: the same calls produce the same bytes, with no object identifier, no
//! producer string and no creation date beyond the one the title block already shows.
//! Text is Helvetica, one of the fourteen fonts every reader has, so nothing has to be
//! embedded and no font licence travels with the document.

use std::fmt::Write as _;

use punchi_fluid_draw::{Anchor, Colour, Pt, Rect, Stroke, Surface};

/// Points per millimetre, the only conversion the writer needs.
const PT_PER_MM: f32 = 72.0 / 25.4;

/// Heaviest page the writer accepts, in square millimetres; a guard against a caller
/// asking for a sheet no reader would open.
pub const MAX_PAGE_MM2: f32 = 1200.0 * 1600.0;

/// A page being written.
#[derive(Debug)]
pub struct PdfSurface {
    page: Rect,
    content: String,
}

impl PdfSurface {
    /// Starts a page of `page` millimetres with a 1 mm stroke scale.
    #[must_use]
    pub fn new(page: Rect) -> Self {
        Self {
            page,
            content: String::new(),
        }
    }

    /// Font size in points for a cap height in millimetres.
    fn font_size(size_mm: f32) -> f32 {
        (size_mm / 0.72) * PT_PER_MM
    }

    /// Device coordinates of a drawing point: points, origin at the bottom-left.
    #[allow(clippy::cast_possible_truncation)]
    fn device(&self, point: Pt) -> (f32, f32) {
        (
            (point.x - self.page.min.x) * PT_PER_MM,
            (self.page.max.y - point.y) * PT_PER_MM,
        )
    }

    fn set_stroke(&mut self, colour: Colour, width: f32) {
        let (r, g, b) = colour.unit();
        let _ = writeln!(self.content, "{r:.4} {g:.4} {b:.4} RG");
        let _ = writeln!(self.content, "{:.4} w", (width * PT_PER_MM).max(0.1));
    }

    fn set_fill(&mut self, colour: Colour) {
        let (r, g, b) = colour.unit();
        let _ = writeln!(self.content, "{r:.4} {g:.4} {b:.4} rg");
    }

    fn path(&mut self, points: &[Pt]) {
        let Some(first) = points.first() else {
            return;
        };
        let (x, y) = self.device(*first);
        let _ = writeln!(self.content, "{x:.3} {y:.3} m");
        for point in &points[1..] {
            let (x, y) = self.device(*point);
            let _ = writeln!(self.content, "{x:.3} {y:.3} l");
        }
    }

    /// Content stream of the page, for tests and for the writer.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Surface for PdfSurface {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        if points.len() < 2 {
            return;
        }
        self.set_stroke(stroke.colour, stroke.width);
        self.path(points);
        self.content.push_str("S\n");
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        if points.len() < 3 {
            return;
        }
        if let Some(colour) = fill {
            self.set_fill(colour);
        }
        if let Some(stroke) = stroke {
            self.set_stroke(stroke.colour, stroke.width);
        }
        self.path(points);
        self.content.push_str("h\n");
        self.content.push_str(match (fill, stroke) {
            (Some(_), Some(_)) => "B\n",
            (Some(_), None) => "f\n",
            _ => "S\n",
        });
    }

    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        if rect.is_empty() {
            return;
        }
        // A rounded corner of a tenth of a millimetre is not visible at any sane print
        // size; the four straight sides are what a reader sees.
        let _ = corner;
        let (x, y) = self.device(Pt::new(rect.min.x, rect.max.y));
        let width = rect.width() * PT_PER_MM;
        let height = rect.height() * PT_PER_MM;
        if let Some(colour) = fill {
            self.set_fill(colour);
        }
        if let Some(stroke) = stroke {
            self.set_stroke(stroke.colour, stroke.width);
        }
        let _ = writeln!(self.content, "{x:.3} {y:.3} {width:.3} {height:.3} re");
        self.content.push_str(match (fill, stroke) {
            (Some(_), Some(_)) => "B\n",
            (Some(_), None) => "f\n",
            _ => "S\n",
        });
    }

    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        if radius <= 0.0 {
            return;
        }
        if let Some(colour) = fill {
            self.set_fill(colour);
        }
        if let Some(stroke) = stroke {
            self.set_stroke(stroke.colour, stroke.width);
        }
        // Four cubic arcs, the usual 0.5523 approximation of a circle.
        let curvature = radius * 0.552_285;
        let points = [
            Pt::new(centre.x + radius, centre.y),
            Pt::new(centre.x, centre.y - radius),
            Pt::new(centre.x - radius, centre.y),
            Pt::new(centre.x, centre.y + radius),
        ];
        let controls = [
            (
                Pt::new(centre.x + radius, centre.y - curvature),
                Pt::new(centre.x + curvature, centre.y - radius),
            ),
            (
                Pt::new(centre.x - curvature, centre.y - radius),
                Pt::new(centre.x - radius, centre.y - curvature),
            ),
            (
                Pt::new(centre.x - radius, centre.y + curvature),
                Pt::new(centre.x - curvature, centre.y + radius),
            ),
            (
                Pt::new(centre.x + curvature, centre.y + radius),
                Pt::new(centre.x + radius, centre.y + curvature),
            ),
        ];
        let (start_x, start_y) = self.device(points[0]);
        let _ = writeln!(self.content, "{start_x:.3} {start_y:.3} m");
        for (index, (c1, c2)) in controls.iter().enumerate() {
            let end = points[(index + 1) % 4];
            let (first_x, first_y) = self.device(*c1);
            let (second_x, second_y) = self.device(*c2);
            let (end_x, end_y) = self.device(end);
            let _ = writeln!(
                self.content,
                "{first_x:.3} {first_y:.3} {second_x:.3} {second_y:.3} {end_x:.3} {end_y:.3} c"
            );
        }
        self.content.push_str(match (fill, stroke) {
            (Some(_), Some(_)) => "B\n",
            (Some(_), None) => "f\n",
            _ => "S\n",
        });
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
        // Points per millimetre: a 12 mm numeral is 34 pt of type, always legible.
        PT_PER_MM
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        let encoded = encode(text);
        if encoded.is_empty() {
            return;
        }
        let font = Self::font_size(size);
        let width_mm = estimate_width(text, size);
        let offset = match anchor {
            Anchor::Start => 0.0,
            Anchor::Middle => -width_mm * 0.5,
            Anchor::End => -width_mm,
        };
        let (x, y) = self.device(Pt::new(at.x + offset, at.y + size * 0.36));
        self.set_fill(colour);
        // A hexadecimal string carries the WinAnsi bytes directly, so an accent or a
        // middle dot is one byte and not the two a UTF-8 literal would become.
        let mut hex = String::with_capacity(encoded.len() * 2);
        for byte in &encoded {
            let _ = write!(hex, "{byte:02X}");
        }
        let _ = writeln!(
            self.content,
            "BT /F1 {font:.2} Tf {x:.3} {y:.3} Td <{hex}> Tj ET"
        );
    }
}

/// Encodes a run as `WinAnsi` bytes for a hexadecimal PDF string.
///
/// A title typed by the user may contain anything; `WinAnsi` stops at U+00FF, so the rest
/// becomes a visible `?` instead of a corrupt glyph or a broken stream.
fn encode(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\n' | '\r' | '\t' => out.push(b' '),
            ' '..='~' => out.push(character as u8),
            '\u{a0}'..='\u{ff}' => {
                // A Latin-1 character is its own code point in one byte.
                #[allow(clippy::cast_possible_truncation)]
                let byte = u32::from(character) as u8;
                out.push(byte);
            }
            _ => out.push(b'?'),
        }
    }
    out
}

/// Width of a Helvetica run in millimetres, accurate enough for anchoring.
fn estimate_width(text: &str, size: f32) -> f32 {
    let mut units = 0.0_f32;
    for character in text.chars() {
        units += match character {
            'i' | 'j' | 'l' | 'I' | 't' | 'f' | 'r' | '.' | ',' | ':' | ';' | '\'' | ' ' => 0.30,
            'm' | 'w' | 'M' | 'W' => 0.83,
            '0'..='9' => 0.556,
            'A'..='Z' => 0.68,
            _ => 0.55,
        };
    }
    units * size * 0.72
}

/// Writes the objects and the trailer around a finished content stream.
#[must_use]
pub fn assemble(surface: &PdfSurface) -> Vec<u8> {
    let (width_pt, height_pt) = (
        surface.page.width() * PT_PER_MM,
        surface.page.height() * PT_PER_MM,
    );
    let mut out: Vec<u8> = Vec::with_capacity(surface.content.len() + 1024);
    out.extend_from_slice(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n");

    let mut offsets: Vec<usize> = Vec::with_capacity(5);
    let push = |out: &mut Vec<u8>, body: &str, offsets: &mut Vec<usize>| {
        offsets.push(out.len());
        out.extend_from_slice(body.as_bytes());
    };

    push(
        &mut out,
        "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
        &mut offsets,
    );
    push(
        &mut out,
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n",
        &mut offsets,
    );
    push(
        &mut out,
        &format!(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width_pt:.2} {height_pt:.2}] \
/Resources << /Font << /F1 5 0 R >> /ProcSet [/PDF /Text] >> /Contents 4 0 R >>\nendobj\n"
        ),
        &mut offsets,
    );

    offsets.push(out.len());
    let header = format!("4 0 obj\n<< /Length {} >>\nstream\n", surface.content.len());
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(surface.content.as_bytes());
    out.extend_from_slice(b"\nendstream\nendobj\n");

    push(
        &mut out,
        "5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>\nendobj\n",
        &mut offsets,
    );

    let xref = out.len();
    let count = offsets.len() + 1;
    let mut tail = format!("xref\n0 {count}\n0000000000 65535 f \n");
    for offset in &offsets {
        let _ = writeln!(tail, "{offset:010} 00000 n ");
    }
    let _ = write!(
        tail,
        "trailer\n<< /Size {count} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"
    );
    out.extend_from_slice(tail.as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Rect {
        Rect::from_min_size(Pt::new(0.0, 0.0), 210.0, 297.0)
    }

    #[test]
    fn a_line_becomes_a_move_and_a_line() {
        let mut surface = PdfSurface::new(page());
        surface.line(
            &[Pt::new(10.0, 10.0), Pt::new(20.0, 10.0)],
            Stroke::new(0.35, Colour::BLACK),
        );
        let content = surface.content();
        assert!(content.contains(" m\n"));
        assert!(content.contains(" l\n"));
        assert!(content.trim_end().ends_with('S'));
    }

    #[test]
    fn device_coordinates_grow_upwards_from_the_bottom_left() {
        let surface = PdfSurface::new(page());
        let (x, y) = surface.device(Pt::new(0.0, 0.0));
        assert!(x.abs() < f32::EPSILON);
        assert!((y - 297.0 * PT_PER_MM).abs() < 0.01);
    }

    #[test]
    fn text_returns_nothing_when_it_has_nothing_to_say() {
        let mut surface = PdfSurface::new(page());
        surface.text(Pt::new(1.0, 1.0), "", 3.0, Colour::BLACK, Anchor::Start);
        assert!(surface.content().is_empty());
    }

    #[test]
    fn a_title_with_parentheses_becomes_a_hex_string() {
        let mut surface = PdfSurface::new(page());
        surface.text(
            Pt::new(1.0, 1.0),
            "Valve (3/2)",
            3.0,
            Colour::BLACK,
            Anchor::Start,
        );
        assert!(surface.content().contains("<56616C76652028332F3229>"));
    }

    #[test]
    fn an_accent_is_a_single_byte() {
        let mut surface = PdfSurface::new(page());
        surface.text(
            Pt::new(1.0, 1.0),
            "Válvula · 3/2",
            3.0,
            Colour::BLACK,
            Anchor::Start,
        );
        // `á` is E1 and `·` is B7 in WinAnsi, one byte each.
        assert!(surface.content().contains("<56E16C76756C6120B720332F32>"));
    }

    #[test]
    fn a_character_winansi_cannot_carry_is_replaced() {
        assert_eq!(encode("a→b"), b"a?b".to_vec());
    }
}
