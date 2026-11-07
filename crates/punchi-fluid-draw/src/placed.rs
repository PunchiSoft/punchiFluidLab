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

use crate::schematic::{self, CYLINDER_ID, Layout, SUPPLY_ID, VALVE_ID};
use crate::surface::{Colour, Pt, Rect, Stroke, Surface};

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

/// A component of the sheet: which definition draws it and the box it occupies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed<'a> {
    /// Catalog id.
    pub catalog_id: &'a str,
    /// Box in millimetres.
    pub rect: Rect,
    /// Extension of a cylinder, `0..=1`, when the model has one for this placement.
    pub extension: f32,
    /// Active position of a valve, `0` for anything that is not one.
    pub valve_position: usize,
}

/// Draws the sheet.
///
/// The three components the model simulates are joined by their pipes when they are all
/// present; everything else is drawn as its own symbol. `selected` gets a ring, drawn last
/// so it is never covered.
pub fn draw_placed(
    surface: &mut dyn Surface,
    placements: &[Placed<'_>],
    pipes: &[Pipe<'_>],
    system: Option<&Network>,
    catalog: &Catalog,
    ink: &Ink,
    selected: Option<usize>,
) {
    for pipe in pipes {
        draw_pipe(surface, placements, pipe, catalog, ink);
    }
    let boxes = |id: &str| {
        placements
            .iter()
            .find(|placed| placed.catalog_id == id)
            .map(|placed| placed.rect)
    };
    let circuit = system.and_then(|system| {
        Some((
            system,
            Layout {
                supply: boxes(SUPPLY_ID)?,
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
        symbol::draw_component(
            surface,
            placed.rect,
            definition,
            placed.valve_position,
            placed.extension,
            ink,
        );
    }

    if let Some((system, layout)) = circuit {
        // The document draws its own symbols and pipes, so the fixed pass adds only the
        // readouts it can place on its own composition.
        schematic::draw_readouts(surface, &layout, system, catalog, ink);
    }

    if let Some(index) = selected {
        if let Some(placed) = placements.get(index) {
            selection_ring(surface, placed.rect, ink.accent);
        }
    }
}

/// Where a port sits, in millimetres.
#[must_use]
pub fn port_point(rect: Rect, definition: &ComponentDefinition, port: &str) -> Option<Pt> {
    definition
        .port
        .iter()
        .find(|candidate| candidate.id == port)
        .map(|candidate| symbol::at(rect, candidate.position[0], candidate.position[1]))
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
        port_point(from.rect, from_definition, pipe.from.port),
        port_point(to.rect, to_definition, pipe.to.port),
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
