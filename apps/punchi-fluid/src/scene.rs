// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The static scene of the document, for the window's drawing.
//!
//! The placements and pipes of the active circuit, with the model's state left at rest: a
//! static sheet is the document's geometry. The live valves, readings and flow pulses arrive
//! with the session's frame path, which is the next slice.

use punchi_fluid_doc::Document;
use punchi_fluid_draw::placed::{Pipe, PipeEnd, Placed};

/// What the drawing needs: the boxes and the hoses of one circuit.
#[derive(Debug)]
pub struct Scene<'a> {
    pub placed: Vec<Placed<'a>>,
    pub pipes: Vec<Pipe<'a>>,
}

/// Builds the scene of a circuit from the document as it is now.
#[must_use]
pub fn build(document: &Document, circuit: u32) -> Scene<'_> {
    let mut scene_of = vec![None; document.placements().len()];
    let mut placed: Vec<Placed<'_>> = Vec::new();
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        scene_of[index] = Some(placed.len());
        placed.push(Placed {
            index,
            catalog_id: placement.catalog_id.as_str(),
            rect: placement.rect(),
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            contact_closed: None,
            orientation: placement.orientation,
            designation: placement.designation.as_deref(),
            closures: &[],
            exhausts: &[],
        });
    }
    let pipes: Vec<Pipe<'_>> = document
        .connections()
        .iter()
        .filter(|connection| connection.circuit == circuit)
        .filter_map(|connection| {
            let from = document.index_of(connection.from.placement)?;
            let to = document.index_of(connection.to.placement)?;
            if document.placements().get(from)?.circuit != circuit
                || document.placements().get(to)?.circuit != circuit
            {
                return None;
            }
            Some(Pipe {
                from: PipeEnd {
                    placement: scene_of[from]?,
                    port: connection.from.port.as_str(),
                },
                to: PipeEnd {
                    placement: scene_of[to]?,
                    port: connection.to.port.as_str(),
                },
                offset_mm: connection.offset_mm(),
                pressure_ratio: 0.0,
                class: connection.class(),
                tube_od_hundredths: connection.tube_od_hundredths(),
                flow_l_min: 0.0,
                phase_s: 0.0,
                animate: false,
            })
        })
        .collect();
    Scene { placed, pipes }
}

#[cfg(test)]
mod tests {
    use punchi_fluid_doc::{Document, Endpoint, Placement};
    use punchi_fluid_draw::surface::Pt;

    use super::build;

    fn document_with_two_valves() -> Document {
        let mut document = Document::new();
        let supply = document.place(Placement::new(
            "supply.air",
            Pt::new(40.0, 40.0),
            24.0,
            24.0,
        ));
        let valve = document.place(Placement::new(
            "valve.5_2",
            Pt::new(120.0, 40.0),
            24.0,
            24.0,
        ));
        let first = document.placements()[supply].id();
        let second = document.placements()[valve].id();
        document.connect(Endpoint::new(first, "1"), Endpoint::new(second, "1"), None);
        document
    }

    #[test]
    fn the_scene_holds_the_circuits_placements_and_pipes() {
        let document = document_with_two_valves();
        let scene = build(&document, document.active_circuit());
        assert_eq!(scene.placed.len(), 2);
        assert_eq!(scene.pipes.len(), 1);
        assert_eq!(scene.pipes[0].from.port, "1");
    }

    #[test]
    fn a_placement_draws_something() {
        use punchi_fluid_draw::surface::Recording;

        let document = document_with_two_valves();
        let scene = build(&document, document.active_circuit());
        let catalog = punchi_fluid_catalog::default_catalog();
        let mut recording = Recording::default();
        punchi_fluid_draw::draw_placed(
            &mut recording,
            &scene.placed,
            &scene.pipes,
            &catalog,
            &punchi_fluid_draw::symbol::Ink::print(),
            &[],
        );
        assert!(!recording.is_empty(), "the circuit must draw its calls");
    }
}
