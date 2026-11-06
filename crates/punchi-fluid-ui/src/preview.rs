// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Catalog sheet: every definition drawn large, for the conformance review.
//!
//! Opened with `PUNCHI_FLUID_PREVIEW=1`. It is not a reader view and not a second
//! editor: it exists so a reviewer — and a screenshot — can see all symbols of the
//! catalog at a size where a numeral, a spring end or a blocking mark is visible.

use egui::{Align2, FontId, Pos2, Rect, Ui, Vec2};
use punchi_fluid_catalog::{Catalog, ComponentDefinition};
use punchi_fluid_draw::{Pt, Rect as MmRect};
use punchi_fluid_theme::{Theme, Token};

use crate::surface::{self, EguiSurface};

/// Side of one cell of the sheet, in points.
pub const CELL: Vec2 = Vec2::new(280.0, 190.0);
/// Margin between the sheet edge and the first cell.
pub const MARGIN: f32 = 20.0;
/// Columns in the sheet.
pub const COLUMNS: usize = 2;

/// Rectangle of the cell that holds the definition at `index`, in sheet coordinates
/// measured from the top-left of the drawing area.
#[must_use]
pub fn cell_rect(index: usize) -> Rect {
    let column = index % COLUMNS;
    let row = index / COLUMNS;
    #[allow(clippy::cast_precision_loss)]
    let origin = Pos2::new(
        MARGIN + column as f32 * CELL.x,
        MARGIN + row as f32 * CELL.y,
    );
    Rect::from_min_size(origin, CELL)
}

/// Draws every definition of the catalog, in identifier order.
pub fn show(ui: &mut Ui, theme: &Theme, catalog: &Catalog) {
    let origin = ui.available_rect_before_wrap().min;
    let painter = ui.painter();
    let ink = surface::ink(theme);

    for (index, definition) in catalog.iter().enumerate() {
        let cell = cell_rect(index).translate(origin.to_vec2());
        painter.rect_stroke(
            cell,
            theme.metrics.radius,
            egui::Stroke::new(theme.metrics.stroke, theme.colour(Token::Outline)),
            egui::StrokeKind::Inside,
        );
        // A symbol box is square: a circle and a triangle only keep their
        // proportions inside one, so the review sheet gives every symbol one.
        let drawing = Rect::from_center_size(
            Pos2::new(cell.center().x, cell.top() + cell.height() * 0.42),
            Vec2::splat(cell.height() * 0.52),
        );
        draw_definition(painter, drawing, definition, &ink, theme);
        painter.text(
            Pos2::new(
                cell.center().x,
                cell.bottom() - theme.metrics.grid_unit * 1.6,
            ),
            Align2::CENTER_TOP,
            &definition.name,
            FontId::proportional(theme.font_size),
            theme.colour(Token::Text),
        );
        painter.text(
            Pos2::new(
                cell.center().x,
                cell.bottom() - theme.metrics.grid_unit * 0.7,
            ),
            Align2::CENTER_TOP,
            &definition.id,
            FontId::proportional(theme.font_size * 0.75),
            theme.colour(Token::TextMuted),
        );
    }
}

fn draw_definition(
    painter: &egui::Painter,
    rect: Rect,
    definition: &ComponentDefinition,
    ink: &punchi_fluid_draw::Ink,
    theme: &Theme,
) {
    let active = match &definition.behaviour {
        punchi_fluid_catalog::Behaviour::Valve {
            default_position, ..
        } => *default_position,
        _ => 0,
    };
    let mut surface = EguiSurface::new(painter, rect.min);
    let area = MmRect::from_min_size(Pt::new(0.0, 0.0), rect.width(), rect.height());
    // Half stroke: the review needs to see the piston, the rod and the spring at a
    // position that is neither end.
    punchi_fluid_draw::draw_component(&mut surface, area, definition, active, 0.5, ink);
    let _ = theme;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_share_edges_and_move_right_then_down() {
        let first = cell_rect(0);
        let second = cell_rect(1);
        let third = cell_rect(2);
        assert!((first.min.y - second.min.y).abs() < f32::EPSILON);
        assert!(second.min.x > first.min.x);
        assert!(third.min.y > first.min.y);
        assert!((third.min.x - first.min.x).abs() < f32::EPSILON);
    }

    #[test]
    fn every_cell_is_the_same_size() {
        for index in 0..8 {
            let cell = cell_rect(index);
            assert!((cell.width() - CELL.x).abs() < f32::EPSILON);
            assert!((cell.height() - CELL.y).abs() < f32::EPSILON);
        }
    }
}
