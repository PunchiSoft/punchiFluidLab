// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the tube size on a pneumatic pipe: the width series and the route length.

use super::*;
use crate::orient::Orientation;
use crate::surface::{Call, Recording};
use punchi_fluid_catalog::Catalog;

fn placed(index: usize, catalog_id: &'static str, rect: Rect) -> Placed<'static> {
    Placed {
        index,
        catalog_id,
        rect,
        extension: 0.0,
        valve_position: 0,
        reading: None,
        series: None,
        contact_closed: None,
        orientation: Orientation::NONE,
        designation: None,
        closures: &[],
    }
}

fn pipe(from: usize, from_port: &'static str, to: usize, to_port: &'static str) -> Pipe<'static> {
    Pipe {
        from: PipeEnd {
            placement: from,
            port: from_port,
        },
        to: PipeEnd {
            placement: to,
            port: to_port,
        },
        offset_mm: 0.0,
        pressure_ratio: 0.0,
        class: LineClass::Working,
        tube_od_hundredths: None,
    }
}

#[test]
fn the_line_width_is_the_series_width_below_the_frame() {
    // The series is discrete on purpose: 4 and 6 mm must differ at 300 dpi, and the
    // widest pipe stays below the 0.7 mm frame.
    let old = symbol::LINE * 1.2;
    assert!((tube_width_mm(None) - 0.42).abs() < 1e-6);
    assert!((tube_width_mm(None) - old).abs() < 1e-6);
    for (od, expected) in [
        (400, 0.25),
        (500, 0.25),
        (600, 0.35),
        (800, 0.35),
        (1000, 0.50),
        (1600, 0.50),
        (5000, 0.50),
    ] {
        assert!(
            (tube_width_mm(Some(od)) - expected).abs() < 1e-6,
            "{od} draws at {}",
            tube_width_mm(Some(od))
        );
    }
    assert!(tube_width_mm(Some(5000)) < 0.7, "the frame stays distinct");
}

#[test]
fn the_electrical_class_keeps_its_stroke_and_has_no_tube() {
    // The wire is detected by the two `PortRole::Electrical` roles, even though the
    // file stores it as a working line.
    let catalog = Catalog::builtin();
    let placed = [
        placed(
            0,
            "electrical.supply",
            Rect::from_center_size(Pt::new(30.0, 40.0), 22.0, 22.0),
        ),
        placed(
            1,
            "electrical.contact",
            Rect::from_center_size(Pt::new(90.0, 40.0), 22.0, 22.0),
        ),
    ];
    let mut cable = pipe(0, "L", 1, "1");
    cable.class = LineClass::Working;
    cable.tube_od_hundredths = Some(1600);
    let pipes = [cable];
    let mut surface = Recording::default();
    draw_placed(&mut surface, &placed, &pipes, &catalog, &Ink::print(), &[]);
    // The symbols draw their own electrical stems, so the pipe is the electrical line
    // that spans the gap between the two boxes: 19 mm to 79 mm on this fixture.
    let wires: Vec<(&Vec<Pt>, &Stroke)> = surface
        .calls()
        .iter()
        .filter_map(|call| match call {
            Call::Line { points, stroke } if stroke.class == LineClass::Electrical => {
                Some((points, stroke))
            }
            _ => None,
        })
        .collect();
    assert!(!wires.is_empty(), "the wire is drawn electrical");
    let cable = wires.iter().find(|(points, _)| {
        points
            .first()
            .is_some_and(|point| (point.x - 19.0).abs() < 0.1)
            && points
                .last()
                .is_some_and(|point| (point.x - 79.0).abs() < 0.1)
    });
    assert!(cable.is_some(), "the hose joins the two terminals");
    if let Some((_, stroke)) = cable {
        assert!(
            (stroke.width - symbol::LINE * 1.2).abs() < 1e-6,
            "an electrical wire keeps its old stroke, not the tube's"
        );
    }
    assert!(
        wires
            .iter()
            .all(|(_, stroke)| (stroke.width - 0.50).abs() > 1e-6),
        "no electrical line takes the width of a 10 mm tube"
    );
    assert_eq!(
        route_length_mm(&placed, &pipes, &catalog, 0),
        None,
        "a wire has no tube length"
    );
}

#[test]
fn the_pilot_and_drain_patterns_survive_the_width() {
    let catalog = Catalog::builtin();
    let placed = [
        placed(
            0,
            "supply.compressor",
            Rect::from_center_size(Pt::new(30.0, 40.0), 22.0, 22.0),
        ),
        placed(
            1,
            "valve.3_2.spring_return",
            Rect::from_center_size(Pt::new(90.0, 90.0), 22.0, 22.0),
        ),
    ];
    let mut hose = pipe(0, "1", 1, "1");
    hose.tube_od_hundredths = Some(1600);
    hose.class = LineClass::Pilot;
    let mut surface = Recording::default();
    draw_placed(&mut surface, &placed, &[hose], &catalog, &Ink::print(), &[]);
    assert!(surface.calls().iter().any(|call| matches!(
        call,
        Call::Line { stroke, .. }
            if stroke.class == LineClass::Pilot
                && (stroke.width - 0.50).abs() < 1e-6
    )));
    let mut drain = hose;
    drain.class = LineClass::Drain;
    let mut surface = Recording::default();
    draw_placed(
        &mut surface,
        &placed,
        &[drain],
        &catalog,
        &Ink::print(),
        &[],
    );
    assert!(surface.calls().iter().any(|call| matches!(
        call,
        Call::Line { stroke, .. }
            if stroke.class == LineClass::Drain
                && (stroke.width - 0.50).abs() < 1e-6
    )));
}

#[test]
fn the_route_length_is_the_drawn_length_not_the_straight_distance() {
    let catalog = Catalog::builtin();
    let placed = [
        placed(
            0,
            "supply.compressor",
            Rect::from_center_size(Pt::new(30.0, 40.0), 22.0, 22.0),
        ),
        placed(
            1,
            "valve.3_2.spring_return",
            Rect::from_center_size(Pt::new(110.0, 120.0), 22.0, 22.0),
        ),
    ];
    let hose = pipe(0, "1", 1, "1");
    let pipes = [hose];
    let length = route_length_mm(&placed, &pipes, &catalog, 0).expect("a route length");
    assert!(length > 0);
    // The router leaves one port and enters the other, so the drawn route is longer
    // than the straight line between the two centres (80, 80) apart.
    let straight = (80.0_f32).hypot(80.0);
    assert!(
        f64::from(length) > f64::from(straight),
        "{length} mm is not longer than the {straight} mm straight line"
    );
    assert!(
        route_length_mm(&placed, &pipes, &catalog, 9).is_none(),
        "a pipe that does not exist has no length"
    );
}
