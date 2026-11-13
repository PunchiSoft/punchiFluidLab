// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A page whose drawing the caller supplies: the seam the session uses to print the sheet it
//! holds, without the export having to know what a document is.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::placed::{Pipe, PipeEnd, Placed, draw_placed};
use punchi_fluid_draw::{Colour, Orientation, Pt, Rect};
use punchi_fluid_export::page::{Page, PaperSize};
use punchi_fluid_export::{TitleBlock, print_ink, render_pdf_with, render_png_with};

/// One cylinder and one hose on the page, drawn from the document shapes rather than from the
/// fixed circuit the export crate knows.
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
    let pipes: [Pipe<'_>; 0] = [];
    draw_placed(surface, &placed, &pipes, &catalog, &print_ink(), &[]);
    let _ = PipeEnd {
        placement: 0,
        port: "1",
    };
    let _ = Colour::WHITE;
}

#[test]
fn a_page_drawn_by_the_caller_is_a_real_pdf() {
    let block = TitleBlock::new("A circuit", "2026-11-07");
    let pdf = render_pdf_with(&Page::a4(block.clone()), &block, draw_a_cylinder)
        .expect("the page renders");
    assert_eq!(&pdf[..5], b"%PDF-", "a PDF header");
    assert!(pdf.len() > 1000, "a page with a symbol is not empty");
}

#[test]
fn the_caller_sheet_keeps_the_paper_it_was_given() {
    let block = TitleBlock::new("A circuit", "2026-11-07");
    let page = Page {
        paper: PaperSize::Oficio,
        ..Page::a4(block.clone())
    };
    let pdf = render_pdf_with(&page, &block, draw_a_cylinder).expect("the page renders");
    let text = String::from_utf8_lossy(&pdf);
    let start = text.find("/MediaBox [").expect("a MediaBox") + "/MediaBox [".len();
    let end = start + text[start..].find(']').expect("the end of the MediaBox");
    let numbers: Vec<f32> = text[start..end]
        .split_whitespace()
        .filter_map(|token| token.parse().ok())
        .collect();
    let ratio = 72.0 / 25.4;
    assert!((numbers[2] - 216.0 * ratio).abs() < 0.6);
    assert!((numbers[3] - 330.0 * ratio).abs() < 0.6);
}

#[test]
fn the_same_caller_drawing_twice_is_the_same_file() {
    let block = TitleBlock::new("A circuit", "2026-11-07");
    let page = Page::a4(block.clone());
    let first = render_png_with(&page, &block, 96.0, draw_a_cylinder).expect("renders");
    let second = render_png_with(&page, &block, 96.0, draw_a_cylinder).expect("renders");
    assert_eq!(first, second, "the export is deterministic");
    assert_eq!(&first[..8], b"\x89PNG\r\n\x1a\n");
}
