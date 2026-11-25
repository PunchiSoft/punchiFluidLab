// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! One drawing, two devices.
//!
//! The schematic is drawn once, in millimetres, against the [`Surface`] trait. The
//! window implements it over `egui`, a PNG or a PDF implements it over a file, and a
//! test compares the recorded calls so the sheet and the paper cannot drift apart.
//!
//! This crate knows nothing about windows, files or fonts: no toolkit here, no I/O
//! here, and no colour that does not come in through [`symbol::Ink`].

mod control_drawing;
mod fit;
pub mod orient;
pub mod placed;
pub mod route;
pub mod schematic;
pub mod surface;
pub mod symbol;

pub use orient::{Orientation, Turned};
pub use placed::{
    Placed, bounds_placed, draw_placed, draw_placed_preview, port_point, route, route_offset,
    selection_ring,
};
pub use route::{route_between_boxes, route_between_boxes_avoiding};
pub use schematic::{
    CYLINDER_RATIO, Layout, SYMBOL_BOX_MM, draw, draw_readouts, draw_with_layout, layout,
    layout_for,
};
pub use surface::{Anchor, Call, Colour, LineClass, Pt, Recording, Rect, Stroke, Surface, dashes};
pub use symbol::{
    Ink, LINE, NUMERAL, draw_component, draw_component_preview, draw_valve, symbol_box_mm,
};
