// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `check` command is a public contract ([ADR 0011]): these tests hold its exit codes and
//! the shape of its JSON, because a course pipeline depends on both.
//!
//! [ADR 0011]: ../../docs/adr/0011-the-check-is-a-public-command.md

// A failing fixture is meant to fail loudly: `expect` in a test is the message the reader
// needs, not a crash, and the workspace denies it in shipped code.
#![allow(clippy::expect_used)]

use std::process::{Command, Output};

/// The command under test, as Cargo built it.
fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_punchi-fluid-cli"))
}

fn run(arguments: &[&str]) -> Output {
    command()
        .args(arguments)
        .output()
        .expect("the command runs")
}

fn write(name: &str, text: &str) -> String {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, text).expect("the fixture is written");
    path.to_string_lossy().into_owned()
}

/// A sheet with a supply, a valve and a cylinder, joined as the worked example is.
const CLEAN: &str = r#"{
  "format": 3,
  "title": "Limpio",
  "placement": [
    {"id": 1, "catalog": "supply.workshop", "x": 34.0, "y": 39.0, "width": 33.8, "height": 33.8},
    {"id": 2, "catalog": "valve.3_2.spring_return", "x": 94.0, "y": 93.6, "width": 33.8, "height": 33.8},
    {"id": 3, "catalog": "cylinder.single_acting_20_100", "x": 144.0, "y": 33.8, "width": 74.4, "height": 33.8}
  ],
  "pipe": [
    {"from": 1, "from_port": "1", "to": 2, "to_port": "1"},
    {"from": 2, "from_port": "2", "to": 3, "to_port": "1"}
  ]
}"#;

/// The same sheet with the cylinder pushed off the drawing: one fault, one exit code.
const FAULTY: &str = r#"{
  "format": 3,
  "title": "Fuera",
  "placement": [
    {"id": 1, "catalog": "cylinder.single_acting_20_100", "x": 250.0, "y": 60.0, "width": 74.4, "height": 33.8}
  ],
  "pipe": []
}"#;

#[test]
fn a_clean_sheet_passes_and_says_so() {
    let path = write("punchi-clean.pfluid", CLEAN);
    let output = run(&["check", &path]);
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("ok"), "{text}");
}

#[test]
fn a_fault_is_listed_and_only_fails_the_job_when_it_is_asked_to() {
    let path = write("punchi-faulty.pfluid", FAULTY);
    // Without `--strict` the list is data, not a failed job: a caller that wants to show
    // warnings must not have to catch an error.
    let lenient = run(&["check", &path]);
    assert!(lenient.status.success(), "{lenient:?}");
    let text = String::from_utf8_lossy(&lenient.stdout);
    assert!(text.contains("hoja"), "{text}");
    // With `--strict` the same file fails the job.
    let strict = run(&["check", &path, "--strict"]);
    assert_eq!(strict.status.code(), Some(3), "{strict:?}");
}

#[test]
fn the_json_carries_its_version_and_one_object_per_fault() {
    let path = write("punchi-json.pfluid", FAULTY);
    let output = run(&["check", &path, "--json"]);
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("\"version\":1"), "{text}");
    // Two faults, not one: the cylinder is off the sheet and its only port is joined to
    // nothing. The test cares about the shape, and about the fault it asked for.
    assert!(text.contains("\"count\":2"), "{text}");
    assert!(text.contains("\"kind\":\"outside_sheet\""), "{text}");
    assert!(text.contains("\"component\":0"), "{text}");
    // Nothing else on stdout: a caller may parse the whole line.
    assert_eq!(text.lines().count(), 1, "{text}");
}

#[test]
fn a_broken_file_is_an_error_in_the_shape_that_was_asked_for() {
    let path = write("punchi-broken.pfluid", "{ no es json");
    let human = run(&["check", &path]);
    assert_eq!(human.status.code(), Some(1), "{human:?}");
    assert!(human.stdout.is_empty(), "the human path writes to stderr");
    let json = run(&["check", &path, "--json"]);
    assert_eq!(json.status.code(), Some(1), "{json:?}");
    let text = String::from_utf8_lossy(&json.stdout);
    assert!(text.contains("\"error\":"), "{text}");
    assert!(text.contains("\"version\":1"), "{text}");
}

#[test]
fn a_wrong_command_line_is_refused_instead_of_guessed() {
    assert_eq!(run(&["check"]).status.code(), Some(2));
    assert_eq!(run(&["check", "--nada", "x"]).status.code(), Some(2));
    // An unknown command with no file is a usage error too; with a file it is a usage error
    // as well, since the command is read first.
    assert_eq!(run(&["inventado"]).status.code(), Some(2));
}
