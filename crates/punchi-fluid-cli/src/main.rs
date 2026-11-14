// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Headless entry point: validates, simulates or checks a circuit, so CI and tests
//! never need a window.
//!
//! `check` works on the **sheet** format the editor saves and speaks a versioned JSON
//! contract ([ADR 0011](../../docs/adr/0011-the-check-is-a-public-command.md)); `validate`
//! and `simulate` work on the legacy circuit format.

use std::process::ExitCode;

use punchi_fluid_catalog::Catalog;
use punchi_fluid_check::Finding;
use punchi_fluid_doc::{Circuit, Document, Endpoint, Placement};
use punchi_fluid_draw::surface::{Pt, Rect};
use punchi_fluid_file::sheet::Sheet;

/// Version of the JSON the `check` command prints.
const CHECK_JSON_VERSION: u32 = 1;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = arguments.first() else {
        return usage();
    };
    if command == "check" {
        return check(&arguments[1..]);
    }

    let Some(path) = arguments.get(1) else {
        eprintln!("missing circuit file");
        return ExitCode::from(2);
    };

    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::from(1);
        }
    };

    let document = match punchi_fluid_file::Document::parse(&contents) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("invalid circuit: {error}");
            return ExitCode::from(1);
        }
    };

    let circuit = match document.into_circuit() {
        Ok(circuit) => circuit,
        Err(error) => {
            eprintln!("invalid circuit: {error}");
            return ExitCode::from(1);
        }
    };

    match command.as_str() {
        "validate" => {
            println!("ok: {} components", circuit.components.len());
            ExitCode::SUCCESS
        }
        "simulate" => {
            let mut solver = punchi_fluid_solver::Solver::default();
            match solver.load(circuit).and_then(|()| solver.step(0.001)) {
                Ok(result) => {
                    println!("ok: {} volumes", result.volumes.len());
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("simulation failed: {error}");
                    ExitCode::from(1)
                }
            }
        }
        other => {
            eprintln!("unknown command {other}");
            ExitCode::from(2)
        }
    }
}

/// `check <file> [--json] [--strict]`.
///
/// Exit codes: `0` the file is a sheet and was checked, `1` it could not be read or parsed,
/// `2` the command line is wrong, `3` faults were found and `--strict` was asked for.
fn check(arguments: &[String]) -> ExitCode {
    let mut path: Option<&str> = None;
    let mut json = false;
    let mut strict = false;
    for argument in arguments {
        match argument.as_str() {
            "--json" => json = true,
            "--strict" => strict = true,
            other if other.starts_with('-') => {
                eprintln!("unknown option {other}");
                return ExitCode::from(2);
            }
            other if path.is_none() => path = Some(other),
            other => {
                eprintln!("unexpected argument {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(path) = path else {
        eprintln!("usage: punchi-fluid-cli check <file> [--json] [--strict]");
        return ExitCode::from(2);
    };

    let sheet = match Sheet::read(std::path::Path::new(path)) {
        Ok(sheet) => sheet,
        Err(error) => return report_error(path, &error.to_string(), json),
    };
    let document = match document_of(&sheet) {
        Ok(document) => document,
        Err(error) => return report_error(path, &error, json),
    };
    let catalog = Catalog::builtin();
    // The sheet is the document's paper: a circuit checked headlessly is measured against
    // the same frame the window draws.
    let paper = document.sheet();
    let (width, height) = paper.size_mm();
    let margins = paper.margins;
    let area = Rect::from_min_size(
        Pt::new(margins.left, margins.top),
        (width - margins.left - margins.right).max(1.0),
        (height - margins.top - margins.bottom).max(1.0),
    );
    let findings = punchi_fluid_check::check(&document, &catalog, area, document.active_circuit());

    if json {
        println!("{}", as_json(path, &findings));
    } else if findings.is_empty() {
        println!("ok: sin faltas");
    } else {
        for finding in &findings {
            println!("{} [{}]", finding.describe(), finding.kind.as_str());
        }
        println!("{} falta(s)", findings.len());
    }

    if findings.is_empty() || !strict {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

/// A file that cannot be read or understood, in whichever shape was asked for.
fn report_error(path: &str, message: &str, json: bool) -> ExitCode {
    if json {
        println!(
            "{{\"version\":{CHECK_JSON_VERSION},\"file\":{},\"count\":0,\"findings\":[],\"error\":{}}}",
            quote(path),
            quote(message)
        );
    } else {
        eprintln!("{path}: {message}");
    }
    ExitCode::from(1)
}

/// The sheet as the document the check consumes.
///
/// It mirrors the mapping the ABI does when it opens a sheet; the test
/// `the_command_and_the_window_agree_on_the_same_file` is what keeps the two honest until a
/// third caller appears and the mapping moves into `punchi-fluid-doc`.
fn document_of(sheet: &Sheet) -> Result<Document, String> {
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
            if let Some(edit) = &stored.edit {
                placement.edits = punchi_fluid_doc::PlacementEdits {
                    pressure_bar: edit.pressure_bar,
                    conductance: edit.conductance,
                    stroke_mm: edit.stroke_mm,
                    bore_mm: edit.bore_mm,
                    rod_mm: edit.rod_mm,
                    moving_mass_kg: edit.moving_mass_kg,
                };
            }
            placement.orientation =
                punchi_fluid_draw::Orientation::new(stored.quarter_turns, stored.mirrored);
            placement.designation.clone_from(&stored.designation);
            placement
        })
        .collect();
    let mut circuits: Vec<Circuit> = sheet
        .circuits
        .iter()
        .map(|circuit| Circuit::new(circuit.id, circuit.title.clone()))
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
    }
    Ok(document)
}

/// The findings as the versioned JSON object of [ADR 0011].
fn as_json(path: &str, findings: &[Finding]) -> String {
    let items: Vec<String> = findings
        .iter()
        .map(|finding| {
            format!(
                "{{\"kind\":{},\"message\":{},\"x\":{:.3},\"y\":{:.3},\"component\":{},\"pipe\":{}}}",
                quote(finding.kind.as_str()),
                quote(finding.kind.message_es()),
                finding.at.x,
                finding.at.y,
                finding
                    .component
                    .map_or_else(|| String::from("null"), |index| index.to_string()),
                finding
                    .pipe
                    .map_or_else(|| String::from("null"), |index| index.to_string()),
            )
        })
        .collect();
    format!(
        "{{\"version\":{CHECK_JSON_VERSION},\"file\":{},\"count\":{},\"findings\":[{}]}}",
        quote(path),
        findings.len(),
        items.join(",")
    )
}

/// A JSON string literal, escaped.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn usage() -> ExitCode {
    eprintln!("usage: punchi-fluid-cli <validate|simulate|check> <file> [--json] [--strict]");
    ExitCode::from(2)
}
