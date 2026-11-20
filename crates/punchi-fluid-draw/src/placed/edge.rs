// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which way a hose leaves a component.
//!
//! [`super::port_edge`] reads the declared position alone. A junction's three ports all sit
//! at the centre, so that answer is a tie; this module takes the other end of the hose as
//! the hint that breaks it, and the check and the drawing both pass the same one.

use punchi_fluid_catalog::ComponentDefinition;

use crate::orient::Orientation;
use crate::surface::{Pt, Rect};

use super::{PortEdge, port_edge, port_point};

/// How near the four edges may be before a port is read as sitting at the centre,
/// as a fraction of the symbol box.
///
/// A junction's three ports are all declared at `[0.5, 0.5]`, so every edge is the
/// same distance away. Below this the four distances are one answer, not a race
/// between `Left` and `Right`; see [`port_edge_towards`].
pub const EDGE_TIE_MM: f32 = 0.01;

/// The edge a port leaves on, using the other end of the hose to break a centre tie.
///
/// [`port_edge`] answers from the declared position alone. That is enough for a port
/// on or near one edge, including a corner where two edges are equally near: the
/// first minimum is kept exactly as before. A component whose ports all sit at the
/// centre — a T junction — has no nearest edge, so the hint picks the side the hose
/// faces. The check and the drawing pass the same hint, which is what keeps the
/// inspection and the printed sheet on one route.
#[must_use]
pub fn port_edge_towards(
    rect: Rect,
    orientation: Orientation,
    definition: &ComponentDefinition,
    id: &str,
    towards: Option<Pt>,
) -> PortEdge {
    let Some(port) = definition.port.iter().find(|port| port.id == id) else {
        return port_edge(definition, id);
    };
    let [x, y] = port.position;
    let distances = [x, 1.0 - x, y, 1.0 - y];
    let min = distances.iter().copied().fold(f64::INFINITY, f64::min);
    let max = distances.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if max - min > f64::from(EDGE_TIE_MM) {
        // An ordinary port, or the corner whose two minima tie: the old answer is the
        // answer, first minimum included.
        return port_edge(definition, id);
    }
    if let Some(towards) = towards {
        if let Some(at) = port_point(rect, orientation, definition, id) {
            let (dx, dy) = at.to(towards);
            if dx.hypot(dy) > EDGE_TIE_MM {
                return if dx.abs() > dy.abs() {
                    if dx > 0.0 {
                        PortEdge::Right
                    } else {
                        PortEdge::Left
                    }
                } else if dy > 0.0 {
                    PortEdge::Bottom
                } else {
                    PortEdge::Top
                };
            }
        }
    }
    // No usable hint: the branch convention of a splice, in the placement's own frame:
    // port 1 left, port 2 right, port 3 up. The catalogue declares no other centre-tied
    // port, so an id outside that set keeps the old answer.
    let local = match id {
        "1" => PortEdge::Left,
        "2" => PortEdge::Right,
        "3" => PortEdge::Top,
        _ => return port_edge(definition, id),
    };
    turned_edge(local, orientation)
}

/// The edge `edge` after the placement's mirror and quarter turns.
fn turned_edge(edge: PortEdge, orientation: Orientation) -> PortEdge {
    let mirrored = match edge {
        PortEdge::Left => PortEdge::Right,
        PortEdge::Right => PortEdge::Left,
        PortEdge::Top => PortEdge::Bottom,
        PortEdge::Bottom => PortEdge::Top,
    };
    let clockwise = |edge| match edge {
        PortEdge::Left => PortEdge::Top,
        PortEdge::Top => PortEdge::Right,
        PortEdge::Right => PortEdge::Bottom,
        PortEdge::Bottom => PortEdge::Left,
    };
    let mut edge = if orientation.mirrored() {
        mirrored
    } else {
        edge
    };
    for _ in 0..orientation.quarter_turns() {
        edge = clockwise(edge);
    }
    edge
}
