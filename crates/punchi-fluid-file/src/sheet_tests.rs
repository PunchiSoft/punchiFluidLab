// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the saved sheet and its migrations.

use std::fs;
use std::path::Path;

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
                port_closures: BTreeMap::default(),
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
                port_closures: BTreeMap::default(),
            },
        ],
        pipe: vec![StoredPipe {
            circuit: 0,
            from: 1,
            from_port: String::from("2"),
            to: 2,
            to_port: String::from("2"),
            offset_tenths: 25,
            class: StoredClass::Working,
            tube_od_hundredths: None,
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
fn the_tube_size_round_trips_and_a_format_twelve_sheet_reads_as_unspecified() {
    let mut current = sheet();
    current.pipe[0].tube_od_hundredths = Some(1000);
    let text = current.to_json().expect("valid");
    assert!(text.contains("\"tube_od_hundredths\": 1000"), "{text}");
    let parsed = Sheet::parse(&text).expect("a format 14 sheet reads");
    assert_eq!(parsed.pipe[0].tube_od_hundredths, Some(1000));
    assert_eq!(parsed.format, SHEET_VERSION);

    // What format 12 wrote: the same pipe without the key.
    let mut older = sheet();
    older.pipe[0].tube_od_hundredths = None;
    let text = older
        .to_json()
        .expect("valid")
        .replace(&format!("\"format\": {SHEET_VERSION}"), "\"format\": 12");
    assert!(!text.contains("tube_od_hundredths"), "{text}");
    let read = Sheet::parse(&text).expect("a format 12 sheet reads");
    assert_eq!(read.format, SHEET_VERSION, "reading it upgrades it");
    assert_eq!(read.pipe[0].tube_od_hundredths, None);
    // An unspecified pipe writes no key, so a sheet that had none keeps looking the same.
    assert!(
        !read
            .to_json()
            .expect("valid")
            .contains("tube_od_hundredths")
    );
}

#[test]
fn a_tube_outside_the_bounds_is_refused() {
    let mut sheet = sheet();
    sheet.pipe[0].tube_od_hundredths = Some(1000);
    let valid = sheet.to_json().expect("valid");
    for value in ["0", "99", "5001", "65000"] {
        let text = valid.replace(
            "\"tube_od_hundredths\": 1000",
            &format!("\"tube_od_hundredths\": {value}"),
        );
        assert!(
            matches!(Sheet::parse(&text), Err(SheetError::Invalid(_))),
            "{value} should be refused"
        );
    }
    // A string and a float are not an integer count of hundredths, so serde refuses them
    // before the range check.
    for value in ["\"1000\"", "1000.5"] {
        let text = valid.replace(
            "\"tube_od_hundredths\": 1000",
            &format!("\"tube_od_hundredths\": {value}"),
        );
        assert!(
            matches!(Sheet::parse(&text), Err(SheetError::Parse(_))),
            "{value} should be refused"
        );
    }
    sheet.pipe[0].tube_od_hundredths = Some(5001);
    assert!(sheet.validate().is_err());
    sheet.pipe[0].tube_od_hundredths = Some(5000);
    assert!(sheet.validate().is_ok());
    sheet.pipe[0].tube_od_hundredths = Some(100);
    assert!(sheet.validate().is_ok());
}

#[test]
fn the_closure_round_trips_and_an_old_sheet_has_none() {
    let mut sheet = sheet();
    sheet.placement[1]
        .port_closures
        .insert(String::from("3"), String::from("silencer.pneumatic"));
    let text = sheet.to_json().expect("valid");
    assert!(text.contains("\"port_closures\""), "{text}");
    assert!(text.contains("\"3\": \"silencer.pneumatic\""), "{text}");
    let parsed = Sheet::parse(&text).expect("a format 14 sheet reads");
    assert_eq!(parsed, sheet);

    // A placement with no closure writes no key at all, so a sheet from before the field
    // looks the same after this build saves it.
    let mut open = self::sheet();
    open.placement[1].port_closures.clear();
    let text = open.to_json().expect("valid");
    assert!(!text.contains("port_closures"), "{text}");
    let older = text.replace(&format!("\"format\": {SHEET_VERSION}"), "\"format\": 13");
    let read = Sheet::parse(&older).expect("a format 13 sheet reads");
    assert_eq!(read.format, SHEET_VERSION, "reading it upgrades it");
    assert!(read.placement[1].port_closures.is_empty());
}

#[test]
fn a_port_closure_outside_the_bounds_is_refused() {
    let mut sheet = sheet();
    let too_long = "x".repeat(MAX_ID + 1);
    sheet.placement[0]
        .port_closures
        .insert(too_long.clone(), String::from("plug.blind"));
    assert!(matches!(sheet.validate(), Err(SheetError::Invalid(_))));
    sheet.placement[0].port_closures.clear();
    sheet.placement[0]
        .port_closures
        .insert(String::from("é"), String::from("plug.blind"));
    assert!(sheet.validate().is_err(), "a non-ASCII port id");
    sheet.placement[0].port_closures.clear();
    sheet.placement[0]
        .port_closures
        .insert(String::from("3"), String::new());
    assert!(sheet.validate().is_err(), "an empty closure id");
    sheet.placement[0].port_closures.clear();
    sheet.placement[0]
        .port_closures
        .insert(String::from("3"), too_long);
    assert!(sheet.validate().is_err(), "a closure id that is too long");

    // At most MAX_PORTS entries; the map itself already prevents a second entry for one
    // port. One past the limit is refused, the limit itself is accepted.
    let mut bounded = self::sheet();
    for index in 0..=MAX_PORTS {
        bounded.placement[0]
            .port_closures
            .insert(format!("p{index}"), String::from("plug.blind"));
    }
    assert!(matches!(bounded.validate(), Err(SheetError::Invalid(_))));
    bounded.placement[0]
        .port_closures
        .remove(&format!("p{MAX_PORTS}"));
    assert!(bounded.validate().is_ok(), "{:?}", bounded.validate());

    // A JSON object with the same port twice is refused by the reader instead of silently
    // keeping the last value.
    let text = self::sheet().to_json().expect("valid");
    let duplicate = text.replace(
        "\"catalog\": \"supply.compressor\"",
        "\"catalog\": \"supply.compressor\", \"port_closures\": \
         {\"3\": \"plug.blind\", \"3\": \"silencer.pneumatic\"}",
    );
    assert!(matches!(
        Sheet::parse(&duplicate),
        Err(SheetError::Parse(_))
    ));
}

#[test]
fn a_pipe_on_a_closed_port_is_a_malformed_sheet() {
    let mut sheet = sheet();
    // The default pipe ends on component 1's delivery, port 2; close exactly that port.
    sheet.placement[0]
        .port_closures
        .insert(String::from("2"), String::from("plug.blind"));
    assert!(matches!(
        sheet.validate(),
        Err(SheetError::Invalid(message)) if message.contains("closed port")
    ));

    // A format-13 sheet has no closures key, so the same pipe is on an open port and is
    // still valid; only a format-14 closure makes it malformed.
    let mut open = sheet;
    open.placement[0].port_closures.clear();
    let text = open
        .to_json()
        .expect("valid")
        .replace(&format!("\"format\": {SHEET_VERSION}"), "\"format\": 13");
    let read = Sheet::parse(&text).expect("a format 13 sheet with the pipe reads");
    assert!(read.placement[0].port_closures.is_empty());
    assert_eq!(read.pipe.len(), 1);
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
        .replace(&format!("\"format\": {SHEET_VERSION}"), "\"format\": 9")
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
            .replace(&format!("\"format\": {SHEET_VERSION}"), "\"format\": 11")
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
        .replace(&format!("\"format\": {SHEET_VERSION}"), "\"format\": 10")
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

#[test]
fn a_stale_temporary_is_replaced_instead_of_refused() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("circuito.pfluid");
    let temporary = path.with_extension("pfluid.tmp");
    fs::write(&temporary, "restos de un guardado que falló").expect("a stale temporary");
    sheet()
        .write(&path)
        .expect("writes over the stale temporary");
    assert!(!temporary.exists());
    assert_eq!(Sheet::read(&path).expect("reads"), sheet());
}

#[cfg(unix)]
#[test]
fn a_symlink_at_the_temporary_name_does_not_redirect_the_save() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("circuito.pfluid");
    let victim = directory.path().join("victima.txt");
    fs::write(&victim, "intacto").expect("a victim file");
    symlink(&victim, path.with_extension("pfluid.tmp")).expect("a planted symlink");
    sheet().write(&path).expect("writes");
    assert_eq!(fs::read_to_string(&victim).expect("reads"), "intacto");
    assert_eq!(Sheet::read(&path).expect("reads"), sheet());
}

#[cfg(unix)]
#[test]
fn saving_keeps_the_permissions_of_the_document_it_replaces() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("circuito.pfluid");
    fs::write(&path, "{}").expect("a private document");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("0600");
    sheet().write(&path).expect("writes");
    let mode = fs::metadata(&path).expect("metadata").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "a save must not widen the permissions");
}

#[test]
fn a_folder_at_the_temporary_name_is_refused_and_kept() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("circuito.pfluid");
    let temporary = path.with_extension("pfluid.tmp");
    fs::create_dir(&temporary).expect("a folder in the way");
    let error = sheet().write(&path).expect_err("refuses");
    assert!(matches!(error, SheetError::Io(_)));
    assert!(temporary.is_dir(), "the folder is not deleted");
    assert!(!path.exists());
}

#[test]
fn a_failed_rename_leaves_no_temporary_behind() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("circuito.pfluid");
    fs::create_dir(&path).expect("a folder where the document should be");
    let error = sheet().write(&path).expect_err("refuses");
    assert!(matches!(error, SheetError::Io(_)));
    assert!(!path.with_extension("pfluid.tmp").exists());
}

#[test]
fn a_device_is_refused_instead_of_read() {
    let zero = Path::new("/dev/zero");
    if !zero.exists() {
        return;
    }
    assert!(matches!(
        Sheet::read(zero).expect_err("refuses"),
        SheetError::NotRegular
    ));
}

#[test]
fn a_file_over_the_limit_is_refused_before_it_is_read() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("grande.pfluid");
    let file = fs::File::create(&path).expect("creates");
    file.set_len(MAX_BYTES + 1).expect("a sparse file");
    drop(file);
    assert!(matches!(
        Sheet::read(&path).expect_err("refuses"),
        SheetError::TooLarge
    ));
}

#[test]
fn a_format_fourteen_sheet_moves_the_compressor_hose_to_its_discharge() {
    // Format 14 stored the compressor's only port as `1`, which was the delivery; the new
    // generation gave the intake `1` and moved the delivery to `2`.
    let text = r#"{
      "format": 14,
      "circuits": [{"id": 0, "title": "Circuito"}],
      "active": 0,
      "placement": [
        {"id": 1, "catalog": "supply.compressor", "x": 10.0, "y": 20.0, "width": 20.0, "height": 20.0},
        {"id": 2, "catalog": "valve.3_2.spring_return", "x": 60.0, "y": 20.0, "width": 20.0, "height": 20.0},
        {"id": 3, "catalog": "cylinder.single_acting_20_100", "x": 110.0, "y": 20.0, "width": 44.0, "height": 20.0}
      ],
      "pipe": [
        {"from": 1, "from_port": "1", "to": 2, "to_port": "1"},
        {"from": 2, "from_port": "2", "to": 3, "to_port": "1"}
      ]
    }"#;
    let sheet = Sheet::parse(text).expect("a format 14 sheet reads");
    assert_eq!(sheet.format, SHEET_VERSION);
    assert_eq!(sheet.pipe[0].from_port, "2", "the hose was on the delivery");
    assert_eq!(sheet.pipe[0].to_port, "1", "the valve end is untouched");
    assert_eq!(sheet.pipe[1].from_port, "2", "another pipe is untouched");
    assert_eq!(
        sheet.pipe[1].to_port, "1",
        "a cylinder's port 1 is not the compressor's"
    );
}

#[path = "sheet/valve_tests.rs"]
mod valve_tests;
