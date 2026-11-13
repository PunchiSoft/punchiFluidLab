// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The circuit on one page: placement, pipes and readouts.
//!
//! This is the drawing the window shows and the PDF prints. It takes an area in
//! millimetres and never looks at the device, so the screen and the paper differ only
//! in the scale the backend applies.

use punchi_fluid_catalog::{Catalog, ComponentDefinition, PortRole};
use punchi_fluid_solver::network::Network;

use crate::surface::{Anchor, Colour, Pt, Rect, Stroke, Surface};
use crate::symbol::{self, Ink};

/// Identifier of the supply of the fixed circuit.
pub const SUPPLY_ID: &str = punchi_fluid_solver::pneumatic::DEFAULT_SUPPLY;
/// Identifier of the valve of the fixed circuit.
pub const VALVE_ID: &str = punchi_fluid_solver::pneumatic::DEFAULT_VALVE;
/// Identifier of the service unit of the worked example.
pub const PREPARATION_ID: &str = "preparation.service_unit";

/// Identifier of the cylinder of the fixed circuit.
pub const CYLINDER_ID: &str = punchi_fluid_solver::pneumatic::DEFAULT_CYLINDER;

/// Where the four components of the worked example sit, in millimetres of the drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// Air source: the compressor. A source is a machine, not the air itself.
    pub supply: Rect,
    /// Service unit that makes the compressor's air usable, between it and the valve.
    pub preparation: Rect,
    /// Valve symbol box.
    pub valve: Rect,
    /// Cylinder symbol box.
    pub cylinder: Rect,
}

/// Fixed placement of the four components inside `area`.
///
/// Returns `None` when the area is too small to draw something readable, so a tiny
/// canvas or a thumbnail does not produce a smudge.
#[must_use]
pub fn layout(area: Rect) -> Option<Layout> {
    let square = (area.height() * 0.26).min(area.width() * 0.20);
    if square < 8.0 {
        return None;
    }
    // The composition a student meets on opening: the source bottom left, the service unit
    // beside it, the valve at the middle right and the cylinder top right. The air reads
    // left to right along the bottom and rises to the actuator, which is the order the parts
    // are met in and the order the hoses travel.
    let supply = Rect::from_center_size(area.point_at(0.13, 0.82), square, square);
    // The service unit sits between the source and the first valve, in series and in that
    // order: what comes out of it is the line pressure the rest of the circuit is designed
    // around, so it cannot be drawn after the valve.
    // Square, like every other accessory: the symbol is drawn in a square box, so a box of
    // another shape would stretch its diamonds and its dial.
    let preparation = Rect::from_center_size(area.point_at(0.38, 0.82), square, square);
    let valve = Rect::from_center_size(area.point_at(0.64, 0.58), square, square);
    let cylinder = Rect::from_center_size(area.point_at(0.76, 0.22), square * 2.2, square);
    Some(Layout {
        supply,
        preparation,
        valve,
        cylinder,
    })
}

/// Draws the circuit and returns where it placed the components.
pub fn draw(
    surface: &mut dyn Surface,
    area: Rect,
    system: &Network,
    catalog: &Catalog,
    ink: &Ink,
) -> Option<Layout> {
    let placed = layout(area)?;
    draw_with_layout(surface, &placed, system, catalog, ink);
    Some(placed)
}

/// The same drawing, at boxes someone else decided.
///
/// The interactive sheet gets its boxes from the document, so a component the user moved
/// stays where they put it; the exported sheet gets them from [`layout`], so a printed
/// page has the composition the drawing was designed for.
pub fn draw_with_layout(
    surface: &mut dyn Surface,
    placed: &Layout,
    system: &Network,
    catalog: &Catalog,
    ink: &Ink,
) {
    draw_hoses(surface, placed, system, catalog, ink);
    draw_circuit(surface, placed, system, catalog, ink);
}

/// The circuit without its hoses: symbols, piston at its extension, and captions.
///
/// The interactive sheet owns its pipes — the document holds them, routes them and lets the
/// user select and delete them — so it asks for this pass and draws them itself. Drawing the
/// hoses here as well would put a second line on the sheet at a second routing.
pub fn draw_circuit(
    surface: &mut dyn Surface,
    placed: &Layout,
    system: &Network,
    catalog: &Catalog,
    ink: &Ink,
) {
    let area = union(placed.supply, placed.cylinder);
    let (Some(supply), Some(valve), Some(cylinder)) = (
        catalog.get(SUPPLY_ID),
        catalog.get(VALVE_ID),
        catalog.get(CYLINDER_ID),
    ) else {
        surface.text(
            area.center(),
            "The catalog is missing a definition for this circuit",
            symbol::NUMERAL * 2.0,
            ink.line,
            Anchor::Middle,
        );
        return;
    };

    symbol::draw_component(surface, placed.supply, supply, 0, 0.0, ink);
    symbol::draw_valve(surface, placed.valve, valve, system.valve_position(), ink);
    // One call: the cylinder draws its own piston and rod at the state of the model, so
    // it cannot be drawn twice at two different extensions.
    #[allow(clippy::cast_possible_truncation)]
    let extension = system.extension() as f32;
    symbol::draw_component(surface, placed.cylinder, cylinder, 0, extension, ink);
    draw_labels(surface, placed, system, supply, valve, cylinder, ink);
}

/// The readouts of the worked circuit, without drawing its symbols or hoses.
///
/// The interactive sheet owns every symbol and pipe, so it asks only for the labels the
/// fixed composition knows how to place.
pub fn draw_readouts(
    surface: &mut dyn Surface,
    placed: &Layout,
    system: &Network,
    catalog: &Catalog,
    ink: &Ink,
) {
    let (Some(supply), Some(valve), Some(cylinder)) = (
        catalog.get(SUPPLY_ID),
        catalog.get(VALVE_ID),
        catalog.get(CYLINDER_ID),
    ) else {
        return;
    };
    draw_labels(surface, placed, system, supply, valve, cylinder, ink);
}

/// The two working hoses of the built-in circuit, coloured by the pressure in them.
///
/// Only the fixed composition uses this: it is the composition that knows which hose is the
/// supply line and which is the working line.
fn draw_hoses(
    surface: &mut dyn Surface,
    placed: &Layout,
    system: &Network,
    catalog: &Catalog,
    ink: &Ink,
) {
    let (Some(supply), Some(preparation), Some(valve), Some(cylinder)) = (
        catalog.get(SUPPLY_ID),
        catalog.get(PREPARATION_ID),
        catalog.get(VALVE_ID),
        catalog.get(CYLINDER_ID),
    ) else {
        return;
    };
    draw_pipes(
        surface,
        placed,
        system,
        supply,
        preparation,
        valve,
        cylinder,
        ink,
    );
}

/// Smallest box containing both.
fn union(a: Rect, b: Rect) -> Rect {
    Rect::from_corners(
        Pt::new(a.min.x.min(b.min.x), a.min.y.min(b.min.y)),
        Pt::new(a.max.x.max(b.max.x), a.max.y.max(b.max.y)),
    )
}

fn port(definition: &ComponentDefinition, rect: Rect, role: PortRole) -> Option<Pt> {
    definition
        .port_with_role(role)
        .map(|port| symbol::at(rect, port.position[0], port.position[1]))
}

#[allow(clippy::too_many_arguments)]
fn draw_pipes(
    surface: &mut dyn Surface,
    placed: &Layout,
    system: &Network,
    supply: &ComponentDefinition,
    preparation: &ComponentDefinition,
    valve: &ComponentDefinition,
    cylinder: &ComponentDefinition,
    ink: &Ink,
) {
    let working = pressure_colour(system, ink);
    let width = symbol::LINE * 1.2;

    let (
        Some(supply_out),
        Some(unit_in),
        Some(unit_out),
        Some(valve_in),
        Some(valve_out),
        Some(cylinder_in),
    ) = (
        port(supply, placed.supply, PortRole::Supply),
        port(preparation, placed.preparation, PortRole::Supply),
        port(preparation, placed.preparation, PortRole::Working),
        port(valve, placed.valve, PortRole::Supply),
        port(valve, placed.valve, PortRole::Working),
        port(cylinder, placed.cylinder, PortRole::Working),
    )
    else {
        return;
    };

    // Source first, then the unit that makes its air usable, then the valve. Offsets are
    // fractions of the symbol size, never millimetres of paper: the composition must be the
    // same on a window and on an A2 sheet.
    let supply_line = [supply_out, unit_in];
    let below = valve_in.y + placed.valve.height() * 0.45;
    let unit_line = [
        unit_out,
        Pt::new(unit_out.x, below),
        Pt::new(valve_in.x, below),
        valve_in,
    ];
    let working_line = [valve_out, Pt::new(valve_out.x, cylinder_in.y), cylinder_in];
    surface.line(&supply_line, Stroke::new(width, ink.supply));
    surface.line(&unit_line, Stroke::new(width, ink.supply));
    surface.line(&working_line, Stroke::new(width, working));

    // A line that carries air says which way it goes, as the reference does: a solid
    // arrowhead on the line, pointing with the flow and turning around when the valve
    // reverses it. A line with no flow carries none, so the arrow is the flow and not a
    // decoration.
    let size = placed.valve.height() * 0.06;
    draw_flow_arrows(surface, &unit_line, system.supply_flow(), size, ink.supply);
    draw_flow_arrows(surface, &working_line, system.working_flow(), size, working);
}

/// Arrowheads along a line, one every `at` fraction of its length.
fn draw_flow_arrows(surface: &mut dyn Surface, line: &[Pt], flow: f64, size: f32, colour: Colour) {
    if flow.abs() <= f64::EPSILON {
        return;
    }
    let sign = if flow > 0.0 { 1.0 } else { -1.0 };
    // One arrow in the middle of every stretch long enough to hold it, so a line that
    // bends keeps saying which way the air is going after the bend.
    for pair in line.windows(2) {
        let (dx, dy) = pair[0].to(pair[1]);
        let length = dx.hypot(dy);
        if length < size * 4.0 {
            continue;
        }
        let point = Pt::new(pair[0].x + dx * 0.5, pair[0].y + dy * 0.5);
        let direction = Pt::new(dx / length * sign, dy / length * sign);
        symbol::draw_flow_arrow(surface, point, direction, size, colour);
    }
}

/// Working line colour: the neutral ink at atmosphere, the working colour at supply
/// pressure. The status line always carries the number too, so the state is never
/// told by colour alone.
fn pressure_colour(system: &Network, ink: &Ink) -> Colour {
    let span = system.supply_pa() - system.atmosphere_pa();
    if span <= 0.0 {
        return ink.line;
    }
    let ratio = ((system.pressure_pa() - system.atmosphere_pa()) / span).clamp(0.0, 1.0);
    #[allow(clippy::cast_possible_truncation)]
    let mix = ratio as f32;
    blend(ink.line, ink.work, mix)
}

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

fn draw_labels(
    surface: &mut dyn Surface,
    placed: &Layout,
    system: &Network,
    supply: &ComponentDefinition,
    valve: &ComponentDefinition,
    cylinder: &ComponentDefinition,
    ink: &Ink,
) {
    // The supply caption goes above its symbol, not under it: its only port leaves downwards
    // and the line to the valve's port 1 leaves from there, so a caption below would be
    // written across the pipe. The cylinder's port is on the left, so it has room under it.
    symbol::haloed_text(
        surface,
        Pt::new(
            placed.supply.center().x,
            placed.supply.min.y - placed.supply.height() * 0.24,
        ),
        supply.display_name(),
        symbol::NUMERAL,
        ink.muted,
        ink.background,
    );
    symbol::haloed_text(
        surface,
        Pt::new(
            placed.cylinder.center().x,
            placed.cylinder.max.y + placed.cylinder.height() * 0.24,
        ),
        cylinder.display_name(),
        symbol::NUMERAL,
        ink.muted,
        ink.background,
    );

    // The valve's caption sits **above** the body and runs leftwards: the ports leave
    // downwards, and the right-hand side of the sheet is the margin. A caption that starts
    // to the right of the valve runs past the content and prints in the margin — which is
    // exactly what the margin test caught when the composition moved. A halo over a hose is
    // the caption doing its job; ink in the margin is a layout bug.
    symbol::haloed_text_anchored(
        surface,
        Pt::new(
            placed.valve.max.x,
            placed.valve.min.y - placed.valve.height() * 0.10,
        ),
        valve.display_name(),
        symbol::NUMERAL,
        ink.muted,
        ink.background,
        Anchor::End,
    );
    if let Some(name) = system
        .valve()
        .and_then(|valve| valve.position_names.get(system.valve_position()))
    {
        symbol::haloed_text_anchored(
            surface,
            Pt::new(
                placed.valve.max.x,
                placed.valve.min.y - placed.valve.height() * 0.02,
            ),
            name,
            symbol::NUMERAL * 0.85,
            ink.line,
            ink.background,
            Anchor::End,
        );
    }

    let gauge = (system.pressure_pa() - system.atmosphere_pa()) / 100_000.0;
    let percent = system.extension() * 100.0;
    symbol::haloed_text(
        surface,
        Pt::new(
            placed.cylinder.center().x,
            placed.cylinder.max.y + placed.cylinder.height() * 0.38,
        ),
        &format!("{percent:.0} % · {gauge:.1} bar"),
        symbol::NUMERAL * 0.85,
        ink.line,
        ink.background,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::Recording;

    fn system() -> Network {
        use punchi_fluid_solver::network::{NetworkPart, PortAddress};
        let catalog = Catalog::builtin();
        let parts: Vec<NetworkPart> = [SUPPLY_ID, VALVE_ID, CYLINDER_ID]
            .into_iter()
            .map(|id| {
                NetworkPart::from_definition(catalog.get(id).expect("the definition exists"))
                    .expect("the definition is a part")
            })
            .collect();
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
        ];
        Network::build(&parts, &pipes).expect("the built-in circuit starts")
    }

    #[test]
    fn a_small_area_produces_no_layout() {
        assert!(layout(Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0)).is_none());
        assert!(layout(Rect::from_min_size(Pt::new(0.0, 0.0), 180.0, 120.0)).is_some());
    }

    #[test]
    fn the_circuit_draws_something_in_every_area_big_enough() {
        let catalog = Catalog::builtin();
        let circuit = system();
        let mut recording = Recording::default();
        let placed = draw(
            &mut recording,
            Rect::from_min_size(Pt::new(0.0, 0.0), 180.0, 120.0),
            &circuit,
            &catalog,
            &Ink::print(),
        );
        assert!(placed.is_some());
        assert!(
            recording.len() > 20,
            "a circuit is more than a couple of calls"
        );
    }

    /// Filled triangles in the gap between valve and cylinder: the flow arrows.
    fn working_arrows(recording: &Recording, placed: &Layout) -> Vec<Vec<Pt>> {
        // Only the gap between the valve and the cylinder: the supply line runs below
        // the valve and a filled arrowhead inside a symbol belongs to that symbol.
        let upper = placed.valve.min.y + placed.valve.height() * 0.05;
        let lower = placed.cylinder.center().y - placed.valve.height() * 0.25;
        let valve = placed.valve.expand(placed.valve.height() * 0.02);
        let cylinder = placed.cylinder.expand(placed.cylinder.height() * 0.02);
        let supply = placed.supply.expand(placed.supply.height() * 0.02);
        recording
            .calls()
            .iter()
            .filter_map(|call| {
                let crate::surface::Call::Polygon {
                    points,
                    fill: Some(_),
                    ..
                } = call
                else {
                    return None;
                };
                let in_the_gap = points.iter().all(|point| {
                    point.y < upper
                        && point.y > lower
                        && !valve.contains(*point)
                        && !cylinder.contains(*point)
                        && !supply.contains(*point)
                });
                in_the_gap.then(|| points.clone())
            })
            .collect()
    }

    /// An arrowhead points to its tip: the vertex farthest from the back of the head.
    fn points_towards(triangle: &[Pt], target: Pt) -> bool {
        #[allow(clippy::cast_precision_loss)]
        let count = triangle.len() as f32;
        let centre = Pt::new(
            triangle.iter().map(|point| point.x).sum::<f32>() / count,
            triangle.iter().map(|point| point.y).sum::<f32>() / count,
        );
        let Some((index, tip)) = triangle.iter().enumerate().max_by(|a, b| {
            let (ax, ay) = centre.to(*a.1);
            let (bx, by) = centre.to(*b.1);
            ax.hypot(ay)
                .partial_cmp(&bx.hypot(by))
                .unwrap_or(std::cmp::Ordering::Equal)
        }) else {
            return false;
        };
        let back = triangle
            .iter()
            .enumerate()
            .filter(|(position, _)| *position != index)
            .fold((0.0f32, 0.0f32), |sum, (_, point)| {
                (sum.0 + point.x, sum.1 + point.y)
            });
        let back = Pt::new(back.0 / 2.0, back.1 / 2.0);
        let (tx, ty) = tip.to(target);
        let (bx, by) = back.to(target);
        tx.hypot(ty) < bx.hypot(by)
    }

    #[test]
    fn a_line_with_air_in_it_carries_an_arrow_pointing_where_the_air_goes() {
        let catalog = Catalog::builtin();
        let mut circuit = system();
        let area = Rect::from_min_size(Pt::new(0.0, 0.0), 180.0, 120.0);

        // Actuated, the valve fills the cylinder: the arrows point at it.
        circuit.press_valve();
        circuit.advance(0.05).expect("the model advances");
        assert!(
            circuit.working_flow() > 0.0,
            "actuated, the supply fills the cylinder"
        );
        let mut filling = Recording::default();
        let placed = draw(&mut filling, area, &circuit, &catalog, &Ink::print())
            .expect("the area is big enough");
        let arrows = working_arrows(&filling, &placed);
        assert!(
            !arrows.is_empty(),
            "the working line carries an arrow while air moves"
        );
        let intake = port(
            catalog.get(CYLINDER_ID).expect("the cylinder exists"),
            placed.cylinder,
            PortRole::Working,
        )
        .expect("the cylinder has a working port");
        assert!(
            arrows.iter().all(|arrow| points_towards(arrow, intake)),
            "filling the cylinder points at its port"
        );

        // Released, the spring vents it and the same arrows turn around.
        circuit.release_valve();
        circuit.advance(0.01).expect("the model advances");
        assert!(
            circuit.working_flow() < 0.0,
            "released, the cylinder vents through the valve"
        );
        let mut venting = Recording::default();
        let placed = draw(&mut venting, area, &circuit, &catalog, &Ink::print())
            .expect("the area is big enough");
        let arrows = working_arrows(&venting, &placed);
        assert!(
            !arrows.is_empty(),
            "the line still carries air, the other way"
        );
        let intake = port(
            catalog.get(CYLINDER_ID).expect("the cylinder exists"),
            placed.cylinder,
            PortRole::Working,
        )
        .expect("the cylinder has a working port");
        assert!(
            arrows.iter().all(|arrow| !points_towards(arrow, intake)),
            "venting points away from the cylinder's port"
        );
    }

    #[test]
    fn the_working_line_colour_follows_the_pressure() {
        let ink = Ink::print();
        let circuit = system();
        let at_rest = pressure_colour(&circuit, &ink);
        assert_eq!(
            at_rest, ink.line,
            "at atmosphere the line keeps the ink colour"
        );
    }
}
