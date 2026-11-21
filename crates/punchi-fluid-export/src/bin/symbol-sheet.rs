// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Renders one built-in symbol to a PNG at print resolution, for the review record.
//!
//! `symbol-sheet <id> <file> [--dpi 300]` draws the symbol in a fixed 40 mm box on a 60 mm
//! square, in print ink, so the reviewer sees the geometry without the window, the palette
//! or a screenshot in between. It lives here because this crate already owns the raster
//! surface, the catalog and the drawing; the CLI does not depend on the export.

use std::process::ExitCode;

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::{Placed, Pt, Rect};
use punchi_fluid_export::{png::PngSurface, print_ink};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(id) = arguments.first() else {
        eprintln!("usage: symbol-sheet <id> <file> [--dpi 300]");
        return ExitCode::from(2);
    };
    let Some(path) = arguments.get(1) else {
        eprintln!("usage: symbol-sheet <id> <file> [--dpi 300]");
        return ExitCode::from(2);
    };
    let mut dpi = 300.0_f32;
    if let Some(position) = arguments.iter().position(|argument| argument == "--dpi") {
        if let Some(parsed) = arguments
            .get(position + 1)
            .and_then(|value| value.parse().ok())
        {
            dpi = parsed;
        }
    }
    let catalog = Catalog::builtin();
    if catalog.get(id).is_none() {
        eprintln!("no such symbol: {id}");
        return ExitCode::from(1);
    }
    let side = 60.0_f32;
    let page = Rect::from_min_size(Pt::new(0.0, 0.0), side, side);
    let mut surface = match PngSurface::new(page, dpi) {
        Ok(surface) => surface,
        Err(error) => {
            eprintln!("cannot start the sheet: {error}");
            return ExitCode::from(1);
        }
    };
    let placed = Placed {
        index: 0,
        catalog_id: id,
        rect: Rect::from_center_size(Pt::new(side * 0.5, side * 0.5), 40.0, 40.0),
        extension: 0.0,
        valve_position: 0,
        reading: None,
        series: None,
        contact_closed: None,
        orientation: punchi_fluid_draw::Orientation::NONE,
        designation: None,
        closures: &[],
        exhausts: &[],
    };
    punchi_fluid_draw::draw_placed(&mut surface, &[placed], &[], &catalog, &print_ink(), &[]);
    match surface.encode() {
        Ok(bytes) => match std::fs::write(path, bytes) {
            Ok(()) => {
                println!("symbol sheet: {id} at {dpi} dpi written to {path}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("cannot write {path}: {error}");
                ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("cannot encode the sheet: {error}");
            ExitCode::from(1)
        }
    }
}
