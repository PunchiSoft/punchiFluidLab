// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the saved sheet and its migrations.

use super::*;
use crate::paper::{StoredMargins, StoredPaper, StoredSheet};
use punchi_fluid_core::paper::{Margins, Orientation, PaperSize};

fn sheet() -> Sheet {
    Sheet {
        format: SHEET_VERSION,
        title: String::from("Prueba"),
        sheet: StoredSheet::a4(),
        circuits: vec![StoredCircuit {
            id: 0,
            title: String::from("Circuito 1"),
            block: StoredBlock {
                title: String::from("Circuito 1"),
                ..StoredBlock::default()
            },
        }],
        placement: vec![
            StoredPlacement {
                id: 1,
                circuit: 0,
                catalog: String::from("supply.compressor"),
                x: 10.0,
                y: 20.0,
                width: 26.0,
                height: 26.0,
                edit: None,
                quarter_turns: 0,
                mirrored: false,
                designation: None,
                source: None,
            },
            StoredPlacement {
                id: 2,
                circuit: 0,
                catalog: String::from("cylinder.double_acting_32_200"),
                x: 120.0,
                y: 80.0,
                width: 62.0,
                height: 26.0,
                edit: Some(StoredEdits {
                    stroke_mm: Some(150.0),
                    ..StoredEdits::default()
                }),
                quarter_turns: 1,
                mirrored: true,
                designation: Some(String::from("1A1")),
                source: None,
            },
        ],
        pipe: vec![StoredPipe {
            circuit: 0,
            from: 1,
            from_port: String::from("1"),
            to: 2,
            to_port: String::from("2"),
            offset_tenths: 25,
            class: StoredClass::Working,
        }],
        active: 0,
    }
}

#[test]
fn a_sheet_round_trips_byte_identically() {
    let sheet = sheet();
    let first = sheet.to_json().expect("valid");
    let parsed = Sheet::parse(&first).expect("parses");
    assert_eq!(parsed, sheet);
    assert_eq!(parsed.to_json().expect("valid"), first);
}

#[test]
fn an_unknown_key_is_rejected_instead_of_ignored() {
    let text = r#"{"format":1,"placement":[],"pipe":[],"surprise":true}"#;
    assert!(matches!(Sheet::parse(text), Err(SheetError::Parse(_))));
}

#[test]
fn a_future_format_is_refused() {
    let text = r#"{"format":99,"placement":[],"pipe":[]}"#;
    assert!(matches!(
        Sheet::parse(text),
        Err(SheetError::UnsupportedVersion { found: 99, .. })
    ));
}

#[test]
fn bounds_are_checked_before_the_document_is_trusted() {
    let mut sheet = sheet();
    sheet.placement[0].width = 0.0;
    assert!(sheet.validate().is_err());
    sheet.placement[0].width = 26.0;
    sheet.placement[1].id = 1;
    assert!(sheet.validate().is_err(), "identities must be unique");
    sheet.placement[1].id = 2;
    sheet.pipe[0].from = 99;
    assert!(sheet.validate().is_err());
    sheet.pipe[0].from = 1;
    sheet.pipe[0].offset_tenths = 200_000;
    assert!(sheet.validate().is_err());
    sheet.pipe[0].offset_tenths = 0;
    sheet.placement[1].edit = Some(StoredEdits {
        rod_mm: Some(40.0),
        bore_mm: Some(32.0),
        ..StoredEdits::default()
    });
    assert!(sheet.validate().is_err(), "the rod must fit the bore");
}

#[test]
fn an_unturned_symbol_writes_no_orientation_at_all() {
    let mut sheet = sheet();
    sheet.placement[1].quarter_turns = 0;
    sheet.placement[1].mirrored = false;
    let text = sheet.to_json().expect("valid");
    assert!(!text.contains("quarter_turns"), "{text}");
    assert!(!text.contains("mirrored"), "{text}");
}

#[test]
fn a_version_two_sheet_still_reads_and_gains_an_unnamed_component() {
    // Exactly what the previous release wrote: orientation but no designation.
    let text = r#"{
          "format": 2,
          "title": "Anterior",
          "placement": [
            {"id": 1, "catalog": "valve.3_2.spring_return", "x": 10.0, "y": 20.0,
             "width": 30.0, "height": 15.0, "quarter_turns": 1, "mirrored": true}
          ],
          "pipe": []
        }"#;
    let sheet = Sheet::parse(text).expect("a version 2 sheet reads");
    assert_eq!(sheet.format, SHEET_VERSION);
    assert_eq!(sheet.placement[0].quarter_turns, 1);
    assert!(sheet.placement[0].mirrored);
    assert_eq!(sheet.placement[0].designation, None);
    assert!(
        sheet
            .to_json()
            .expect("valid")
            .contains(&format!(r#""format": {SHEET_VERSION}"#))
    );
}

#[test]
fn a_designation_that_cannot_be_read_is_refused() {
    let mut sheet = sheet();
    sheet.placement[0].designation = Some(String::from("  "));
    assert!(sheet.validate().is_err(), "whitespace is not a name");
    sheet.placement[0].designation = Some("x".repeat(MAX_DESIGNATION + 1));
    assert!(sheet.validate().is_err(), "too long");
    sheet.placement[0].designation = Some(String::from("1A1"));
    assert!(sheet.validate().is_ok());
    // An unnamed component writes nothing at all, so a sheet without names looks as it
    // did before the field existed.
    for placement in &mut sheet.placement {
        placement.designation = None;
    }
    assert!(!sheet.to_json().expect("valid").contains("designation"));
}

#[test]
fn a_version_one_sheet_still_reads_and_gains_an_unturned_orientation() {
    // Exactly what the previous release wrote: no orientation field, format 1.
    let text = r#"{
          "format": 1,
          "title": "Viejo",
          "placement": [
            {"id": 1, "catalog": "supply.compressor", "x": 10.0, "y": 20.0,
             "width": 26.0, "height": 26.0}
          ],
          "pipe": []
        }"#;
    let sheet = Sheet::parse(text).expect("a version 1 sheet reads");
    assert_eq!(sheet.format, SHEET_VERSION, "reading it upgrades it");
    assert_eq!(sheet.placement[0].quarter_turns, 0);
    assert!(!sheet.placement[0].mirrored);
    // Saving it writes the format this build understands, with the fields it added.
    let saved = sheet.to_json().expect("valid");
    assert!(
        saved.contains(&format!(r#""format": {SHEET_VERSION}"#)),
        "{saved}"
    );
    assert_eq!(sheet.placement[0].designation, None, "and no name either");
}

#[test]
fn a_turn_that_is_not_one_of_the_four_is_refused() {
    let mut sheet = sheet();
    sheet.placement[0].quarter_turns = 4;
    assert!(sheet.validate().is_err());
}

#[test]
fn a_version_seven_sheet_reads_with_its_circuit_title_in_the_block() {
    // Exactly what format 7 wrote: circuits and a sheet, no control block.
    let text = r#"{
          "format": 7,
          "title": "Anterior",
          "sheet": {},
          "circuits": [{"id": 0, "title": "Circuito 1"}],
          "placement": [],
          "pipe": []
        }"#;
    let sheet = Sheet::parse(text).expect("a version 7 sheet reads");
    assert_eq!(sheet.format, SHEET_VERSION);
    assert_eq!(sheet.circuits[0].block.title, "Circuito 1");
    assert_eq!(sheet.circuits[0].block.revision, "A");
    assert!(sheet.circuits[0].block.owner.is_empty());
}

#[test]
fn a_version_eight_sheet_reads_with_the_guide_defaults() {
    // Exactly what format 8 wrote: a block with the original eight fields.
    let text = r#"{
          "format": 8,
          "title": "Anterior",
          "sheet": {},
          "circuits": [{
            "id": 0,
            "title": "Circuito 1",
            "block": {
              "owner": "Curso de Neumática",
              "title": "Circuito 1",
              "number": "PF-2026-07",
              "revision": "B",
              "date": "2026-01-01",
              "prepared": "Ana",
              "checked": "Luis",
              "approved": "Marta",
              "position": "right"
            }
          }],
          "placement": [],
          "pipe": []
        }"#;
    let sheet = Sheet::parse(text).expect("a version 8 sheet reads");
    assert_eq!(sheet.format, SHEET_VERSION);
    let block = &sheet.circuits[0].block;
    assert_eq!(block.owner, "Curso de Neumática");
    assert!(block.project.is_empty());
    assert!(block.equipment.is_empty());
    assert_eq!(block.scale, "S/E");
    assert!(block.norms.is_empty());
    assert_eq!(block.model, "industrial");
    assert_eq!(block.visible, None, "the model supplies the full field set");
}

#[test]
fn a_version_nine_sheet_gains_the_industrial_model() {
    // Format 9 already had the twelve fields but no model or visibility.
    let saved = sheet().to_json().expect("valid");
    let json = saved
        .replace("\"format\": 12", "\"format\": 9")
        .replace("\"model\": \"industrial\",", "");
    let read = Sheet::parse(&json).expect("a version 9 sheet reads");
    assert_eq!(read.format, SHEET_VERSION);
    assert_eq!(read.circuits[0].block.model, "industrial");
    assert_eq!(read.circuits[0].block.visible, None);
}

#[test]
fn a_version_eleven_sheet_reads_its_retired_model_names() {
    // Format 11 named the two shorter models in Spanish; the reader renames them and
    // saving writes the English name, so no file keeps the old token.
    for (retired, current) in [("escolar", "school"), ("tecnico", "technical")] {
        let json = sheet()
            .to_json()
            .expect("valid")
            .replace("\"format\": 12", "\"format\": 11")
            .replace(
                "\"model\": \"industrial\",",
                &format!("\"model\": \"{retired}\","),
            );
        let read = Sheet::parse(&json).expect("a format 11 sheet reads");
        assert_eq!(read.format, SHEET_VERSION);
        assert_eq!(read.circuits[0].block.model, current);
        let saved = read.to_json().expect("valid");
        assert!(saved.contains(&format!("\"model\": \"{current}\"")));
    }
}

#[test]
fn the_content_model_and_visibility_round_trip() {
    let mut source = sheet();
    source.circuits[0].block.model = String::from("school");
    source.circuits[0].block.visible = Some(vec![
        String::from("owner"),
        String::from("title"),
        String::from("prepared"),
    ]);
    let saved = source.to_json().expect("valid");
    let read = Sheet::parse(&saved).expect("reads");
    assert_eq!(read.circuits[0].block, source.circuits[0].block);

    source.circuits[0].block.model = String::from("mechanical");
    assert!(source.validate().is_err(), "an unknown model is refused");
    source.circuits[0].block.model = String::from("school");
    source.circuits[0].block.visible = Some(vec![String::from("material")]);
    assert!(source.validate().is_err(), "an unknown field is refused");
}

#[test]
fn the_block_defaults_are_the_ones_the_guide_starts_at() {
    let block = StoredBlock::default();
    assert_eq!(block.scale, "S/E");
    assert_eq!(block.revision, "A");
    assert!(block.project.is_empty());
    assert!(block.equipment.is_empty());
    assert!(block.norms.is_empty());
    assert_eq!(block.model, "industrial");
    assert_eq!(block.visible, None);
    assert_eq!(block.position, "bottom-right");
    assert!(block.remark.is_empty());
}

#[test]
fn a_block_that_breaks_a_bound_is_refused() {
    let mut sheet = sheet();
    sheet.circuits[0].block.owner = String::from("  Curso");
    assert!(sheet.validate().is_err(), "untrimmed");
    sheet.circuits[0].block.owner = "x".repeat(MAX_BLOCK_FIELD + 1);
    assert!(sheet.validate().is_err(), "too long");
    sheet.circuits[0].block.owner = String::from("Curso\nSegundo");
    assert!(sheet.validate().is_err(), "control character");
    sheet.circuits[0].block.owner = String::from("Curso de Neumática");
    sheet.circuits[0].block.norms = String::from(" NCh 14");
    assert!(sheet.validate().is_err(), "the new fields are checked too");
    sheet.circuits[0].block.norms = String::from("NCh 14 · NCh 1768/1");
    assert!(sheet.validate().is_ok());
}

#[test]
fn the_block_survives_a_save_and_an_open() {
    let mut source = sheet();
    source.circuits[0].block.owner = String::from("Curso de Neumática");
    source.circuits[0].block.project = String::from("Línea 2");
    source.circuits[0].block.equipment = String::from("Prensa 4");
    source.circuits[0].block.number = String::from("PF-2026-07");
    source.circuits[0].block.prepared = String::from("Ana");
    source.circuits[0].block.scale = String::from("1:50");
    source.circuits[0].block.norms = String::from("NCh 14 · NCh 1768/1");
    let saved = source.to_json().expect("valid");
    let read = Sheet::parse(&saved).expect("reads");
    assert_eq!(read.circuits[0].block, source.circuits[0].block);
}

#[test]
fn the_block_position_is_checked_and_round_trips() {
    let mut sheet = sheet();
    sheet.circuits[0].block.position = String::from("bottom-full");
    let saved = sheet.to_json().expect("valid");
    let read = Sheet::parse(&saved).expect("reads");
    assert_eq!(read.circuits[0].block.position, "bottom-full");
    sheet.circuits[0].block.position = String::from("middle");
    assert!(sheet.validate().is_err(), "an unknown position is refused");
}

#[test]
fn a_version_ten_sheet_maps_the_old_positions_and_gains_the_note() {
    let saved = sheet().to_json().expect("valid");
    let json = saved
        .replace("\"format\": 12", "\"format\": 10")
        .replace("\"position\": \"bottom-right\"", "\"position\": \"left\"");
    let read = Sheet::parse(&json).expect("a version 10 sheet reads");
    assert_eq!(read.circuits[0].block.position, "bottom-right");
    assert!(read.circuits[0].block.remark.is_empty());
}

#[test]
fn the_remark_round_trips_and_allows_line_breaks() {
    let mut source = sheet();
    source.circuits[0].block.remark = String::from("Revisar la línea 2.\nPedir cotización.");
    let saved = source.to_json().expect("valid");
    let read = Sheet::parse(&saved).expect("reads");
    assert_eq!(
        read.circuits[0].block.remark,
        source.circuits[0].block.remark
    );
    source.circuits[0].block.remark = String::from("  sangría");
    assert!(source.validate().is_err(), "untrimmed");
    source.circuits[0].block.remark = "x".repeat(punchi_fluid_core::title_block::MAX_REMARK + 1);
    assert!(source.validate().is_err(), "too long");
    source.circuits[0].block.remark = String::from("Nota\u{0007}");
    assert!(source.validate().is_err(), "a control character");
}

#[test]
fn a_sheet_is_written_atomically_and_read_back() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("circuito.pfluid");
    let sheet = sheet();
    sheet.write(&path).expect("writes");
    assert!(!path.with_extension("pfluid.tmp").exists());
    assert_eq!(Sheet::read(&path).expect("reads"), sheet);
}

#[test]
fn a_version_six_sheet_reads_as_an_unframed_a4() {
    // A format 6 file was drawn on a bare 200 × 130 mm area: it reads as A4 with no frame,
    // so every placement it holds is still inside the drawing space.
    let text = r#"{
          "format": 6,
          "title": "Viejo",
          "circuits": [{"id": 0, "title": "Viejo"}],
          "placement": [
            {"id": 1, "catalog": "supply.compressor", "x": 190.0, "y": 125.0,
             "width": 20.0, "height": 20.0}
          ],
          "pipe": []
        }"#;
    let sheet = Sheet::parse(text).expect("a version 6 sheet reads");
    assert_eq!(sheet.format, SHEET_VERSION);
    assert_eq!(sheet.sheet.paper, StoredPaper::A4);
    assert!(!sheet.sheet.landscape);
    assert_eq!(
        sheet.sheet.margins,
        StoredMargins::from(Margins::unframed())
    );
    let paper = sheet.paper_sheet();
    assert_eq!(paper.size_mm(), (210.0, 297.0));
    assert_eq!(paper.margins, Margins::unframed());
}

#[test]
fn the_sheet_keeps_its_paper_orientation_and_margins_in_the_file() {
    let mut sheet = sheet();
    sheet.sheet = StoredSheet {
        paper: StoredPaper::A3,
        landscape: true,
        margins: StoredMargins {
            top: 5.0,
            right: 6.0,
            bottom: 7.0,
            left: 8.0,
        },
    };
    let saved = sheet.to_json().expect("valid");
    let read = Sheet::parse(&saved).expect("reads");
    assert_eq!(read.sheet, sheet.sheet);
    let paper = read.paper_sheet();
    assert_eq!(paper.paper, PaperSize::A3);
    assert_eq!(paper.orientation, Orientation::Landscape);
    assert_eq!(paper.size_mm(), (420.0, 297.0));
    assert_eq!(
        paper.margins,
        Margins {
            top: 5.0,
            right: 6.0,
            bottom: 7.0,
            left: 8.0,
        }
    );
    assert!(saved.contains(r#""paper": "a3""#), "{saved}");
    assert!(saved.contains(r#""landscape": true"#), "{saved}");
}

#[test]
fn a_custom_paper_is_stored_and_read_back_as_the_document_sheet() {
    let mut sheet = sheet();
    sheet.sheet = StoredSheet {
        paper: StoredPaper::Custom {
            width_mm: 180.0,
            height_mm: 250.0,
        },
        landscape: true,
        margins: StoredMargins::from(Margins::iso5457()),
    };
    let saved = sheet.to_json().expect("valid");
    let read = Sheet::parse(&saved).expect("reads");
    assert_eq!(read.sheet, sheet.sheet);
    let paper = read.paper_sheet();
    assert_eq!(
        paper.paper,
        PaperSize::custom(180.0, 250.0).expect("the size is inside the bounds")
    );
    assert_eq!(paper.orientation, Orientation::Landscape);
    assert_eq!(paper.size_mm(), (250.0, 180.0));
}

#[test]
fn a_sheet_whose_frame_leaves_no_drawing_space_is_refused() {
    let mut sheet = sheet();
    sheet.sheet.margins = StoredMargins {
        top: 10.0,
        right: 10.0,
        bottom: 10.0,
        left: 1_000.0,
    };
    assert!(
        sheet.validate().is_err(),
        "the frame leaves nothing to draw"
    );
    sheet.sheet.margins = StoredMargins::from(Margins::iso5457());
    sheet.sheet.paper = StoredPaper::Custom {
        width_mm: 10.0,
        height_mm: 10.0,
    };
    assert!(sheet.validate().is_err(), "a paper outside the bounds");
}
