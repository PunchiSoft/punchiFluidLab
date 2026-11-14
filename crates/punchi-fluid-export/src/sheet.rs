// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet around the drawing: frame, identification block and legend.
//!
//! Everything a printed diagram needs to be readable away from the program that made
//! it: which sheet this is, who drew it, when, and what the symbols mean.

use punchi_fluid_draw::{Anchor, Colour, LineClass, Pt, Stroke, Surface};

use crate::page::{Page, TitleBlock};

/// Width of the frame and the block rules, in millimetres.
const RULE: f32 = 0.35;
/// Width of the frame line and the centring marks, in millimetres (ISO 5457).
const FRAME: f32 = 0.7;
/// Cap height of a field label.
const LABEL: f32 = 2.2;
/// Cap height of a field value.
const VALUE: f32 = 3.0;
/// Draws the frame, the centring marks, the title block and the legend of the page.
pub fn draw(surface: &mut dyn Surface, page: &Page, block: &TitleBlock) {
    let content = page.content_rect();
    surface.rect(content, 0.0, None, Some(Stroke::new(FRAME, Colour::BLACK)));
    draw_centring_marks(surface, page);

    let title = page.title_rect();
    surface.rect(title, 0.0, None, Some(Stroke::new(RULE, Colour::BLACK)));

    let row = title.height() / 4.0;
    for index in 1..4 {
        #[allow(clippy::cast_precision_loss)]
        let y = title.min.y + row * index as f32;
        surface.line(
            &[Pt::new(title.min.x, y), Pt::new(title.max.x, y)],
            Stroke::new(RULE * 0.6, Colour::new(90, 90, 90)),
        );
    }

    let left = title.min.x + 2.0;
    let field = |surface: &mut dyn Surface, index: usize, label: &str, value: &str| {
        #[allow(clippy::cast_precision_loss)]
        let top = title.min.y + row * index as f32;
        surface.text(
            Pt::new(left, top + row * 0.30),
            label,
            LABEL,
            Colour::new(90, 90, 90),
            Anchor::Start,
        );
        surface.text(
            Pt::new(left, top + row * 0.72),
            value,
            VALUE,
            Colour::BLACK,
            Anchor::Start,
        );
    };

    field(surface, 0, "Título", &block.title);
    field(surface, 1, "Proyecto / circuito", &block.project);
    field(
        surface,
        2,
        "Revisión / fecha",
        &format!("{} · {}", block.revision, block.date),
    );
    // A diagram is not to scale: the sheet count is the only page field here. The export
    // may fit a large drawing onto the paper, but that fit is not a drawing scale and is
    // not written in the block.
    let sheet = format!("1/1 · {}", page.paper.label());
    field(surface, 3, "Hoja", &sheet);

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
