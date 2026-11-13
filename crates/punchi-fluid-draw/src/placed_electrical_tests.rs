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
            contact_closed: None,
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
            contact_closed: None,
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
            contact_closed: None,
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
            contact_closed: None,
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

#[test]
fn a_contact_shows_whether_it_is_open_or_closed() {
    // The bar is the moving part: above the gap when open, across it when closed. The file
    // holds one symbol and the run shows two states (ADR 0018).
    let catalog = Catalog::builtin();
    let rect = Rect::from_center_size(Pt::new(50.0, 50.0), 24.0, 24.0);
    let bar_of = |closed: Option<bool>| {
        let placed = [Placed {
            index: 0,
            catalog_id: "electrical.contact",
            rect,
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            contact_closed: closed,
            orientation: Orientation::NONE,
            designation: None,
        }];
        let mut surface = Recording::default();
        draw_placed(&mut surface, &placed, &[], &catalog, &Ink::print(), &[]);
        // The bar is the horizontal line nearest the middle, between the two terminals.
        surface
            .calls()
            .iter()
            .filter_map(|call| match call {
                Call::Line { points, .. } if points.len() == 2 => Some(points.clone()),
                _ => None,
            })
            .filter(|points| {
                (points[0].x - rect.min.x).abs() < 12.0 && (points[1].x - points[0].x).abs() > 5.0
            })
            .map(|points| points[0].y)
            .next()
            .unwrap_or_default()
    };
    let open = bar_of(Some(false));
    let closed = bar_of(Some(true));
    let middle = rect.center().y;
    assert!(
        open < middle - 1.0,
        "the open bar sits above the gap: {open:.1} against {middle:.1}"
    );
    assert!(
        (closed - middle).abs() < 1.0,
        "the closed bar bridges the terminals: {closed:.1} against {middle:.1}"
    );
}

#[test]
fn a_lamp_shows_its_state_as_rays_not_only_colour() {
    let catalog = Catalog::builtin();
    let rect = Rect::from_center_size(Pt::new(60.0, 45.0), 30.0, 30.0);
    let placed = |lit: bool| {
        [Placed {
            index: 0,
            catalog_id: "electrical.lamp",
            rect,
            extension: 0.0,
            valve_position: 0,
            reading: Some(Reading::Indicator(lit)),
            series: None,
            contact_closed: None,
            orientation: Orientation::NONE,
            designation: None,
        }]
    };
    let lines = |lit: bool| {
        let mut surface = Recording::default();
        draw_placed(
            &mut surface,
            &placed(lit),
            &[],
            &catalog,
            &Ink::print(),
            &[],
        );
        surface
            .calls()
            .iter()
            .filter(|call| matches!(call, Call::Line { .. }))
            .count()
    };
    assert!(
        lines(true) > lines(false),
        "a lit lamp draws rays, not only a different colour"
    );
}
