// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Selecting a hose: the route it draws, the click that finds it and the accent that marks
//! it.
//!
//! The window's selection is the placements' (`Vec<usize>`) and, separately, one hose, and
//! the two are never selected together: `Supr` has to know which one it removes. The route
//! is rebuilt here from the document and the catalog with the same router the sheet draws
//! with, so a click lands on the line the reader sees and not on an older shape.

use eframe::egui;
use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::{Document, Placement};
use punchi_fluid_draw::placed::{Placed, port_edge_towards, port_point};
use punchi_fluid_draw::{Pt, Rect, route_between_boxes_avoiding};

use crate::Window;
use crate::theme::Tokens;
use crate::view::Placement as ViewPlacement;

/// How far from the drawn route a click still takes the hose, in millimetres: about three
/// pixels at the fit, wide enough for the hand and narrow enough not to steal the sheet.
const PIPE_SLACK_MM: f32 = 1.5;

/// The polyline of one pipe, in millimetres of the sheet.
///
/// `None` when the document or the catalog cannot name both ends: a half-read file draws
/// nothing and cannot be clicked either.
#[must_use]
pub fn route_of(document: &Document, catalog: &Catalog, index: usize) -> Option<Vec<Pt>> {
    let connection = document.connections().get(index)?;
    let from_index = document.index_of(connection.from.placement)?;
    let to_index = document.index_of(connection.to.placement)?;
    let from = document.placements().get(from_index)?;
    let to = document.placements().get(to_index)?;
    let from_definition = catalog.get(&from.catalog_id)?;
    let to_definition = catalog.get(&to.catalog_id)?;
    let start = port_point(
        from.rect(),
        from.orientation,
        from_definition,
        &connection.from.port,
    )?;
    let end = port_point(
        to.rect(),
        to.orientation,
        to_definition,
        &connection.to.port,
    )?;
    let boxes: Vec<Rect> = document.placements().iter().map(Placement::rect).collect();
    Some(route_between_boxes_avoiding(
        start,
        port_edge_towards(
            from.rect(),
            from.orientation,
            from_definition,
            &connection.from.port,
            Some(end),
        ),
        from.rect(),
        end,
        port_edge_towards(
            to.rect(),
            to.orientation,
            to_definition,
            &connection.to.port,
            Some(start),
        ),
        to.rect(),
        &boxes,
        [from_index, to_index],
        connection.offset_mm(),
    ))
}

/// The screen rectangle of a placement's selection envelope: the symbol box in sheet
/// millimetres mapped to the canvas, expanded so the envelope never covers the symbol.
pub(crate) fn envelope_rect(
    rect: Rect,
    page_min: egui::Vec2,
    placement: &ViewPlacement,
    expansion: f32,
) -> egui::Rect {
    let min = placement.origin + (egui::vec2(rect.min.x, rect.min.y) - page_min) * placement.scale;
    let size = egui::vec2(rect.max.x - rect.min.x, rect.max.y - rect.min.y) * placement.scale;
    egui::Rect::from_min_size(min, size).expand(expansion)
}

/// Draws the exterior envelope and its four corner handles around every selected placement.
/// The symbol's own lines keep the normative ink; selection is a view layer over the sheet.
pub(crate) fn draw_component_envelopes(
    painter: &egui::Painter,
    placed: &[Placed<'_>],
    selected: &[usize],
    placement: &ViewPlacement,
    page_min: egui::Vec2,
    tokens: &Tokens,
) {
    let expansion = tokens.unit * 0.15;
    let handle = tokens.unit * 0.18;
    for component in placed
        .iter()
        .filter(|placed| selected.contains(&placed.index))
    {
        let envelope = envelope_rect(component.rect, page_min, placement, expansion);
        painter.rect_stroke(
            envelope,
            tokens.radius_small,
            egui::Stroke::new(tokens.focus_width, tokens.accent),
            egui::StrokeKind::Inside,
        );
        for corner in [
            envelope.left_top(),
            envelope.right_top(),
            envelope.left_bottom(),
            envelope.right_bottom(),
        ] {
            let handle = egui::Rect::from_center_size(corner, egui::vec2(handle, handle));
            painter.rect_filled(handle, tokens.radius_small, tokens.paper);
            painter.rect_stroke(
                handle,
                tokens.radius_small,
                egui::Stroke::new(1.0_f32, tokens.accent),
                egui::StrokeKind::Inside,
            );
        }
    }
}

impl Window {
    /// Draws the selection layer over the sheet: the exterior envelopes of the selected
    /// placements, while the symbol's own lines keep the normative ink.
    pub(crate) fn draw_selection_overlay(
        &self,
        painter: &egui::Painter,
        placed: &[Placed<'_>],
        placement: &ViewPlacement,
        page_min: egui::Vec2,
        tokens: &Tokens,
    ) {
        draw_component_envelopes(painter, placed, &self.selected, placement, page_min, tokens);
    }
}

/// The pipe under `point` in the active circuit, the nearest one when two routes overlap.
#[must_use]
pub fn pipe_at(document: &Document, catalog: &Catalog, point: Pt) -> Option<usize> {
    let active = document.active_circuit();
    let mut best: Option<(f32, usize)> = None;
    for (index, connection) in document.connections().iter().enumerate() {
        if connection.circuit != active {
            continue;
        }
        let Some(route) = route_of(document, catalog, index) else {
            continue;
        };
        let distance = distance_to_path(point, &route);
        if distance <= PIPE_SLACK_MM && best.is_none_or(|(closest, _)| distance < closest) {
            best = Some((distance, index));
        }
    }
    best.map(|(_, index)| index)
}

/// The distance from `point` to the nearest segment of a polyline, in millimetres.
#[must_use]
pub fn distance_to_path(point: Pt, path: &[Pt]) -> f32 {
    path.windows(2)
        .map(|segment| match segment {
            [start, end] => distance_to_segment(point, *start, *end),
            _ => f32::INFINITY,
        })
        .fold(f32::INFINITY, f32::min)
}

/// The distance from `point` to one segment, in millimetres.
fn distance_to_segment(point: Pt, start: Pt, end: Pt) -> f32 {
    let (dx, dy) = start.to(end);
    let length_squared = dx * dx + dy * dy;
    if length_squared <= f32::EPSILON {
        return point.to(start).0.hypot(point.to(start).1);
    }
    let (px, py) = start.to(point);
    let along = ((px * dx + py * dy) / length_squared).clamp(0.0, 1.0);
    (px - along * dx).hypot(py - along * dy)
}

impl Window {
    /// Makes the pipe under `point` the selection, or clears it when there is no hose there.
    /// A hose and components are never selected together.
    pub(crate) fn select_pipe_at(&mut self, point: Pt) {
        self.selected_pipe = pipe_at(&self.document, &self.catalog, point);
        if self.selected_pipe.is_some() {
            self.selected.clear();
            self.sync_selection_notice();
        }
    }

    /// Removes the selected hose, as one undo, when it is the selection. `false` when the
    /// selection is not a hose, so the components' own removal carries on.
    pub(crate) fn remove_selected_pipe(&mut self) -> bool {
        let Some(index) = self.selected_pipe else {
            return false;
        };
        self.selected_pipe = None;
        if !self.document.disconnect(index) {
            return false;
        }
        self.notice = Some(self.language.text("notice.pipe_deleted").to_owned());
        true
    }

    /// The status line's note for what the sheet has selected.
    pub(crate) fn sync_selection_notice(&mut self) {
        self.notice = if self.selected_pipe.is_some() && self.selected.is_empty() {
            Some(self.language.text("selection.pipe").to_owned())
        } else {
            match self.selected.len() {
                0 => None,
                1 => Some(self.language.text("selection.one").to_owned()),
                count => {
                    let count = count.to_string();
                    Some(self.language.format("selection.many", &[("count", &count)]))
                }
            }
        };
    }

    /// Marks the selected hose with the accent, as the component's own lines are marked: a
    /// second drawing would say nothing the line does not.
    pub(crate) fn highlight_selected_pipe(&self, ui: &egui::Ui, origin: egui::Pos2, scale: f32) {
        let Some(index) = self.selected_pipe else {
            return;
        };
        let Some(route) = route_of(&self.document, &self.catalog, index) else {
            return;
        };
        if route.len() < 2 {
            return;
        }
        let points: Vec<egui::Pos2> = route
            .iter()
            .map(|point| origin + egui::vec2(point.x * scale, point.y * scale))
            .collect();
        let width = (self.theme.tokens.unit * 0.1 * scale).max(2.0);
        let stroke = egui::Stroke::new(width, self.theme.tokens.accent);
        let painter = ui.painter().with_clip_rect(ui.max_rect());
        painter.add(egui::Shape::line(points, stroke));
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use punchi_fluid_doc::{Document, Endpoint, Placement};
    use punchi_fluid_draw::Pt;

    use crate::Window;
    use crate::theme::Theme;

    use super::{distance_to_path, envelope_rect, pipe_at, route_of};

    /// A compressor piped to a valve, and the index of the pipe.
    fn circuit() -> (Document, usize) {
        let mut document = Document::new();
        let supply = document.place(Placement::new(
            "supply.compressor",
            Pt::new(50.0, 50.0),
            24.0,
            24.0,
        ));
        let valve = document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(140.0, 50.0),
            24.0,
            24.0,
        ));
        let from = document.placements()[supply].id();
        let to = document.placements()[valve].id();
        assert!(document.connect(Endpoint::new(from, "1"), Endpoint::new(to, "1"), Some(1000)));
        (document, 0)
    }

    fn window() -> Window {
        Window::new(Theme::fallback(), Document::new(), None, None)
    }

    #[test]
    fn a_pipe_has_a_route_between_its_two_ports() {
        let (document, index) = circuit();
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let route = route_of(&document, &catalog, index).expect("the pipe routes");
        assert!(route.len() >= 2, "a hose is a polyline: {route:?}");
    }

    #[test]
    fn a_click_on_the_line_takes_the_pipe_and_a_click_away_does_not() {
        let (document, index) = circuit();
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let route = route_of(&document, &catalog, index).expect("the pipe routes");
        let middle = route[route.len() / 2];
        assert_eq!(pipe_at(&document, &catalog, middle), Some(index));
        assert_eq!(
            pipe_at(&document, &catalog, Pt::new(400.0, 300.0)),
            None,
            "the empty sheet is not a hose"
        );
    }

    #[test]
    fn the_distance_to_a_path_is_zero_on_it_and_grows_away_from_it() {
        let path = [Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), Pt::new(10.0, 10.0)];
        assert!(distance_to_path(Pt::new(5.0, 0.0), &path) < 1e-4);
        assert!((distance_to_path(Pt::new(5.0, 3.0), &path) - 3.0).abs() < 1e-4);
        assert!(distance_to_path(Pt::new(13.0, 4.0), &path) >= 3.0);
    }

    #[test]
    fn selecting_a_pipe_replaces_the_components_selection_and_marks_it() {
        let (document, index) = circuit();
        let mut window = window();
        window.document = document;
        window.selected = vec![0, 1];
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let route = route_of(&window.document, &catalog, index).expect("the pipe routes");
        window.select_pipe_at(route[route.len() / 2]);
        assert_eq!(window.selected_pipe, Some(index));
        assert!(window.selected.is_empty(), "a hose wins the selection");
        assert_eq!(window.notice.as_deref(), Some("pipe selected"));
    }

    #[test]
    fn removing_the_selected_pipe_is_one_undo() {
        let (document, index) = circuit();
        let mut window = window();
        window.document = document;
        let catalog = punchi_fluid_catalog::Catalog::builtin();
        let route = route_of(&window.document, &catalog, index).expect("the pipe routes");
        window.select_pipe_at(route[route.len() / 2]);
        assert!(window.remove_selected_pipe(), "the hose is removed");
        assert!(window.document.connections().is_empty());
        assert!(window.document.can_undo(), "one undo entry");
        assert!(window.document.undo(), "the removal is undone");
        assert_eq!(window.document.connections().len(), 1);
    }

    #[test]
    fn the_envelope_sits_outside_the_symbol() {
        let placement = crate::view::Placement {
            scale: 2.0,
            origin: egui::pos2(10.0, 20.0),
        };
        let rect = punchi_fluid_draw::Rect::from_corners(Pt::new(5.0, 5.0), Pt::new(25.0, 15.0));
        let envelope = envelope_rect(rect, egui::vec2(0.0, 0.0), &placement, 3.0);
        assert!((envelope.min.x - 17.0).abs() < 0.01);
        assert!((envelope.min.y - 27.0).abs() < 0.01);
        assert!((envelope.width() - 46.0).abs() < 0.01);
        assert!((envelope.height() - 26.0).abs() < 0.01);
    }
}
