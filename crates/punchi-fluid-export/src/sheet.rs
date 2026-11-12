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
/// Cap height of a field label.
const LABEL: f32 = 2.2;
/// Cap height of a field value.
const VALUE: f32 = 3.0;

/// Draws the frame, the title block and the legend of the page.
pub fn draw(surface: &mut dyn Surface, page: &Page, block: &TitleBlock) {
    let content = page.content_rect();
    surface.rect(
        content,
        0.0,
        None,
        Some(Stroke::new(RULE * 1.6, Colour::BLACK)),
    );

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

    field(surface, 0, "title", &block.title);
    field(surface, 1, "project", &block.project);
    field(
        surface,
        2,
        "revision / date",
        &format!("{} · {}", block.revision, block.date),
    );
    let sheet = match &block.scale {
        Some(scale) => format!("1/1 · {} · {scale}", page.paper.label()),
        None => format!("1/1 · {}", page.paper.label()),
    };
    field(surface, 3, "sheet", &sheet);

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
        assert!(texts.contains(&"title"));
        assert!(texts.iter().any(|text| text.contains("1 supply")));
        assert!(texts.contains(&"working"));
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
