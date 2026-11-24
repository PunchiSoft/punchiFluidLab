// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Renders the built-in symbols to PNGs for the review record.
//!
//! `symbol-sheet <id> <file> [--dpi 300] [--ink print|light|dark] [--fit]` draws the
//! symbol in a fixed 40 mm box on a 60 mm square, in print ink by default, so the
//! reviewer sees the geometry without the window, the palette or a screenshot in
//! between. `--ink dark` paints the dark surface, `--fit` crops to the drawing instead
//! of the 60 mm page, and the light and dark captures of a review are the print and the
//! dark ink fitted to that drawing.
//!
//! `symbol-sheet --all <dir>` writes the three captures of every built-in symbol into
//! `dir`, with the names and options a review already uses: `<id>-300.png` (print ink on
//! the 60 mm page), `<id>-light.png` and `<id>-dark.png` (light and dark ink fitted to
//! the drawing). One run keeps the whole review folder in step with the catalog.

use std::path::Path;
use std::process::ExitCode;

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::{Colour, Ink, Orientation, Placed, Pt, Rect, bounds_placed, draw_placed};
use punchi_fluid_export::{png::PngSurface, print_ink};

/// Resolution at which a review capture is written, in dots per inch.
const CAPTURE_DPI: f32 = 300.0;
/// Side of the page the 300 dpi capture draws, in millimetres.
const PAGE_MM: f32 = 60.0;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|argument| argument == "--all")
    {
        every(&arguments)
    } else {
        one(&arguments)
    }
}

/// The per-symbol command: one capture of `<id>` into `<file>`.
fn one(arguments: &[String]) -> ExitCode {
    let (Some(id), Some(path)) = (arguments.first(), arguments.get(1)) else {
        usage();
        return ExitCode::from(2);
    };
    let dpi = option(arguments, "--dpi")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(CAPTURE_DPI);
    let ink = match option(arguments, "--ink").unwrap_or("print") {
        "print" | "light" => print_ink(),
        "dark" => dark_ink(),
        other => {
            eprintln!("symbol-sheet: --ink takes `print`, `light` or `dark`, not `{other}`");
            return ExitCode::from(2);
        }
    };
    let fit = arguments.iter().any(|argument| argument == "--fit");
    match capture(&Catalog::builtin(), id, Path::new(path), dpi, ink, fit) {
        Ok(()) => {
            println!("symbol sheet: {id} at {dpi} dpi written to {path}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

/// The batch command: the three review captures of every built-in symbol in `dir`.
///
/// The names and options are the ones the review template documents for one symbol, so a
/// batch and a hand-run per-symbol command cannot disagree about what a capture is.
fn every(arguments: &[String]) -> ExitCode {
    if arguments.len() > 2 {
        eprintln!("symbol-sheet: --all takes the output directory and nothing else");
        return ExitCode::from(2);
    }
    let Some(dir) = arguments.get(1) else {
        usage();
        return ExitCode::from(2);
    };
    let dir = Path::new(dir);
    if let Err(error) = std::fs::create_dir_all(dir) {
        eprintln!("cannot create {}: {error}", dir.display());
        return ExitCode::from(1);
    }
    let catalog = Catalog::builtin();
    let mut written = 0_usize;
    for definition in catalog.iter() {
        let id = definition.id.as_str();
        for (suffix, ink, fit) in [
            ("-300.png", print_ink(), false),
            ("-light.png", print_ink(), true),
            ("-dark.png", dark_ink(), true),
        ] {
            let path = dir.join(format!("{id}{suffix}"));
            if let Err(message) = capture(&catalog, id, &path, CAPTURE_DPI, ink, fit) {
                eprintln!("{message}");
                return ExitCode::from(1);
            }
            written += 1;
        }
    }
    println!(
        "symbol sheet: {written} captures of {} built-in symbols written to {}",
        catalog.len(),
        dir.display()
    );
    ExitCode::SUCCESS
}

/// Draws `id` into `path` with the given options, or says why it cannot.
fn capture(
    catalog: &Catalog,
    id: &str,
    path: &Path,
    dpi: f32,
    ink: Ink,
    fit: bool,
) -> Result<(), String> {
    if catalog.get(id).is_none() {
        return Err(format!("no such symbol: {id}"));
    }
    let mut placed = Placed {
        index: 0,
        catalog_id: id,
        rect: Rect::from_center_size(Pt::new(PAGE_MM * 0.5, PAGE_MM * 0.5), 40.0, 40.0),
        extension: 0.0,
        moving: None,
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
        let Some(bounds) = bounds_placed(&[placed], &[], catalog) else {
            return Err(format!("nothing to draw for {id}"));
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
        Rect::from_min_size(Pt::new(0.0, 0.0), PAGE_MM, PAGE_MM)
    };
    let mut surface =
        PngSurface::new(page, dpi).map_err(|error| format!("cannot start the sheet: {error}"))?;
    surface.fill(ink.background);
    draw_placed(&mut surface, &[placed], &[], catalog, &ink, &[]);
    let bytes = surface
        .encode()
        .map_err(|error| format!("cannot encode the sheet: {error}"))?;
    std::fs::write(path, bytes).map_err(|error| format!("cannot write {}: {error}", path.display()))
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
    eprintln!("       symbol-sheet --all <dir>");
}
