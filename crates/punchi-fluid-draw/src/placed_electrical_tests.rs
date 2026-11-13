// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the electrical line class.

use super::*;
use crate::orient::Orientation;
use crate::surface::{Call, Recording};
use punchi_fluid_catalog::Catalog;

#[test]
fn a_hose_between_two_terminals_is_drawn_dotted() {
    // The class follows the ports: two electrical terminals make a wire, and the sheet
    // says so without the user choosing anything.
    let catalog = Catalog::builtin();
    let supply = Rect::from_center_size(Pt::new(30.0, 40.0), 22.0, 22.0);
    let contact = Rect::from_center_size(Pt::new(90.0, 40.0), 22.0, 22.0);
    let placed = [
        Placed {
            index: 0,
            catalog_id: "electrical.supply",
            rect: supply,
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            orientation: Orientation::NONE,
            designation: None,
        },
        Placed {
            index: 1,
            catalog_id: "electrical.contact",
            rect: contact,
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            orientation: Orientation::NONE,
            designation: None,
        },
    ];
    let pipes = [Pipe {
        from: PipeEnd {
            placement: 0,
            port: "L",
        },
        to: PipeEnd {
            placement: 1,
            port: "1",
        },
        offset_mm: 0.0,
        class: LineClass::Working,
        pressure_ratio: 0.0,
    }];
    let mut surface = Recording::default();
    draw_placed(&mut surface, &placed, &pipes, &catalog, &Ink::print(), &[]);
    let dotted = surface.calls().iter().any(|call| match call {
        Call::Line { stroke, .. } => stroke.class == LineClass::Electrical,
        _ => false,
    });
    assert!(dotted, "the wire kept the solid working class");
}

#[test]
fn a_pneumatic_hose_keeps_its_own_class() {
    let catalog = Catalog::builtin();
    let supply = Rect::from_center_size(Pt::new(30.0, 40.0), 22.0, 22.0);
    let valve = Rect::from_center_size(Pt::new(90.0, 40.0), 22.0, 22.0);
    let placed = [
        Placed {
            index: 0,
            catalog_id: "supply.compressor",
            rect: supply,
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            orientation: Orientation::NONE,
            designation: None,
        },
        Placed {
            index: 1,
            catalog_id: "valve.3_2.spring_return",
            rect: valve,
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            orientation: Orientation::NONE,
            designation: None,
        },
    ];
    let pipes = [Pipe {
        from: PipeEnd {
            placement: 0,
            port: "1",
        },
        to: PipeEnd {
            placement: 1,
            port: "1",
        },
        offset_mm: 0.0,
        class: LineClass::Working,
        pressure_ratio: 0.0,
    }];
    let mut surface = Recording::default();
    draw_placed(&mut surface, &placed, &pipes, &catalog, &Ink::print(), &[]);
    assert!(
        !surface.calls().iter().any(|call| match call {
            Call::Line { stroke, .. } => stroke.class == LineClass::Electrical,
            _ => false,
        }),
        "a pneumatic hose became a wire"
    );
}
