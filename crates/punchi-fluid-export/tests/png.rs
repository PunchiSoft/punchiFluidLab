// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The raster acceptance criteria: size, margin band and determinism.
#![cfg_attr(
    test,
    allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)
)]

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::Ink;
use punchi_fluid_export::page::{Page, PaperSize};
use punchi_fluid_export::png::PngSurface;
use punchi_fluid_export::{TitleBlock, render_png};
use punchi_fluid_solver::network::Network;

fn system() -> Network {
    let catalog = Catalog::builtin();
    Network::demo(&catalog).expect("the built-in circuit starts")
}

/// Reads width and height from the PNG header: signature, length, type, IHDR.
fn header_size(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "a PNG signature");
    assert_eq!(&bytes[12..16], b"IHDR");
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    (width, height)
}

#[test]
fn a4_landscape_at_300_dpi_produces_a_3508_by_2480_png() {
    let catalog = Catalog::builtin();
    let block = TitleBlock::new("A circuit", "2026-11-06");
    let bytes = render_png(&Page::a4(block.clone()), &block, &system(), &catalog, 300.0)
        .expect("the page renders");
    assert_eq!(header_size(&bytes), (3508, 2480));
}

#[test]
fn a3_landscape_at_300_dpi_produces_a_4961_by_3508_png() {
    let catalog = Catalog::builtin();
    let block = TitleBlock::new("A circuit", "2026-11-06");
    let page = Page {
        paper: PaperSize::A3,
        orientation: punchi_fluid_export::Orientation::Landscape,
        ..Page::a4(block.clone())
    };
    let bytes = render_png(&page, &block, &system(), &catalog, 300.0).expect("the page renders");
    assert_eq!(header_size(&bytes), (4961, 3508));
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn the_margin_band_holds_no_ink() {
    let catalog = Catalog::builtin();
    let block = punchi_fluid_export::block_with_review_state("A circuit", "2026-11-06");
    let dpi = 150.0;
    let scale = dpi / 25.4;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let side = |mm: f32| ((mm * scale).floor() as u32).saturating_sub(4);
    let clear = 3.0 * scale;

    // Draws a page and returns the darkest pixel of a band, leaving out the centring
    // marks. `band` picks the region; the mark that legitimately crosses it is skipped.
    let darkest = |paper: PaperSize, band: &str| {
        let page = Page {
            paper,
            ..Page::a4(block.clone())
        };
        let mut surface = PngSurface::new(page.rect(), dpi).expect("the page is allowed");
        punchi_fluid_export::sheet::draw(&mut surface, &page, &block);
        punchi_fluid_draw::draw(
            &mut surface,
            page.drawing_rect(),
            &system(),
            &catalog,
            &Ink::print(),
        );
        let left = side(page.margins.left);
        let right = side(page.margins.right);
        let top = side(page.margins.top);
        let bottom = side(page.margins.bottom);
        let pixmap = surface.pixmap();
        let mark_x = pixmap.width() as f32 * 0.5;
        let mark_y = pixmap.height() as f32 * 0.5;
        let mut darkest = 255_u8;
        for y in 0..pixmap.height() {
            for x in 0..pixmap.width() {
                let in_left = x < left;
                let in_right = x >= pixmap.width() - right;
                let in_top = y < top;
                let in_bottom = y >= pixmap.height() - bottom;
                let outside = in_left || in_right || in_top || in_bottom;
                if !outside {
                    continue;
                }
                let wanted = match band {
                    "left" => in_left,
                    "right" => in_right,
                    "top" => in_top,
                    "bottom" => in_bottom,
                    _ => outside,
                };
                if !wanted {
                    continue;
                }
                // ISO 5457 puts the centring marks in the margin at the middle of each
                // side: that ink is intended. A horizontal mark crosses the left and right
                // bands; a vertical one crosses the top and bottom bands.
                if (in_left || in_right) && (y as f32 - mark_y).abs() < clear {
                    continue;
                }
                if (in_top || in_bottom) && (x as f32 - mark_x).abs() < clear {
                    continue;
                }
                if let Some(pixel) = pixmap.pixel(x, y) {
                    darkest = darkest.min(pixel.demultiply().red());
                }
            }
        }
        darkest
    };

    // The sheets without reference zones keep every band clean but the marks.
    for paper in [
        PaperSize::A5,
        PaperSize::Letter,
        PaperSize::Legal,
        PaperSize::Oficio,
    ] {
        let value = darkest(paper, "all");
        assert!(
            value > 250,
            "{}: the margin band must stay white, found a pixel at {value}",
            paper.label()
        );
    }

    // A4 now carries the standard's two-sided grid. Its top and right bands are meant to
    // be inked; the left and bottom bands must stay clean, because A4 is not four-sided.
    let left = darkest(PaperSize::A4, "left");
    assert!(
        left > 250,
        "A4: the left band must stay white, found a pixel at {left}"
    );
    let bottom = darkest(PaperSize::A4, "bottom");
    assert!(
        bottom > 250,
        "A4: the bottom band must stay white, found a pixel at {bottom}"
    );
    // The two bands it keeps must carry the grid: a clean sheet here would pass the two
    // assertions above while losing the zones altogether.
    let top = darkest(PaperSize::A4, "top");
    assert!(
        top < 250,
        "A4: the top band carries the numerals, found a blank band at {top}"
    );
    let right = darkest(PaperSize::A4, "right");
    assert!(
        right < 250,
        "A4: the right band carries the letters, found a blank band at {right}"
    );
}

#[test]
fn the_drawing_marks_pixels_inside_the_content() {
    let catalog = Catalog::builtin();
    let block = TitleBlock::new("A circuit", "2026-11-06");
    let page = Page::a4(block.clone());
    let dpi = 150.0;
    let mut surface = PngSurface::new(page.rect(), dpi).expect("the page is allowed");
    punchi_fluid_export::sheet::draw(&mut surface, &page, &block);
    punchi_fluid_draw::draw(
        &mut surface,
        page.drawing_rect(),
        &system(),
        &catalog,
        &Ink::print(),
    );

    let content = page.content_rect();
    let scale = dpi / 25.4;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let centre = (
        ((content.center().x) * scale) as u32,
        ((content.center().y) * scale) as u32,
    );
    let pixmap = surface.pixmap();
    assert!(pixmap.pixel(centre.0, centre.1).is_some());
    let dark = pixmap
        .pixels()
        .iter()
        .filter(|pixel| pixel.demultiply().red() < 200)
        .count();
    assert!(
        dark > 200,
        "the sheet must contain the drawing, found {dark} dark pixels"
    );
}

#[test]
fn two_renders_of_the_same_input_are_identical() {
    let catalog = Catalog::builtin();
    let block = TitleBlock::new("A circuit", "2026-11-06");
    let page = Page::a4(block.clone());
    let first = render_png(&page, &block, &system(), &catalog, 96.0).expect("renders");
    let second = render_png(&page, &block, &system(), &catalog, 96.0).expect("renders");
    assert_eq!(first, second);
}
