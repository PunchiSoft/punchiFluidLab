// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether a command may run in the ribbon's current state.

use super::{Command, Ribbon};

impl Ribbon {
    /// Whether the command may run in the current state. The transport is the demo: it
    /// starts stopped, so Pausa, Paso, Hasta cambio and Detener wait for Iniciar. Guardar is
    /// always live: without a path the window asks for one. A control whose command is not
    /// here yet is not a command at all and is painted disabled.
    pub(super) fn enabled(&self, command: Command) -> bool {
        match command {
            Command::Start => !self.running,
            Command::Pause | Command::Stop => self.running,
            Command::Step | Command::UntilChange => self.can_step,
            Command::Delete
            | Command::Rotate
            | Command::Mirror
            | Command::Duplicate
            | Command::Copy => self.selected >= 1,
            Command::Paste => self.has_clipboard,
            Command::AlignLeft | Command::AlignRight | Command::AlignTop | Command::AlignBottom => {
                self.selected >= 2
            }
            Command::Undo => self.can_undo,
            Command::Redo => self.can_redo,
            Command::ViewBack => self.can_go_back,
            Command::BlockReset => self.can_reset_block,
            Command::CircuitClose => self.can_close_circuit,
            Command::PipeClass(_) | Command::Tube => self.has_pipe,
            Command::Distribute(_) => self.selected >= 2,
            _ => true,
        }
    }
}
