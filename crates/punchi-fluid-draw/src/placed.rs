// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sheet of components the user positioned, not the fixed circuit.
//!
//! The editor draws what the document says: every component at its own box, the
//! simulated ones joined by the pipes the model describes, and a ring around the
//! selection. The passive components are drawn as symbols and nothing else — they have no
//! physics until connections exist.

use punchi_fluid_catalog::{Catalog, ComponentDefinition, PortRole};

use crate::orient::{Orientation, Turned};
use crate::surface::{Bounds, Colour, LineClass, Pt, Rect, Stroke, Surface};

mod closure;
mod edge;
mod flow;

pub use closure::{CLOSURE_MM, closure_boxes, draw_closures};
pub use edge::{EDGE_TIE_MM, port_edge_towards};

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
    /// Class of the line: working, pilot or drain, which decides its pattern.
    pub class: LineClass,
    /// Nominal outside diameter of a pneumatic pipe, in hundredths of a millimetre.
    ///
    /// `None` is an old sheet or an electrical wire; the width then stays the one the
    /// drawing always used. The size is stored, not measured: its physics is P-024b.
    pub tube_od_hundredths: Option<u16>,
    /// Signed flow through the hose over the model's last frame, normal litres per minute.
    ///
    /// Positive runs from `from` to `to`. `0.0` is a still hose or a scene that was not
    /// given a live model; the drawing reads it only to place the flow pulse.
    pub flow_l_min: f32,
    /// How far the flow pulse has travelled, in simulated seconds.
    ///
    /// The front end owns the clock, so the drawing never reads time: it maps the phase
    /// onto the route. An export leaves it at zero and sets [`Self::animate`] to `false`.
    pub phase_s: f32,
    /// Whether the flow pulse travels. `false` freezes it for reduced motion and the print.
    pub animate: bool,
}
use crate::control_drawing::{indicator_rays, moving_bar};
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
    /// Whether a lamp is lit.
    Indicator(bool),
    /// Whether a buzzer sounds.
    Audible(bool),
}

impl Reading {
    /// The text drawn beside the symbol, with its unit.
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Self::Pressure(value) => format!("{value:.1} bar"),
            Self::Flow(value) => format!("{value:.1} l/min"),
            Self::Position(value) => format!("{value:.1} mm"),
            Self::Indicator(true) => "encendida".to_owned(),
            Self::Indicator(false) => "apagada".to_owned(),
            Self::Audible(true) => "sonando".to_owned(),
            Self::Audible(false) => "en silencio".to_owned(),
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

/// A port that vents to the atmosphere and how much air left it on the frame.
///
/// The solver reports the flow through the port's atmosphere boundary; the drawing only
/// places the marks, so a plugged port and a still hose draw nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExhaustMark<'a> {
    /// Port identifier as the definition names it.
    pub port: &'a str,
    /// Signed flow leaving the port, normal litres per minute.
    pub flow_l_min: f32,
}

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
    /// Whether a contact is closed, when this component is one (ADR 0018).
    pub contact_closed: Option<bool>,
    /// How the symbol is turned on the sheet.
    pub orientation: Orientation,
    /// Name the user gave the component, drawn under its box.
    pub designation: Option<&'a str>,
    /// Closures chosen on its ports, by port id, with the definitions already resolved.
    ///
    /// The document has no catalog, so the caller resolves each id once and the drawing
    /// stays a function of what it is given. An empty slice is a component with every
    /// port open, which is what most of the sheet is.
    pub closures: &'a [(&'a str, &'a ComponentDefinition)],
    /// Ports of this placement that vent to the atmosphere, with their reading.
    pub exhausts: &'a [ExhaustMark<'a>],
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

/// Draws the sheet: the symbols with the connection points a hose arrives at.
///
/// The three components the model simulates are joined by their pipes when they are all
/// present; everything else is drawn as its own symbol. Every index in `selected` gets a
/// ring, drawn last so it is never covered.
pub fn draw_placed(
    surface: &mut dyn Surface,
    placements: &[Placed<'_>],
    pipes: &[Pipe<'_>],
    catalog: &Catalog,
    ink: &Ink,
    selected: &[usize],
) {
    draw_placed_with(surface, placements, pipes, catalog, ink, selected, true);
}

/// Draws the catalog panel's thumbnail: the same symbols **without** the sheet's connection
/// points. The panel lists symbols; the sheet marks where a hose arrives.
pub fn draw_placed_preview(
    surface: &mut dyn Surface,
    placements: &[Placed<'_>],
    pipes: &[Pipe<'_>],
    catalog: &Catalog,
    ink: &Ink,
    selected: &[usize],
) {
    draw_placed_with(surface, placements, pipes, catalog, ink, selected, false);
}

fn draw_placed_with(
    surface: &mut dyn Surface,
    placements: &[Placed<'_>],
    pipes: &[Pipe<'_>],
    catalog: &Catalog,
    ink: &Ink,
    selected: &[usize],
    connections: bool,
) {
    // One box per placement, built once: the router needs them all to skip the symbols a
    // hose does not join. One vector per frame, not one per pipe.
    let boxes: Vec<Rect> = placements.iter().map(|placed| placed.rect).collect();
    for pipe in pipes {
        draw_pipe(surface, placements, &boxes, pipe, catalog, ink);
    }
    let draw_symbol: fn(&mut dyn Surface, Rect, &ComponentDefinition, usize, f32, &Ink) =
        if connections {
            symbol::draw_component
        } else {
            symbol::draw_component_preview
        };
    // Every placement is drawn from the document and its own state: a cylinder with the
    // extension the network reports for it, a valve with its own position. There is no
    // list of the three components the model knows, because the model follows the sheet.
    for placed in placements {
        // A selected component is marked by its own lines taking the accent, not by a box
        // drawn around it: the reference marks the symbol, and an extra rectangle is a
        // second drawing that says nothing the symbol does not.
        let mut selected_ink = *ink;
        if selected.contains(&placed.index) {
            selected_ink.line = ink.accent;
            selected_ink.text = ink.accent;
        }
        let ink = &selected_ink;
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
            draw_symbol(
                surface,
                placed.rect,
                definition,
                placed.valve_position,
                placed.extension,
                ink,
            );
        } else {
            let mut turned = Turned::new(surface, placed.orientation, placed.rect.center());
            draw_symbol(
                &mut turned,
                placed.rect,
                definition,
                placed.valve_position,
                placed.extension,
                ink,
            );
        }
        // A closure is drawn after the body so it covers the port stub it is fitted to
        // instead of hiding behind it. It keeps the component's ink: selecting a valve
        // selects its closures too, without a ring of their own.
        draw_closures(
            surface,
            placed.rect,
            placed.orientation,
            definition,
            placed.closures,
            ink,
        );
        // The air leaving an exhaust port is drawn outside the symbol, in the same stream
        // language as the hoses: the port says where, the marks say how much. A plugged
        // port has no reading and draws nothing.
        for mark in placed.exhausts {
            draw_exhaust_jet(surface, placed, definition, mark, ink);
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
        if let Some(definition) = catalog.get(placed.catalog_id) {
            moving_bar(surface, placed, definition, ink);
            // A lamp gains rays when lit; a buzzer's waves are in its file, and the word
            // beside it plus the sound carry its state (ADR 0022).
            if matches!(placed.reading, Some(Reading::Indicator(true))) {
                indicator_rays(surface, placed, definition, ink);
            }
        }
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

/// Draws the air leaving an exhaust port, outside the symbol, along the port's edge.
///
/// The marks are the same solid arrowheads as the hose stream and the port's edge gives the
/// direction, so a bottom exhaust blows down and a right one blows right. A plugged port has
/// no reading and a still one is below the flow floor, so neither draws anything.
fn draw_exhaust_jet(
    surface: &mut dyn Surface,
    placed: &Placed<'_>,
    definition: &ComponentDefinition,
    mark: &ExhaustMark<'_>,
    ink: &Ink,
) {
    if mark.flow_l_min.abs() <= flow::FLOW_FLOOR {
        return;
    }
    let Some(at) = port_point(placed.rect, placed.orientation, definition, mark.port) else {
        return;
    };
    let direction = match port_edge(definition, mark.port) {
        PortEdge::Left => Pt::new(-1.0, 0.0),
        PortEdge::Right => Pt::new(1.0, 0.0),
        PortEdge::Top => Pt::new(0.0, -1.0),
        PortEdge::Bottom => Pt::new(0.0, 1.0),
    };
    // A closure fitted to the port owns the mouth: the air leaves through the silencer's
    // own symbol, so the marks start past its box instead of crossing it.
    let mouth = if placed.closures.iter().any(|(port, _)| *port == mark.port) {
        CLOSURE_MM
    } else {
        0.0
    };
    // Fixed distances, not a loop over an index: the jet is three marks, no allocation.
    for distance in [2.0_f32, 4.5, 7.0] {
        let distance = distance + mouth;
        let point = Pt::new(at.x + direction.x * distance, at.y + direction.y * distance);
        symbol::draw_flow_arrow(surface, point, direction, flow::PULSE_HEAD_MM, ink.line);
    }
}

/// Draws one pipe as an orthogonal run, which is how a schematic is read.
///
/// A straight line between the ports when they line up, otherwise an elbow with a single
/// jog in the middle: two segments would cut across the symbols, and more than three is a
/// path nobody draws by hand.
fn draw_pipe(
    surface: &mut dyn Surface,
    placements: &[Placed<'_>],
    boxes: &[Rect],
    pipe: &Pipe<'_>,
    catalog: &Catalog,
    ink: &Ink,
) {
    let Some(route) = pipe_route(placements, boxes, pipe, catalog) else {
        return;
    };
    // The neutral ink at atmosphere, the working colour at supply pressure. The status
    // bar carries the number too, so pressure is never told by colour alone.
    let colour = blend(ink.line, ink.work, pipe.pressure_ratio);
    // An all-electrical hose is drawn with the electrical class: the wire says what it is
    // before anyone reads a label, and a student who has just placed a contact does not
    // have to choose a class for the first wire of the control circuit. Its stroke stays
    // the one it always had; only a pneumatic pipe follows the tube.
    let class = if route.electrical {
        LineClass::Electrical
    } else {
        pipe.class
    };
    let width = if route.electrical {
        symbol::LINE * 1.2
    } else {
        tube_width_mm(pipe.tube_od_hundredths)
    };
    // The pattern is the class's, never a literal: a pilot or drain line is read by its
    // dashes before anyone reads its label.
    surface.line(&route.points, Stroke::new(width, colour).with_class(class));
    if !route.electrical {
        flow::draw_flow_marks(surface, &route.points, pipe, width, colour);
    }
}

/// The width a pneumatic pipe draws at, from its nominal outside diameter.
///
/// The series is the drawing's, not a continuous map: 0.25, 0.35 and 0.50 mm are visibly
/// different at 300 dpi, and the cap stays below the 0.7 mm frame so a pipe never competes
/// with the sheet's own border. `None` keeps the width the drawing always used for an old
/// sheet; an electrical wire never reaches here.
#[must_use]
pub fn tube_width_mm(tube: Option<u16>) -> f32 {
    match tube {
        None => symbol::LINE * 1.2,
        Some(od) if od < 600 => 0.25,
        Some(od) if od < 1000 => 0.35,
        Some(_) => 0.50,
    }
}

/// The route a pipe takes, and whether its two ports make it an electrical wire.
pub(crate) struct PipeRoute {
    pub(crate) points: Vec<Pt>,
    pub(crate) electrical: bool,
}

/// Builds the polyline of one pipe from the same router the sheet draws with.
pub(crate) fn pipe_route(
    placements: &[Placed<'_>],
    boxes: &[Rect],
    pipe: &Pipe<'_>,
    catalog: &Catalog,
) -> Option<PipeRoute> {
    let from = placements.get(pipe.from.placement)?;
    let to = placements.get(pipe.to.placement)?;
    let from_definition = catalog.get(from.catalog_id)?;
    let to_definition = catalog.get(to.catalog_id)?;
    let start = port_point(from.rect, from.orientation, from_definition, pipe.from.port)?;
    let end = port_point(to.rect, to.orientation, to_definition, pipe.to.port)?;
    let electrical = pipe.class == LineClass::Electrical
        || (pipe.class == LineClass::Working
            && is_electrical(from_definition, pipe.from.port, to_definition, pipe.to.port));
    let points = route_between_boxes_avoiding(
        start,
        port_edge_towards(
            from.rect,
            from.orientation,
            from_definition,
            pipe.from.port,
            Some(end),
        ),
        from.rect,
        end,
        port_edge_towards(
            to.rect,
            to.orientation,
            to_definition,
            pipe.to.port,
            Some(start),
        ),
        to.rect,
        boxes,
        [pipe.from.placement, pipe.to.placement],
        pipe.offset_mm,
    );
    Some(PipeRoute { points, electrical })
}

/// The edge of its box a port sits on, which is the direction the hose leaves in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortEdge {
    /// The left side: the hose leaves horizontally.
    Left,
    /// The right side: the hose leaves horizontally.
    Right,
    /// The top side: the hose leaves vertically.
    Top,
    /// The bottom side: the hose leaves vertically.
    Bottom,
}

impl PortEdge {
    /// `true` when a hose leaves this edge on the horizontal axis.
    #[must_use]
    pub const fn horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// The edge a port sits on, read from its declared position inside the symbol box.
///
/// A port declared exactly on an edge leaves along that edge's axis. A port declared inside
/// the box — a rounded symbol, a file drawn by hand — takes the axis of the edge it is
/// nearest, so the route still leaves the symbol instead of cutting through it.
#[must_use]
pub fn port_edge(definition: &ComponentDefinition, id: &str) -> PortEdge {
    let Some(port) = definition.port.iter().find(|port| port.id == id) else {
        return PortEdge::Left;
    };
    let [x, y] = port.position;
    let distances = [
        (PortEdge::Left, x),
        (PortEdge::Right, 1.0 - x),
        (PortEdge::Top, y),
        (PortEdge::Bottom, 1.0 - y),
    ];
    distances
        .into_iter()
        .min_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(PortEdge::Left, |(edge, _)| edge)
}

/// `true` when both ends of a hose are electrical terminals.
///
/// The class of the line follows the ports: two terminals make a wire, and the drawing says
/// so without the user choosing a class for it.
fn is_electrical(
    from: &ComponentDefinition,
    from_port: &str,
    to: &ComponentDefinition,
    to_port: &str,
) -> bool {
    let role = |definition: &ComponentDefinition, id: &str| {
        definition
            .port
            .iter()
            .find(|port| port.id == id)
            .map(|port| port.role)
    };
    matches!(
        (role(from, from_port), role(to, to_port)),
        (Some(PortRole::Electrical), Some(PortRole::Electrical))
    )
}

pub use crate::route::{
    route, route_between, route_between_boxes, route_between_boxes_avoiding, route_length_mm,
    route_offset,
};

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
/// The real drawing runs once through [`Bounds`], so the symbols, the designations, the
/// readings and the pipe routes are all measured by the same code the page will use. The result has a margin for strokes and glyph overhang, so a caller that
/// scales it to the page cannot crop the drawing. `None` means there is nothing to draw.
#[must_use]
pub fn bounds_placed(
    placements: &[Placed<'_>],
    pipes: &[Pipe<'_>],
    catalog: &Catalog,
) -> Option<Rect> {
    let mut bounds = Bounds::default();
    // The symbol box is the placement's footprint even when its ink does not reach the
    // edge: a rotated cylinder and a sparse symbol must not shrink the fit.
    for placed in placements {
        bounds.rect(placed.box_rect(), 0.0, None, None);
    }
    draw_placed(&mut bounds, placements, pipes, catalog, &Ink::print(), &[]);
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
#[path = "placed_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "placed_routing_tests.rs"]
mod routing_tests;

#[cfg(test)]
#[path = "placed_electrical_tests.rs"]
mod electrical_tests;

#[cfg(test)]
mod selection_tests {
    use crate::orient::Orientation;
    use crate::placed::{Placed, draw_placed};
    use crate::surface::{Call, Pt, Recording, Rect};
    use crate::symbol::Ink;
    use punchi_fluid_catalog::Catalog;

    fn one() -> Placed<'static> {
        Placed {
            index: 0,
            catalog_id: "supply.compressor",
            rect: Rect::from_min_size(Pt::new(10.0, 10.0), 22.0, 22.0),
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            contact_closed: None,
            orientation: Orientation::NONE,
            designation: None,
            closures: &[],
            exhausts: &[],
        }
    }

    #[test]
    fn a_selected_component_is_drawn_in_accent_and_not_boxed() {
        let catalog = Catalog::builtin();
        let ink = Ink::print();
        let plain = {
            let mut recording = Recording::default();
            draw_placed(&mut recording, &[one()], &[], &catalog, &ink, &[]);
            recording
        };
        let picked = {
            let mut recording = Recording::default();
            draw_placed(&mut recording, &[one()], &[], &catalog, &ink, &[0]);
            recording
        };

        // No extra stroke: the selection is the symbol's own lines, not a ring around it.
        assert_eq!(picked.calls().len(), plain.calls().len());
        let accent = picked.calls().iter().any(|call| match call {
            Call::Line { stroke, .. } | Call::Arc { stroke, .. } => stroke.colour == ink.accent,
            Call::Rect { stroke, .. }
            | Call::Circle { stroke, .. }
            | Call::Polygon { stroke, .. } => {
                matches!(stroke, Some(stroke) if stroke.colour == ink.accent)
            }
            Call::Text { colour, .. } => *colour == ink.accent,
        });
        assert!(accent, "the selected symbol carries the accent");
    }
}

#[cfg(test)]
#[path = "placed_tube_tests.rs"]
mod tube_tests;

#[cfg(test)]
#[path = "placed_flow_tests.rs"]
mod flow_tests;

#[cfg(test)]
#[path = "placed_closure_tests.rs"]
mod closure_tests;
