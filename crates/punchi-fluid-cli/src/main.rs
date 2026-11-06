// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Headless entry point: validates or simulates a circuit file, so CI and tests
//! never need a window.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);

    let Some(command) = arguments.next() else {
        eprintln!("usage: punchi-fluid <validate|simulate> <file>");
        return ExitCode::from(2);
    };
    let Some(path) = arguments.next() else {
        eprintln!("missing circuit file");
        return ExitCode::from(2);
    };

    let contents = match std::fs::read_to_string(&path) {
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
