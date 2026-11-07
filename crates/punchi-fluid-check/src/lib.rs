// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Design checks: what is wrong with a circuit before it is run.
//!
//! The reference offers a "check the circuit" command before simulating, and its manual
//! lists the faults it looks for. This crate implements the ones that can be decided from
//! the document and the catalog alone — geometry and topology, never physics:
//!
//! - a component outside the sheet;
//! - two components whose boxes overlap;
//! - a pipe crossing a component it does not belong to;
//! - two pipes running over each other;
//! - a pneumatic port left with nothing attached to it.
//!
//! The reference lists nine faults. Four of the nine are deliberately absent, not
//! forgotten, and each is blocked on a feature rather than on this crate:
//!
//! - a mismatched connection, which in the reference means a pneumatic line on a terminal
//!   that is not pneumatic, and needs the electrical library (F6) to exist at all;
//! - two cylinders sharing a designation, a label that does not match, and a pipe through
//!   an unrelated circuit, which all three need component designations and circuit
//!   grouping (F4).
//!
//! A check that guesses is worse than no check, so [`FindingKind`] grows when the missing
//! feature lands, not before.
//!
//! Every finding carries where it happens, so the interface can point at it instead of
//! making the user map a sentence back onto the drawing.

use punchi_fluid_catalog::{Catalog, PortRole};
use punchi_fluid_doc::Document;
use punchi_fluid_draw::placed::{port_point, route};
use punchi_fluid_draw::surface::{Pt, Rect};

/// What kind of fault was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    /// A component does not fit inside the sheet.
    OutsideSheet,
    /// Two component boxes overlap.
    OverlappingComponents,
    /// A pipe crosses a component that is not one of its ends.
    PipeThroughComponent,
    /// Two pipes run over each other.
    OverlappingPipes,
    /// A port that carries air has no pipe on it.
    OpenConnection,
}

impl FindingKind {
    /// Stable name, for a message or a test.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OutsideSheet => "outside_sheet",
            Self::OverlappingComponents => "overlapping_components",
            Self::PipeThroughComponent => "pipe_through_component",
            Self::OverlappingPipes => "overlapping_pipes",
            Self::OpenConnection => "open_connection",
        }
    }

    /// The sentence the interface shows.
    #[must_use]
    pub const fn message_es(self) -> &'static str {
        match self {
            Self::OutsideSheet => "un componente se sale de la hoja",
            Self::OverlappingComponents => "dos componentes se solapan",
            Self::PipeThroughComponent => "un tubo cruza un componente que no es su extremo",
            Self::OverlappingPipes => "dos tubos se superponen",
            Self::OpenConnection => "una conexión neumática queda abierta",
        }
    }
}

/// One fault, with the place it happens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Finding {
    /// What is wrong.
    pub kind: FindingKind,
    /// Where to point, in millimetres of the sheet.
    pub at: Pt,
    /// Index of the component involved, when there is one.
    pub component: Option<usize>,
    /// Index of the pipe involved, when there is one.
    pub pipe: Option<usize>,
}

impl Finding {
    /// One line for a report.
    #[must_use]
    pub fn describe(&self) -> String {
        format!(
            "{} en ({:.0}, {:.0}) mm",
            self.kind.message_es(),
            self.at.x,
            self.at.y
        )
    }
}

/// Checks a document against `sheet`, using `catalog` for the ports of every pipe.
///
/// The result is ordered: every fault kind in [`FindingKind`] order, and inside a kind by
/// document index, so two runs on the same document produce the same list.
#[must_use]
pub fn check(document: &Document, catalog: &Catalog, sheet: Rect) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (index, placement) in document.placements().iter().enumerate() {
        let rect = placement.rect();
        if !contains_rect(sheet, rect) {
            findings.push(Finding {
                kind: FindingKind::OutsideSheet,
                at: rect.center(),
                component: Some(index),
                pipe: None,
            });
        }
    }

    for (index, placement) in document.placements().iter().enumerate() {
        for (other, candidate) in document.placements().iter().enumerate().skip(index + 1) {
            let overlap = intersect(placement.rect(), candidate.rect());
            if let Some(area) = overlap {
                findings.push(Finding {
                    kind: FindingKind::OverlappingComponents,
                    at: area.center(),
                    component: Some(other),
                    pipe: None,
                });
            }
        }
    }

    for (index, connection) in document.connections().iter().enumerate() {
        let Some(path) = pipe_path(document, catalog, index) else {
            continue;
        };
        let ends = [connection.from.placement, connection.to.placement];
        for (component, placement) in document.placements().iter().enumerate() {
            if ends.contains(&placement.id()) {
                continue;
            }
            let rect = placement.rect();
            if let Some(at) = crosses(&path, rect) {
                findings.push(Finding {
                    kind: FindingKind::PipeThroughComponent,
                    at,
                    component: Some(component),
                    pipe: Some(index),
                });
                break;
            }
        }
    }

    for (index, _) in document.connections().iter().enumerate() {
        let Some(path) = pipe_path(document, catalog, index) else {
            continue;
        };
        for other in (index + 1)..document.connections().len() {
            let Some(other_path) = pipe_path(document, catalog, other) else {
                continue;
            };
            if let Some(at) = overlap(&path, &other_path) {
                findings.push(Finding {
                    kind: FindingKind::OverlappingPipes,
                    at,
                    component: None,
                    pipe: Some(other),
                });
                break;
            }
        }
    }

    for (index, placement) in document.placements().iter().enumerate() {
        let Some(definition) = catalog.get(&placement.catalog_id) else {
            continue;
        };
        for port in &definition.port {
            if !must_be_wired(definition, port.role) {
                continue;
            }
            let joined = document.connections().iter().any(|connection| {
                (connection.from.placement == placement.id() && connection.from.port == port.id)
                    || (connection.to.placement == placement.id() && connection.to.port == port.id)
            });
            if joined {
                continue;
            }
            let Some(at) = port_point(placement.rect(), definition, &port.id) else {
                continue;
            };
            findings.push(Finding {
                kind: FindingKind::OpenConnection,
                at,
                component: Some(index),
                pipe: None,
            });
        }
    }

    findings.sort_by(|a, b| {
        (a.kind as u8, a.component, a.pipe).cmp(&(b.kind as u8, b.component, b.pipe))
    });
    findings
}

/// The ports that carry air and therefore may not be left open.
///
/// An exhaust port is exempt: the reference's own worked example is a 3/2 valve whose port
/// 3 breathes into the room, and a check that flags the example is a check the user turns
/// off. A mechanical port is not a conduit at all.
fn carries_air(role: PortRole) -> bool {
    matches!(role, PortRole::Supply | PortRole::Working)
}

/// `true` when the port has to be wired, which a junction's never does.
///
/// A junction is a point where hoses meet, modelled as a component with three ports on one
/// spot (ADR 0006). An unused port of it is an artefact of that model, not a circuit waiting
/// to be finished, so reporting it would teach the user to ignore the check.
fn must_be_wired(definition: &punchi_fluid_catalog::ComponentDefinition, role: PortRole) -> bool {
    carries_air(role)
        && !matches!(
            definition.behaviour,
            punchi_fluid_catalog::Behaviour::Junction
        )
}

/// The routed polyline of one pipe, in millimetres.
#[must_use]
pub fn pipe_path(document: &Document, catalog: &Catalog, index: usize) -> Option<Vec<Pt>> {
    let connection = document.connections().get(index)?;
    let from_index = document.index_of(connection.from.placement)?;
    let to_index = document.index_of(connection.to.placement)?;
    let from = document.placements().get(from_index)?;
    let to = document.placements().get(to_index)?;
    let from_definition = catalog.get(&from.catalog_id)?;
    let to_definition = catalog.get(&to.catalog_id)?;
    let start = port_point(from.rect(), from_definition, &connection.from.port)?;
    let end = port_point(to.rect(), to_definition, &connection.to.port)?;
    Some(route(start, end))
}

fn contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.min.x >= outer.min.x
        && inner.min.y >= outer.min.y
        && inner.max.x <= outer.max.x
        && inner.max.y <= outer.max.y
}

/// The overlap of two rectangles, when it has area.
///
/// The bounds are compared before they are turned into a `Rect`, because building one from
/// two corners normalises them: two boxes far apart would produce a rectangle that spans
/// the gap between them and a positive area out of nothing.
fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let min_x = a.min.x.max(b.min.x);
    let min_y = a.min.y.max(b.min.y);
    let max_x = a.max.x.min(b.max.x);
    let max_y = a.max.y.min(b.max.y);
    // Touching edges is not an overlap: two components may sit side by side.
    if max_x - min_x > 0.5 && max_y - min_y > 0.5 {
        Some(Rect::from_corners(
            Pt::new(min_x, min_y),
            Pt::new(max_x, max_y),
        ))
    } else {
        None
    }
}

/// Where a polyline enters a rectangle, if it does.
fn crosses(path: &[Pt], rect: Rect) -> Option<Pt> {
    for pair in path.windows(2) {
        if let Some(at) = segment_meets_rect(pair[0], pair[1], rect) {
            return Some(at);
        }
    }
    None
}

/// The first point at which a segment is inside the rectangle.
fn segment_meets_rect(start: Pt, end: Pt, rect: Rect) -> Option<Pt> {
    // Both ends outside on the same side: no crossing, and the common case.
    if (start.x < rect.min.x && end.x < rect.min.x)
        || (start.x > rect.max.x && end.x > rect.max.x)
        || (start.y < rect.min.y && end.y < rect.min.y)
        || (start.y > rect.max.y && end.y > rect.max.y)
    {
        return None;
    }
    if rect.contains(start) {
        return Some(start);
    }
    // Sample the segment; a schematic segment is at most a few tens of millimetres, so a
    // sample every half millimetre cannot step over a component.
    let length = start.to(end);
    let length = length.0.hypot(length.1);
    let steps = (length / 0.5).ceil().max(1.0);
    if !steps.is_finite() {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let steps = steps as u32;
    for step in 1..=steps {
        let t = f64::from(step) / f64::from(steps);
        #[allow(clippy::cast_possible_truncation)]
        let point = Pt::new(
            start.x + (end.x - start.x) * t as f32,
            start.y + (end.y - start.y) * t as f32,
        );
        if rect.contains(point) {
            return Some(point);
        }
    }
    None
}

/// Where two polylines run over each other, if they do.
fn overlap(path: &[Pt], other: &[Pt]) -> Option<Pt> {
    for a in path.windows(2) {
        for b in other.windows(2) {
            if let Some(at) = collinear_overlap(a[0], a[1], b[0], b[1]) {
                return Some(at);
            }
        }
    }
    None
}

/// The middle of the shared stretch of two collinear segments, if they share one.
fn collinear_overlap(a0: Pt, a1: Pt, b0: Pt, b1: Pt) -> Option<Pt> {
    let horizontal = (a0.y - a1.y).abs() < 0.1;
    let vertical = (a0.x - a1.x).abs() < 0.1;
    if !horizontal && !vertical {
        return None;
    }
    if horizontal {
        if (b0.y - a0.y).abs() > 0.5 || (b1.y - a0.y).abs() > 0.5 {
            return None;
        }
        let (a_low, a_high) = ordered(a0.x, a1.x);
        let (b_low, b_high) = ordered(b0.x, b1.x);
        let low = a_low.max(b_low);
        let high = a_high.min(b_high);
        if high - low > 1.0 {
            return Some(Pt::new(f32::midpoint(low, high), a0.y));
        }
        return None;
    }
    if (b0.x - a0.x).abs() > 0.5 || (b1.x - a0.x).abs() > 0.5 {
        return None;
    }
    let (a_low, a_high) = ordered(a0.y, a1.y);
    let (b_low, b_high) = ordered(b0.y, b1.y);
    let low = a_low.max(b_low);
    let high = a_high.min(b_high);
    if high - low > 1.0 {
        return Some(Pt::new(a0.x, f32::midpoint(low, high)));
    }
    None
}

fn ordered(a: f32, b: f32) -> (f32, f32) {
    if a <= b { (a, b) } else { (b, a) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use punchi_fluid_doc::{Endpoint, Placement};

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
    fn a_clean_circuit_has_no_findings() {
        let mut document = Document::new();
        let supply = place(&mut document, "supply.workshop", 40.0, 40.0);
        let valve = place(&mut document, "valve.3_2.spring_return", 100.0, 90.0);
        let cylinder = place(&mut document, "cylinder.single_acting_20_100", 160.0, 40.0);
        join(&mut document, (supply, "1"), (valve, "1"));
        join(&mut document, (valve, "2"), (cylinder, "1"));
        // The valve's port 3 is its exhaust: open to the room, and not a fault.
        let findings = check(&document, &catalog(), sheet());
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn a_component_outside_the_sheet_is_reported() {
        let mut document = Document::new();
        place(&mut document, "supply.workshop", 195.0, 60.0);
        let findings = check(&document, &catalog(), sheet());
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
        let supply = place(&mut document, "supply.workshop", 40.0, 40.0);
        let valve = place(&mut document, "valve.3_2.spring_return", 100.0, 90.0);
        let findings = check(&document, &catalog(), sheet());

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
        let after = check(&document, &catalog(), sheet());
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
        let working = port_point(rect, &valve, "2").expect("the valve has port 2");
        let exhaust = port_point(rect, &valve, "3").expect("the valve has port 3");
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
        let findings = check(&document, &catalog(), sheet());
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn two_components_on_top_of_each_other_are_reported() {
        let mut document = Document::new();
        place(&mut document, "supply.workshop", 60.0, 60.0);
        place(&mut document, "throttle.fixed", 62.0, 60.0);
        let findings = check(&document, &catalog(), sheet());
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
        place(&mut document, "supply.workshop", 40.0, 60.0);
        place(&mut document, "throttle.fixed", 71.0, 60.0);
        let findings = check(&document, &catalog(), sheet());
        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == FindingKind::OverlappingComponents),
            "{findings:?}"
        );
    }

    #[test]
    fn a_pipe_through_a_third_component_is_reported() {
        let mut document = Document::new();
        let supply = place(&mut document, "supply.workshop", 30.0, 60.0);
        let valve = place(&mut document, "valve.3_2.spring_return", 150.0, 60.0);
        // The throttle sits between them, right on the run.
        place(&mut document, "throttle.fixed", 90.0, 60.0);
        assert!(
            document.connect(Endpoint::new(supply, "1"), Endpoint::new(valve, "1")),
            "the pipe must exist for the check to have anything to look at"
        );
        let findings = check(&document, &catalog(), sheet());
        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == FindingKind::PipeThroughComponent),
            "{findings:?}"
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
        place(&mut document, "supply.workshop", 195.0, 60.0);
        place(&mut document, "throttle.fixed", 60.0, 60.0);
        place(&mut document, "throttle.fixed", 61.0, 60.0);
        let first = check(&document, &catalog(), sheet());
        let second = check(&document, &catalog(), sheet());
        assert_eq!(first, second);
        assert!(first.len() >= 2);
    }
}
