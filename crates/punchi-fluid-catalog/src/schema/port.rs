// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Ports, drawing primitives and their defaults.
//!
//! Split from `schema.rs` to keep both files under the architecture budget; the
//! types are re-exported from the schema module, so callers keep importing
//! `crate::schema::{Port, Primitive, PrimitiveKind}`.

use serde::{Deserialize, Serialize};

use super::{LineClass, PortRole, default_true};

/// Drawing primitive supported by the symbol renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveKind {
    /// Two points.
    Line,
    /// Open sequence of segments.
    Polyline,
    /// Closed sequence of segments.
    Polygon,
    /// `[x, y, width, height]`.
    Rect,
    /// `[cx, cy, radius]`.
    Circle,
    /// `[cx, cy, radius, start_degrees, end_degrees]`.
    Arc,
}

/// One drawing primitive, in coordinates normalised to the symbol box `0..=1`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Primitive {
    /// Geometry to draw.
    pub kind: PrimitiveKind,
    /// Flat coordinate list, interpreted according to `kind`.
    #[serde(default)]
    pub points: Vec<f64>,
    /// Fills the inside instead of only stroking the outline.
    #[serde(default)]
    pub filled: bool,
    /// Draws the primitive with a dashed pattern, instead of the pattern of its
    /// line class. Kept for parts that are genuinely dashed, like a spring.
    #[serde(default)]
    pub dashed: bool,
    /// Class of the line. Required on every line and polyline.
    #[serde(default)]
    pub line_class: Option<LineClass>,
    /// Stroke width multiplier relative to the base line width.
    #[serde(default = "default_stroke")]
    pub stroke: f64,
}

fn default_stroke() -> f64 {
    1.0
}

/// Where the renderer places a port numeral, when the automatic side is not the reference's.
///
/// The automatic rule puts a left side port's numeral above its stub and a right one's
/// below; a file whose reference puts both on the same side says so here. It is a placement
/// of the standard's numeral (`RF053`), not a different numeral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NumeralSide {
    /// Above the left stub, below the right one (the default).
    #[default]
    Auto,
    /// Above the stub, whichever side the port is on.
    Above,
    /// Below the stub, whichever side the port is on.
    Below,
}

/// Connection point of a component.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    /// Identifier used by the connections of a valve, typically `"1"`, `"2"`.
    pub id: String,
    /// What the port does.
    pub role: PortRole,
    /// Position inside the symbol box, `[x, y]` in `0..=1`.
    pub position: [f64; 2],
    /// Whether the renderer prints this port's id beside it.
    ///
    /// A coil label such as `Y1` is a connection mark the user sets, not a fixed numeral;
    /// a port with this flag off still connects and routes, but prints no number.
    #[serde(default = "default_true")]
    pub show_numeral: bool,
    /// Number the renderer prints, when it differs from the id.
    ///
    /// The id is the unique key a pipe, the document and the solver use; the standard can
    /// print the same numeral on two ports, as the two supply inlets of a shuttle do
    /// (`1a` and `1b`, both drawn as `1`). Absent means the id is the numeral (ADR 0039).
    #[serde(default)]
    pub numeral: Option<String>,
    /// Where to place this port's numeral, when the automatic side is wrong for the
    /// reference. The default keeps the automatic rule.
    #[serde(default)]
    pub numeral_side: NumeralSide,
}

impl Port {
    /// Number the renderer prints: the declared numeral, or the id.
    #[must_use]
    pub fn numeral(&self) -> &str {
        self.numeral.as_deref().unwrap_or(&self.id)
    }
}
