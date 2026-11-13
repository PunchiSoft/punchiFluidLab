// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Writes a real PDF to the target directory so the drawing can be looked at.
//!
//! Not an assertion: the file is the evidence for the visual review, and it is written
//! only when `PUNCHI_FLUID_WRITE_SAMPLE` is set, so a normal test run stays quiet.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::{Colour, Ink, Pt, Rect, Surface};
use punchi_fluid_export::page::Page;
use punchi_fluid_export::png::PngSurface;
use punchi_fluid_export::render_pdf;
use punchi_fluid_solver::network::Network;

#[test]
fn write_a_sample_sheet_on_request() {
    if std::env::var_os("PUNCHI_FLUID_WRITE_SAMPLE").is_none() {
        return;
    }
    let catalog = Catalog::builtin();
    let system = Network::demo(&catalog).expect("the circuit starts");
    let block = punchi_fluid_export::block_with_review_state("Pneumatic circuit", "2026-11-06");
    for (paper, name) in [
        (punchi_fluid_export::PaperSize::A4, "a4"),
        (punchi_fluid_export::PaperSize::A3, "a3"),
    ] {
        let page = Page {
            paper,
            ..Page::a4(block.clone())
        };
        let bytes = render_pdf(&page, &block, &system, &catalog).expect("renders");
        let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(format!("target/sample-{name}.pdf"));
        std::fs::write(&target, bytes).expect("writes the sample");
        println!("wrote {}", target.display());
    }

    let png =
        punchi_fluid_export::render_png(&Page::a4(block.clone()), &block, &system, &catalog, 300.0)
            .expect("renders the raster");
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target/sample-a4-300dpi.png");
    std::fs::write(&target, png).expect("writes the raster sample");
    println!("wrote {}", target.display());

    write_review_images(&catalog);
}

/// Writes the light and dark image of every built-in symbol for its review.
///
/// Same drawing code as the sheet, at a size a reader can check a numeral in: 60 × 45 mm
/// at 200 dpi is 472 × 354 px, and the symbol is drawn at half stroke.
fn write_review_images(catalog: &Catalog) {
    // One symbol at a time when a review only needs its own images; the whole catalog
    // otherwise.
    let only =
        std::env::var_os("PUNCHI_FLUID_REVIEW_ID").map(|id| id.to_string_lossy().into_owned());
    for definition in catalog.iter() {
        if only.as_deref().is_some_and(|id| id != definition.id) {
            continue;
        }
        for (theme, ink, background) in [
            ("light", Ink::print(), Colour::WHITE),
            (
                "dark",
                Ink {
                    line: Colour::new(240, 240, 240),
                    dim: Colour::new(150, 150, 150),
                    muted: Colour::new(180, 180, 180),
                    text: Colour::new(240, 240, 240),
                    supply: Colour::new(240, 240, 240),
                    work: Colour::new(240, 240, 240),
                    accent: Colour::new(45, 55, 80),
                    background: Colour::new(30, 30, 30),
                },
                Colour::new(30, 30, 30),
            ),
        ] {
            // The thumbnail's box has the aspect the application gives the symbol, so the
            // review image cannot hide a deformation: a thin barrel is drawn in a wide box
            // and everything else in a square one, exactly as the sheet does it.
            let wide = definition.kinematics == punchi_fluid_catalog::Kinematics::LinearActuator;
            let (inner_width, inner_height) = if wide {
                (48.0, 48.0 / 2.2)
            } else {
                (30.0, 30.0)
            };
            let page = Rect::from_min_size(Pt::new(0.0, 0.0), 60.0, 45.0);
            let mut surface = PngSurface::new(page, 200.0).expect("the thumbnail page");
            surface.rect(page, 0.0, Some(background), None);
            let inner = Rect::from_min_size(
                Pt::new((60.0 - inner_width) * 0.5, (45.0 - inner_height) * 0.5),
                inner_width,
                inner_height,
            );
            // A contact and a relay have a moving bar the renderer draws from the run
            // state; the review image shows it at rest, not only the file's static part.
            let stateful = matches!(
                definition.behaviour,
                punchi_fluid_catalog::Behaviour::Contact { .. }
                    | punchi_fluid_catalog::Behaviour::LimitSwitch { .. }
                    | punchi_fluid_catalog::Behaviour::PressureSwitch { .. }
                    | punchi_fluid_catalog::Behaviour::Relay { .. }
                    | punchi_fluid_catalog::Behaviour::Lamp
                    | punchi_fluid_catalog::Behaviour::Buzzer
            );
            if stateful {
                let placed = punchi_fluid_draw::Placed {
                    index: 0,
                    catalog_id: &definition.id,
                    rect: inner,
                    extension: 0.0,
                    valve_position: 0,
                    reading: match definition.behaviour {
                        punchi_fluid_catalog::Behaviour::Lamp => {
                            Some(punchi_fluid_draw::placed::Reading::Indicator(true))
                        }
                        punchi_fluid_catalog::Behaviour::Buzzer => {
                            Some(punchi_fluid_draw::placed::Reading::Audible(true))
                        }
                        _ => None,
                    },
                    series: None,
                    contact_closed: Some(false),
                    orientation: punchi_fluid_draw::Orientation::NONE,
                    designation: None,
                };
                punchi_fluid_draw::draw_placed(&mut surface, &[placed], &[], catalog, &ink, &[]);
            } else {
                punchi_fluid_draw::draw_component(&mut surface, inner, definition, 0, 0.5, &ink);
            }
            let bytes = surface.encode().expect("encodes");
            let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets/symbols/review")
                .join(format!("{}-{theme}.png", definition.id));
            std::fs::write(&target, bytes).expect("writes the review image");
            println!("wrote {}", target.display());
        }
    }
}
