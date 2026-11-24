// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet around the drawing: frame and identification block.
//!
//! Everything a printed diagram needs to be identified away from the program that made
//! it: which sheet this is, who drew it and when.

use punchi_fluid_draw::{Anchor, Colour, Pt, Stroke, Surface};

use crate::labels::Labels;
use crate::page::{Page, TitleBlock};
use crate::reference_zones::draw_reference_zones;
use crate::title_block::draw_title_block;

/// Width of the frame and the block rules, in millimetres.
pub(crate) const RULE: f32 = 0.35;
/// Width of the frame line and the centring marks, in millimetres (ISO 5457).
pub(crate) const FRAME: f32 = 0.7;
/// Cap height of a field label, from ISO 3098 / `NCh` 15.
pub(crate) const LABEL: f32 = 2.5;
/// Cap height of a field value.
pub(crate) const VALUE: f32 = 3.5;
/// Cap height of the title and the company, the two values that lead the block.
pub(crate) const HEADING: f32 = 5.0;
/// The muted ink of a label and of a reference-zone character.
pub(crate) const MUTED: Colour = Colour::new(90, 90, 90);
/// Draws the frame, the reference zones, the centring marks and the title block.
pub fn draw(surface: &mut dyn Surface, page: &Page, block: &TitleBlock) {
    // The paper belongs to the drawing, not to the window: the print surface paints it
    // white, and the window's surface maps that white to the theme's `paper` token. It is
    // the first call so the frame, the block and the circuit sit on top of it.
    surface.rect(page.rect(), 0.0, Some(Colour::WHITE), None);
    let content = page.content_rect();
    surface.rect(content, 0.0, None, Some(Stroke::new(FRAME, Colour::BLACK)));
    draw_reference_zones(surface, page);
    draw_centring_marks(surface, page);
    draw_title_block(surface, page, block);

    if let Some(note) = &block.note {
        // Inside the frame, never in the margin: the margin band belongs to the paper.
        surface.text(
            Pt::new(content.min.x + 2.0, content.min.y + 2.5),
            note,
            LABEL,
            Colour::new(170, 40, 40),
            Anchor::Start,
        );
    }

    draw_parts(surface, page);
}

/// Length of a centring mark beyond the frame, in millimetres (ISO 5457 §4.3).
pub(crate) const MARK_EXTENT: f32 = 10.0;

/// The four centring marks of ISO 5457 §4.3: continuous lines of 0.7 mm, at the ends of the
/// symmetry axes, starting at the frame and extending 10 mm beyond it.
///
/// They are lines and nothing else. The standard fixes their width, their place and their
/// length; an arrowhead would be our own invention drawn over a normative element, and the
/// 20 mm filing edge would grow a longer mark than the other three. On that edge the mark
/// stops 10 mm inside the paper, because the standard measures from the frame.
fn draw_centring_marks(surface: &mut dyn Surface, page: &Page) {
    let paper = page.rect();
    let content = page.content_rect();
    if content.is_empty() {
        return;
    }
    let cx = f32::midpoint(paper.min.x, paper.max.x);
    let cy = f32::midpoint(paper.min.y, paper.max.y);
    let stroke = Stroke::new(FRAME, Colour::BLACK);
    // A mark shorter than half a millimetre means the frame touches the paper: there is
    // nothing to draw, and a zero-length line would only add noise to the recording.
    let mut mark = |from: Pt, to: Pt| {
        let (dx, dy) = from.to(to);
        if dx.hypot(dy) >= 0.5 {
            surface.line(&[from, to], stroke);
        }
    };
    if content.min.y > paper.min.y + 0.5 {
        mark(
            Pt::new(cx, content.min.y),
            Pt::new(cx, (content.min.y - MARK_EXTENT).max(paper.min.y)),
        );
    }
    if content.max.y < paper.max.y - 0.5 {
        mark(
            Pt::new(cx, content.max.y),
            Pt::new(cx, (content.max.y + MARK_EXTENT).min(paper.max.y)),
        );
    }
    if content.min.x > paper.min.x + 0.5 {
        mark(
            Pt::new(content.min.x, cy),
            Pt::new((content.min.x - MARK_EXTENT).max(paper.min.x), cy),
        );
    }
    if content.max.x < paper.max.x - 0.5 {
        mark(
            Pt::new(content.max.x, cy),
            Pt::new((content.max.x + MARK_EXTENT).min(paper.max.x), cy),
        );
    }
}

/// The parts list, in its own column above the block.
///
/// A schematic without it is a drawing a technician cannot build from: the symbols say
/// what kind of valve it is, the list says how many and with which designation. The column
/// shares the block's width so the bottom-right corner reads as one block of information.
fn draw_parts(surface: &mut dyn Surface, page: &Page) {
    let rect = page.parts_rect();
    if rect.is_empty() {
        return;
    }
    let Some(parts) = &page.parts else {
        return;
    };
    surface.rect(rect, 0.0, None, Some(Stroke::new(RULE, Colour::BLACK)));
    surface.text(
        Pt::new(rect.min.x + 2.0, rect.min.y + 3.6),
        page.labels.parts_heading,
        LABEL,
        Colour::new(90, 90, 90),
        Anchor::Start,
    );
    // A rule under the heading, so the rows do not read as one paragraph with the title.
    let rule_y = rect.min.y + 5.0;
    surface.line(
        &[Pt::new(rect.min.x, rule_y), Pt::new(rect.max.x, rule_y)],
        Stroke::new(RULE * 0.6, Colour::new(90, 90, 90)),
    );

    // The quantity column is as wide as three digits; the name takes the rest, and the
    // designations close the row, which is the order a parts list is read in.
    let quantity_x = rect.min.x + 2.0;
    let name_x = quantity_x + 8.0;
    let designation_x = rect.min.x + rect.width() * 0.62;
    let mut y = rule_y + 4.0;
    for row in &parts.rows {
        surface.text(
            Pt::new(quantity_x, y),
            &row.quantity.to_string(),
            VALUE,
            Colour::BLACK,
            Anchor::Start,
        );
        surface.text(
            Pt::new(name_x, y),
            &row.name,
            LABEL,
            Colour::BLACK,
            Anchor::Start,
        );
        if !row.designations.is_empty() {
            surface.text(
                Pt::new(designation_x, y),
                &row.designations,
                LABEL,
                Colour::new(60, 60, 60),
                Anchor::Start,
            );
        }
        y += 4.4;
    }
    if parts.omitted > 0 {
        surface.text(
            Pt::new(name_x, y),
            &Labels::with_count(page.labels.parts_omitted, parts.omitted),
            LABEL,
            Colour::new(90, 90, 90),
            Anchor::Start,
        );
        y += 4.4;
    }
    draw_closure_rows(surface, parts, quantity_x, name_x, &mut y, &page.labels);
    draw_tube_rows(surface, parts, quantity_x, name_x, &mut y, &page.labels);
}

/// Draws the fitted closures and their cut note, from the current vertical position.
fn draw_closure_rows(
    surface: &mut dyn Surface,
    parts: &crate::parts::PartsList,
    quantity_x: f32,
    name_x: f32,
    y: &mut f32,
    labels: &crate::labels::Labels,
) {
    // A closure is a fitted accessory, not a free placement: its row names the closure, the
    // component that carries it and the port, in the same columns as the components.
    for row in &parts.closures {
        surface.text(
            Pt::new(quantity_x, *y),
            &row.quantity.to_string(),
            VALUE,
            Colour::BLACK,
            Anchor::Start,
        );
        surface.text(
            Pt::new(name_x, *y),
            &row.text(),
            LABEL,
            Colour::BLACK,
            Anchor::Start,
        );
        *y += 4.4;
    }
    if parts.omitted_closures > 0 {
        surface.text(
            Pt::new(name_x, *y),
            &crate::labels::Labels::with_count(labels.closures_omitted, parts.omitted_closures),
            LABEL,
            Colour::new(90, 90, 90),
            Anchor::Start,
        );
        *y += 4.4;
    }
}

/// Draws the tube rows and their cut-list note, from the current vertical position.
fn draw_tube_rows(
    surface: &mut dyn Surface,
    parts: &crate::parts::PartsList,
    quantity_x: f32,
    name_x: f32,
    y: &mut f32,
    labels: &crate::labels::Labels,
) {
    // The tubes close the list: one row per size and class, with the length of the route
    // the drawing shows. The note says what that length is, because a reader must not take
    // it for a cut list with a bend allowance it does not have.
    for row in &parts.tubes {
        surface.text(
            Pt::new(quantity_x, *y),
            &row.text(labels),
            LABEL,
            Colour::BLACK,
            Anchor::Start,
        );
        *y += 4.4;
    }
    if !parts.tubes.is_empty() {
        surface.text(
            Pt::new(name_x, *y),
            labels.tube_note,
            LABEL * 0.8,
            MUTED,
            Anchor::Start,
        );
        *y += 3.6;
    }
    if parts.omitted_tubes > 0 {
        surface.text(
            Pt::new(name_x, *y),
            &crate::labels::Labels::with_count(labels.tubes_omitted, parts.omitted_tubes),
            LABEL,
            Colour::new(90, 90, 90),
            Anchor::Start,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use punchi_fluid_draw::Recording;

    #[test]
    fn the_sheet_draws_frame_and_block_without_a_legend() {
        let mut recording = Recording::default();
        let block = TitleBlock::new("A circuit", "2026-11-06");
        let mut page = Page::a4(block.clone());
        page.labels = crate::test_labels::english();
        draw(&mut recording, &page, &block);
        assert!(recording.len() > 10);

        let texts: Vec<&str> = recording
            .calls()
            .iter()
            .filter_map(|call| match call {
                punchi_fluid_draw::Call::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"Title"));
        assert!(
            !texts.iter().any(|text| text.contains("1 supply")),
            "the sheet no longer carries the port legend"
        );
        assert!(
            !texts
                .iter()
                .any(|text| matches!(*text, "working" | "pilot" | "drain"))
        );
        // ISO 5457 §4.3 draws the centring marks as lines: a filled triangle anywhere on
        // the sheet is a mistake, not a decoration.
        let triangles = recording
            .calls()
            .iter()
            .filter(|call| matches!(call, punchi_fluid_draw::Call::Polygon { .. }))
            .count();
        assert_eq!(triangles, 0, "a centring mark is a line, not an arrow");
    }

    #[test]
    fn the_centring_marks_are_ten_millimetre_lines_on_the_symmetry_axes() {
        let mut recording = Recording::default();
        let block = TitleBlock::new("A circuit", "2026-11-06");
        let page = Page::a4(block.clone());
        draw(&mut recording, &page, &block);

        let paper = page.rect();
        let content = page.content_rect();
        let cx = f32::midpoint(paper.min.x, paper.max.x);
        let cy = f32::midpoint(paper.min.y, paper.max.y);
        let expected = [
            (
                Pt::new(cx, content.min.y),
                Pt::new(cx, content.min.y - MARK_EXTENT),
            ),
            (
                Pt::new(cx, content.max.y),
                Pt::new(cx, content.max.y + MARK_EXTENT),
            ),
            // The filing edge is 20 mm wide, so its mark ends 10 mm inside the paper.
            (
                Pt::new(content.min.x, cy),
                Pt::new(paper.min.x + MARK_EXTENT, cy),
            ),
            (Pt::new(content.max.x, cy), Pt::new(paper.max.x, cy)),
        ];

        let lines: Vec<Vec<Pt>> = recording
            .calls()
            .iter()
            .filter_map(|call| match call {
                punchi_fluid_draw::Call::Line { points, stroke }
                    if (stroke.width - FRAME).abs() < f32::EPSILON =>
                {
                    Some(points.clone())
                }
                _ => None,
            })
            .collect();

        for (from, to) in expected {
            let found = lines.iter().any(|points| {
                points.len() == 2
                    && (points[0].x - from.x).abs() < 0.01
                    && (points[0].y - from.y).abs() < 0.01
                    && (points[1].x - to.x).abs() < 0.01
                    && (points[1].y - to.y).abs() < 0.01
            });
            assert!(found, "missing centring line from {from:?} to {to:?}");
        }
    }

    #[test]
    fn the_parts_list_is_drawn_with_its_counts() {
        let mut page = Page::a4(TitleBlock::new("A circuit", "2026-11-12"));
        page.parts = Some(crate::parts::PartsList::group(
            &punchi_fluid_catalog::Catalog::builtin(),
            [
                ("cylinder.single_acting_20_100", Some("1A1")),
                ("cylinder.single_acting_20_100", Some("1A2")),
                ("supply.compressor", None),
            ],
            [],
            [],
            &crate::test_labels::spanish(),
        ));
        page.labels = crate::test_labels::spanish();
        let block = page.title_block.clone().unwrap_or_default();
        let mut recording = Recording::default();
        draw(&mut recording, &page, &block);
        let texts: Vec<String> = recording
            .calls()
            .iter()
            .filter_map(|call| match call {
                punchi_fluid_draw::Call::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert!(texts.iter().any(|text| text == "lista de piezas"));
        assert!(texts.iter().any(|text| text == "2"), "the count of two");
        assert!(texts.iter().any(|text| text.contains("Cilindro")));
        assert!(texts.iter().any(|text| text.contains("1A1, 1A2")));
    }

    #[test]
    fn the_control_block_prints_its_guide_fields_and_marks_the_scale() {
        let mut block = TitleBlock::new("Un circuito", "2026-11-14");
        block.owner = String::from("Curso de Neumática");
        block.project = String::from("Línea 2");
        block.equipment = String::from("Prensa 4");
        block.number = String::from("PF-2026-07");
        block.revision = String::from("B");
        block.prepared = String::from("Ana");
        block.checked = String::from("Luis");
        block.approved = String::from("Marta");
        block.norms = String::from("NCh 14 · NCh 1768/1");
        block.remark = String::from("NOTA QUE NO SE IMPRIME");
        block.sheet = String::from("2/3");
        let mut recording = Recording::default();
        let mut page = Page::a4(block.clone());
        page.labels = crate::test_labels::english();
        draw(&mut recording, &page, &block);
        let texts: Vec<String> = recording
            .calls()
            .iter()
            .filter_map(|call| match call {
                punchi_fluid_draw::Call::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        for expected in [
            "Company / owner",
            "Project / installation",
            "Equipment / system",
            "Title",
            "Drawing no.",
            "Revision",
            "Sheet",
            "Date",
            "Scale",
            "Drawn by",
            "Checked by",
            "Approved by",
            "Standards applied",
            "Curso de Neumática",
            "Línea 2",
            "Prensa 4",
            "PF-2026-07",
            "2/3",
            "Marta",
            "S/E",
            "NCh 14 · NCh 1768/1",
        ] {
            assert!(
                texts.iter().any(|text| text == expected),
                "missing {expected}"
            );
        }
        // The block no longer borrows the mechanical fields a part drawing would need.
        for forbidden in [
            "Material",
            "Unidad",
            "Proyección",
            "Empresa o curso",
            "Nota",
            "NOTA QUE NO SE IMPRIME",
        ] {
            assert!(
                !texts.iter().any(|text| text == forbidden),
                "a schematic has no {forbidden} field"
            );
        }
    }

    #[test]
    fn the_unverified_note_appears_where_it_can_be_read() {
        let block = TitleBlock::new("A circuit", "2026-11-06").with_unverified_note();
        let mut recording = Recording::default();
        draw(&mut recording, &Page::a4(block.clone()), &block);
        let found = recording.calls().iter().any(|call| {
            matches!(call, punchi_fluid_draw::Call::Text { text, .. } if text.contains("not yet verified"))
        });
        assert!(found, "the note must be drawn on the sheet");
    }
}
