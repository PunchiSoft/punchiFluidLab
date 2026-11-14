// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet around the drawing: frame, identification block and legend.
//!
//! Everything a printed diagram needs to be readable away from the program that made
//! it: which sheet this is, who drew it, when, and what the symbols mean.

use punchi_fluid_draw::{Anchor, Colour, LineClass, Pt, Stroke, Surface};

use crate::page::{Page, TitleBlock};
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
/// Draws the frame, the reference zones, the centring marks, the title block and the legend.
pub fn draw(surface: &mut dyn Surface, page: &Page, block: &TitleBlock) {
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

    draw_legend(surface, page);
    draw_parts(surface, page);
}

/// The optional reference zones of ISO 5457 / `NCh` 13.Of93.
///
/// The drawing space is divided into fields of at most 50 mm: numerals down the left and
/// right bands and letters across the top and bottom, 3.5 mm high with 0.35 mm rules. The
/// rules and the characters stay inside the margin bands, so the drawing space itself
/// carries no extra ink. A3 and larger get them; A4 stays clean (see `Page`).
fn draw_reference_zones(surface: &mut dyn Surface, page: &Page) {
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
    if bands.iter().any(|band| *band <= 0.0) {
        return;
    }

    let columns = field_count(content.width());
    let rows = field_count(content.height());
    #[allow(clippy::cast_precision_loss)]
    let step_x = content.width() / columns as f32;
    #[allow(clippy::cast_precision_loss)]
    let step_y = content.height() / rows as f32;
    let stroke = Stroke::new(RULE, Colour::BLACK);

    // Numerals down the left and right bands: one per row.
    let left_centre = f32::midpoint(paper.min.x, content.min.x);
    let right_centre = f32::midpoint(content.max.x, paper.max.x);
    for row in 0..rows {
        #[allow(clippy::cast_precision_loss)]
        let centre = content.min.y + (row as f32 + 0.5) * step_y;
        let label = (row + 1).to_string();
        let baseline = centre + VALUE * 0.36;
        for x in [left_centre, right_centre] {
            surface.text(Pt::new(x, baseline), &label, VALUE, MUTED, Anchor::Middle);
        }
    }

    // Letters across the top and bottom bands: one per column.
    let top_centre = f32::midpoint(paper.min.y, content.min.y);
    let bottom_centre = f32::midpoint(content.max.y, paper.max.y);
    for column in 0..columns {
        #[allow(clippy::cast_precision_loss)]
        let centre = content.min.x + (column as f32 + 0.5) * step_x;
        let label = column_label(column);
        for y in [top_centre, bottom_centre] {
            surface.text(
                Pt::new(centre, y + VALUE * 0.36),
                &label,
                VALUE,
                MUTED,
                Anchor::Middle,
            );
        }
    }

    // The division rules live only in the bands, never in the drawing space.
    for column in 1..columns {
        #[allow(clippy::cast_precision_loss)]
        let x = content.min.x + column as f32 * step_x;
        surface.line(
            &[Pt::new(x, paper.min.y), Pt::new(x, content.min.y)],
            stroke,
        );
        surface.line(
            &[Pt::new(x, content.max.y), Pt::new(x, paper.max.y)],
            stroke,
        );
    }
    for row in 1..rows {
        #[allow(clippy::cast_precision_loss)]
        let y = content.min.y + row as f32 * step_y;
        surface.line(
            &[Pt::new(paper.min.x, y), Pt::new(content.min.x, y)],
            stroke,
        );
        surface.line(
            &[Pt::new(content.max.x, y), Pt::new(paper.max.x, y)],
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

/// Letter of a reference column: `A`..`Z`, then `AA`, `AB`… for a very large sheet.
fn column_label(mut index: usize) -> String {
    let mut label = String::new();
    loop {
        #[allow(clippy::cast_possible_truncation)]
        let letter = (b'A' + (index % 26) as u8) as char;
        label.insert(0, letter);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    label
}

/// The four centring marks of ISO 5457: at the ends of the symmetry axes of the paper, a
/// line from the frame across the border to the paper edge and a small arrow. The form is
/// free; the line is the recommended part, and spanning the whole border keeps the 20 mm
/// filing margin looking like the other three sides instead of leaving a gap on the left.
fn draw_centring_marks(surface: &mut dyn Surface, page: &Page) {
    let paper = page.rect();
    let content = page.content_rect();
    if content.is_empty() {
        return;
    }
    let cx = f32::midpoint(paper.min.x, paper.max.x);
    let cy = f32::midpoint(paper.min.y, paper.max.y);
    let stroke = Stroke::new(FRAME, Colour::BLACK);
    let mut mark = |from: Pt, to: Pt, tip: Pt, base: [Pt; 2]| {
        let (dx, dy) = from.to(to);
        if dx.hypot(dy) < 0.5 {
            return;
        }
        surface.line(&[from, to], stroke);
        surface.polygon(&[tip, base[0], base[1]], Some(Colour::BLACK), None);
    };
    if content.min.y > paper.min.y + 0.5 {
        let tip = Pt::new(cx, (paper.min.y + 3.0).min(content.min.y));
        mark(
            Pt::new(cx, content.min.y),
            Pt::new(cx, paper.min.y),
            tip,
            [
                Pt::new(cx - 1.5, paper.min.y),
                Pt::new(cx + 1.5, paper.min.y),
            ],
        );
    }
    if content.max.y < paper.max.y - 0.5 {
        let tip = Pt::new(cx, (paper.max.y - 3.0).max(content.max.y));
        mark(
            Pt::new(cx, content.max.y),
            Pt::new(cx, paper.max.y),
            tip,
            [
                Pt::new(cx - 1.5, paper.max.y),
                Pt::new(cx + 1.5, paper.max.y),
            ],
        );
    }
    if content.min.x > paper.min.x + 0.5 {
        let tip = Pt::new((paper.min.x + 3.0).min(content.min.x), cy);
        mark(
            Pt::new(content.min.x, cy),
            Pt::new(paper.min.x, cy),
            tip,
            [
                Pt::new(paper.min.x, cy - 1.5),
                Pt::new(paper.min.x, cy + 1.5),
            ],
        );
    }
    if content.max.x < paper.max.x - 0.5 {
        let tip = Pt::new((paper.max.x - 3.0).max(content.max.x), cy);
        mark(
            Pt::new(content.max.x, cy),
            Pt::new(paper.max.x, cy),
            tip,
            [
                Pt::new(paper.max.x, cy - 1.5),
                Pt::new(paper.max.x, cy + 1.5),
            ],
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
        "lista de piezas",
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
            &format!("… y {} más", parts.omitted),
            LABEL,
            Colour::new(90, 90, 90),
            Anchor::Start,
        );
    }
}

/// Legend of line classes and port numerals, above the title block.
fn draw_legend(surface: &mut dyn Surface, page: &Page) {
    let content = page.content_rect();
    let title = page.title_rect();
    let base_y = title.min.y - 5.0;
    let mut x = content.min.x + 1.0;

    let sample = |surface: &mut dyn Surface, x: &mut f32, class: LineClass, label: &str| {
        let end = *x + 12.0;
        let stroke = Stroke::new(RULE, Colour::BLACK).with_class(class);
        if let Some((dash, gap)) = class.pattern() {
            // Draw the sample with the same geometry the drawing uses, so the legend
            // cannot promise a pattern the diagram does not use.
            let mut position = 0.0_f32;
            while position < 12.0 {
                let from = position;
                let to = (position + dash).min(12.0);
                surface.line(
                    &[Pt::new(*x + from, base_y), Pt::new(*x + to, base_y)],
                    stroke,
                );
                position += dash + gap;
            }
        } else {
            surface.line(&[Pt::new(*x, base_y), Pt::new(end, base_y)], stroke);
        }
        surface.text(
            Pt::new(end + 2.0, base_y + LABEL * 0.36),
            label,
            LABEL,
            Colour::BLACK,
            Anchor::Start,
        );
        #[allow(clippy::cast_precision_loss)]
        let width = label.len() as f32 * LABEL * 0.55;
        *x = end + 2.0 + width + 6.0;
    };

    sample(surface, &mut x, LineClass::Working, "working");
    sample(surface, &mut x, LineClass::Pilot, "pilot");
    sample(surface, &mut x, LineClass::Drain, "drain");

    surface.text(
        Pt::new(content.min.x + 1.0, base_y - LABEL * 1.8),
        "ports: 1 supply · 2 and 4 working · 3 and 5 exhaust · 12 and 14 pilot",
        LABEL,
        Colour::BLACK,
        Anchor::Start,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::PaperSize;
    use punchi_fluid_draw::Recording;

    #[test]
    fn the_sheet_draws_frame_block_and_legend() {
        let mut recording = Recording::default();
        let block = TitleBlock::new("A circuit", "2026-11-06");
        draw(&mut recording, &Page::a4(block.clone()), &block);
        assert!(recording.len() > 10);

        let texts: Vec<&str> = recording
            .calls()
            .iter()
            .filter_map(|call| match call {
                punchi_fluid_draw::Call::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"Título"));
        assert!(texts.iter().any(|text| text.contains("1 supply")));
        assert!(texts.contains(&"working"));
        // The four centring marks are the only filled triangles on the sheet.
        let triangles = recording
            .calls()
            .iter()
            .filter(|call| matches!(call, punchi_fluid_draw::Call::Polygon { .. }))
            .count();
        assert_eq!(triangles, 4, "one centring arrow on each side");
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
        ));
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
        draw(&mut recording, &Page::a4(block.clone()), &block);
        let texts: Vec<String> = recording
            .calls()
            .iter()
            .filter_map(|call| match call {
                punchi_fluid_draw::Call::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        for expected in [
            "Empresa / propietario",
            "Proyecto / instalación",
            "Equipo / sistema",
            "Título",
            "Nº de plano",
            "Revisión",
            "Hoja",
            "Fecha",
            "Escala",
            "Dibujado",
            "Revisado",
            "Aprobado",
            "Normas aplicadas",
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
    fn a3_gets_the_reference_zones_and_a4_stays_clean() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        let zones = |paper, orientation| {
            let page = Page {
                paper,
                orientation,
                ..Page::a4(block.clone())
            };
            let content = page.content_rect();
            let mut recording = Recording::default();
            draw(&mut recording, &page, &block);
            // A reference character is 3.5 mm, in muted ink, inside a margin band. The
            // block's own revision `A` is black and inside the drawing space, so size and
            // position tell the two apart.
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
                    } if (*size - 3.5).abs() < 0.01
                        && *colour == Colour::new(90, 90, 90)
                        && (at.x < content.min.x - 0.01
                            || at.x > content.max.x + 0.01
                            || at.y < content.min.y - 0.01
                            || at.y > content.max.y + 0.01) =>
                    {
                        Some(text.clone())
                    }
                    _ => None,
                })
                .collect::<Vec<String>>()
        };

        let a4 = zones(PaperSize::A4, punchi_fluid_core::Orientation::Portrait);
        assert!(a4.is_empty(), "A4 carries no reference zones: {a4:?}");

        let a3 = zones(PaperSize::A3, punchi_fluid_core::Orientation::Landscape);
        for expected in ["A", "B", "1", "2"] {
            assert!(
                a3.iter().any(|text| text == expected),
                "A3 is missing the reference character {expected}: {a3:?}"
            );
        }
    }

    #[test]
    fn reference_rules_and_characters_never_enter_the_drawing_space() {
        let block = TitleBlock::new("Un circuito", "2026-11-14");
        let page = Page {
            paper: PaperSize::A3,
            orientation: punchi_fluid_core::Orientation::Portrait,
            ..Page::a4(block)
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
