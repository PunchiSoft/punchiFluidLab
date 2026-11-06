// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Schematic editor widgets.
//!
//! The UI crate owns drawing and interaction only. It reads a
//! [`punchi_fluid_core::Circuit`], asks the solver for state, and never touches
//! files or numbers behind the solver's public API. Swapping the toolkit is a
//! change confined to this crate.

pub mod canvas;
pub mod preview;
pub mod ribbon;
pub mod shell;
pub mod sidebar;
pub mod surface;

pub use canvas::{CanvasOutcome, SchematicCanvas};
pub mod icons;
pub mod menu;

pub use ribbon::{Action, RibbonInfo, RibbonState, Tab};
pub use shell::Mode;
pub use sidebar::{SidebarEvent, SidebarState};
