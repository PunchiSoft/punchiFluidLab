// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A plug or a silencer drawn on the port it closes.
//!
//! A closure belongs to the port, not to the sheet: it is not a placement with its own
//! box and its own hose, it is the choice made on one port. The drawing therefore puts
//! the closure's own symbol outside that port, along the edge the port sits on, with the
//! closure's own port facing the host. The symbol comes from the closure's catalog file,
//! so the plug's bar and the silencer's cone are the ones the file declares and no new
//! geometry is written here.

use punchi_fluid_catalog::ComponentDefinition;

use crate::orient::{Orientation, Turned};
use crate::surface::{Pt, Rect, Surface};
use crate::symbol::{self, Ink};

use super::edge::turned_edge;
use super::{PortEdge, port_edge, port_point};

/// Side of the small square a closure is drawn in, in millimetres.
///
/// It is smaller than a component's 20 mm box because it is an accessory fitted to a
/// port, not a part of the circuit: the primitives scale to this square, so a new closure
/// symbol is a file and not a different size.
pub const CLOSURE_MM: f32 = 10.0;

/// The box and orientation of one closure, outside its host port.
///
/// The closure's own port touches the host port's outer point and faces the host, so the
/// air leaves the host into the closure. It sits on the host's **turned** edge: a host
/// rotated a quarter turn takes its closures with it instead of leaving them behind.
fn closure_frame(
    rect: Rect,
    orientation: Orientation,
    definition: &ComponentDefinition,
    port: &str,
) -> Option<(Rect, Orientation)> {
    let point = port_point(rect, orientation, definition, port)?;
    let host = orientation.box_of(rect);
    let half = CLOSURE_MM * 0.5;
    let (min, closure_orientation) = match turned_edge(port_edge(definition, port), orientation) {
        PortEdge::Right => (
            Pt::new(host.max.x, point.y.clamp(host.min.y, host.max.y) - half),
            Orientation::NONE,
        ),
        PortEdge::Left => (
            Pt::new(
                host.min.x - CLOSURE_MM,
                point.y.clamp(host.min.y, host.max.y) - half,
            ),
            Orientation::NONE.reflected(),
        ),
        PortEdge::Bottom => (
            Pt::new(point.x.clamp(host.min.x, host.max.x) - half, host.max.y),
            Orientation::new(1, false),
        ),
        PortEdge::Top => (
            Pt::new(
                point.x.clamp(host.min.x, host.max.x) - half,
                host.min.y - CLOSURE_MM,
            ),
            Orientation::new(3, false),
        ),
    };
    Some((
        Rect::from_min_size(min, CLOSURE_MM, CLOSURE_MM),
        closure_orientation,
    ))
}

/// The box of every closure of a host, by port id.
///
/// The box grows the footprint the check measures and the export fits, even though the
/// closure is not a placement of its own. A closure naming a port the host does not have
/// is left out: the check reports it, the drawing has nothing to put beside.
#[must_use]
pub fn closure_boxes(
    rect: Rect,
    orientation: Orientation,
    definition: &ComponentDefinition,
    closures: &[(&str, &ComponentDefinition)],
) -> Vec<(String, Rect)> {
    closures
        .iter()
        .filter_map(|(port, _closure)| {
            let (box_rect, _) = closure_frame(rect, orientation, definition, port)?;
            Some(((*port).to_owned(), box_rect))
        })
        .collect()
}

/// Draws the closures of one host on top of its body, outside its ports.
///
/// The caller resolved the closure definitions from the catalog (the document has no
/// catalog), so this is the closure's own symbol at its own box and orientation: the
/// plug's blocking bar or the silencer's cone, never a second drawing of them here.
pub fn draw_closures(
    surface: &mut dyn Surface,
    rect: Rect,
    orientation: Orientation,
    definition: &ComponentDefinition,
    closures: &[(&str, &ComponentDefinition)],
    ink: &Ink,
) {
    for (port, closure) in closures {
        let Some((box_rect, closure_orientation)) =
            closure_frame(rect, orientation, definition, port)
        else {
            continue;
        };
        // The closure's own port numeral is suppressed: it is part of the host port, not a
        // component with a number a reader should follow.
        if closure_orientation.is_identity() {
            symbol::draw_component_without_numerals(surface, box_rect, closure, 0, 0.0, None, ink);
        } else {
            let mut turned = Turned::new(surface, closure_orientation, box_rect.center());
            symbol::draw_component_without_numerals(
                &mut turned,
                box_rect,
                closure,
                0,
                0.0,
                None,
                ink,
            );
        }
    }
}
