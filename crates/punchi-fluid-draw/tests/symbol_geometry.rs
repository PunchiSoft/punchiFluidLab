// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The geometry of every built-in symbol, frozen.
//!
//! A symbol is normative: an arc moved by half a millimetre is a different symbol. This test
//! draws each definition at a fixed box and compares the drawing calls with the approved
//! fixture in `assets/symbols/fixtures`. When a change is intended, regenerate with
//! `UPDATE_SYMBOL_FIXTURES=1 cargo test -p punchi-fluid-draw --test symbol_geometry` and
//! review the diff before committing it.

use std::fmt::Write as _;

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::{Ink, Orientation, Placed, Pt, Recording, Rect, draw_placed};

#[test]
fn every_symbol_keeps_its_geometry() {
    let catalog = Catalog::builtin();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = root.join("assets/symbols/fixtures");
    let update = std::env::var_os("UPDATE_SYMBOL_FIXTURES").is_some();
    let mut failures = 0;
    for definition in catalog.iter() {
        let placed = Placed {
            index: 0,
            catalog_id: &definition.id,
            rect: Rect::from_min_size(Pt::new(0.0, 0.0), 40.0, 40.0),
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            contact_closed: None,
            orientation: Orientation::NONE,
            designation: None,
        };
        let mut recording = Recording::default();
        draw_placed(&mut recording, &[placed], &[], &catalog, &Ink::print(), &[]);
        let mut text = String::new();
        for call in recording.calls() {
            writeln!(text, "{call:?}").expect("a drawing call is written");
        }
        let path = dir.join(format!("{}.txt", definition.id));
        if update {
            std::fs::create_dir_all(&dir).expect("fixture folder");
            std::fs::write(&path, &text).expect("fixture written");
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(expected) if expected == text => {}
            Ok(_) => {
                eprintln!("symbol geometry changed: {}", definition.id);
                failures += 1;
            }
            Err(_) => {
                eprintln!("symbol has no fixture: {}", definition.id);
                failures += 1;
            }
        }
    }
    assert_eq!(
        failures, 0,
        "approve the change with UPDATE_SYMBOL_FIXTURES=1 after reading the diff"
    );
}
