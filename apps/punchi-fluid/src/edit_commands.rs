// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The edit and simulation commands the ribbon dispatches: the pipe's line class, the tube
//! control, even spacing, select all and the simulation reset.

use punchi_fluid_doc::Axis;
use punchi_fluid_draw::LineClass;

use crate::Window;

impl Window {
    /// **Conducto ▸ clase**: the selected pipe takes the line class, as one undo.
    pub(crate) fn set_pipe_class(&mut self, class: LineClass) {
        let Some(index) = self.selected_pipe else {
            self.notice = Some(self.language.text("notice.select_hose").to_owned());
            return;
        };
        if self.document.set_pipe_class(index, class) {
            self.notice = Some(self.language.text("notice.hose_updated").to_owned());
        }
    }

    /// **Conducto ▸ Diámetro**: opens the properties window, where the tube series lives; the
    /// series is not duplicated in the ribbon.
    pub(crate) fn open_pipe_control(&mut self) {
        if self.selected_pipe.is_none() {
            self.notice = Some(self.language.text("notice.select_hose_diameter").to_owned());
        }
        self.inspector.open();
    }

    /// **Repartir**: even spacing along the axis, one undo.
    pub(crate) fn distribute(&mut self, axis: Axis) {
        if self.document.space_evenly(&self.selected, axis) {
            self.notice = Some(self.language.text("notice.distributed").to_owned());
        }
    }

    /// **Restablecer simulación**: the model is thrown away, so the next start is at `t = 0`.
    pub(crate) fn reset_simulation(&mut self) {
        self.stop();
        self.notice = Some(self.language.text("notice.sim_reset").to_owned());
    }
}
