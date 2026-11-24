// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Creating a connection from a port drag, including its preview and document edit.

use eframe::egui;
use punchi_fluid_catalog::PortRole;
use punchi_fluid_core::DEFAULT_TUBE_HUNDREDTHS;
use punchi_fluid_doc::Endpoint;
use punchi_fluid_draw::Pt;
use punchi_fluid_draw::placed::port_point;

use crate::Window;

/// One endpoint stays on its port while the other follows the pointer.
#[derive(Debug)]
pub(crate) struct ConnectionDrag {
    pub(crate) from: Endpoint,
    pub(crate) start: Pt,
    pub(crate) pointer: Pt,
}

/// Draws the temporary orthogonal route over the sheet in the theme's accent.
pub(crate) fn draw_preview(
    painter: &egui::Painter,
    drag: &ConnectionDrag,
    placement: &crate::view::Placement,
    page_min: egui::Vec2,
    tokens: &crate::theme::Tokens,
) {
    let preview = punchi_fluid_draw::route(drag.start, drag.pointer);
    let screen = |point: Pt| {
        placement.origin
            + egui::vec2(
                (point.x - page_min.x) * placement.scale,
                (point.y - page_min.y) * placement.scale,
            )
    };
    for segment in preview.windows(2) {
        if let [from, to] = segment {
            painter.line_segment(
                [screen(*from), screen(*to)],
                egui::Stroke::new(tokens.unit * 0.25, tokens.accent),
            );
        }
    }
    painter.circle_stroke(
        screen(drag.start),
        tokens.unit * 0.8,
        egui::Stroke::new(tokens.unit * 0.2, tokens.accent),
    );
}

impl Window {
    /// Handles starting, updating, cancelling and completing a port-to-port drag.
    pub(crate) fn handle_connection_gesture(
        &mut self,
        response: &egui::Response,
        to_mm: &impl Fn(egui::Pos2) -> Pt,
    ) -> bool {
        if response.drag_started_by(egui::PointerButton::Primary)
            && !self.running
            && !response.ctx.input(|input| input.modifiers.shift)
        {
            let press = response.ctx.input(|input| input.pointer.press_origin());
            if let Some((from, start)) = press.and_then(|screen| self.port_at(to_mm(screen))) {
                self.release_pressed_valve();
                self.connection_drag = Some(ConnectionDrag {
                    from,
                    start,
                    pointer: start,
                });
            }
        }
        if self.connection_drag.is_none() {
            return false;
        }
        if response
            .ctx
            .input(|input| input.key_pressed(egui::Key::Escape))
        {
            self.connection_drag = None;
            return true;
        }
        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(position) = response.interact_pointer_pos() {
                if let Some(drag) = self.connection_drag.as_mut() {
                    drag.pointer = to_mm(position);
                }
            }
        }
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            let drop = response.interact_pointer_pos().map(to_mm);
            if let (Some(drag), Some(point)) = (self.connection_drag.take(), drop) {
                self.finish_connection_drag(drag, point);
            } else {
                self.connection_drag = None;
            }
        }
        true
    }

    /// The nearest port of the active circuit within the grid's connection magnet.
    pub(crate) fn port_at(&self, point: Pt) -> Option<(Endpoint, Pt)> {
        let slack = f32::from(self.grid.step_mm) * 0.5 + 1.0;
        let active = self.document.active_circuit();
        let mut nearest: Option<(f32, Endpoint, Pt)> = None;
        for placement in self
            .document
            .placements()
            .iter()
            .filter(|placement| placement.circuit == active)
        {
            let Some(definition) = self.catalog.get(&placement.catalog_id) else {
                continue;
            };
            for port in &definition.port {
                let Some(at) = port_point(
                    placement.rect(),
                    placement.orientation,
                    definition,
                    &port.id,
                ) else {
                    continue;
                };
                let (dx, dy) = point.to(at);
                let distance = dx.hypot(dy);
                if distance <= slack
                    && nearest
                        .as_ref()
                        .is_none_or(|(closest, _, _)| distance < *closest)
                {
                    nearest = Some((distance, Endpoint::new(placement.id(), &port.id), at));
                }
            }
        }
        nearest.map(|(_, endpoint, at)| (endpoint, at))
    }

    /// Completes a tubing gesture on a port, or reports a refusal from the document.
    pub(crate) fn finish_connection_drag(&mut self, drag: ConnectionDrag, point: Pt) {
        let Some((to, _)) = self.port_at(point) else {
            return;
        };
        let tube = if self.port_role(&drag.from) == Some(PortRole::Electrical)
            && self.port_role(&to) == Some(PortRole::Electrical)
        {
            None
        } else {
            Some(DEFAULT_TUBE_HUNDREDTHS)
        };
        if self.document.connect(drag.from, to, tube) {
            self.selected.clear();
            self.selected_pipe = self.document.connections().len().checked_sub(1);
            self.notice = Some(self.language.text("notice.connection_created").to_owned());
        } else {
            self.notice = Some(self.language.text("notice.connection_refused").to_owned());
        }
    }

    /// The role of an endpoint, when its placement and port still exist.
    fn port_role(&self, endpoint: &Endpoint) -> Option<PortRole> {
        let index = self.document.index_of(endpoint.placement)?;
        let placement = self.document.placements().get(index)?;
        self.catalog
            .get(&placement.catalog_id)?
            .port
            .iter()
            .find(|port| port.id == endpoint.port)
            .map(|port| port.role)
    }
}
