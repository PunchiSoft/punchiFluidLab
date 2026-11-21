// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for a closure drawn on its port: where the box sits, that the numeral stays
//! readable, and that the footprint grows with it.

use super::*;
use crate::surface::{Call, Recording};
use punchi_fluid_catalog::Catalog;

fn host<'a>(
    definition: &'a ComponentDefinition,
    rect: Rect,
    orientation: Orientation,
    closures: &'a [(&'a str, &'a ComponentDefinition)],
) -> Placed<'a> {
    Placed {
        index: 0,
        catalog_id: definition.id.as_str(),
        rect,
        extension: 0.0,
        valve_position: 0,
        reading: None,
        series: None,
        contact_closed: None,
        orientation,
        designation: None,
        closures,
    }
}

fn overlapping(a: Rect, b: Rect) -> bool {
    a.min.x < b.max.x && b.min.x < a.max.x && a.min.y < b.max.y && b.min.y < a.max.y
}

#[test]
fn the_closure_is_drawn_at_the_port_and_does_not_cover_the_numeral() {
    let catalog = Catalog::builtin();
    let valve = catalog
        .get("valve.3_2.spring_return")
        .expect("the valve exists");
    let silencer = catalog
        .get("silencer.pneumatic")
        .expect("the silencer exists");
    let rect = Rect::from_min_size(Pt::new(100.0, 100.0), 20.0, 20.0);
    let closures = [("3", silencer)];

    let boxes = closure_boxes(rect, Orientation::NONE, valve, &closures);
    assert_eq!(boxes.len(), 1, "one closure, one box");
    let (port, box_rect) = &boxes[0];
    assert_eq!(port, "3");
    // Port 3 is on the bottom edge, so the closure hangs below the host and touches it
    // without entering it: the body is never covered.
    assert!(
        (box_rect.min.y - rect.max.y).abs() < 1.0e-4,
        "the closure does not touch the bottom edge: {box_rect:?}"
    );
    assert!(
        !overlapping(*box_rect, rect),
        "{box_rect:?} inside {rect:?}"
    );
    let point = port_point(rect, Orientation::NONE, valve, "3").expect("port 3 has a point");
    assert!(
        (box_rect.center().x - point.x).abs() < 1.0e-4,
        "the closure is not centred on the port: {box_rect:?}"
    );

    let placed = [host(valve, rect, Orientation::NONE, &closures)];
    let mut recording = Recording::default();
    draw_placed(&mut recording, &placed, &[], &catalog, &Ink::print(), &[]);
    // The silencer's cone is the closure's own file symbol, drawn inside its box. Its
    // outline is a stroked polygon, which the surface receives as closed line segments.
    let cone = recording.calls().iter().any(|call| match call {
        Call::Line { points, .. } => {
            points.len() >= 2
                && points
                    .iter()
                    .all(|point| box_rect.expand(0.01).contains(*point))
        }
        _ => false,
    });
    assert!(cone, "the closure's own symbol was not drawn at its box");

    // The host numeral is the run at the numeral cap, not the halo; a digit is narrower
    // than it is tall, so one cap each way is a conservative glyph box.
    let numeral = recording.calls().iter().find_map(|call| match call {
        Call::Text { at, text, size, .. }
            if text == "3" && (*size - symbol::NUMERAL).abs() < 1.0e-3 =>
        {
            Some(*at)
        }
        _ => None,
    });
    let numeral = numeral.expect("the host port keeps its numeral");
    let glyph = Rect::from_center_size(numeral, symbol::NUMERAL, symbol::NUMERAL);
    assert!(
        !overlapping(glyph, *box_rect),
        "the closure covers the numeral: {glyph:?} against {box_rect:?}"
    );
}

#[test]
fn the_closure_keeps_its_own_numeral_silent_and_the_host_numeral_visible() {
    let catalog = Catalog::builtin();
    let valve = catalog
        .get("valve.3_2.spring_return")
        .expect("the valve exists");
    let plug = catalog.get("plug.blind").expect("the plug exists");
    let rect = Rect::from_min_size(Pt::new(100.0, 100.0), 20.0, 20.0);
    let closures = [("3", plug)];
    let placed = [host(valve, rect, Orientation::NONE, &closures)];
    let mut recording = Recording::default();
    draw_placed(&mut recording, &placed, &[], &catalog, &Ink::print(), &[]);
    let numeral = |id: &str| {
        recording
            .calls()
            .iter()
            .filter(|call| match call {
                Call::Text { text, size, .. } => {
                    text == id && (*size - symbol::NUMERAL).abs() < 1.0e-3
                }
                _ => false,
            })
            .count()
    };
    // The host's supply port carries `1`; the plug's own `1` must not appear beside it.
    assert_eq!(numeral("1"), 1, "the closure drew its own port numeral");
    // The host's exhaust numeral stays: it is the host port the closure belongs to.
    assert_eq!(
        numeral("3"),
        1,
        "the host numeral vanished under the closure"
    );
}

#[test]
fn a_turned_host_takes_its_closure_to_the_turned_edge() {
    // Port 3 is on the bottom edge as the file draws it; a clockwise quarter turn puts it
    // on the left, so the closure goes there instead of staying under the host.
    let catalog = Catalog::builtin();
    let valve = catalog
        .get("valve.3_2.spring_return")
        .expect("the valve exists");
    let plug = catalog.get("plug.blind").expect("the plug exists");
    let rect = Rect::from_min_size(Pt::new(100.0, 100.0), 20.0, 20.0);
    let orientation = Orientation::new(1, false);
    let closures = [("3", plug)];

    let boxes = closure_boxes(rect, orientation, valve, &closures);
    assert_eq!(boxes.len(), 1);
    let box_rect = boxes[0].1;
    assert!(
        (box_rect.max.x - rect.min.x).abs() < 1.0e-4,
        "the closure is not on the turned left edge: {box_rect:?}"
    );
    assert!(
        (box_rect.min.x - (rect.min.x - CLOSURE_MM)).abs() < 1.0e-4,
        "{box_rect:?}"
    );
}

#[test]
fn the_closure_grows_the_drawn_bounds() {
    let catalog = Catalog::builtin();
    let valve = catalog
        .get("valve.3_2.spring_return")
        .expect("the valve exists");
    let silencer = catalog
        .get("silencer.pneumatic")
        .expect("the silencer exists");
    let rect = Rect::from_min_size(Pt::new(100.0, 100.0), 20.0, 20.0);
    let closures = [("3", silencer)];

    let without = [host(valve, rect, Orientation::NONE, &[])];
    let with = [host(valve, rect, Orientation::NONE, &closures)];
    let plain = bounds_placed(&without, &[], &catalog).expect("the host has bounds");
    let grown = bounds_placed(&with, &[], &catalog).expect("the closure has bounds");
    let box_rect = closure_boxes(rect, Orientation::NONE, valve, &closures)[0].1;

    assert!(
        grown.contains(box_rect.center()),
        "the bounds miss the closure: {grown:?} against {box_rect:?}"
    );
    assert!(
        grown.max.y > plain.max.y + CLOSURE_MM * 0.5,
        "the footprint only grew by {:.1} mm: {plain:?} -> {grown:?}",
        grown.max.y - plain.max.y
    );
}

#[test]
fn a_closure_on_a_port_the_host_lacks_is_not_drawn() {
    let catalog = Catalog::builtin();
    let valve = catalog
        .get("valve.3_2.spring_return")
        .expect("the valve exists");
    let silencer = catalog
        .get("silencer.pneumatic")
        .expect("the silencer exists");
    let rect = Rect::from_min_size(Pt::new(100.0, 100.0), 20.0, 20.0);
    let closures = [("9", silencer)];
    assert!(closure_boxes(rect, Orientation::NONE, valve, &closures).is_empty());

    let plain = {
        let placed = [host(valve, rect, Orientation::NONE, &[])];
        let mut recording = Recording::default();
        draw_placed(&mut recording, &placed, &[], &catalog, &Ink::print(), &[]);
        recording.calls().len()
    };
    let unresolved = {
        let placed = [host(valve, rect, Orientation::NONE, &closures)];
        let mut recording = Recording::default();
        draw_placed(&mut recording, &placed, &[], &catalog, &Ink::print(), &[]);
        recording.calls().len()
    };
    assert_eq!(plain, unresolved, "a closure with no port drew something");
}
