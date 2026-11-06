// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The drawing surface: everything the schematic needs to be drawn once.
//!
//! Coordinates are **millimetres** of the drawing, never pixels: the screen
//! multiplies by pixels per millimetre, the paper divides by 25.4 to reach points.
//! That is what lets the sheet and the exported page come from the same calls.

/// Point (or vector) in millimetres of the drawing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pt {
    /// Horizontal coordinate, rightwards.
    pub x: f32,
    /// Vertical coordinate, downwards.
    pub y: f32,
}

impl Pt {
    /// Builds a point.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Vector from `self` to `other`.
    #[must_use]
    pub fn to(self, other: Self) -> (f32, f32) {
        (other.x - self.x, other.y - self.y)
    }

    /// Length in millimetres.
    #[must_use]
    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    /// This point moved by `distance` in both axes.
    #[must_use]
    pub fn offset(self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}

/// Axis-aligned rectangle in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// Top-left corner (smaller coordinates).
    pub min: Pt,
    /// Bottom-right corner (larger coordinates).
    pub max: Pt,
}

impl Rect {
    /// Rectangle from two opposite corners.
    #[must_use]
    pub fn from_corners(a: Pt, b: Pt) -> Self {
        Self {
            min: Pt::new(a.x.min(b.x), a.y.min(b.y)),
            max: Pt::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    /// Rectangle from its origin and size.
    #[must_use]
    pub fn from_min_size(min: Pt, width: f32, height: f32) -> Self {
        Self {
            min,
            max: Pt::new(min.x + width, min.y + height),
        }
    }

    /// Rectangle centred on `centre`.
    #[must_use]
    pub fn from_center_size(centre: Pt, width: f32, height: f32) -> Self {
        Self::from_min_size(
            Pt::new(centre.x - width * 0.5, centre.y - height * 0.5),
            width,
            height,
        )
    }

    /// Width in millimetres.
    #[must_use]
    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height in millimetres.
    #[must_use]
    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    /// Centre of the rectangle.
    #[must_use]
    pub fn center(self) -> Pt {
        Pt::new(
            self.min.x + self.width() * 0.5,
            self.min.y + self.height() * 0.5,
        )
    }

    /// `true` when the rectangle has no area.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.width() <= 0.0 || self.height() <= 0.0
    }

    /// This rectangle grown by `amount` on every side.
    #[must_use]
    pub fn expand(self, amount: f32) -> Self {
        Self {
            min: Pt::new(self.min.x - amount, self.min.y - amount),
            max: Pt::new(self.max.x + amount, self.max.y + amount),
        }
    }

    /// Point inside the rectangle at a normalised position, `0..=1`.
    #[must_use]
    pub fn point_at(self, u: f32, v: f32) -> Pt {
        Pt::new(
            self.min.x + self.width() * u,
            self.min.y + self.height() * v,
        )
    }

    /// Sub-rectangle described in normalised coordinates, `0..=1`.
    #[must_use]
    pub fn sub(self, u0: f32, v0: f32, width: f32, height: f32) -> Self {
        Self::from_min_size(
            self.point_at(u0, v0),
            self.width() * width,
            self.height() * height,
        )
    }

    /// `true` when `point` lies inside, edges included.
    #[must_use]
    pub fn contains(self, point: Pt) -> bool {
        (self.min.x..=self.max.x).contains(&point.x) && (self.min.y..=self.max.y).contains(&point.y)
    }
}

/// Opaque colour of the drawing, independent of any toolkit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Colour {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl Colour {
    /// Black, the printed default.
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    /// White, for paper.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };

    /// Builds a colour from its channels.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Hexadecimal `#rrggbb`, the form a PDF content stream wants.
    #[must_use]
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Channels normalised to `0..=1`, the form PDF operators want.
    #[must_use]
    pub fn unit(self) -> (f32, f32, f32) {
        (
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        )
    }
}

/// Kind of line, which decides the dash pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineClass {
    /// Working line: solid.
    #[default]
    Working,
    /// Pilot line: dashed.
    Pilot,
    /// Drain line: a longer dash.
    Drain,
    /// Electrical line: dotted.
    Electrical,
}

impl LineClass {
    /// Dash and gap in millimetres, or `None` for a solid line.
    #[must_use]
    pub fn pattern(self) -> Option<(f32, f32)> {
        match self {
            Self::Working => None,
            Self::Pilot => Some((1.6, 0.9)),
            Self::Drain => Some((2.6, 1.3)),
            Self::Electrical => Some((0.5, 0.7)),
        }
    }
}

/// One stroke of a drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// Width in millimetres.
    pub width: f32,
    /// Colour.
    pub colour: Colour,
    /// Class, which decides the pattern.
    pub class: LineClass,
}

impl Stroke {
    /// Solid working stroke.
    #[must_use]
    pub const fn new(width: f32, colour: Colour) -> Self {
        Self {
            width,
            colour,
            class: LineClass::Working,
        }
    }

    /// Same stroke, another class.
    #[must_use]
    pub const fn with_class(self, class: LineClass) -> Self {
        Self { class, ..self }
    }
}

/// Horizontal anchoring of a text run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    /// `at` is the left edge.
    Start,
    /// `at` is the middle.
    #[default]
    Middle,
    /// `at` is the right edge.
    End,
}

/// One recorded call, for the test that proves screen and paper agree.
#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    /// A polyline.
    Line {
        /// Points in millimetres.
        points: Vec<Pt>,
        /// Stroke.
        stroke: Stroke,
    },
    /// A filled or outlined polygon.
    Polygon {
        /// Points.
        points: Vec<Pt>,
        /// Fill.
        fill: Option<Colour>,
        /// Outline.
        stroke: Option<Stroke>,
    },
    /// A rectangle with rounded corners.
    Rect {
        /// Geometry.
        rect: Rect,
        /// Corner radius.
        corner: f32,
        /// Fill.
        fill: Option<Colour>,
        /// Outline.
        stroke: Option<Stroke>,
    },
    /// A circle.
    Circle {
        /// Centre.
        centre: Pt,
        /// Radius.
        radius: f32,
        /// Fill.
        fill: Option<Colour>,
        /// Outline.
        stroke: Option<Stroke>,
    },
    /// An arc, degrees clockwise from the positive x axis.
    Arc {
        /// Centre.
        centre: Pt,
        /// Radius.
        radius: f32,
        /// Start angle.
        start: f32,
        /// End angle.
        end: f32,
        /// Stroke.
        stroke: Stroke,
    },
    /// A text run.
    Text {
        /// Anchor point.
        at: Pt,
        /// Content.
        text: String,
        /// Cap height in millimetres.
        size: f32,
        /// Colour.
        colour: Colour,
        /// Anchoring.
        anchor: Anchor,
    },
}

/// Everything the drawing may ask of a device: screen, sheet or file.
pub trait Surface {
    /// A polyline. A single segment is a line of two points.
    fn line(&mut self, points: &[Pt], stroke: Stroke);
    /// A closed polygon, filled and/or outlined.
    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>);
    /// A rectangle, with a corner radius in millimetres (`0` for square corners).
    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>);
    /// A circle.
    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>);
    /// An arc from `start` to `end` degrees, clockwise on screen.
    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke);
    /// A text run whose `size` is the cap height in millimetres.
    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor);

    /// Device units per millimetre of the drawing.
    ///
    /// Legibility is a property of the device, not of the drawing: the same 30 mm symbol
    /// is unreadable in a 30 px thumbnail and perfectly clear on paper. The drawing asks
    /// this to decide whether a numeral is worth drawing.
    fn device_scale(&self) -> f32 {
        1.0
    }
}

/// A surface that keeps every call, for tests and for the diff between two devices.
#[derive(Debug, Default)]
pub struct Recording {
    calls: Vec<Call>,
}

impl Recording {
    /// The calls recorded so far.
    #[must_use]
    pub fn calls(&self) -> &[Call] {
        &self.calls
    }

    /// Number of calls.
    #[must_use]
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// `true` when nothing was drawn.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }
}

impl Surface for Recording {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        self.calls.push(Call::Line {
            points: points.to_vec(),
            stroke,
        });
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        self.calls.push(Call::Polygon {
            points: points.to_vec(),
            fill,
            stroke,
        });
    }

    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        self.calls.push(Call::Rect {
            rect,
            corner,
            fill,
            stroke,
        });
    }

    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        self.calls.push(Call::Circle {
            centre,
            radius,
            fill,
            stroke,
        });
    }

    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke) {
        self.calls.push(Call::Arc {
            centre,
            radius,
            start,
            end,
            stroke,
        });
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        self.calls.push(Call::Text {
            at,
            text: text.to_owned(),
            size,
            colour,
            anchor,
        });
    }
}

/// Shortest step the dash walk takes, in millimetres.
///
/// Neither `dash + gap` nor the phases it produces are representable in binary, so a step
/// can round to zero. Without a floor the walk never advances: it keeps appending the same
/// point until the process runs out of memory. A step this short is invisible at any scale,
/// so nudging the phase past the boundary is lossless.
const DASH_STEP_FLOOR: f32 = 1.0e-4;

/// Splits a polyline into the solid segments of a dashed pattern.
///
/// Dashing is done once here, in drawing millimetres, so the screen and the paper
/// agree on where the gaps fall instead of each device inventing its own.
#[must_use]
pub fn dashes(points: &[Pt], pattern: Option<(f32, f32)>) -> Vec<Vec<Pt>> {
    let Some((dash, gap)) = pattern else {
        return vec![points.to_vec()];
    };
    let period = dash + gap;
    if period <= 0.0 || points.len() < 2 {
        return vec![points.to_vec()];
    }

    let mut segments = Vec::new();
    let mut position = 0.0_f32;
    let mut current: Vec<Pt> = Vec::new();

    for pair in points.windows(2) {
        let (dx, dy) = pair[0].to(pair[1]);
        let length = dx.hypot(dy);
        if length <= f32::EPSILON {
            continue;
        }
        let (ux, uy) = (dx / length, dy / length);
        let mut travelled = 0.0_f32;
        // The most boundaries a segment of this length can hold, with slack. The bound
        // turns any future defect of the same kind into a slightly wrong drawing instead
        // of an allocation that takes the machine with it.
        let mut budget = (length / period).ceil() * 4.0 + 16.0;
        while travelled < length {
            if budget <= 0.0 {
                break;
            }
            budget -= 1.0;
            let phase = position % period;
            let in_dash = phase < dash;
            let remaining = if in_dash {
                dash - phase
            } else {
                period - phase
            };
            let step = remaining.min(length - travelled);
            if step <= DASH_STEP_FLOOR {
                position += DASH_STEP_FLOOR;
                continue;
            }
            let from = Pt::new(pair[0].x + ux * travelled, pair[0].y + uy * travelled);
            let to = Pt::new(
                pair[0].x + ux * (travelled + step),
                pair[0].y + uy * (travelled + step),
            );
            if in_dash {
                if current.is_empty() {
                    current.push(from);
                }
                current.push(to);
            } else if !current.is_empty() {
                segments.push(std::mem::take(&mut current));
            }
            travelled += step;
            position += step;
        }
    }
    if current.len() >= 2 {
        segments.push(current);
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_solid_line_is_one_segment() {
        let points = [Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)];
        let segments = dashes(&points, None);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], points);
    }

    #[test]
    fn dashing_respects_the_period() {
        let points = [Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)];
        let segments = dashes(&points, Some((2.0, 1.0)));
        // 10 mm at 2 on, 1 off: 2-3, 5-6, 8-9 and the last dash is cut at 10.
        assert_eq!(segments.len(), 4, "{segments:?}");
        assert!((segments[0][0].x - 0.0).abs() < 1.0e-4);
        assert!((segments[0][1].x - 2.0).abs() < 1.0e-4);
        assert!((segments[1][0].x - 3.0).abs() < 1.0e-4);
        assert!((segments[3][1].x - 10.0).abs() < 1.0e-4);
    }

    #[test]
    fn dashing_keeps_corners() {
        let points = [Pt::new(0.0, 0.0), Pt::new(4.0, 0.0), Pt::new(4.0, 4.0)];
        let segments = dashes(&points, Some((3.0, 1.0)));
        assert!(segments.len() >= 2);
        assert!(segments.iter().all(|segment| segment.len() >= 2));
    }

    #[test]
    fn rectangles_are_normalised() {
        let rect = Rect::from_corners(Pt::new(10.0, 5.0), Pt::new(2.0, 1.0));
        assert_eq!(rect.min, Pt::new(2.0, 1.0));
        assert!((rect.width() - 8.0).abs() < f32::EPSILON);
        assert!((rect.height() - 4.0).abs() < f32::EPSILON);
        assert!(rect.contains(Pt::new(2.0, 1.0)));
        assert!(!rect.contains(Pt::new(11.0, 1.0)));
    }

    #[test]
    fn the_recording_keeps_every_call_in_order() {
        let mut recording = Recording::default();
        recording.line(
            &[Pt::new(0.0, 0.0), Pt::new(1.0, 1.0)],
            Stroke::new(0.3, Colour::BLACK),
        );
        recording.text(Pt::new(2.0, 2.0), "1", 2.5, Colour::BLACK, Anchor::Middle);
        assert_eq!(recording.len(), 2);
        assert!(matches!(recording.calls()[1], Call::Text { .. }));
    }
}
