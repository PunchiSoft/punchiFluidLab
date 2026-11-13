// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The same page as SVG: vector, measurable in millimetres and readable as text.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::placed::{Placed, draw_placed};
use punchi_fluid_draw::{Orientation, Pt, Rect};
use punchi_fluid_export::page::{Page, PaperSize};
use punchi_fluid_export::parts::PartsList;
use punchi_fluid_export::{TitleBlock, print_ink, render_pdf_with, render_svg_with};

fn draw_a_cylinder(surface: &mut dyn punchi_fluid_draw::Surface, area: Rect) {
    let catalog = Catalog::builtin();
    let definition = catalog
        .get("cylinder.single_acting_20_100")
        .expect("the built-in cylinder exists");
    let rect = Rect::from_min_size(Pt::new(area.min.x + 10.0, area.min.y + 10.0), 74.4, 33.8);
    let placed = [Placed {
        index: 0,
        catalog_id: definition.id.as_str(),
        rect,
        extension: 0.5,
        valve_position: 0,
        reading: None,
        series: None,
        orientation: Orientation::NONE,
        designation: Some("1A"),
    }];
    draw_placed(surface, &placed, &[], None, &catalog, &print_ink(), &[]);
}

fn page_with_parts() -> Page {
    let mut page = Page::a4(TitleBlock::new("A circuit", "2026-11-12"));
    page.parts = Some(PartsList::group(
        &Catalog::builtin(),
        [
            ("cylinder.single_acting_20_100", Some("1A")),
            ("supply.compressor", None),
        ],
    ));
    page
}

#[test]
fn the_sheet_becomes_a_vector_document() {
    let page = page_with_parts();
    let block = page.title_block.clone().unwrap_or_default();
    let svg = render_svg_with(&page, &block, draw_a_cylinder).expect("the page renders");
    assert!(svg.starts_with("<?xml"), "an XML header");
    assert!(svg.contains("<svg"), "the document element");
    assert!(
        svg.contains("viewBox=\"0 0 210.0 297.0\""),
        "A4 in millimetres"
    );
    assert!(svg.contains("lista de piezas"), "the parts list is text");
    assert!(svg.contains("Cilindro"), "with the catalog name");
    assert!(svg.contains("1A"), "and the designation");
    assert!(svg.contains("A circuit"), "the title block");
    assert!(svg.trim_end().ends_with("</svg>"));
}

#[test]
fn the_same_page_is_the_same_text_on_both_devices() {
    // One drawing, two devices: the texts the PDF carries are the texts the SVG carries,
    // so a reader who opens either sees the same sheet.
    let page = page_with_parts();
    let block = page.title_block.clone().unwrap_or_default();
    let svg = render_svg_with(&page, &block, draw_a_cylinder).expect("the SVG renders");
    let pdf = render_pdf_with(&page, &block, draw_a_cylinder).expect("the PDF renders");
    let pdf_text = decode_pdf_text(&pdf);
    for text in ["A circuit", "2026-11-12", "Cilindro", "lista de piezas"] {
        assert!(svg.contains(text), "the SVG is missing {text}");
        assert!(
            pdf_text.contains(text),
            "the PDF is missing {text}: {pdf_text:?}"
        );
    }
}

/// The text runs of a PDF content stream, which the writer stores as hexadecimal strings.
fn decode_pdf_text(pdf: &[u8]) -> String {
    let stream = String::from_utf8_lossy(pdf);
    let mut out = String::new();
    for chunk in stream.split('<').skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        let hex = &chunk[..end];
        if hex.is_empty() || hex.len() % 2 != 0 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        for pair in hex.as_bytes().chunks(2) {
            let Ok(pair) = std::str::from_utf8(pair) else {
                continue;
            };
            let Ok(byte) = u8::from_str_radix(pair, 16) else {
                continue;
            };
            out.push(char::from(byte));
        }
        out.push('\n');
    }
    out
}

#[test]
fn a_title_with_markup_is_escaped_instead_of_breaking_the_document() {
    let block = TitleBlock::new("A & B <C>", "2026-11-12");
    let mut page = Page::a4(block.clone());
    page.paper = PaperSize::A5;
    let svg = render_svg_with(&page, &block, draw_a_cylinder).expect("the page renders");
    assert!(svg.contains("A &amp; B &lt;C&gt;"), "the markup is escaped");
    assert!(!svg.contains("<C>"), "no raw element survives the title");
}

#[test]
fn the_drawing_stays_inside_the_sheet() {
    let page = page_with_parts();
    let block = page.title_block.clone().unwrap_or_default();
    let svg = render_svg_with(&page, &block, draw_a_cylinder).expect("the page renders");
    // Every coordinate the writer emits is in millimetres of the paper, so a negative one
    // or one past the sheet is a layout bug the test can see without a raster.
    for line in svg.lines().filter(|line| {
        line.starts_with("<polyline") || line.starts_with("<rect") || line.starts_with("<circle")
    }) {
        for token in ["x=\"", "cx=\""] {
            if let Some(rest) = line.split(token).nth(1) {
                let value: f32 = rest
                    .split('"')
                    .next()
                    .and_then(|number| number.parse().ok())
                    .unwrap_or_default();
                assert!(
                    (-0.5..=210.5).contains(&value),
                    "x is out of the paper: {line}"
                );
            }
        }
    }
}
