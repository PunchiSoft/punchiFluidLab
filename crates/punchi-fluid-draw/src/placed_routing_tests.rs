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
