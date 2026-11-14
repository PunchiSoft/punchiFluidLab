// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the route between two ports.

use super::*;
use punchi_fluid_catalog::Catalog;

#[test]
fn perpendicular_ports_are_joined_with_one_bend() {
    // A hose that leaves a side and enters a bottom is one turn, not a staircase. The
    // owner's sheet showed the old Z and called it what it looked like.
    let points = route_between(
        Pt::new(10.0, 50.0),
        PortEdge::Right,
        Pt::new(80.0, 90.0),
        PortEdge::Bottom,
        0.0,
    );
    assert_eq!(points.len(), 3, "one bend has three points: {points:?}");
    assert!(
        (points[1].x - 80.0).abs() < 1e-6,
        "the turn is above the arrival"
    );
    assert!(
        (points[1].y - 50.0).abs() < 1e-6,
        "and level with the departure"
    );
    // Every segment is orthogonal.
    for pair in points.windows(2) {
        assert!(
            (pair[0].x - pair[1].x).abs() < 1e-6 || (pair[0].y - pair[1].y).abs() < 1e-6,
            "a diagonal segment in {points:?}"
        );
    }
}

#[test]
fn parallel_ports_keep_the_z_that_can_be_dragged() {
    let direct = route_between(
        Pt::new(50.0, 10.0),
        PortEdge::Bottom,
        Pt::new(50.0, 90.0),
        PortEdge::Top,
        0.0,
    );
    assert_eq!(direct.len(), 2, "aligned ports are a straight line");
    let offset = route_between(
        Pt::new(50.0, 10.0),
        PortEdge::Bottom,
        Pt::new(90.0, 90.0),
        PortEdge::Top,
        5.0,
    );
    assert_eq!(
        offset.len(),
        4,
        "a jog between parallel ports has both bends"
    );
}

#[test]
fn a_port_reads_the_edge_it_sits_on() {
    let catalog = Catalog::builtin();
    let valve = catalog.get("valve.3_2.spring_return").expect("a valve");
    assert_eq!(port_edge(valve, "1"), PortEdge::Bottom);
    assert_eq!(port_edge(valve, "2"), PortEdge::Top);
    let unit = catalog
        .get("preparation.service_unit_simple")
        .expect("a unit");
    assert_eq!(port_edge(unit, "1"), PortEdge::Left);
    assert_eq!(port_edge(unit, "2"), PortEdge::Right);
}

/// `true` when a segment runs through the inside of the box, not just along its edge.
fn crosses_inside(start: Pt, end: Pt, rect: Rect) -> bool {
    let inset = 0.5;
    let left = rect.min.x + inset;
    let right = rect.max.x - inset;
    let top = rect.min.y + inset;
    let bottom = rect.max.y - inset;
    if left >= right || top >= bottom {
        return false;
    }
    let length = (end.x - start.x).hypot(end.y - start.y);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let steps = (length / 0.25).ceil().max(1.0) as u32;
    (1..=steps).any(|step| {
        #[allow(clippy::cast_precision_loss)]
        let t = step as f32 / steps as f32;
        let point = Pt::new(
            start.x + (end.x - start.x) * t,
            start.y + (end.y - start.y) * t,
        );
        point.x > left && point.x < right && point.y > top && point.y < bottom
    })
}

/// The automatic route keeps both boxes clear; the leads from a port inside a rounded
/// symbol to its edge are the only segments allowed to touch it.
fn assert_clear(points: &[Pt], boxes: [Rect; 2]) {
    for pair in points.windows(2) {
        assert!(
            (pair[0].x - pair[1].x).abs() < 1e-6 || (pair[0].y - pair[1].y).abs() < 1e-6,
            "a diagonal segment in {points:?}"
        );
    }
    if points.len() <= 3 {
        return;
    }
    for pair in points[1..points.len() - 1].windows(2) {
        for rect in boxes {
            assert!(
                !crosses_inside(pair[0], pair[1], rect),
                "the hose goes through {rect:?}: {points:?}"
            );
        }
    }
}

#[test]
fn a_hose_goes_round_the_symbols_it_joins() {
    let start_box = Rect::from_min_size(Pt::new(0.0, 0.0), 30.0, 30.0);
    let end_box = Rect::from_min_size(Pt::new(10.0, 40.0), 30.0, 30.0);
    let points = route_between_boxes(
        Pt::new(30.0, 15.0),
        PortEdge::Right,
        start_box,
        Pt::new(25.0, 40.0),
        PortEdge::Top,
        end_box,
        0.0,
    );
    assert!(points.len() > 3, "the bend needs a detour: {points:?}");
    assert!(
        points[1].x > points[0].x,
        "the hose leaves the right edge outwards"
    );
    let last = points.len() - 1;
    assert!(
        points[last].y > points[last - 1].y,
        "the hose enters the top edge from above"
    );
    assert_clear(&points, [start_box, end_box]);
}

#[test]
fn the_compressor_hose_goes_over_the_compressor() {
    // The real geometry: the service unit's left port is inside its box and the compressor's
    // top port is just below its top edge. The old one-bend route cut through the compressor
    // at the departure's height.
    let unit = Rect::from_corners(Pt::new(73.1, 221.84), Pt::new(103.7, 252.44));
    let compressor = Rect::from_corners(Pt::new(28.1, 221.84), Pt::new(58.7, 252.44));
    let points = route_between_boxes(
        Pt::new(74.94, 237.14),
        PortEdge::Left,
        unit,
        Pt::new(43.4, 223.68),
        PortEdge::Top,
        compressor,
        0.0,
    );
    assert!(
        points.iter().any(|point| point.y < compressor.min.y),
        "the hose goes above the compressor: {points:?}"
    );
    assert_clear(&points, [unit, compressor]);
}

#[test]
fn a_perpendicular_hose_ignores_an_offset_and_still_goes_round() {
    // A perpendicular pair has no middle segment to drag. An offset left over from another
    // arrangement must not turn the route back into the one-bend crossing.
    let unit = Rect::from_corners(Pt::new(73.1, 221.84), Pt::new(103.7, 252.44));
    let compressor = Rect::from_corners(Pt::new(28.1, 221.84), Pt::new(58.7, 252.44));
    let points = route_between_boxes(
        Pt::new(74.94, 237.14),
        PortEdge::Left,
        unit,
        Pt::new(43.4, 223.68),
        PortEdge::Top,
        compressor,
        5.0,
    );
    assert!(
        points.iter().any(|point| point.y < compressor.min.y),
        "the hose still goes above the compressor: {points:?}"
    );
    assert_clear(&points, [unit, compressor]);
}

#[test]
fn a_tight_gap_still_gets_a_route_around() {
    // The owner moved the service unit until it almost touched the compressor. The 2 mm
    // clearance no longer fits; the route uses the real gap rather than crossing the body.
    let compressor = Rect::from_corners(Pt::new(28.1, 221.84), Pt::new(58.7, 252.44));
    let unit = Rect::from_corners(Pt::new(59.7, 221.84), Pt::new(90.3, 252.44));
    let points = route_between_boxes(
        Pt::new(59.7, 237.14),
        PortEdge::Left,
        unit,
        Pt::new(43.4, 223.68),
        PortEdge::Top,
        compressor,
        0.0,
    );
    assert_clear(&points, [unit, compressor]);
}

#[test]
fn aligned_boxes_keep_a_straight_hose() {
    let top = Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0);
    let bottom = Rect::from_min_size(Pt::new(0.0, 60.0), 20.0, 20.0);
    let points = route_between_boxes(
        Pt::new(10.0, 20.0),
        PortEdge::Bottom,
        top,
        Pt::new(10.0, 60.0),
        PortEdge::Top,
        bottom,
        0.0,
    );
    assert_eq!(points, vec![Pt::new(10.0, 20.0), Pt::new(10.0, 60.0)]);
}

#[test]
fn a_dragged_offset_still_takes_its_own_shape() {
    // The user moved the jog on purpose, so the box-aware router keeps the stored shape; the
    // check is the one that reports a crossing it creates.
    let top = Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0);
    let bottom = Rect::from_min_size(Pt::new(0.0, 60.0), 20.0, 20.0);
    let points = route_between_boxes(
        Pt::new(10.0, 20.0),
        PortEdge::Bottom,
        top,
        Pt::new(30.0, 60.0),
        PortEdge::Top,
        bottom,
        5.0,
    );
    assert_eq!(points.len(), 4, "{points:?}");
    assert!((points[1].y - 45.0).abs() < 1e-6, "the middle moved to 45");
}
