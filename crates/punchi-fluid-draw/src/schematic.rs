// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The circuit on one page: placement, pipes and readouts.
//!
//! This is the drawing the window shows and the PDF prints. It takes an area in
//! millimetres and never looks at the device, so the screen and the paper differ only
//! in the scale the backend applies.

use punchi_fluid_catalog::{Catalog, ComponentDefinition, PortRole};
use punchi_fluid_solver::pneumatic::PneumaticSystem;

use crate::surface::{Anchor, Colour, Pt, Rect, Stroke, Surface};
use crate::symbol::{self, Ink};

/// Identifier of the supply of the fixed circuit.
pub const SUPPLY_ID: &str = punchi_fluid_solver::pneumatic::DEFAULT_SUPPLY;
/// Identifier of the valve of the fixed circuit.
pub const VALVE_ID: &str = punchi_fluid_solver::pneumatic::DEFAULT_VALVE;
/// Identifier of the cylinder of the fixed circuit.
pub const CYLINDER_ID: &str = punchi_fluid_solver::pneumatic::DEFAULT_CYLINDER;

/// Where the three components sit, in millimetres of the drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// Supply symbol box.
    pub supply: Rect,
    /// Valve symbol box.
    pub valve: Rect,
    /// Cylinder symbol box.
    pub cylinder: Rect,
}

/// Fixed placement of the three components inside `area`.
///
/// Returns `None` when the area is too small to draw something readable, so a tiny
/// canvas or a thumbnail does not produce a smudge.
#[must_use]
pub fn layout(area: Rect) -> Option<Layout> {
    let square = (area.height() * 0.26).min(area.width() * 0.20);
    if square < 8.0 {
        return None;
    }
    let supply = Rect::from_center_size(area.point_at(0.17, 0.30), square, square);
    let valve = Rect::from_center_size(area.point_at(0.47, 0.72), square, square);
    let cylinder = Rect::from_center_size(area.point_at(0.72, 0.26), square * 2.2, square);
    Some(Layout {
        supply,
        valve,
        cylinder,
    })
}

/// Draws the circuit and returns where it placed the components.
pub fn draw(
    surface: &mut dyn Surface,
    area: Rect,
    system: &PneumaticSystem,
    catalog: &Catalog,
    ink: &Ink,
) -> Option<Layout> {
    let placed = layout(area)?;
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
        return Some(placed);
    };

    draw_pipes(surface, &placed, system, supply, valve, cylinder, ink);
    symbol::draw_component(surface, placed.supply, supply, 0, 0.0, ink);
    symbol::draw_valve(surface, placed.valve, valve, system.valve_position(), ink);
    // One call: the cylinder draws its own piston and rod at the state of the model, so
    // it cannot be drawn twice at two different extensions.
    #[allow(clippy::cast_possible_truncation)]
    let extension = system.extension() as f32;
    symbol::draw_component(surface, placed.cylinder, cylinder, 0, extension, ink);
    draw_labels(surface, &placed, system, supply, valve, cylinder, ink);
    Some(placed)
}

fn port(definition: &ComponentDefinition, rect: Rect, role: PortRole) -> Option<Pt> {
    definition
        .port_with_role(role)
        .map(|port| symbol::at(rect, port.position[0], port.position[1]))
}

fn draw_pipes(
    surface: &mut dyn Surface,
    placed: &Layout,
    system: &PneumaticSystem,
    supply: &ComponentDefinition,
    valve: &ComponentDefinition,
    cylinder: &ComponentDefinition,
    ink: &Ink,
) {
    let working = pressure_colour(system, ink);
    let width = symbol::LINE * 1.2;

    let (Some(supply_out), Some(valve_in), Some(valve_out), Some(cylinder_in)) = (
        port(supply, placed.supply, PortRole::Supply),
        port(valve, placed.valve, PortRole::Supply),
        port(valve, placed.valve, PortRole::Working),
        port(cylinder, placed.cylinder, PortRole::Working),
    ) else {
        return;
    };

    // Offsets are fractions of the symbol size, never millimetres of paper: the
    // composition must be the same on a window and on an A2 sheet.
    let below = supply_out.y.max(valve_in.y) + placed.supply.height() * 0.45;
    let supply_line = [
        supply_out,
        Pt::new(supply_out.x, below),
        Pt::new(valve_in.x, below),
        valve_in,
    ];
    let working_line = [valve_out, Pt::new(valve_out.x, cylinder_in.y), cylinder_in];
    surface.line(&supply_line, Stroke::new(width, ink.supply));
    surface.line(&working_line, Stroke::new(width, working));

    // A line that carries air says which way it goes, as the reference does: a solid
    // arrowhead on the line, pointing with the flow and turning around when the valve
    // reverses it. A line with no flow carries none, so the arrow is the flow and not a
    // decoration.
    let size = placed.valve.height() * 0.06;
    draw_flow_arrows(
        surface,
        &supply_line,
        system.supply_flow(),
        size,
        ink.supply,
    );
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
fn pressure_colour(system: &PneumaticSystem, ink: &Ink) -> Colour {
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
    system: &PneumaticSystem,
    supply: &ComponentDefinition,
    valve: &ComponentDefinition,
    cylinder: &ComponentDefinition,
    ink: &Ink,
) {
    for (rect, definition) in [(placed.supply, supply), (placed.cylinder, cylinder)] {
        symbol::haloed_text(
            surface,
            Pt::new(rect.center().x, rect.max.y + rect.height() * 0.24),
            &definition.name,
            symbol::NUMERAL,
            ink.muted,
            ink.background,
        );
    }

    // The valve's caption sits beside the body, not under it: the ports leave downwards,
    // so a caption underneath would be written across the supply line climbing to port 1.
    symbol::haloed_text_anchored(
        surface,
        Pt::new(
            placed.valve.max.x + placed.valve.width() * 0.04,
            placed.valve.max.y + placed.valve.height() * 0.18,
        ),
        &valve.name,
        symbol::NUMERAL,
        ink.muted,
        ink.background,
        Anchor::Start,
    );
    if let Some(name) = system.valve().position_names.get(system.valve_position()) {
        symbol::haloed_text_anchored(
            surface,
            Pt::new(
                placed.valve.max.x + placed.valve.width() * 0.04,
                placed.valve.max.y + placed.valve.height() * 0.32,
            ),
            name,
            symbol::NUMERAL * 0.85,
            ink.line,
            ink.background,
            Anchor::Start,
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

    fn system() -> PneumaticSystem {
        let catalog = Catalog::builtin();
        PneumaticSystem::from_catalog(&catalog).expect("the built-in circuit starts")
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
