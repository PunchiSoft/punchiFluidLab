// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet and title block commands the ribbon dispatches: each one is an undoable document
//! change, and the page the window draws and prints follows the document.

use eframe::egui;
use punchi_fluid_doc::paper::{Orientation, PaperSize, Sheet as PaperSheet};
use punchi_fluid_doc::title_block::{BlockPosition, ContentModel};
use punchi_fluid_draw::Pt;
use punchi_fluid_export::TitleBlock;

use crate::Drag;
use crate::Window;
use crate::files;
use crate::ribbon::MarginPreset;
use crate::view;

impl Window {
    /// **Cajetín ▸ Contenido**: the model chooses its default field set, one undo.
    pub(crate) fn set_block_model(&mut self, model: ContentModel) {
        self.edit_block(|block| {
            block.model = model;
            block.visible = model.defaults();
        });
    }

    /// **Cajetín ▸ Posición**: where the block sits on the sheet, one undo.
    pub(crate) fn set_block_position(&mut self, position: BlockPosition) {
        self.edit_block(|block| block.position = position);
    }

    /// Applies a change to the active circuit's block as one undo, and rebinds the page the
    /// window prints and draws.
    fn edit_block(&mut self, change: impl FnOnce(&mut TitleBlock)) {
        let id = self.document.active_circuit();
        let Some(mut block) = self.document.block(id).cloned() else {
            return;
        };
        change(&mut block);
        if !self.document.set_block(id, block) {
            return;
        }
        self.refresh_page();
        self.notice = Some(self.language.text("block.updated").to_owned());
    }

    /// **Cajetín ▸ Restablecer**: the block a circuit is born with, one undo.
    pub(crate) fn reset_block(&mut self) {
        let id = self.document.active_circuit();
        let Some(title) = self
            .document
            .circuits()
            .iter()
            .find(|circuit| circuit.id == id)
            .map(|circuit| circuit.title.clone())
        else {
            return;
        };
        if !self.document.set_block(id, TitleBlock::new(title, "")) {
            return;
        }
        self.refresh_page();
        self.notice = Some(self.language.text("notice.block_reset").to_owned());
    }

    /// Whether the active circuit's block differs from the one it is born with.
    pub(crate) fn block_is_customized(&self) -> bool {
        let id = self.document.active_circuit();
        let Some(title) = self
            .document
            .circuits()
            .iter()
            .find(|circuit| circuit.id == id)
            .map(|circuit| circuit.title.clone())
        else {
            return false;
        };
        let default = TitleBlock::new(title, "");
        self.document
            .block(id)
            .is_some_and(|block| block != &default)
    }

    /// Rebinds the page the window draws after a block or sheet change.
    pub(crate) fn refresh_page(&mut self) {
        let (page, block) = files::page_and_block(&self.document);
        self.page = page;
        self.block = block;
    }

    /// **Márgenes**: the named preset in millimetres, through the document's sheet.
    pub(crate) fn set_sheet_margins(&mut self, preset: MarginPreset) {
        let mut sheet = self.document.sheet();
        sheet.margins = preset.margins();
        self.apply_sheet(sheet, false);
    }

    /// **Orientación**: turns the paper, one undo.
    pub(crate) fn set_sheet_orientation(&mut self, orientation: Orientation) {
        let mut sheet = self.document.sheet();
        sheet.orientation = orientation;
        self.apply_sheet(sheet, true);
    }

    /// **Tamaño**: the named paper, one undo.
    pub(crate) fn set_sheet_paper(&mut self, paper: PaperSize) {
        let mut sheet = self.document.sheet();
        sheet.paper = paper;
        self.apply_sheet(sheet, true);
    }

    /// Applies a validated sheet, refits the view and rebinds the page the window draws. The
    /// document records the change; the window only follows it.
    fn apply_sheet(&mut self, sheet: PaperSheet, refit: bool) {
        if !self.document.set_sheet(sheet) {
            self.notice = Some(self.language.text("page.refused").to_owned());
            return;
        }
        let (page, block) = files::page_and_block(&self.document);
        self.page = page;
        self.block = block;
        if refit {
            self.view.reset();
        }
        self.notice = Some(self.language.text("page.updated").to_owned());
    }
}

impl Window {
    /// The sheet's editing gestures: a click selects the placement under the pointer, a
    /// primary drag moves the selection with a live preview, and the release commits one
    /// undo entry.
    pub(crate) fn handle_edit(
        &mut self,
        response: &egui::Response,
        placement: &view::Placement,
        page_min: egui::Vec2,
    ) {
        let to_mm = |screen: egui::Pos2| {
            Pt::new(
                page_min.x + (screen.x - placement.origin.x) / placement.scale,
                page_min.y + (screen.y - placement.origin.y) / placement.scale,
            )
        };
        // The hand mode owns the primary drag: the pointer pans the sheet, and no selection,
        // valve press or menu happens underneath.
        if self.hand_mode {
            if response.dragged_by(egui::PointerButton::Primary) {
                let delta = response.ctx.input(|input| input.pointer.delta());
                self.view.pan_by(delta);
            }
            return;
        }
        // The armed move, the keyboard menu and the right-click menu own the frame when the
        // sheet is under them: no selection, valve press or drag happens underneath.
        if self.handle_menu_gestures(response, placement, page_min) {
            return;
        }
        if self.handle_connection_gesture(response, &to_mm) {
            return;
        }
        // Play turns a primary press into valve input; a stopped or paused press stays an
        // editing gesture, so it can select or drag the valve without changing the model.
        if self.running
            && response.is_pointer_button_down_on()
            && response
                .ctx
                .input(|input| input.pointer.button_down(egui::PointerButton::Primary))
        {
            if let Some(position) = response.interact_pointer_pos() {
                let hit = self.hit_placement(to_mm(position));
                if hit != self.pressed_valve {
                    if let Some(previous) = self.pressed_valve.take() {
                        if let Some(model) = self.model.as_mut() {
                            let _ = model.release_valve(previous);
                        }
                    }
                    if let Some(index) = hit {
                        let pressed = self
                            .model
                            .as_mut()
                            .is_some_and(|model| model.press_valve(index));
                        if pressed {
                            self.pressed_valve = Some(index);
                        }
                    }
                }
            }
        } else if let Some(index) = self.pressed_valve.take() {
            if let Some(model) = self.model.as_mut() {
                let _ = model.release_valve(index);
            }
        }
        if response.clicked() {
            if let Some(position) = response.interact_pointer_pos() {
                let additive = response.ctx.input(|input| input.modifiers.shift);
                self.select_at(to_mm(position), additive);
            }
        }
        if response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(position) = response.interact_pointer_pos() {
                let additive = response.ctx.input(|input| input.modifiers.shift);
                if additive || self.pressed_valve.is_some() {
                    return;
                }
                let index = self.select_at(to_mm(position), false);
                if index.is_some() {
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
                    self.drag = Some(Drag {
                        pointer: position,
                        starts,
                        delta: egui::Vec2::ZERO,
                    });
                }
            }
        }
        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(drag) = &mut self.drag {
                if let Some(position) = response.interact_pointer_pos() {
                    drag.delta = egui::vec2(
                        (position.x - drag.pointer.x) / placement.scale,
                        (position.y - drag.pointer.y) / placement.scale,
                    );
                }
            }
        }
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            match response.interact_pointer_pos() {
                Some(position) => self.commit_drag(placement.scale, position),
                None => self.drag = None,
            }
        }
    }
}
