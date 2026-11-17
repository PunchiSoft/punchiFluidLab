// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Domain model for pneumatic and hydraulic circuits.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

//!
//! This crate knows what a circuit *is*: components, ports, connections and
//! units. It knows nothing about numerics, files or widgets, so it can be used
//! from the solver, the file format, the desktop app and a headless test alike.

pub mod component;
pub mod network;
pub mod paper;
pub mod title_block;
pub mod units;

pub use component::{Component, ComponentKind, Port, PortId, PortKind};
pub use network::{Circuit, Connection, NodeId};
pub use paper::{Margins, Orientation, PaperSize, Sheet};

/// The most circuits one project may hold.
///
/// A teaching project uses two or three: the power circuit, its control and a variant. The
/// cap bounds the file, the undo history and the tab strip at once, and it lives here so the
/// document, the file and the window cannot disagree about it.
pub const MAX_CIRCUITS: usize = 32;
pub use title_block::{BlockPosition, ContentModel, FieldSet, TitleBlock, TitleField};
pub use units::{Pressure, Temperature, Volume};

/// Everything that can go wrong while building a domain object.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Two ports of the same component were connected to each other.
    #[error("a component cannot connect to itself")]
    SelfConnection,

    /// A port was referenced but the component does not expose it.
    #[error("unknown port {port} on component {component}")]
    UnknownPort {
        /// Component that was queried.
        component: String,
        /// Port that was requested.
        port: String,
    },

    /// A required physical value was outside its meaningful range.
    #[error("{quantity} is out of range: {value}")]
    OutOfRange {
        /// Name of the quantity, for diagnostics.
        quantity: &'static str,
        /// Rejected value.
        value: f64,
    },
}
