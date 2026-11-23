// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The start-page commands the ribbon dispatches: Inicio and the remembered documents.

use crate::Window;

impl Window {
    /// **Inicio**: the start page, with its templates and the remembered documents.
    pub(crate) fn open_start(&mut self) {
        self.start_open = true;
    }

    /// **Recientes**: the remembered document at `index`, when its file is still there. The
    /// ribbon already disables a missing one; the command refuses it too, so a stale click or
    /// a script cannot pretend it opened.
    pub(crate) fn open_recent(&mut self, index: usize) {
        let Some(recent) = self.recents.entries().get(index).cloned() else {
            return;
        };
        let path = std::path::PathBuf::from(&recent.path);
        if let Err(cause) = self.open_path(&path) {
            self.notice = Some(format!("no se pudo abrir el documento: {cause}"));
        }
    }
}
