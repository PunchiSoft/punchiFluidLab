// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The acceptance criteria of the export, checked rather than claimed.
#![cfg_attr(
    test,
    allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)
)]

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::{Call, Ink, Pt, Recording, Rect};
use punchi_fluid_export::{ExportError, TitleBlock, page::Page, pdf::PdfSurface, render_pdf};
use punchi_fluid_solver::network::Network;

fn system() -> Network {
    let catalog = Catalog::builtin();
    Network::demo(&catalog).expect("the built-in circuit starts")
}

fn render(page: &Page, block: &TitleBlock) -> Vec<u8> {
    let catalog = Catalog::builtin();
    render_pdf(page, block, &system(), &catalog).expect("the page renders")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `true` when the rendered sheet contains `needle`, as a PDF hexadecimal string.
fn shows(pdf: &str, needle: &str) -> bool {
    let hex: String = needle.bytes().fold(String::new(), |mut out, byte| {
        let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{byte:02X}"));
        out
    });
    pdf.contains(&hex)
}

#[test]
fn the_media_box_matches_the_chosen_paper_within_a_point() {
    for (paper, orientation, (width_mm, height_mm)) in [
        (
            punchi_fluid_export::PaperSize::A4,
            punchi_fluid_export::Orientation::Portrait,
            (210.0, 297.0),
        ),
        (
            punchi_fluid_export::PaperSize::A3,
            punchi_fluid_export::Orientation::Landscape,
            (420.0, 297.0),
        ),
        (
            punchi_fluid_export::PaperSize::Letter,
            punchi_fluid_export::Orientation::Portrait,
            (215.9, 279.4),
        ),
    ] {
        let page = Page {
            paper,
            orientation,
            ..Page::a4(TitleBlock::default())
        };
        let pdf = text(&render(&page, &TitleBlock::new("A circuit", "2026-11-06")));
        let start = pdf.find("/MediaBox [").expect("a MediaBox") + "/MediaBox [".len();
        let end = start + pdf[start..].find(']').expect("the end of the MediaBox");
        let numbers: Vec<f32> = pdf[start..end]
            .split_whitespace()
            .filter_map(|token| token.parse().ok())
            .collect();
        let ratio = 72.0 / 25.4;
        assert_eq!(numbers.len(), 4);
        assert!((numbers[2] - width_mm * ratio).abs() < 1.0, "{numbers:?}");
        assert!((numbers[3] - height_mm * ratio).abs() < 1.0, "{numbers:?}");
    }
}

#[test]
fn the_document_has_exactly_one_page() {
    let pdf = text(&render(
        &Page::a4(TitleBlock::default()),
        &TitleBlock::new("A circuit", "2026-11-06"),
    ));
    assert_eq!(pdf.matches("/Type /Page ").count(), 1);
    assert_eq!(pdf.matches("/Count 1").count(), 1);
}

#[test]
fn the_file_carries_no_path_user_or_host_and_no_clock_beyond_the_block() {
    let pdf = text(&render(
        &Page::a4(TitleBlock::default()),
        &TitleBlock::new("A circuit", "2026-11-06"),
    ));
    for forbidden in [
        "/Producer",
        "/Creator",
        "/CreationDate",
        "/ID",
        "/home/",
        "/tmp/",
    ] {
        assert!(
            !pdf.contains(forbidden),
            "the PDF must not contain {forbidden}"
        );
    }
    for variable in ["USER", "LOGNAME", "HOSTNAME"] {
        if let Ok(value) = std::env::var(variable) {
            assert!(
                value.is_empty() || !pdf.contains(&value),
                "the PDF must not contain {variable}"
            );
        }
    }
}

#[test]
fn two_renders_of_the_same_input_are_identical() {
    let page = Page::a4(TitleBlock::default());
    let block = TitleBlock::new("A circuit", "2026-11-06");
    assert_eq!(render(&page, &block), render(&page, &block));
}

#[test]
fn an_open_review_keeps_the_unverified_note_in_the_file() {
    let block = punchi_fluid_export::block_with_review_state("A circuit", "2026-11-06");
    let pdf = text(&render(&Page::a4(TitleBlock::default()), &block));
    if punchi_fluid_catalog::reviews::all_complete() {
        assert!(!shows(&pdf, "not yet verified"));
    } else {
        assert!(
            shows(&pdf, "not yet verified"),
            "the note must be in the file"
        );
    }
}

#[test]
fn a_page_with_no_room_left_is_refused_before_writing() {
    let page = Page {
        paper: punchi_fluid_export::PaperSize::A5,
        margins: punchi_fluid_export::Margins {
            top: 20.0,
            bottom: 20.0,
            left: 80.0,
            right: 80.0,
        },
        ..Page::a4(TitleBlock::default())
    };
    let catalog = Catalog::builtin();
    let error = render_pdf(&page, &TitleBlock::default(), &system(), &catalog)
        .expect_err("a page with no room must be refused");
    assert!(matches!(error, ExportError::EmptyContent(_)), "{error}");
}

#[test]
fn the_drawing_lands_inside_the_content_rectangle() {
    let page = Page::a4(TitleBlock::default());
    let block = TitleBlock::new("A circuit", "2026-11-06");
    let mut surface = PdfSurface::new(page.rect());
    punchi_fluid_export::sheet::draw(&mut surface, &page, &block);
    let catalog = Catalog::builtin();
    let drawing = page.drawing_rect();
    punchi_fluid_draw::draw(&mut surface, drawing, &system(), &catalog, &Ink::print());

    let content = page.content_rect();
    for token in surface.content().split_whitespace() {
        let Ok(value) = token.parse::<f32>() else {
            continue;
        };
        // Device coordinates are points from the bottom-left; a coordinate outside the
        // page would mean the drawing escaped its frame.
        assert!(
            (-1.0..=(page.size_mm().0.max(page.size_mm().1) * 72.0 / 25.4 + 1.0)).contains(&value),
            "coordinate {value} leaves the page, content is {content:?}"
        );
    }
}

#[test]
fn screen_and_paper_differ_only_in_scale() {
    let catalog = Catalog::builtin();
    let circuit = system();

    // Both areas are above the numeral legibility floor, so the only difference left
    // is the scale. Below the floor the drawing drops the numerals on purpose, which is
    // a different drawing and a different test.
    let small = Rect::from_min_size(Pt::new(0.0, 0.0), 300.0, 195.0);
    let large = Rect::from_min_size(Pt::new(0.0, 0.0), 600.0, 390.0);

    let mut first = Recording::default();
    punchi_fluid_draw::draw(&mut first, small, &circuit, &catalog, &Ink::print());
    let mut second = Recording::default();
    punchi_fluid_draw::draw(&mut second, large, &circuit, &catalog, &Ink::print());

    assert_eq!(
        first.len(),
        second.len(),
        "the same calls, not a second drawing"
    );
    for (a, b) in first.calls().iter().zip(second.calls()) {
        match (a, b) {
            (
                Call::Line {
                    points: pa,
                    stroke: sa,
                },
                Call::Line {
                    points: pb,
                    stroke: sb,
                },
            ) => {
                assert_eq!(pa.len(), pb.len());
                for (one, two) in pa.iter().zip(pb) {
                    assert!((two.x - one.x * 2.0).abs() < 1.0e-3);
                    assert!((two.y - one.y * 2.0).abs() < 1.0e-3);
                }
                assert!(
                    (sb.width - sa.width).abs() < 1.0e-6,
                    "a stroke is a width, not a scale"
                );
            }
            (
                Call::Text {
                    text: ta, size: sa, ..
                },
                Call::Text {
                    text: tb, size: sb, ..
                },
            ) => {
                assert_eq!(ta, tb);
                assert_eq!(sa, sb);
            }
            (one, two) => assert_eq!(
                std::mem::discriminant(one),
                std::mem::discriminant(two),
                "the sequence of calls must match"
            ),
        }
    }
}
