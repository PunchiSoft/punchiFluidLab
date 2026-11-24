// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The armed move: **Mover** in the sheet menu carries the selection with the pointer and a
//! click drops it.
//!
//! It is not a second drag. A drag starts on the symbol and lives while the primary button is
//! down; this gesture starts from the menu, needs no press on the symbol, and survives while
//! the pointer moves. It changes the document exactly once, at the drop, so the whole gesture
//! is one undo entry.

use eframe::egui;
use punchi_fluid_draw::Pt;

use crate::Window;
use crate::view;

/// The selection being carried: where it started, the pointer in millimetres, the nudge the
/// arrow keys add and the document revision the gesture belongs to.
#[derive(Debug)]
pub(crate) struct Moving {
    starts: Vec<(usize, Pt)>,
    /// The pointer millimetres when it first entered the sheet; `None` until then, so the
    /// symbol does not jump to a pointer that is still over a menu or another panel.
    anchor: Option<Pt>,
    pointer: Pt,
    nudge: egui::Vec2,
    revision: u64,
}

impl Moving {
    /// The displacement to preview, in millimetres of the sheet.
    #[must_use]
    pub(crate) fn delta(&self) -> egui::Vec2 {
        let follow = self.anchor.map_or(egui::Vec2::ZERO, |anchor| {
            let (dx, dy) = anchor.to(self.pointer);
            egui::vec2(dx, dy)
        });
        follow + self.nudge
    }

    /// The placements the gesture carries and their starting centres.
    #[must_use]
    pub(crate) fn starts(&self) -> &[(usize, Pt)] {
        &self.starts
    }
}

impl Window {
    /// **Mover**: arms the gesture for the current selection. It is view state until the
    /// drop, so arming it does not enter the history.
    pub(crate) fn arm_move(&mut self) {
        let starts: Vec<(usize, Pt)> = self
            .selected
            .iter()
            .filter_map(|index| {
                self.document
                    .placements()
                    .get(*index)
                    .map(|placement| (*index, placement.centre))
            })
            .collect();
        if starts.is_empty() {
            return;
        }
        let revision = self.document.revision();
        self.moving = Some(Moving {
            starts,
            anchor: None,
            pointer: Pt::new(0.0, 0.0),
            nudge: egui::Vec2::ZERO,
            revision,
        });
        self.notice = Some(self.language.text("notice.moving").to_owned());
    }

    /// The pointer's frame while the gesture is armed: follow it, take the arrow nudge, drop
    /// on a click or `Enter`, and cancel on `Esc` or the right button.
    pub(crate) fn handle_moving(
        &mut self,
        response: &egui::Response,
        placement: &view::Placement,
        page_min: egui::Vec2,
    ) {
        let context = &response.ctx;
        let stale = self
            .moving
            .as_ref()
            .is_some_and(|moving| moving.revision != self.document.revision());
        if stale {
            // Another command changed the document underneath the gesture: the preview no
            // longer describes it, so it goes away without a second notice.
            self.moving = None;
            return;
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
            || response.secondary_clicked()
        {
            self.cancel_move();
            return;
        }
        let (left, right, up, down, shift) = context.input(|input| {
            (
                input.key_pressed(egui::Key::ArrowLeft),
                input.key_pressed(egui::Key::ArrowRight),
                input.key_pressed(egui::Key::ArrowUp),
                input.key_pressed(egui::Key::ArrowDown),
                input.modifiers.shift,
            )
        });
        let step = if shift { 10.0 } else { 1.0 };
        let mut nudge = egui::Vec2::ZERO;
        if left {
            nudge.x -= step;
        }
        if right {
            nudge.x += step;
        }
        if up {
            nudge.y -= step;
        }
        if down {
            nudge.y += step;
        }
        // The pointer moves before the drop is read, so a click at a new point commits the
        // point the reader clicked and not the previous frame's.
        {
            let Some(moving) = self.moving.as_mut() else {
                return;
            };
            if response.hovered() {
                if let Some(position) = context.pointer_hover_pos() {
                    let point = Pt::new(
                        page_min.x + (position.x - placement.origin.x) / placement.scale,
                        page_min.y + (position.y - placement.origin.y) / placement.scale,
                    );
                    if moving.anchor.is_none() {
                        moving.anchor = Some(point);
                    }
                    moving.pointer = point;
                }
            }
            if nudge != egui::Vec2::ZERO {
                moving.nudge += nudge;
                // The arrow key moved the preview without a pointer event; ask for the frame
                // that draws it.
                context.request_repaint();
            }
        }
        if context.input(|input| input.key_pressed(egui::Key::Enter))
            || response.clicked()
            || response.drag_stopped_by(egui::PointerButton::Primary)
        {
            self.commit_move();
        }
    }

    /// Drops the carried selection as one undo entry. A drop with no displacement cancels,
    /// so a stray click does not fill the history with nothing.
    pub(crate) fn commit_move(&mut self) {
        let Some(moving) = self.moving.take() else {
            return;
        };
        let delta = moving.delta();
        if delta.length() < 0.5 {
            self.notice = Some(self.language.text("notice.move_cancelled").to_owned());
            return;
        }
        let moves: Vec<(usize, Pt)> = moving
            .starts
            .iter()
            .map(|(index, start)| (*index, Pt::new(start.x + delta.x, start.y + delta.y)))
            .collect();
        if self.document.move_many(&moves) {
            self.notice = Some(self.language.text("notice.moved").to_owned());
        }
    }

    /// `Esc` or the right button: the selection stays where it was and the history untouched.
    pub(crate) fn cancel_move(&mut self) {
        if self.moving.take().is_some() {
            self.notice = Some(self.language.text("notice.move_cancelled").to_owned());
        }
    }
}
