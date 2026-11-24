// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the route between two ports.

use super::*;
use punchi_fluid_catalog::{Behaviour, Catalog, Category, Kinematics, Origin, Port};

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
fn assert_clear(points: &[Pt], boxes: &[Rect]) {
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
        for &rect in boxes {
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
    assert_clear(&points, &[start_box, end_box]);
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
    assert_clear(&points, &[unit, compressor]);
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
    assert_clear(&points, &[unit, compressor]);
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
    assert_clear(&points, &[unit, compressor]);
}

#[test]
fn a_third_component_on_the_way_is_also_avoided() {
    let start = Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0);
    let end = Rect::from_min_size(Pt::new(0.0, 80.0), 20.0, 20.0);
    let middle = Rect::from_min_size(Pt::new(0.0, 45.0), 20.0, 10.0);
    let boxes = [start, middle, end];
    let points = route_between_boxes_avoiding(
        Pt::new(10.0, 20.0),
        PortEdge::Bottom,
        start,
        Pt::new(10.0, 80.0),
        PortEdge::Top,
        end,
        &boxes,
        [0, 2],
        0.0,
    );
    assert!(
        points
            .iter()
            .any(|point| point.x < middle.min.x || point.x > middle.max.x),
        "the hose goes round the middle box: {points:?}"
    );
    assert_clear(&points, &boxes);
}

#[test]
fn two_components_on_the_way_are_both_avoided() {
    let start = Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0);
    let end = Rect::from_min_size(Pt::new(0.0, 100.0), 20.0, 20.0);
    let first = Rect::from_min_size(Pt::new(0.0, 35.0), 20.0, 10.0);
    let second = Rect::from_min_size(Pt::new(0.0, 60.0), 20.0, 10.0);
    let boxes = [start, first, second, end];
    let points = route_between_boxes_avoiding(
        Pt::new(10.0, 20.0),
        PortEdge::Bottom,
        start,
        Pt::new(10.0, 100.0),
        PortEdge::Top,
        end,
        &boxes,
        [0, 3],
        0.0,
    );
    assert_clear(&points, &boxes);
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

#[test]
fn port_edge_towards_resolves_the_centre_from_the_hint() {
    // The three ports of a junction are all at the centre: the hint is what says which
    // side the hose leaves towards. The old function answered Left for every one.
    let catalog = Catalog::builtin();
    let tee = catalog.get("junction.tee").expect("a tee");
    let rect = Rect::from_center_size(Pt::new(100.0, 80.0), 8.0, 8.0);
    let at = Pt::new(100.0, 80.0);
    assert_eq!(
        port_edge_towards(
            rect,
            Orientation::NONE,
            tee,
            "3",
            Some(Pt::new(100.0, 60.0))
        ),
        PortEdge::Top,
        "a gauge above the tee leaves through the top"
    );
    assert_eq!(
        port_edge_towards(rect, Orientation::NONE, tee, "3", Some(Pt::new(80.0, 80.0))),
        PortEdge::Left
    );
    assert_eq!(
        port_edge_towards(
            rect,
            Orientation::NONE,
            tee,
            "3",
            Some(Pt::new(120.0, 80.0))
        ),
        PortEdge::Right
    );
    assert_eq!(
        port_edge_towards(
            rect,
            Orientation::NONE,
            tee,
            "3",
            Some(Pt::new(100.0, 100.0))
        ),
        PortEdge::Bottom
    );
    // Equal horizontal and vertical reach is the vertical case: a diagonal drag would
    // otherwise leave the route's axis to whichever comparison came first.
    assert_eq!(
        port_edge_towards(
            rect,
            Orientation::NONE,
            tee,
            "3",
            Some(Pt::new(120.0, 60.0))
        ),
        PortEdge::Top,
        "a diagonal hint is vertical"
    );
    // A coincident hint is no hint: the splice convention answers instead.
    assert_eq!(
        port_edge_towards(rect, Orientation::NONE, tee, "3", Some(at)),
        PortEdge::Top
    );
    assert_eq!(
        port_edge_towards(rect, Orientation::NONE, tee, "1", None),
        PortEdge::Left
    );
    assert_eq!(
        port_edge_towards(rect, Orientation::NONE, tee, "2", None),
        PortEdge::Right
    );
    assert_eq!(
        port_edge_towards(rect, Orientation::NONE, tee, "3", None),
        PortEdge::Top
    );
    // The convention is in the placement's local frame, so a turn moves it.
    assert_eq!(
        port_edge_towards(rect, Orientation::new(1, false), tee, "1", None),
        PortEdge::Top,
        "a left port turned clockwise is the top one"
    );
    assert_eq!(
        port_edge_towards(rect, Orientation::NONE.reflected(), tee, "1", None),
        PortEdge::Right,
        "a mirror swaps the left port to the right"
    );
    // An id this convention does not name keeps the old answer.
    assert_eq!(
        port_edge_towards(
            rect,
            Orientation::NONE,
            tee,
            "9",
            Some(Pt::new(100.0, 60.0))
        ),
        port_edge(tee, "9")
    );
}

#[test]
fn an_ordinary_port_keeps_its_edge() {
    // A port near one edge has a unique answer already; the hint must not move it, or
    // every hose in the project would drift.
    let catalog = Catalog::builtin();
    let unit = catalog
        .get("preparation.service_unit_simple")
        .expect("a unit");
    let rect = Rect::from_center_size(Pt::new(100.0, 80.0), 30.0, 30.0);
    for port in ["1", "2"] {
        assert_eq!(
            port_edge_towards(rect, Orientation::NONE, unit, port, Some(Pt::new(0.0, 0.0))),
            port_edge(unit, port),
            "the hint changed port {port}"
        );
    }

    // A corner port ties two edges at the same distance. `port_edge` keeps the first
    // minimum, and the new function must preserve exactly that even with a hint.
    let corner = ComponentDefinition {
        moving: Vec::new(),
        schema: 3,
        id: "test.corner".to_owned(),
        name: "Corner".to_owned(),
        name_es: None,
        category: Category::Accessory,
        summary: None,
        summary_es: None,
        disabled: false,
        port: vec![Port {
            id: "1".to_owned(),
            role: PortRole::Working,
            position: [0.0, 0.0],
            show_numeral: true,
            numeral: None,
        }],
        symbol: Vec::new(),
        body: None,
        envelope: None,
        spring: None,
        return_part: None,
        actuator: None,
        behaviour: Behaviour::Junction,
        kinematics: Kinematics::None,
        source: None,
        origin: Origin::Builtin,
    };
    assert_eq!(port_edge(&corner, "1"), PortEdge::Left);
    assert_eq!(
        port_edge_towards(
            Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0),
            Orientation::NONE,
            &corner,
            "1",
            Some(Pt::new(50.0, 40.0))
        ),
        PortEdge::Left,
        "the corner keeps the first minimum"
    );
}

#[test]
fn a_gauge_above_a_tee_gets_a_straight_vertical_stub() {
    // The owner's sheet: a horizontal line with the junction under the gauge. The gauge's
    // only port is at the bottom of its dial, and the branch leaves the tee's top, so the
    // whole stub is one vertical run — no horizontal lead around the junction.
    let catalog = Catalog::builtin();
    let gauge = catalog.get("gauge.pressure").expect("a gauge");
    let tee = catalog.get("junction.tee").expect("a tee");
    let gauge_rect = Rect::from_center_size(Pt::new(100.0, 55.0), 22.0, 22.0);
    let tee_rect = Rect::from_center_size(Pt::new(100.0, 80.0), 8.0, 8.0);
    let start = port_point(gauge_rect, Orientation::NONE, gauge, "1").expect("the gauge port");
    let end = port_point(tee_rect, Orientation::NONE, tee, "3").expect("the branch port");
    assert!((start.x - 100.0).abs() < 1e-4 && (start.y - 66.0).abs() < 1e-4);
    assert!((end.x - 100.0).abs() < 1e-4 && (end.y - 80.0).abs() < 1e-4);
    let points = route_between_boxes_avoiding(
        start,
        port_edge_towards(gauge_rect, Orientation::NONE, gauge, "1", Some(end)),
        gauge_rect,
        end,
        port_edge_towards(tee_rect, Orientation::NONE, tee, "3", Some(start)),
        tee_rect,
        &[gauge_rect, tee_rect],
        [0, 1],
        0.0,
    );
    assert_eq!(
        points.len(),
        2,
        "the branch is one straight run: {points:?}"
    );
    assert!(
        points.iter().all(|point| (point.x - 100.0).abs() < 1e-4),
        "every point shares the gauge's x: {points:?}"
    );
    for pair in points.windows(2) {
        assert!(
            (pair[0].x - pair[1].x).abs() < 1e-4,
            "a segment leaves the vertical: {points:?}"
        );
    }
}
