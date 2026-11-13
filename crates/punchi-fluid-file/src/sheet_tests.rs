// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the saved sheet and its migrations.

use super::*;

fn sheet() -> Sheet {
    Sheet {
        format: SHEET_VERSION,
        title: String::from("Prueba"),
        circuits: vec![StoredCircuit {
            id: 0,
            title: String::from("Circuito 1"),
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
    assert!(sheet.to_json().expect("valid").contains(r#""format": 6"#));
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
    assert!(saved.contains(r#""format": 6"#), "{saved}");
    assert_eq!(sheet.placement[0].designation, None, "and no name either");
}

#[test]
fn a_turn_that_is_not_one_of_the_four_is_refused() {
    let mut sheet = sheet();
    sheet.placement[0].quarter_turns = 4;
    assert!(sheet.validate().is_err());
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
