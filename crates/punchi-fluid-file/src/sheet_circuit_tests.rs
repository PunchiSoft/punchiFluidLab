// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the saved sheet and its migrations.

use super::*;

fn project() -> Sheet {
    let mut sheet = Sheet::new("Proyecto");
    sheet.circuits.push(StoredCircuit {
        id: 1,
        title: String::from("Control"),
        block: StoredBlock {
            title: String::from("Control"),
            ..StoredBlock::default()
        },
    });
    sheet.placement.push(StoredPlacement {
        id: 1,
        circuit: 0,
        catalog: String::from("supply.compressor"),
        x: 20.0,
        y: 20.0,
        width: 30.0,
        height: 16.0,
        edit: None,
        quarter_turns: 0,
        mirrored: false,
        designation: None,
        source: None,
    });
    sheet.placement.push(StoredPlacement {
        id: 2,
        circuit: 1,
        catalog: String::from("valve.3_2.spring_return"),
        x: 100.0,
        y: 80.0,
        width: 30.0,
        height: 16.0,
        edit: None,
        quarter_turns: 0,
        mirrored: false,
        designation: None,
        source: None,
    });
    // A pipe whose ends belong to different circuits is accepted by the format: the
    // check is what reports it, so the loader must not hide it.
    sheet.pipe.push(StoredPipe {
        circuit: 1,
        from: 1,
        from_port: String::from("1"),
        to: 2,
        to_port: String::from("1"),
        offset_tenths: 0,
        class: StoredClass::Working,
    });
    sheet
}

#[test]
fn a_project_keeps_two_circuits_and_a_pipe_between_them() {
    let sheet = project();
    let text = sheet.to_json().expect("valid");
    let parsed = Sheet::parse(&text).expect("parses");
    assert_eq!(parsed, sheet);
    assert_eq!(parsed.format, SHEET_VERSION);
    assert_eq!(parsed.circuits.len(), 2);
    assert_eq!(parsed.placement[1].circuit, 1);
    assert_eq!(parsed.pipe[0].circuit, 1);
}

#[test]
fn a_version_three_sheet_reads_as_one_circuit() {
    let text = r#"{
          "format": 3,
          "title": "Anterior",
          "placement": [
            {"id": 1, "catalog": "supply.compressor", "x": 10.0, "y": 20.0,
             "width": 26.0, "height": 26.0}
          ],
          "pipe": []
        }"#;
    let sheet = Sheet::parse(text).expect("a version 3 sheet reads");
    assert_eq!(sheet.format, SHEET_VERSION);
    assert_eq!(sheet.circuits.len(), 1);
    assert_eq!(sheet.circuits[0].id, 0);
    assert_eq!(sheet.placement[0].circuit, 0);
    assert_eq!(sheet.active, 0);
}

#[test]
fn a_version_four_project_keeps_its_circuits_and_has_unbound_recorders() {
    // The trap this pins: a version 4 file already has its circuits, and a migration
    // that reset them would silently merge every sheet into the first one.
    let text = r#"{
          "format": 4,
          "title": "Proyecto anterior",
          "circuits": [
            {"id": 0, "title": "Potencia"},
            {"id": 7, "title": "Control"}
          ],
          "placement": [
            {"id": 1, "circuit": 0, "catalog": "supply.compressor", "x": 10.0, "y": 20.0,
             "width": 26.0, "height": 26.0},
            {"id": 2, "circuit": 7, "catalog": "chart.recorder", "x": 90.0, "y": 20.0,
             "width": 23.8, "height": 23.8}
          ],
          "pipe": [],
          "active": 7
        }"#;
    let sheet = Sheet::parse(text).expect("a version 4 project reads");
    assert_eq!(sheet.format, SHEET_VERSION);
    assert_eq!(sheet.circuits.len(), 2);
    assert_eq!(sheet.placement[1].circuit, 7, "the second circuit survives");
    assert_eq!(sheet.active, 7);
    assert_eq!(
        sheet.placement[1].source, None,
        "an older file has no recorder bound"
    );
}

#[test]
fn a_recorder_source_round_trips_and_is_validated() {
    let mut sheet = project();
    sheet.placement.push(StoredPlacement {
        id: 3,
        circuit: 1,
        catalog: String::from("chart.recorder"),
        x: 60.0,
        y: 20.0,
        width: 23.8,
        height: 23.8,
        edit: None,
        quarter_turns: 0,
        mirrored: false,
        designation: None,
        source: Some(StoredSource {
            placement: 2,
            quantity: String::from("pressure"),
        }),
    });
    let text = sheet.to_json().expect("valid");
    let parsed = Sheet::parse(&text).expect("parses");
    assert_eq!(parsed, sheet);
    assert_eq!(
        parsed.placement[2]
            .source
            .as_ref()
            .map(|s| s.quantity.as_str()),
        Some("pressure")
    );

    let mut broken = sheet.clone();
    broken.placement[2].source = Some(StoredSource {
        placement: 2,
        quantity: String::from("temperature"),
    });
    assert!(broken.validate().is_err(), "an unknown quantity is refused");

    let mut missing = sheet.clone();
    missing.placement[2].source = Some(StoredSource {
        placement: 99,
        quantity: String::from("pressure"),
    });
    assert!(missing.validate().is_err(), "a missing source is refused");

    let mut self_chart = sheet.clone();
    self_chart.placement[2].source = Some(StoredSource {
        placement: 3,
        quantity: String::from("pressure"),
    });
    assert!(
        self_chart.validate().is_err(),
        "a recorder cannot chart itself"
    );

    let mut other_circuit = sheet.clone();
    other_circuit.placement[2].source = Some(StoredSource {
        placement: 1,
        quantity: String::from("pressure"),
    });
    assert!(
        other_circuit.validate().is_err(),
        "a source in another circuit is refused"
    );
}

#[test]
fn a_version_five_project_reads_its_pipes_as_working_lines() {
    // The trap: format 6 added the line class, and every pipe an older release wrote is a
    // working line. A migration that guessed anything else would redraw a whole sheet.
    let text = r#"{
      "format": 5,
      "title": "Anterior",
      "circuits": [{"id": 0, "title": "Potencia"}],
      "placement": [
        {"id": 1, "circuit": 0, "catalog": "supply.compressor", "x": 10.0, "y": 20.0,
         "width": 26.0, "height": 26.0},
        {"id": 2, "circuit": 0, "catalog": "cylinder.single_acting_20_100", "x": 90.0,
         "y": 20.0, "width": 60.0, "height": 26.0, "source": null}
      ],
      "pipe": [
        {"circuit": 0, "from": 1, "from_port": "1", "to": 2, "to_port": "1"}
      ],
      "active": 0
    }"#;
    let sheet = Sheet::parse(text).expect("a version 5 project reads");
    assert_eq!(sheet.format, SHEET_VERSION);
    assert_eq!(sheet.pipe[0].class, StoredClass::Working);
    let saved = sheet.to_json().expect("valid");
    assert!(
        !saved.contains("\"class\""),
        "a working line is not written, so an old sheet looks as it always did: {saved}"
    );
}

#[test]
fn a_pipe_keeps_its_class() {
    let mut sheet = project();
    sheet.pipe[0].class = StoredClass::Pilot;
    let text = sheet.to_json().expect("valid");
    assert!(text.contains("\"class\": \"pilot\""), "{text}");
    let parsed = Sheet::parse(&text).expect("parses");
    assert_eq!(parsed.pipe[0].class, StoredClass::Pilot);
}

#[test]
fn a_component_or_pipe_with_an_unknown_circuit_is_refused() {
    let mut sheet = project();
    sheet.placement[0].circuit = 9;
    assert!(sheet.validate().is_err());
    sheet.placement[0].circuit = 0;
    sheet.pipe[0].circuit = 9;
    assert!(sheet.validate().is_err());
}
