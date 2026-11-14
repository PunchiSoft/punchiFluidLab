// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The control-diagram block of the printed sheet.
//!
//! The rows come from [`crate::block_layout`], the same layout the ABI hands to the canvas.
//! This module only draws them: 0.7 mm outer border, 0.35 mm rules, 5 mm values for the
//! company and the title, 3.5 mm for the rest and 2.5 mm labels.

use punchi_fluid_core::TitleField;
use punchi_fluid_draw::{Anchor, Colour, Pt, Stroke, Surface};

use crate::block_layout::{self, BlockField};
use crate::page::{Page, TitleBlock};
use crate::sheet::{FRAME, HEADING, LABEL, MUTED, RULE, VALUE};

/// Draws the block at `page.title_rect()`, in the rows its content model resolved.
pub(crate) fn draw_title_block(surface: &mut dyn Surface, page: &Page, block: &TitleBlock) {
    let title = page.title_rect();
    if title.is_empty() {
        return;
    }
    let layout = block_layout::layout(block, page.paper);
    if layout.height <= 0.0 || layout.width <= 0.0 {
        return;
    }
    surface.rect(title, 0.0, None, Some(Stroke::new(FRAME, Colour::BLACK)));

    let x_scale = title.width() / layout.width;
    let y_scale = title.height() / layout.height;
    let mut y = title.min.y;
    for (row_index, row) in layout.rows.iter().enumerate() {
        let height = row.height * y_scale;
        if row_index > 0 {
            surface.line(
                &[Pt::new(title.min.x, y), Pt::new(title.max.x, y)],
                Stroke::new(RULE, Colour::BLACK),
            );
        }
        let mut x = title.min.x;
        for (cell_index, cell) in row.cells.iter().enumerate() {
            let width = cell.width * x_scale;
            if cell_index > 0 {
                surface.line(
                    &[Pt::new(x, y), Pt::new(x, y + height)],
                    Stroke::new(RULE, Colour::BLACK),
                );
            }
            draw_field(
                surface,
                x,
                y,
                width,
                height,
                cell.field,
                value(block, cell.field),
            );
            x += width;
        }
        y += height;
    }
}

/// The Spanish label of one cell, shared with the canvas by the field index.
fn label(field: BlockField) -> &'static str {
    match field {
        BlockField::Stored(TitleField::Owner) => "Empresa / propietario",
        BlockField::Stored(TitleField::Project) => "Proyecto / instalación",
        BlockField::Stored(TitleField::Equipment) => "Equipo / sistema",
        BlockField::Stored(TitleField::Title) => "Título",
        BlockField::Stored(TitleField::Number) => "Nº de plano",
        BlockField::Stored(TitleField::Revision) => "Revisión",
        BlockField::Stored(TitleField::Date) => "Fecha",
        BlockField::Stored(TitleField::Prepared) => "Dibujado",
        BlockField::Stored(TitleField::Checked) => "Revisado",
        BlockField::Stored(TitleField::Approved) => "Aprobado",
        BlockField::Stored(TitleField::Scale) => "Escala",
        BlockField::Stored(TitleField::Norms) => "Normas aplicadas",
        BlockField::Sheet => "Hoja",
    }
}

/// The value of one cell.
fn value(block: &TitleBlock, field: BlockField) -> &str {
    match field {
        BlockField::Stored(field) => block.field(field),
        BlockField::Sheet => &block.sheet,
    }
}

/// One cell: a small label over the value, both clipped to the cell so a long document
/// number cannot run into its neighbour.
fn draw_field(
    surface: &mut dyn Surface,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    field: BlockField,
    value: &str,
) {
    let pad = 1.6;
    let top_pad = 0.6;
    let bottom_pad = 0.4;
    // Label first, then the value below it. Both baselines are measured from the cell top,
    // never from a fraction of a row designed for 65 mm, and the value shrinks one step
    // when the scaled row cannot hold its preferred height.
    let label_baseline = top + top_pad + LABEL;
    surface.text(
        Pt::new(left + pad, label_baseline),
        label(field),
        LABEL,
        MUTED,
        Anchor::Start,
    );
    if value.is_empty() {
        return;
    }
    let value_top = label_baseline + 0.4;
    let preferred = if field.is_heading() { HEADING } else { VALUE };
    let available = top + height - bottom_pad - value_top;
    let value_size = preferred.min(available);
    if value_size < 2.0 {
        return;
    }
    let room = width - pad * 2.0;
    surface.text(
        Pt::new(left + pad, value_top + value_size),
        &elide(value, value_size, room),
        value_size,
        Colour::BLACK,
        Anchor::Start,
    );
}

/// Cuts `text` to what fits `room` millimetres at `size`, with a plain `...`.
///
/// The writer has no glyph metrics; 0.6 of the cap height per character is a conservative
/// estimate for the technical lettering of ISO 3098, so a value never crosses its cell rule.
fn elide(text: &str, size: f32, room: f32) -> String {
    let per_character = size * 0.6;
    if !per_character.is_finite() || per_character <= 0.0 || room <= 0.0 {
        return String::new();
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let max = (room / per_character).floor() as usize;
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let keep = max.saturating_sub(3);
    let mut out: String = text.chars().take(keep).collect();
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::{PaperSize, TitleBlock};
    use punchi_fluid_core::ContentModel;
    use punchi_fluid_draw::{Call, Recording};

    #[test]
    fn every_text_baseline_and_cap_stays_inside_its_cell() {
        for paper in [PaperSize::A4, PaperSize::A3] {
            for model in ContentModel::ALL {
                let mut block = TitleBlock::default();
                block.set_model(model);
                block.title = String::from("Circuito 1");
                block.number = String::from("PF-2026-07");
                block.norms = String::from("NCh 14 · NCh 1768/1");
                let page = Page {
                    paper,
                    ..Page::a4(block.clone())
                };
                let rect = page.title_rect();
                let layout = block_layout::layout(&block, paper);
                let mut recording = Recording::default();
                draw_title_block(&mut recording, &page, &block);

                let mut y = rect.min.y;
                for row in &layout.rows {
                    let height = row.height * rect.height() / layout.height;
                    let mut x = rect.min.x;
                    for cell in &row.cells {
                        let width = cell.width * rect.width() / layout.width;
                        for call in recording.calls() {
                            let Call::Text { at, size, .. } = call else {
                                continue;
                            };
                            let inside_x = at.x >= x - 0.01 && at.x <= x + width + 0.01;
                            let inside_y = at.y >= y - 0.01 && at.y <= y + height + 0.01;
                            if !inside_x || !inside_y {
                                continue;
                            }
                            assert!(
                                at.y - size >= y - 0.01,
                                "{} {model:?}: a text starts above its cell: {at:?} size {size}",
                                paper.label()
                            );
                            assert!(
                                at.y <= y + height + 0.01,
                                "{} {model:?}: a text baseline crosses the bottom rule: {at:?} at {y}..{}",
                                paper.label(),
                                y + height
                            );
                        }
                        x += width;
                    }
                    y += height;
                }
            }
        }
    }

    #[test]
    fn the_a4_title_value_keeps_its_height_without_overlapping_the_next_row() {
        let mut block = TitleBlock::default();
        block.set_model(ContentModel::Industrial);
        block.title = String::from("Circuito 1");
        let page = Page::a4(block.clone());
        assert!((page.title_rect().height() - 45.0).abs() < 0.01);
        let layout = block_layout::layout(&block, PaperSize::A4);
        let title_row = &layout.rows[1];
        assert!(
            title_row.height >= 9.0,
            "the title row has room for 5 mm text"
        );
    }
}
