// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the design checks, one per fault.

use super::*;
use punchi_fluid_doc::{Endpoint, Placement};
use punchi_fluid_draw::Orientation;

fn catalog() -> Catalog {
    Catalog::builtin()
}

fn sheet() -> Rect {
    Rect::from_min_size(Pt::new(0.0, 0.0), 200.0, 130.0)
}

fn place(document: &mut Document, id: &str, x: f32, y: f32) -> u64 {
    let index = document.place(Placement::new(id, Pt::new(x, y), 30.0, 16.0));
    document.placements()[index].id()
}

fn join(document: &mut Document, from: (u64, &str), to: (u64, &str)) {
    assert!(
        document.connect(Endpoint::new(from.0, from.1), Endpoint::new(to.0, to.1)),
        "the test pipe was refused"
    );
}

#[test]
fn two_cylinders_with_one_designation_are_reported_at_the_second_one() {
    let mut document = Document::new();
    let supply = place(&mut document, "supply.compressor", 40.0, 40.0);
    let valve = place(&mut document, "valve.5_2.spring_return", 100.0, 90.0);
    let first = place(&mut document, "cylinder.double_acting_32_200", 160.0, 40.0);
    let second = place(&mut document, "cylinder.double_acting_32_200", 160.0, 100.0);
    join(&mut document, (supply, "1"), (valve, "1"));
    join(&mut document, (valve, "2"), (first, "1"));
    join(&mut document, (valve, "4"), (second, "1"));
    // Unnamed, the two cylinders are not duplicates: a name is what makes them clash.
    let findings = check(&document, &catalog(), sheet(), 0);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.kind == FindingKind::DuplicateDesignation),
        "{findings:?}"
    );
    let index = document
        .placements()
        .iter()
        .position(|placement| placement.id() == second)
        .unwrap_or_default();
    assert!(document.designate(index, Some("1A1")));
    let index = document
        .placements()
        .iter()
        .position(|placement| placement.id() == first)
        .unwrap_or_default();
    assert!(document.designate(index, Some("1a1")));
    let findings = check(&document, &catalog(), sheet(), 0);
    let clash = findings
        .iter()
        .find(|finding| finding.kind == FindingKind::DuplicateDesignation)
        .copied();
    assert!(clash.is_some(), "{findings:?}");
    let clash = clash.unwrap_or(Finding {
        kind: FindingKind::DuplicateDesignation,
        at: Pt::new(0.0, 0.0),
        component: None,
        pipe: None,
    });
    // Case does not matter to a reader, and the report points at the second cylinder.
    assert_eq!(clash.component, Some(index + 1));
    assert!((clash.at.y - 100.0).abs() < 20.0, "{:?}", clash.at);
}

#[test]
fn a_clean_circuit_has_no_findings() {
    let mut document = Document::new();
    let supply = place(&mut document, "supply.compressor", 40.0, 40.0);
    let valve = place(&mut document, "valve.3_2.spring_return", 100.0, 90.0);
    let cylinder = place(&mut document, "cylinder.single_acting_20_100", 160.0, 40.0);
    join(&mut document, (supply, "1"), (valve, "1"));
    join(&mut document, (valve, "2"), (cylinder, "1"));
    // The valve's port 3 is its exhaust: open to the room, and not a fault.
    let findings = check(&document, &catalog(), sheet(), 0);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn a_component_outside_the_sheet_is_reported() {
    let mut document = Document::new();
    place(&mut document, "supply.compressor", 195.0, 60.0);
    let findings = check(&document, &catalog(), sheet(), 0);
    let outside: Vec<&Finding> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::OutsideSheet)
        .collect();
    assert_eq!(outside.len(), 1, "{findings:?}");
    assert_eq!(outside[0].component, Some(0));
}

#[test]
fn a_port_with_no_pipe_is_reported_but_an_exhaust_is_not() {
    let mut document = Document::new();
    let supply = place(&mut document, "supply.compressor", 40.0, 40.0);
    let valve = place(&mut document, "valve.3_2.spring_return", 100.0, 90.0);
    let findings = check(&document, &catalog(), sheet(), 0);

    // The fault points at a port, not at the component's middle, and names the component
    // so the drawing can mark it.
    let open: Vec<&Finding> = findings
        .iter()
        .filter(|finding| finding.kind == FindingKind::OpenConnection)
        .collect();
    assert_eq!(open.len(), 3, "supply 1, valve 1 and valve 2: {findings:?}");
    assert!(open.iter().all(|finding| finding.component.is_some()));

    // Joining the supply removes exactly one of them.
    join(&mut document, (supply, "1"), (valve, "1"));
    let after = check(&document, &catalog(), sheet(), 0);
    let open_after = after
        .iter()
        .filter(|finding| finding.kind == FindingKind::OpenConnection)
        .count();
    assert_eq!(open_after, 1, "{after:?}");
    // What is left is the valve's working port 2, never its exhaust 3.
    let valve = catalog()
        .get("valve.3_2.spring_return")
        .expect("the valve is built in")
        .clone();
    let rect = document.placements()[1].rect();
    let working = port_point(rect, Orientation::NONE, &valve, "2").expect("port 2");
    let exhaust = port_point(rect, Orientation::NONE, &valve, "3").expect("port 3");
    let remaining = after
        .iter()
        .find(|finding| finding.kind == FindingKind::OpenConnection)
        .expect("one port is still open");
    assert!((remaining.at.x - working.x).abs() < 0.01);
    assert!((remaining.at.y - working.y).abs() < 0.01);
    assert!((remaining.at.y - exhaust.y).abs() > 0.01);
}

#[test]
fn a_junction_alone_is_not_an_open_connection() {
    // A junction is a point where hoses meet, not a device that has to be wired, so its
    // unused ports must not be reported — otherwise the check cries wolf and the user
    // learns to dismiss it.
    let mut document = Document::new();
    place(&mut document, "junction.tee", 100.0, 60.0);
    let findings = check(&document, &catalog(), sheet(), 0);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn two_components_on_top_of_each_other_are_reported() {
    let mut document = Document::new();
    place(&mut document, "supply.compressor", 60.0, 60.0);
    place(&mut document, "throttle.fixed", 62.0, 60.0);
    let findings = check(&document, &catalog(), sheet(), 0);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == FindingKind::OverlappingComponents),
        "{findings:?}"
    );
}

#[test]
fn components_side_by_side_do_not_overlap() {
    let mut document = Document::new();
    place(&mut document, "supply.compressor", 40.0, 60.0);
    place(&mut document, "throttle.fixed", 71.0, 60.0);
    let findings = check(&document, &catalog(), sheet(), 0);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.kind == FindingKind::OverlappingComponents),
        "{findings:?}"
    );
}

#[test]
fn a_dragged_pipe_through_a_third_component_is_reported() {
    let mut document = Document::new();
    let supply = place(&mut document, "supply.compressor", 30.0, 60.0);
    let valve = place(&mut document, "valve.3_2.spring_return", 150.0, 60.0);
    // The throttle sits between them, right on the run.
    place(&mut document, "throttle.fixed", 90.0, 60.0);
    assert!(
        document.connect(Endpoint::new(supply, "1"), Endpoint::new(valve, "1")),
        "the pipe must exist for the check to have anything to look at"
    );
    // The automatic route goes round the throttle now; a stored offset is the user's own
    // drag, and a drag that leaves the hose over the body is still a fault.
    assert!(document.route_to(0, 0.1), "the middle segment can be moved");
    let findings = check(&document, &catalog(), sheet(), 0);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == FindingKind::PipeThroughComponent),
        "{findings:?}"
    );
}

#[test]
fn a_dragged_hose_through_its_own_end_is_reported() {
    let mut document = Document::new();
    let valve = place(&mut document, "valve.3_2.spring_return", 100.0, 100.0);
    let cylinder = place(&mut document, "cylinder.single_acting_20_100", 90.0, 40.0);
    assert!(
        document.connect(Endpoint::new(valve, "2"), Endpoint::new(cylinder, "1")),
        "the hose must exist"
    );
    // Move the jog into the cylinder's body: the hose now crosses the component it ends on,
    // which the old check skipped because it was one of the ends.
    assert!(
        document.route_to(0, -30.0),
        "the middle segment can be moved"
    );
    let cylinder_index = document.index_of(cylinder);
    let findings = check(&document, &catalog(), sheet(), 0);
    assert!(
        findings.iter().any(|finding| {
            finding.kind == FindingKind::PipeThroughComponent && finding.component == cylinder_index
        }),
        "the hose crosses its own end: {findings:?}"
    );
}

#[test]
fn two_pipes_over_the_same_line_are_reported() {
    // Straight from the geometry, because whether two catalog pipes end up collinear
    // depends on port geometry that is not what this test is about.
    let a = [Pt::new(10.0, 30.0), Pt::new(90.0, 30.0)];
    let b = [Pt::new(50.0, 30.0), Pt::new(140.0, 30.0)];
    assert!(
        overlap(&a, &b).is_some(),
        "the shared stretch from 50 to 90 is an overlap"
    );
    let apart = [Pt::new(50.0, 60.0), Pt::new(140.0, 60.0)];
    assert!(
        overlap(&a, &apart).is_none(),
        "different rows do not overlap"
    );
    let crossing = [Pt::new(70.0, 0.0), Pt::new(70.0, 60.0)];
    assert!(
        overlap(&a, &crossing).is_none(),
        "crossing is not running over the same line"
    );
}

#[test]
fn the_same_document_always_reports_the_same_list() {
    let mut document = Document::new();
    place(&mut document, "supply.compressor", 195.0, 60.0);
    place(&mut document, "throttle.fixed", 60.0, 60.0);
    place(&mut document, "throttle.fixed", 61.0, 60.0);
    let first = check(&document, &catalog(), sheet(), 0);
    let second = check(&document, &catalog(), sheet(), 0);
    assert_eq!(first, second);
    assert!(first.len() >= 2);
}
