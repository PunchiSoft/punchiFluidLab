// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The document's life between the file and the window.
//!
//! The sheet format stores circuits, placements, pipes and the paper; the document is the
//! live model the window edits. This crate is the one place that converts one to the other,
//! so the ABI, the window and the tests open and save the same way. The live model of a
//! circuit is [`Model`]: the window runs it without the ABI. The editor state (selection,
//! tools, undo history) joins it with the next slice.

use std::path::Path;

use punchi_fluid_doc::{Circuit, Document, Endpoint, Placement, PlacementEdits, Quantity, Source};
use punchi_fluid_draw::Orientation;
use punchi_fluid_draw::surface::{LineClass, Pt};
use punchi_fluid_file::paper::StoredSheet;
mod model;
pub mod templates;

pub use model::Model;

use punchi_fluid_file::sheet::{
    SHEET_VERSION, Sheet, StoredBlock, StoredCircuit, StoredEdits, StoredPipe, StoredPlacement,
};

/// Builds a document from a parsed sheet, or explains why it cannot.
///
/// # Errors
///
/// Returns a message when the project has no usable circuit or a pipe names an end that
/// cannot be connected.
pub fn load(sheet: &Sheet) -> Result<Document, String> {
    let placements: Vec<Placement> = sheet
        .placement
        .iter()
        .map(|stored| {
            let mut placement = Placement::with_id(
                stored.id,
                stored.catalog.clone(),
                Pt::new(stored.x, stored.y),
                stored.width,
                stored.height,
            );
            placement.circuit = stored.circuit;
            if let Some(edit) = stored.edit {
                placement.edits = PlacementEdits {
                    pressure_bar: edit.pressure_bar,
                    conductance: edit.conductance,
                    stroke_mm: edit.stroke_mm,
                    bore_mm: edit.bore_mm,
                    rod_mm: edit.rod_mm,
                    moving_mass_kg: edit.moving_mass_kg,
                };
            }
            placement.orientation = Orientation::new(stored.quarter_turns, stored.mirrored);
            placement.source = stored.source.as_ref().and_then(|source| {
                let quantity = Quantity::from_name(&source.quantity)?;
                Some(Source::new(source.placement, quantity))
            });
            placement.designation.clone_from(&stored.designation);
            placement.port_closures.clone_from(&stored.port_closures);
            placement
        })
        .collect();
    let mut circuits: Vec<Circuit> = sheet
        .circuits
        .iter()
        .map(|stored| {
            let mut circuit = Circuit::new(stored.id, stored.title.clone());
            circuit.block = stored.block.to_block(&stored.title);
            circuit
        })
        .collect();
    if circuits.is_empty() {
        circuits.push(Circuit::new(0, "Circuito 1"));
    }
    let active = if circuits.iter().any(|circuit| circuit.id == sheet.active) {
        sheet.active
    } else {
        circuits[0].id
    };
    let mut document = Document::new();
    if !document.set_project(circuits, active) {
        return Err(String::from("the project has no usable circuit"));
    }
    // The sheet is part of the file: a drawing opened on A3 stays on A3, and a file written
    // before the paper existed opens on the unframed A4 its geometry expects.
    document.seed_sheet(sheet.paper_sheet());
    document.seed(placements);
    for pipe in &sheet.pipe {
        if !document.seed_connect(
            Endpoint::new(pipe.from, pipe.from_port.clone()),
            Endpoint::new(pipe.to, pipe.to_port.clone()),
        ) {
            return Err(format!(
                "the pipe {}:{} -> {}:{} cannot be connected",
                pipe.from, pipe.from_port, pipe.to, pipe.to_port
            ));
        }
        let index = document.connections().len() - 1;
        document.seed_pipe_circuit(index, pipe.circuit);
        document.seed_route(index, pipe.offset_tenths);
        document.seed_pipe_class(
            index,
            match pipe.class {
                punchi_fluid_file::sheet::StoredClass::Working => LineClass::Working,
                punchi_fluid_file::sheet::StoredClass::Pilot => LineClass::Pilot,
                punchi_fluid_file::sheet::StoredClass::Drain => LineClass::Drain,
            },
        );
        // The size is read verbatim: an old sheet has no key and stays unspecified, which is
        // not the same as a new pipe's documented 10 mm.
        document.seed_pipe_tube(index, pipe.tube_od_hundredths);
    }
    Ok(document)
}

/// The document as the sheet format stores it.
#[must_use]
pub fn to_sheet(document: &Document) -> Sheet {
    let placement = document
        .placements()
        .iter()
        .map(|placement| {
            let edits = &placement.edits;
            let edit = if edits.is_empty() {
                None
            } else {
                Some(StoredEdits {
                    pressure_bar: edits.pressure_bar,
                    conductance: edits.conductance,
                    stroke_mm: edits.stroke_mm,
                    bore_mm: edits.bore_mm,
                    rod_mm: edits.rod_mm,
                    moving_mass_kg: edits.moving_mass_kg,
                })
            };
            // Tenths of a millimetre are the precision the sheet draws at; storing the raw
            // `f32` only writes the layout's arithmetic noise into the file.
            let round = |value: f32| (value * 10.0).round() / 10.0;
            StoredPlacement {
                id: placement.id(),
                circuit: placement.circuit,
                catalog: placement.catalog_id.clone(),
                x: round(placement.centre.x),
                y: round(placement.centre.y),
                width: round(placement.width),
                height: round(placement.height),
                edit,
                quarter_turns: placement.orientation.quarter_turns(),
                mirrored: placement.orientation.mirrored(),
                designation: placement.designation.clone(),
                source: placement
                    .source
                    .map(|source| punchi_fluid_file::sheet::StoredSource {
                        placement: source.placement,
                        quantity: source.quantity.as_str().to_owned(),
                    }),
                port_closures: placement.port_closures.clone(),
            }
        })
        .collect();
    let pipe = document
        .connections()
        .iter()
        .map(|connection| StoredPipe {
            circuit: connection.circuit,
            from: connection.from.placement,
            from_port: connection.from.port.clone(),
            to: connection.to.placement,
            to_port: connection.to.port.clone(),
            offset_tenths: connection.offset_tenths(),
            class: match connection.class() {
                LineClass::Pilot => punchi_fluid_file::sheet::StoredClass::Pilot,
                LineClass::Drain => punchi_fluid_file::sheet::StoredClass::Drain,
                // The file has no electrical class yet, so a line the format cannot name is
                // written as working until F6 gives it one.
                LineClass::Working | LineClass::Electrical => {
                    punchi_fluid_file::sheet::StoredClass::Working
                }
            },
            tube_od_hundredths: connection.tube_od_hundredths(),
        })
        .collect();
    let circuits = document
        .circuits()
        .iter()
        .map(|circuit| StoredCircuit {
            id: circuit.id,
            title: circuit.title.clone(),
            block: StoredBlock::from_block(&circuit.block),
        })
        .collect();
    Sheet {
        format: SHEET_VERSION,
        title: String::from("Circuito"),
        sheet: StoredSheet::from(document.sheet()),
        circuits,
        placement,
        pipe,
        active: document.active_circuit(),
    }
}

/// Writes the document to `path` as a sheet file.
///
/// The same text `load` reads: what the window saves is what it can open again.
///
/// # Errors
///
/// Returns a message when the file cannot be written.
pub fn save(document: &Document, path: &Path) -> Result<(), String> {
    to_sheet(document)
        .write(path)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use punchi_fluid_doc::{Document, Placement};
    use punchi_fluid_draw::surface::Pt;
    use punchi_fluid_file::sheet::Sheet;

    use super::{load, save, to_sheet};

    #[test]
    fn a_new_document_survives_the_round_trip() {
        let document = Document::new();
        let sheet = to_sheet(&document);
        let loaded = load(&sheet).expect("a new document round-trips");
        assert_eq!(loaded.circuits().len(), document.circuits().len());
        assert_eq!(loaded.active_circuit(), document.active_circuit());
        assert_eq!(loaded.sheet().size_mm(), document.sheet().size_mm());
    }

    #[test]
    fn a_placement_survives_the_round_trip() {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.5_2",
            Pt::new(100.0, 80.0),
            24.0,
            24.0,
        ));
        let sheet = to_sheet(&document);
        let loaded = load(&sheet).expect("a document with a placement round-trips");
        assert_eq!(loaded.placements().len(), 1);
        assert_eq!(loaded.placements()[0].catalog_id, "valve.5_2");
        assert!((loaded.placements()[0].centre.x - 100.0).abs() < 0.01);
    }

    #[test]
    fn a_saved_document_is_the_one_that_loads_again() {
        let mut document = Document::new();
        document.place(Placement::new(
            "valve.5_2",
            Pt::new(100.0, 80.0),
            24.0,
            24.0,
        ));
        let path = std::env::temp_dir().join(format!(
            "punchi-fluid-session-save-{}.pfluid",
            std::process::id()
        ));
        save(&document, &path).expect("the document saves");
        let text = std::fs::read_to_string(&path).expect("the file was written");
        let sheet = Sheet::parse(&text).expect("the saved text is a sheet");
        let loaded = load(&sheet).expect("the saved sheet loads");
        assert_eq!(loaded.placements().len(), 1);
        assert_eq!(loaded.placements()[0].catalog_id, "valve.5_2");
        let _ = std::fs::remove_file(&path);
    }
}
