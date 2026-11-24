// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The static scene of the document, for the window's drawing.
//!
//! The placements and pipes of the active circuit, with model readings carried into the
//! drawing. The document owns geometry; the session owns the moving state.

use punchi_fluid_catalog::Catalog;
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
pub fn build<'a>(
    document: &'a Document,
    catalog: &'a Catalog,
    circuit: u32,
    model: Option<&punchi_fluid_session::Model>,
) -> Scene<'a> {
    build_for_frame(document, catalog, circuit, model, false)
}

/// Builds a live scene whose hose marks follow the model's clock when motion is enabled.
#[must_use]
pub fn build_for_frame<'a>(
    document: &'a Document,
    catalog: &'a Catalog,
    circuit: u32,
    model: Option<&punchi_fluid_session::Model>,
    animate_flow: bool,
) -> Scene<'a> {
    let mut scene_of = vec![None; document.placements().len()];
    let mut placed: Vec<Placed<'_>> = Vec::new();
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        scene_of[index] = Some(placed.len());
        let definition = catalog.get(placement.catalog_id.as_str());
        placed.push(Placed {
            index,
            catalog_id: placement.catalog_id.as_str(),
            rect: placement.rect(),
            extension: model
                .and_then(|model| model.extension(index))
                .unwrap_or(0.0),
            moving: model.and_then(|model| model.moving_seat(index)),
            valve_position: model.map_or(0, |model| model.valve_position(index)),
            reading: model.and_then(|model| {
                definition.and_then(|definition| model.reading(index, definition))
            }),
            series: None,
            contact_closed: None,
            orientation: placement.orientation,
            designation: placement.designation.as_deref(),
            closures: &[],
            exhausts: &[],
        });
    }
    #[allow(clippy::cast_possible_truncation)]
    let phase_s = model.map_or(0.0, |model| model.network().telemetry().time as f32);
    let pipes: Vec<Pipe<'_>> = document
        .connections()
        .iter()
        .enumerate()
        .filter(|(_, connection)| connection.circuit == circuit)
        .filter_map(|(index, connection)| {
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
                pressure_ratio: model.map_or(0.0, |model| model.hose_pressure(document, index)),
                class: connection.class(),
                tube_od_hundredths: connection.tube_od_hundredths(),
                flow_l_min: model
                    .and_then(|model| model.pipe_flow_l_min(index))
                    .unwrap_or(0.0),
                phase_s,
                animate: animate_flow,
            })
        })
        .collect();
    Scene { placed, pipes }
}

#[cfg(test)]
mod tests {
    use punchi_fluid_doc::{Document, Endpoint, Placement};
    use punchi_fluid_draw::surface::{Pt, Rect};

    use super::{build, build_for_frame};

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
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let scene = build(&document, &catalog, document.active_circuit(), None);
        assert_eq!(scene.placed.len(), 2);
        assert_eq!(scene.pipes.len(), 1);
        assert_eq!(scene.pipes[0].from.port, "1");
    }

    #[test]
    fn a_placement_draws_something() {
        use punchi_fluid_draw::surface::Recording;

        let document = document_with_two_valves();
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let scene = build(&document, &catalog, document.active_circuit(), None);
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

    #[test]
    fn the_worked_example_draws_solver_flow_and_uses_simulated_time() {
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let area = Rect::from_corners(Pt::new(0.0, 0.0), Pt::new(300.0, 200.0));
        let document = punchi_fluid_session::templates::document(0, &catalog, area)
            .expect("the worked example is available");
        let mut model = punchi_fluid_session::Model::build(&document, &catalog)
            .expect("the worked example has a running model");
        assert!(model.press_valve(2), "the first working valve is momentary");
        model.advance(0.016).expect("one frame advances the model");

        let live = build_for_frame(
            &document,
            &catalog,
            document.active_circuit(),
            Some(&model),
            true,
        );
        let cylinder = live
            .placed
            .iter()
            .find(|part| part.catalog_id == "cylinder.single_acting_20_100")
            .expect("the worked example contains its single-acting cylinder");
        assert!(cylinder.extension > 0.0, "the model moves the cylinder rod");
        assert!(
            live.pipes
                .iter()
                .any(|pipe| { pipe.flow_l_min.abs() > 0.0 && pipe.animate && pipe.phase_s > 0.0 })
        );

        let reduced_motion = build_for_frame(
            &document,
            &catalog,
            document.active_circuit(),
            Some(&model),
            false,
        );
        assert!(
            reduced_motion
                .pipes
                .iter()
                .any(|pipe| pipe.flow_l_min.abs() > 0.0 && !pipe.animate)
        );
    }
}
