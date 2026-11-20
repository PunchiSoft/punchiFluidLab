// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The optional reference zones of ISO 5457 §4.4.
//!
//! The same geometry the canvas repeats in `apps/punchi-fluid-qt/src/sheetframe.cpp`:
//! fields of at most 50 mm, numerals left to right on the horizontal bands and letters
//! top to bottom on the vertical bands, with `I` and `O` skipped. A4 uses the standard's
//! two-sided form; A3 and larger draw all four bands.

use punchi_fluid_draw::{Anchor, Colour, Pt, Rect, Stroke, Surface};

use crate::page::{Page, PaperSize};
use crate::sheet::{MUTED, RULE, VALUE};

/// The optional reference zones of ISO 5457 / `NCh` 13.Of93.
///
/// The drawing space is divided into fields of at most 50 mm. On A3 and larger the numerals
/// `1`, `2`, … read left to right along the top and bottom bands and the letters `A`, `B`, …
/// read top to bottom along the left and right bands, 3.5 mm high with 0.35 mm rules. A4
/// uses the standard's A4 form: numerals on the top band only and letters on the right band
/// only. The rules and the characters stay inside the margin bands, so the drawing space
/// itself carries no extra ink.
pub(crate) fn draw_reference_zones(surface: &mut dyn Surface, page: &Page) {
    if !page.has_reference_zones() {
        return;
    }
    let paper = page.rect();
    let content = page.content_rect();
    if content.is_empty() {
        return;
    }
    let bands = [
        content.min.x - paper.min.x,
        paper.max.x - content.max.x,
        content.min.y - paper.min.y,
        paper.max.y - content.max.y,
    ];
    // A band narrower than the character plus its clearance cannot hold a label without
    // printing it on the paper's edge or outside it. Draw the frame only, never a clipped
    // or overflowing reference grid.
    if bands.iter().any(|band| *band < VALUE + LABEL_CLEARANCE) {
        return;
    }

    let columns = field_count(content.width());
    let rows = field_count(content.height());
    #[allow(clippy::cast_precision_loss)]
    let step_x = content.width() / columns as f32;
    #[allow(clippy::cast_precision_loss)]
    let step_y = content.height() / rows as f32;
    let grid = Grid {
        paper,
        content,
        columns,
        rows,
        step_x,
        step_y,
        a4: page.paper == PaperSize::A4,
    };

    draw_column_labels(surface, &grid);
    draw_row_labels(surface, &grid);
    draw_division_rules(surface, &grid);
}

/// The geometry of one reference grid, shared by the three drawing steps.
#[derive(Clone, Copy)]
struct Grid {
    paper: Rect,
    content: Rect,
    columns: usize,
    rows: usize,
    step_x: f32,
    step_y: f32,
    a4: bool,
}

/// Numerals across the top band, and across the bottom band too on A3 and larger.
///
/// A4 carries them on the top band only.
fn draw_column_labels(surface: &mut dyn Surface, grid: &Grid) {
    let mark_x = f32::midpoint(grid.paper.min.x, grid.paper.max.x);
    let top_centre = f32::midpoint(grid.paper.min.y, grid.content.min.y);
    let bottom_centre = f32::midpoint(grid.content.max.y, grid.paper.max.y);
    for column in 0..grid.columns {
        #[allow(clippy::cast_precision_loss)]
        let centre = grid.content.min.x + (column as f32 + 0.5) * grid.step_x;
        // The top and bottom centring marks are vertical lines through the bands, so the
        // numeral gives way along the band, not across it.
        let Some(x) = clear_of_mark(centre, mark_x, grid.step_x) else {
            continue;
        };
        let label = (column + 1).to_string();
        let mut places = vec![top_centre];
        if !grid.a4 {
            places.push(bottom_centre);
        }
        for y in places {
            surface.text(
                Pt::new(x, y + VALUE * 0.36),
                &label,
                VALUE,
                MUTED,
                Anchor::Middle,
            );
        }
    }
}

/// Letters down the left and right bands on A3 and larger; on A4, down the right band only.
fn draw_row_labels(surface: &mut dyn Surface, grid: &Grid) {
    let mark_y = f32::midpoint(grid.paper.min.y, grid.paper.max.y);
    let left_centre = f32::midpoint(grid.paper.min.x, grid.content.min.x);
    let right_centre = f32::midpoint(grid.content.max.x, grid.paper.max.x);
    for row in 0..grid.rows {
        #[allow(clippy::cast_precision_loss)]
        let centre = grid.content.min.y + (row as f32 + 0.5) * grid.step_y;
        // The left and right centring marks are horizontal lines through the bands.
        let Some(y) = clear_of_mark(centre, mark_y, grid.step_y) else {
            continue;
        };
        let label = row_label(row);
        let mut places = Vec::with_capacity(2);
        if grid.a4 {
            places.push(right_centre);
        } else {
            places.push(left_centre);
            places.push(right_centre);
        }
        for x in places {
            surface.text(
                Pt::new(x, y + VALUE * 0.36),
                &label,
                VALUE,
                MUTED,
                Anchor::Middle,
            );
        }
    }
}

/// The division rules, only in the bands that carry a label.
///
/// They never enter the drawing space and never enter one of A4's two empty bands.
fn draw_division_rules(surface: &mut dyn Surface, grid: &Grid) {
    let stroke = Stroke::new(RULE, Colour::BLACK);
    for column in 1..grid.columns {
        #[allow(clippy::cast_precision_loss)]
        let x = grid.content.min.x + column as f32 * grid.step_x;
        surface.line(
            &[Pt::new(x, grid.paper.min.y), Pt::new(x, grid.content.min.y)],
            stroke,
        );
        if !grid.a4 {
            surface.line(
                &[Pt::new(x, grid.content.max.y), Pt::new(x, grid.paper.max.y)],
                stroke,
            );
        }
    }
    for row in 1..grid.rows {
        #[allow(clippy::cast_precision_loss)]
        let y = grid.content.min.y + row as f32 * grid.step_y;
        if !grid.a4 {
            surface.line(
                &[Pt::new(grid.paper.min.x, y), Pt::new(grid.content.min.x, y)],
                stroke,
            );
        }
        surface.line(
            &[Pt::new(grid.content.max.x, y), Pt::new(grid.paper.max.x, y)],
            stroke,
        );
    }
}

/// Number of reference fields along one side: 50 mm each, the last one never longer.
fn field_count(length: f32) -> usize {
    if !length.is_finite() || length <= 0.0 {
        return 1;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = (length / 50.0).ceil() as usize;
    count.max(1)
}

/// The minimum distance from a label's centre to a centring mark, in millimetres.
const LABEL_CLEARANCE: f32 = 1.0;

/// Moves a label centre clear of a centring mark, or `None` when it cannot give way.
///
/// A centring mark runs through the middle of its band, so only the coordinate along the
/// band can move. The label keeps half its cap height plus [`LABEL_CLEARANCE`] from the
/// line and never travels further than half a field, which would make it read as its
/// neighbour's; when even that cannot clear the mark it is omitted, not printed over it.
fn clear_of_mark(centre: f32, mark: f32, field: f32) -> Option<f32> {
    if !centre.is_finite() || !mark.is_finite() || !field.is_finite() || field <= 0.0 {
        return None;
    }
    let clearance = VALUE * 0.5 + LABEL_CLEARANCE;
    if (centre - mark).abs() >= clearance {
        return Some(centre);
    }
    let wanted = if centre >= mark {
        mark + clearance
    } else {
        mark - clearance
    };
    if (wanted - centre).abs() <= field * 0.5 && (wanted - mark).abs() >= LABEL_CLEARANCE {
        Some(wanted)
    } else {
        None
    }
}

/// The 24 field letters of ISO 5457 §4.4: `A`..`Z` without `I` and `O`.
///
/// They are also the alphabet of the rare sheet that needs more rows than A0: a custom
/// paper can reach 2000 mm, which is 40 fields, so the sequence continues with `AA`, `AB`, …
/// over the same 24 letters. No label ever contains a forbidden letter.
const FIELD_LETTERS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";

/// Letter of a reference row: `A`, `B`, … skipping `I` and `O`.
fn row_label(mut index: usize) -> String {
    let base = FIELD_LETTERS.len();
    let mut label = String::new();
    loop {
        let letter = FIELD_LETTERS[index % base];
        label.insert(0, char::from(letter));
        if index < base {
            break;
        }
        index = index / base - 1;
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::{Margins, TitleBlock};
    use punchi_fluid_draw::Recording;

    /// Zone labels in draw order: the text and the anchor point the sheet records.
    fn zone_labels(page: &Page) -> Vec<(String, Pt)> {
        let mut recording = Recording::default();
        draw_reference_zones(&mut recording, page);
        recording
            .calls()
            .iter()
            .filter_map(|call| match call {
                punchi_fluid_draw::Call::Text {
                    at,
                    text,
                    size,
                    colour,
                    ..
                } if (*size - VALUE).abs() < 0.01 && *colour == MUTED => Some((text.clone(), *at)),
                _ => None,
            })
            .collect()
    }

    fn is_numeral(text: &str) -> bool {
        !text.is_empty() && text.chars().all(|c| c.is_ascii_digit())
    }

    fn is_letter(text: &str) -> bool {
        !text.is_empty() && text.chars().all(|c| c.is_ascii_alphabetic())
    }

    #[test]
    fn a3_gets_the_reference_zones_on_four_sides() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        let page = Page {
            paper: PaperSize::A3,
            orientation: punchi_fluid_core::Orientation::Landscape,
            ..Page::a4(block)
        };
        let content = page.content_rect();
        let labels = zone_labels(&page);

        // A letter sits left of the drawing space and a numeral above it.
        assert!(
            labels
                .iter()
                .any(|(text, at)| is_letter(text) && at.x < content.min.x),
            "A3 has no letter in the left band: {labels:?}"
        );
        assert!(
            labels
                .iter()
                .any(|(text, at)| is_numeral(text) && at.y < content.min.y),
            "A3 has no numeral in the top band: {labels:?}"
        );

        // The negative half is what fails if the axes are swapped back: a letter must not
        // appear in a horizontal band, nor a numeral in a vertical one.
        for (text, at) in &labels {
            assert!(
                is_letter(text) || is_numeral(text),
                "a zone label is neither a letter nor a numeral: {text}"
            );
            if is_letter(text) {
                assert!(
                    at.x < content.min.x || at.x > content.max.x,
                    "the letter {text} sits in a horizontal band at {at:?}"
                );
            } else {
                assert!(
                    at.y < content.min.y || at.y > content.max.y,
                    "the numeral {text} sits in a vertical band at {at:?}"
                );
            }
        }
    }

    #[test]
    fn a4_gets_its_reference_zones_on_top_and_right() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        for orientation in [
            punchi_fluid_core::Orientation::Landscape,
            punchi_fluid_core::Orientation::Portrait,
        ] {
            let page = Page {
                paper: PaperSize::A4,
                orientation,
                ..Page::a4(block.clone())
            };
            let content = page.content_rect();
            let labels = zone_labels(&page);

            assert!(
                labels
                    .iter()
                    .any(|(text, at)| is_numeral(text) && at.y < content.min.y),
                "{orientation:?}: A4 has no numeral in the top band: {labels:?}"
            );
            assert!(
                labels
                    .iter()
                    .any(|(text, at)| is_letter(text) && at.x > content.max.x),
                "{orientation:?}: A4 has no letter in the right band: {labels:?}"
            );
            // The other bands carry no label: a two-sided form, not four.
            for (text, at) in &labels {
                assert!(
                    is_letter(text) || is_numeral(text),
                    "a zone label is neither a letter nor a numeral: {text}"
                );
                if is_letter(text) {
                    assert!(
                        at.x > content.max.x,
                        "{orientation:?}: the letter {text} left the right band at {at:?}"
                    );
                } else {
                    assert!(
                        at.y < content.min.y,
                        "{orientation:?}: the numeral {text} left the top band at {at:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_row_labels_skip_i_and_o() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        let page = Page {
            paper: PaperSize::A0,
            orientation: punchi_fluid_core::Orientation::Landscape,
            ..Page::a4(block)
        };
        let rows = field_count(page.content_rect().height());
        // The current division, pinned: A0 landscape has 17 rows. The standard's table
        // records 16; that open question is not settled here.
        assert_eq!(rows, 17, "A0 landscape rows");
        for (index, expected) in [(7, "H"), (8, "J"), (12, "N"), (13, "P"), (16, "S")] {
            assert_eq!(row_label(index), expected, "row {index}");
        }

        let labels = zone_labels(&page);
        let letters: Vec<&str> = labels
            .iter()
            .filter(|(text, _)| is_letter(text))
            .map(|(text, _)| text.as_str())
            .collect();
        assert_eq!(
            letters.len(),
            rows * 2,
            "every row keeps one letter per vertical band"
        );
        for forbidden in ["I", "O"] {
            assert!(
                !letters.contains(&forbidden),
                "the row letter {forbidden} is forbidden"
            );
        }
    }

    #[test]
    fn a_custom_sheet_labels_beyond_z_as_aa_without_i_or_o() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        let page = Page {
            paper: PaperSize::custom(2000.0, 2000.0).expect("a custom sheet inside the bound"),
            orientation: punchi_fluid_core::Orientation::Landscape,
            ..Page::a4(block)
        };
        let rows = field_count(page.content_rect().height());
        // The A-series stops at 24 fields; a custom sheet reaches 40 on its 2 000 mm side.
        assert_eq!(rows, 40, "a 2 000 mm sheet has 40 fields");

        let labels = zone_labels(&page);
        let letters: Vec<&str> = labels
            .iter()
            .filter(|(text, _)| is_letter(text))
            .map(|(text, _)| text.as_str())
            .collect();
        assert_eq!(
            letters.len(),
            rows * 2,
            "every row keeps one letter per vertical band"
        );
        for expected in ["AA", "AB"] {
            assert!(
                letters.contains(&expected),
                "the sequence continues past Z with {expected}: {letters:?}"
            );
        }
        for (text, _) in &labels {
            assert!(
                !text.contains('I') && !text.contains('O'),
                "the label {text} uses a forbidden letter"
            );
        }
    }

    #[test]
    fn reference_rules_and_characters_never_enter_the_drawing_space() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        let page = Page {
            paper: PaperSize::A3,
            orientation: punchi_fluid_core::Orientation::Portrait,
            ..Page::a4(block.clone())
        };
        let content = page.content_rect();
        let mut recording = Recording::default();
        draw_reference_zones(&mut recording, &page);
        assert!(!recording.is_empty(), "A3 carries reference zones");
        let in_band = |point: Pt| {
            point.x <= content.min.x + 0.01
                || point.x >= content.max.x - 0.01
                || point.y <= content.min.y + 0.01
                || point.y >= content.max.y - 0.01
        };
        assert!(
            recording.calls().iter().all(|call| matches!(
                call,
                punchi_fluid_draw::Call::Line { .. } | punchi_fluid_draw::Call::Text { .. }
            )),
            "the zones draw only rules and characters"
        );
        for call in recording.calls() {
            match call {
                punchi_fluid_draw::Call::Line { points, .. } => {
                    for point in points {
                        assert!(
                            in_band(*point),
                            "a reference rule entered the drawing space at {point:?}"
                        );
                    }
                }
                punchi_fluid_draw::Call::Text { at, .. } => {
                    assert!(
                        in_band(*at),
                        "a reference character entered the drawing space at {at:?}"
                    );
                }
                _ => {}
            }
        }

        // A4 draws the top and right bands and leaves the left and bottom ones blank, rules
        // included: a two-sided form, not four.
        for orientation in [
            punchi_fluid_core::Orientation::Landscape,
            punchi_fluid_core::Orientation::Portrait,
        ] {
            let page = Page {
                paper: PaperSize::A4,
                orientation,
                ..Page::a4(block.clone())
            };
            let content = page.content_rect();
            let mut recording = Recording::default();
            draw_reference_zones(&mut recording, &page);
            assert!(!recording.is_empty(), "{orientation:?}: A4 carries zones");
            let in_top_or_right =
                |point: Pt| point.y <= content.min.y + 0.01 || point.x >= content.max.x - 0.01;
            for call in recording.calls() {
                match call {
                    punchi_fluid_draw::Call::Line { points, .. } => {
                        for point in points {
                            assert!(
                                in_top_or_right(*point),
                                "{orientation:?}: A4 ink left the top and right bands at {point:?}"
                            );
                        }
                    }
                    punchi_fluid_draw::Call::Text { at, .. } => {
                        assert!(
                            in_top_or_right(*at),
                            "{orientation:?}: an A4 label left the top and right bands at {at:?}"
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn every_zone_label_keeps_clear_of_the_centring_marks() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        let presets = [
            ("normal", Margins::iso5457()),
            (
                "narrow",
                Margins {
                    top: 10.0,
                    right: 10.0,
                    bottom: 10.0,
                    left: 10.0,
                },
            ),
            (
                "wide",
                Margins {
                    top: 20.0,
                    right: 20.0,
                    bottom: 20.0,
                    left: 20.0,
                },
            ),
        ];
        let mut papers = PaperSize::ALL.to_vec();
        papers.push(PaperSize::custom(2000.0, 2000.0).expect("a custom sheet inside the bound"));
        for paper in papers {
            for orientation in [
                punchi_fluid_core::Orientation::Landscape,
                punchi_fluid_core::Orientation::Portrait,
            ] {
                for (preset, margins) in presets {
                    let page = Page {
                        paper,
                        orientation,
                        margins,
                        ..Page::a4(block.clone())
                    };
                    if !page.has_reference_zones() {
                        continue;
                    }
                    let content = page.content_rect();
                    if content.is_empty() {
                        continue;
                    }
                    let paper_rect = page.rect();
                    let mark_x = f32::midpoint(paper_rect.min.x, paper_rect.max.x);
                    let mark_y = f32::midpoint(paper_rect.min.y, paper_rect.max.y);
                    let labels = zone_labels(&page);
                    let mut numerals = 0;
                    let mut letters = 0;
                    for (text, at) in &labels {
                        if is_numeral(text) {
                            numerals += 1;
                            // A numeral sits in a horizontal band, so the vertical centring
                            // mark is the one it can touch.
                            assert!(
                                (at.x - mark_x).abs() >= LABEL_CLEARANCE - 0.001,
                                "{} {orientation:?} {preset}: numeral {text} at {at:?} \
                                 is {} mm from the mark",
                                paper.label(),
                                (at.x - mark_x).abs()
                            );
                        } else if is_letter(text) {
                            letters += 1;
                            // The recorded point is the baseline; the glyph centre is above
                            // it, and the horizontal mark crosses the vertical bands.
                            let centre = at.y - VALUE * 0.36;
                            assert!(
                                (centre - mark_y).abs() >= LABEL_CLEARANCE - 0.001,
                                "{} {orientation:?} {preset}: letter {text} at {at:?} \
                                 is {} mm from the mark",
                                paper.label(),
                                (centre - mark_y).abs()
                            );
                        }
                    }
                    // No label is omitted: a shift is enough on every supported sheet.
                    let columns = field_count(content.width());
                    let rows = field_count(content.height());
                    let sides = if paper == PaperSize::A4 { 1 } else { 2 };
                    assert_eq!(
                        numerals,
                        columns * sides,
                        "{} {orientation:?} {preset}: numerals",
                        paper.label()
                    );
                    assert_eq!(
                        letters,
                        rows * sides,
                        "{} {orientation:?} {preset}: letters",
                        paper.label()
                    );
                }
            }
        }
    }

    #[test]
    fn a_margin_too_narrow_for_the_labels_draws_none() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        let page = Page {
            paper: PaperSize::custom(500.0, 400.0).expect("a custom sheet"),
            margins: Margins {
                top: 2.0,
                right: 2.0,
                bottom: 2.0,
                left: 2.0,
            },
            ..Page::a4(block)
        };
        assert!(page.has_reference_zones());
        assert!(!page.content_rect().is_empty(), "the frame still has room");
        let mut recording = Recording::default();
        draw_reference_zones(&mut recording, &page);
        assert!(
            recording.is_empty(),
            "a 2 mm band holds neither a 3.5 mm glyph nor a rule: {:?}",
            recording.calls()
        );
    }
}
