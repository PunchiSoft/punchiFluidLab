// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sheet of components the user positioned, not the fixed circuit.
//!
//! The editor draws what the document says: every component at its own box, the
//! simulated ones joined by the pipes the model describes, and a ring around the
//! selection. The passive components are drawn as symbols and nothing else — they have no
//! physics until connections exist.

use punchi_fluid_catalog::{Catalog, ComponentDefinition};
use punchi_fluid_solver::network::Network;

use crate::orient::{Orientation, Turned};
use crate::schematic::{self, CYLINDER_ID, Layout, PREPARATION_ID, SUPPLY_ID, VALVE_ID};
use crate::surface::{Bounds, Colour, Pt, Rect, Stroke, Surface};

/// One end of a pipe: a component of the sheet and one of its ports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeEnd<'a> {
    /// Index of the component in the slice passed to [`draw_placed`].
    pub placement: usize,
    /// Port identifier as the definition names it.
    pub port: &'a str,
}

/// A pipe to draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pipe<'a> {
    /// One end.
    pub from: PipeEnd<'a>,
    /// The other end.
    pub to: PipeEnd<'a>,
    /// How far the middle segment was dragged from the automatic route, in millimetres.
    pub offset_mm: f32,
    /// Pressure in the hose as a fraction of the supply span, `0` at atmosphere.
    ///
    /// The document owns the pipe; the model owns the gas; this is the model's answer for
    /// this pipe, so the sheet can colour it without knowing the network.
    pub pressure_ratio: f32,
}
use crate::symbol::{self, Ink};

/// What an instrument shows, in the unit its definition declares.
///
/// The value is the model's; the unit and the precision are the drawing's, so every device
/// prints the same number and a new instrument is one more variant here rather than a string
/// built in the front end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reading {
    /// Gauge pressure, `bar`.
    Pressure(f32),
    /// Volumetric flow at normal conditions, `l/min`.
    Flow(f32),
    /// Extension of a cylinder, `mm`.
    Position(f32),
}

impl Reading {
    /// The text drawn beside the symbol, with its unit.
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Self::Pressure(value) => format!("{value:.1} bar"),
            Self::Flow(value) => format!("{value:.1} l/min"),
            Self::Position(value) => format!("{value:.1} mm"),
        }
    }
}

/// A bounded series of samples to draw inside a component's box.
///
/// The recorder owns the samples and the window; the drawing only maps them, so a series
/// longer than the box can show is subsampled and never truncated to a lie.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Series<'a> {
    /// Samples, oldest first, in the quantity's own unit.
    pub values: &'a [f32],
    /// Value drawn at the bottom of the box.
    pub min: f32,
    /// Value drawn at the top of the box.
    pub max: f32,
}

/// Most vertices a series uses; a longer one is subsampled, keeping the last sample.
pub const SERIES_POINTS: usize = 64;

/// A component of the sheet: which definition draws it and the box it occupies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed<'a> {
    /// Index of the placement in the document, for the selection rings.
    pub index: usize,
    /// Catalog id.
    pub catalog_id: &'a str,
    /// Box in millimetres.
    pub rect: Rect,
    /// Extension of a cylinder, `0..=1`, when the model has one for this placement.
    pub extension: f32,
    /// Active position of a valve, `0` for anything that is not one.
    pub valve_position: usize,
    /// Reading of an instrument, in the unit its definition declares, if it has one.
    pub reading: Option<Reading>,
    /// Samples of a recorder, when this component is one and it is bound.
    pub series: Option<Series<'a>>,
    /// How the symbol is turned on the sheet.
    pub orientation: Orientation,
    /// Name the user gave the component, drawn under its box.
    pub designation: Option<&'a str>,
}

impl Placed<'_> {
    /// The box the component occupies once it is turned.
    #[must_use]
    pub fn box_rect(&self) -> Rect {
        self.orientation.box_of(self.rect)
    }
}

/// The trace of a recorder inside its own box.
///
/// At most [`SERIES_POINTS`] vertices, so a minute of samples costs one polyline and no
/// allocation; the last sample is always the last vertex, which is what makes the right
/// edge of the curve the value the reading prints.
// A window holds a bounded number of samples — a few hundred — so turning an index into a
// fraction of the box is exact, and the cast says so instead of rounding silently.
#[allow(clippy::cast_precision_loss)]
fn draw_series(surface: &mut dyn Surface, placed: &Placed<'_>, ink: &Ink) {
    let Some(series) = placed.series else {
        return;
    };
    if series.values.len() < 2 {
        return;
    }
    let box_rect = placed.box_rect();
    // Inside the symbol's own frame, so the trace never covers the paper it is drawn on.
    let margin = box_rect.width().min(box_rect.height()) * 0.14;
    let inner = box_rect.expand(-margin);
    if inner.width() <= 0.0 || inner.height() <= 0.0 {
        return;
    }
    let span = series.max - series.min;
    let span = if span.abs() < f32::EPSILON { 1.0 } else { span };
    let stride = series.values.len().div_ceil(SERIES_POINTS).max(1);
    let mut points = [Pt::new(0.0, 0.0); SERIES_POINTS + 1];
    let mut count = 0;
    let last = series.values.len() - 1;
    for (index, value) in series.values.iter().enumerate().step_by(stride) {
        let t = index as f32 / last as f32;
        let normalised = ((value - series.min) / span).clamp(0.0, 1.0);
        points[count] = Pt::new(
            inner.min.x + inner.width() * t,
            inner.max.y - inner.height() * normalised,
        );
        count += 1;
    }
    // The newest sample closes the curve even when the stride skipped it.
    if last % stride != 0 {
        let normalised = ((series.values[last] - series.min) / span).clamp(0.0, 1.0);
        points[count] = Pt::new(inner.max.x, inner.max.y - inner.height() * normalised);
        count += 1;
    }
    if count >= 2 {
        surface.line(&points[..count], Stroke::new(symbol::LINE * 0.8, ink.work));
    }
}

/// Draws the sheet.
///
/// The three components the model simulates are joined by their pipes when they are all
/// present; everything else is drawn as its own symbol. Every index in `selected` gets a
/// ring, drawn last so it is never covered.
pub fn draw_placed(
    surface: &mut dyn Surface,
    placements: &[Placed<'_>],
    pipes: &[Pipe<'_>],
    system: Option<&Network>,
    catalog: &Catalog,
    ink: &Ink,
    selected: &[usize],
) {
    for pipe in pipes {
        draw_pipe(surface, placements, pipe, catalog, ink);
    }
    let boxes = |id: &str| {
        placements
            .iter()
            .find(|placed| placed.catalog_id == id)
            .map(Placed::box_rect)
    };
    let circuit = system.and_then(|system| {
        Some((
            system,
            Layout {
                supply: boxes(SUPPLY_ID)?,
                preparation: boxes(PREPARATION_ID)?,
                valve: boxes(VALVE_ID)?,
                cylinder: boxes(CYLINDER_ID)?,
            },
        ))
    });

    // Every placement is drawn from the document and its own state: a cylinder with the
    // extension the network reports for it, a valve with its own position. There is no
    // list of the three components the model knows, because the model follows the sheet.
    for placed in placements {
        let Some(definition) = catalog.get(placed.catalog_id) else {
            // A document may name a component this machine does not have: the box says so
            // instead of the component vanishing.
            surface.rect(
                placed.rect,
                1.0,
                None,
                Some(Stroke::new(symbol::LINE, ink.muted)),
            );
            continue;
        };
        // A turned symbol is the same drawing seen through a transform, so a direction is
        // never a second copy of the geometry.
        if placed.orientation.is_identity() {
            symbol::draw_component(
                surface,
                placed.rect,
                definition,
                placed.valve_position,
                placed.extension,
                ink,
            );
        } else {
            let mut turned = Turned::new(surface, placed.orientation, placed.rect.center());
            symbol::draw_component(
                &mut turned,
                placed.rect,
                definition,
                placed.valve_position,
                placed.extension,
                ink,
            );
        }
        // The designation sits under the box, as the reference draws it: it names the
        // component for the reader without touching the symbol itself.
        if let Some(designation) = placed.designation.filter(|text| !text.is_empty()) {
            let box_rect = placed.box_rect();
            symbol::haloed_text(
                surface,
                Pt::new(box_rect.center().x, box_rect.max.y + symbol::NUMERAL * 1.4),
                designation,
                symbol::NUMERAL * 0.95,
                ink.text,
                ink.background,
            );
        }
        // A reading is text, not a primitive of the symbol: the dial is the file's and
        // the number is the model's.
        if let Some(reading) = placed.reading {
            let box_rect = placed.box_rect();
            let at = Pt::new(
                box_rect.center().x,
                box_rect.min.y + box_rect.height() * 0.72,
            );
            symbol::haloed_text(
                surface,
                at,
                &reading.text(),
                symbol::NUMERAL * 1.1,
                ink.text,
                ink.background,
            );
        }
        if placed.series.is_some() {
            draw_series(surface, placed, ink);
        }
    }

    if let Some((system, layout)) = circuit {
        // The document draws its own symbols and pipes, so the fixed pass adds only the
        // readouts it can place on its own composition.
        schematic::draw_readouts(surface, &layout, system, catalog, ink);
    }

    // The ring marks the selection, not the last component touched: a drag of six
    // components shows six rings until it is done.
    for placed in placements
        .iter()
        .filter(|placed| selected.contains(&placed.index))
    {
        selection_ring(surface, placed.box_rect(), ink.accent);
    }
}

/// Where a port sits, in millimetres, once the symbol is turned.
#[must_use]
pub fn port_point(
    rect: Rect,
    orientation: Orientation,
    definition: &ComponentDefinition,
    port: &str,
) -> Option<Pt> {
    definition
        .port
        .iter()
        .find(|candidate| candidate.id == port)
        .map(|candidate| {
            let local = symbol::at(rect, candidate.position[0], candidate.position[1]);
            orientation.map(rect.center(), local)
        })
}

/// Draws one pipe as an orthogonal run, which is how a schematic is read.
///
/// A straight line between the ports when they line up, otherwise an elbow with a single
/// jog in the middle: two segments would cut across the symbols, and more than three is a
/// path nobody draws by hand.
fn draw_pipe(
    surface: &mut dyn Surface,
    placements: &[Placed<'_>],
    pipe: &Pipe<'_>,
    catalog: &Catalog,
    ink: &Ink,
) {
    let (Some(from), Some(to)) = (
        placements.get(pipe.from.placement),
        placements.get(pipe.to.placement),
    ) else {
        return;
    };
    let (Some(from_definition), Some(to_definition)) =
        (catalog.get(from.catalog_id), catalog.get(to.catalog_id))
    else {
        return;
    };
    let (Some(start), Some(end)) = (
        port_point(from.rect, from.orientation, from_definition, pipe.from.port),
        port_point(to.rect, to.orientation, to_definition, pipe.to.port),
    ) else {
        return;
    };

    let points = route_offset(start, end, pipe.offset_mm);
    // The neutral ink at atmosphere, the working colour at supply pressure. The status
    // bar carries the number too, so pressure is never told by colour alone.
    let colour = blend(ink.line, ink.work, pipe.pressure_ratio);
    surface.line(&points, Stroke::new(symbol::LINE * 1.2, colour));
}

/// The polyline between two points: straight, or an elbow with one jog.
#[must_use]
pub fn route(start: Pt, end: Pt) -> Vec<Pt> {
    route_offset(start, end, 0.0)
}

/// The polyline with its middle segment displaced by `offset` millimetres.
///
/// The offset applies along the axis the elbow jogs in: horizontally when the ports are
/// mostly side by side, vertically when they are mostly one above the other. That keeps
/// the stored value meaningful after a component moves, and it is the segment a user
/// drags in the reference.
#[must_use]
pub fn route_offset(start: Pt, end: Pt, offset: f32) -> Vec<Pt> {
    let horizontal = (end.x - start.x).abs() >= (end.y - start.y).abs();
    if horizontal && (end.y - start.y).abs() < 0.5 {
        return vec![start, end];
    }
    if !horizontal && (end.x - start.x).abs() < 0.5 {
        return vec![start, end];
    }
    if horizontal {
        let middle = f32::midpoint(start.x, end.x) + offset;
        vec![start, Pt::new(middle, start.y), Pt::new(middle, end.y), end]
    } else {
        let middle = f32::midpoint(start.y, end.y) + offset;
        vec![start, Pt::new(start.x, middle), Pt::new(end.x, middle), end]
    }
}

/// Mixes two colours; `amount` is clamped to `0..=1`.
fn blend(from: Colour, to: Colour, amount: f32) -> Colour {
    let channel = |a: u8, b: u8| {
        let value = (f32::from(a) + (f32::from(b) - f32::from(a)) * amount.clamp(0.0, 1.0)).round();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let byte = value.clamp(0.0, 255.0) as u8;
        byte
    };
    Colour::new(
        channel(from.r, to.r),
        channel(from.g, to.g),
        channel(from.b, to.b),
    )
}

/// Bounding box of everything the sheet draws, for fitting it onto a page.
///
/// The real drawing runs once through [`Bounds`], so the symbols, the captions, the
/// designations, the readings and the pipe routes are all measured by the same code the
/// page will use. The result has a margin for strokes and glyph overhang, so a caller that
/// scales it to the page cannot crop the drawing. `None` means there is nothing to draw.
#[must_use]
pub fn bounds_placed(
    placements: &[Placed<'_>],
    pipes: &[Pipe<'_>],
    system: Option<&Network>,
    catalog: &Catalog,
) -> Option<Rect> {
    let mut bounds = Bounds::default();
    // The symbol box is the placement's footprint even when its ink does not reach the
    // edge: a rotated cylinder and a sparse symbol must not shrink the fit.
    for placed in placements {
        bounds.rect(placed.box_rect(), 0.0, None, None);
    }
    draw_placed(
        &mut bounds,
        placements,
        pipes,
        system,
        catalog,
        &Ink::print(),
        &[],
    );
    bounds.bounds().map(|rect| rect.expand(symbol::LINE * 3.0))
}

/// A ring around a box: the selection, and nothing else.
pub fn selection_ring(surface: &mut dyn Surface, rect: Rect, colour: Colour) {
    let ring = rect.expand(2.0);
    let corners = [
        Pt::new(ring.min.x, ring.min.y),
        Pt::new(ring.max.x, ring.min.y),
        Pt::new(ring.max.x, ring.max.y),
        Pt::new(ring.min.x, ring.max.y),
        Pt::new(ring.min.x, ring.min.y),
    ];
    surface.line(&corners, Stroke::new(symbol::LINE, colour));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orient::Orientation;
    use crate::surface::{Call, Recording};

    #[test]
    fn a_series_draws_a_bounded_curve_inside_its_box() {
        // A minute of samples is 600 values; the drawing keeps at most 64 vertices, inside
        // the box, and the newest sample is the right-hand end of the curve.
        // 600 samples fit in `u16`, so the conversion is exact and says so.
        let values: Vec<f32> = (0_u16..600).map(f32::from).collect();
        let catalog = Catalog::builtin();
        let rect = Rect::from_center_size(Pt::new(50.0, 50.0), 24.0, 24.0);
        let placed = [Placed {
            index: 0,
            catalog_id: "chart.recorder",
            rect,
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: Some(Series {
                values: &values,
                min: 0.0,
                max: 599.0,
            }),
            orientation: Orientation::NONE,
            designation: None,
        }];
        let mut surface = Recording::default();
        draw_placed(
            &mut surface,
            &placed,
            &[],
            None,
            &catalog,
            &Ink::print(),
            &[],
        );
        let curve = surface
            .calls()
            .iter()
            .filter_map(|call| match call {
                Call::Line { points, .. } => Some(points),
                _ => None,
            })
            .max_by_key(|points| points.len())
            .expect("the recorder draws a line");
        assert!(
            curve.len() <= SERIES_POINTS + 1,
            "the curve has {} vertices, more than the {} the drawing allows",
            curve.len(),
            SERIES_POINTS
        );
        assert!(curve.len() > 2, "a curve is more than one segment");
        assert!(
            curve.iter().all(|point| rect.expand(0.01).contains(*point)),
            "the curve left the recorder's box"
        );
        // The first sample sits on the bottom edge and the last in the top-right corner of
        // the plot, which is what makes the end of the curve the value the reading prints.
        let plot = rect.expand(-rect.height() * 0.14);
        let first = curve.first().expect("a first point");
        assert!(
            (first.y - plot.max.y).abs() < 0.01,
            "the oldest sample is at {first:?}, not on the bottom edge"
        );
        let last = curve.last().expect("a last point");
        assert!(
            (last.x - plot.max.x).abs() < 0.01 && (last.y - plot.min.y).abs() < 0.01,
            "the last sample is at {last:?}, not at the top-right corner of the plot"
        );
    }

    #[test]
    fn a_reading_carries_the_unit_of_its_instrument() {
        // The drawing owns the unit and the precision, so every device prints the same
        // number and the front end cannot invent a second format.
        assert_eq!(Reading::Pressure(3.24).text(), "3.2 bar");
        assert_eq!(Reading::Flow(41.0).text(), "41.0 l/min");
        assert_eq!(Reading::Flow(-2.55).text(), "-2.5 l/min");
    }

    #[test]
    fn the_bounds_cover_the_boxes_and_the_pipe_route() {
        let catalog = Catalog::builtin();
        let placements = [
            Placed {
                index: 0,
                catalog_id: "supply.compressor",
                rect: Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0),
                extension: 0.0,
                valve_position: 0,
                reading: None,
                series: None,
                orientation: Orientation::NONE,
                designation: None,
            },
            Placed {
                index: 1,
                catalog_id: "cylinder.single_acting_20_100",
                rect: Rect::from_min_size(Pt::new(100.0, 0.0), 40.0, 20.0),
                extension: 0.0,
                valve_position: 0,
                reading: None,
                series: None,
                orientation: Orientation::NONE,
                designation: None,
            },
        ];
        let pipes = [Pipe {
            from: PipeEnd {
                placement: 0,
                port: "1",
            },
            to: PipeEnd {
                placement: 1,
                port: "1",
            },
            offset_mm: 10.0,
            pressure_ratio: 0.0,
        }];

        let bounds =
            bounds_placed(&placements, &pipes, None, &catalog).expect("a drawing has bounds");
        assert!(bounds.contains(Pt::new(0.0, 0.0)), "{bounds:?}");
        assert!(bounds.contains(Pt::new(140.0, 20.0)), "{bounds:?}");
    }
}
