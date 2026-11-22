// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Renders one built-in symbol to a PNG for the review record.
//!
//! `symbol-sheet <id> <file> [--dpi 300] [--ink print|light|dark] [--fit]` draws the
//! symbol in a fixed 40 mm box on a 60 mm square, in print ink by default, so the
//! reviewer sees the geometry without the window, the palette or a screenshot in
//! between. `--ink dark` paints the dark surface, `--fit` crops to the drawing instead
//! of the 60 mm page, and the light and dark captures of a review are the print and the
//! dark ink fitted to that drawing.

use std::process::ExitCode;

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::{Colour, Ink, Orientation, Placed, Pt, Rect, bounds_placed, draw_placed};
use punchi_fluid_export::{png::PngSurface, print_ink};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(id) = arguments.first() else {
        usage();
        return ExitCode::from(2);
    };
    let Some(path) = arguments.get(1) else {
        usage();
        return ExitCode::from(2);
    };
    let dpi = option(&arguments, "--dpi")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(300.0);
    let ink = match option(&arguments, "--ink").unwrap_or("print") {
        "print" | "light" => print_ink(),
        "dark" => dark_ink(),
        other => {
            eprintln!("symbol-sheet: --ink takes `print`, `light` or `dark`, not `{other}`");
            return ExitCode::from(2);
        }
    };
    let fit = arguments.iter().any(|argument| argument == "--fit");
    let catalog = Catalog::builtin();
    if catalog.get(id).is_none() {
        eprintln!("no such symbol: {id}");
        return ExitCode::from(1);
    }
    let side = 60.0_f32;
    let mut placed = Placed {
        index: 0,
        catalog_id: id,
        rect: Rect::from_center_size(Pt::new(side * 0.5, side * 0.5), 40.0, 40.0),
        extension: 0.0,
        valve_position: 0,
        reading: None,
        series: None,
        contact_closed: None,
        orientation: Orientation::NONE,
        designation: None,
        closures: &[],
        exhausts: &[],
    };
    let page = if fit {
        let Some(bounds) = bounds_placed(&[placed], &[], &catalog) else {
            eprintln!("nothing to draw for {id}");
            return ExitCode::from(1);
        };
        placed.rect = Rect::from_min_size(
            Pt::new(
                placed.rect.min.x - bounds.min.x,
                placed.rect.min.y - bounds.min.y,
            ),
            placed.rect.width(),
            placed.rect.height(),
        );
        Rect::from_min_size(Pt::new(0.0, 0.0), bounds.width(), bounds.height())
    } else {
        Rect::from_min_size(Pt::new(0.0, 0.0), side, side)
    };
    let mut surface = match PngSurface::new(page, dpi) {
        Ok(surface) => surface,
        Err(error) => {
            eprintln!("cannot start the sheet: {error}");
            return ExitCode::from(1);
        }
    };
    surface.fill(ink.background);
    draw_placed(&mut surface, &[placed], &[], &catalog, &ink, &[]);
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

/// Value of a `--name value` option.
fn option<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    let index = arguments.iter().position(|argument| argument == name)?;
    arguments.get(index + 1).map(String::as_str)
}

/// The dark surface: light strokes on the window's dark background, with the dark accent
/// the active square uses. The values are the ones the review captures already carry.
fn dark_ink() -> Ink {
    Ink {
        line: Colour::new(240, 240, 240),
        dim: Colour::new(135, 135, 135),
        muted: Colour::new(188, 188, 188),
        text: Colour::new(240, 240, 240),
        supply: Colour::new(240, 240, 240),
        work: Colour::new(240, 240, 240),
        accent: Colour::new(45, 55, 80),
        background: Colour::new(30, 30, 30),
    }
}

fn usage() {
    eprintln!("usage: symbol-sheet <id> <file> [--dpi 300] [--ink print|light|dark] [--fit]");
}
