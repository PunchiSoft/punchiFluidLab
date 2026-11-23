// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet and title block commands the ribbon dispatches: each one is an undoable document
//! change, and the page the window draws and prints follows the document.

use punchi_fluid_doc::paper::{Orientation, PaperSize, Sheet as PaperSheet};
use punchi_fluid_doc::title_block::{BlockPosition, ContentModel};
use punchi_fluid_export::TitleBlock;

use crate::Window;
use crate::files;
use crate::ribbon::MarginPreset;

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
        self.notice = Some(String::from("cajetín actualizado"));
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
        self.notice = Some(String::from("cajetín restablecido"));
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
            self.notice = Some(String::from("la hoja no admite ese cambio"));
            return;
        }
        let (page, block) = files::page_and_block(&self.document);
        self.page = page;
        self.block = block;
        if refit {
            self.view.reset();
        }
        self.notice = Some(String::from("hoja actualizada"));
    }
}
