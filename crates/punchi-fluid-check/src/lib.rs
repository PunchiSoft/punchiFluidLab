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

use punchi_fluid_catalog::{Catalog, Category, ComponentDefinition, PortRole};
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
    /// Two cylinders carry the same designation.
    DuplicateDesignation,
    /// A pipe joins two circuits.
    PipeAcrossCircuits,
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
            Self::DuplicateDesignation => "duplicate_designation",
            Self::PipeAcrossCircuits => "pipe_across_circuits",
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
            Self::DuplicateDesignation => "dos cilindros llevan la misma designación",
            Self::PipeAcrossCircuits => "un tubo une dos circuitos",
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
pub fn check(document: &Document, catalog: &Catalog, sheet: Rect, circuit: u32) -> Vec<Finding> {
    let mut findings = Vec::new();

    findings.extend(placement_faults(document, sheet, circuit));

    for (index, connection) in document.connections().iter().enumerate() {
        if connection.circuit != circuit {
            continue;
        }
        let Some(path) = pipe_path(document, catalog, index) else {
            continue;
        };
        let ends = [connection.from.placement, connection.to.placement];
        for (component, placement) in document.placements().iter().enumerate() {
            if placement.circuit != circuit || ends.contains(&placement.id()) {
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

    for (index, connection) in document.connections().iter().enumerate() {
        if connection.circuit != circuit {
            continue;
        }
        let Some(path) = pipe_path(document, catalog, index) else {
            continue;
        };
        for other in (index + 1)..document.connections().len() {
            if document.connections()[other].circuit != circuit {
                continue;
            }
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

    findings.extend(duplicate_designations(document, catalog, circuit));

    findings.extend(open_connections(document, catalog, circuit));
    findings.extend(pipes_across_circuits(document, catalog));

    findings.sort_by(|a, b| {
        (a.kind as u8, a.component, a.pipe).cmp(&(b.kind as u8, b.component, b.pipe))
    });
    findings
}

/// A port that carries air and has nothing attached to it.
fn open_connections(document: &Document, catalog: &Catalog, circuit: u32) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        let Some(definition) = catalog.get(&placement.catalog_id) else {
            continue;
        };
        for port in &definition.port {
            if is_electrical(port.role) || !must_be_wired(definition, port.role) {
                continue;
            }
            let joined = document.connections().iter().any(|connection| {
                (connection.from.placement == placement.id() && connection.from.port == port.id)
                    || (connection.to.placement == placement.id() && connection.to.port == port.id)
            });
            if joined {
                continue;
            }
            let Some(at) = port_point(
                placement.rect(),
                placement.orientation,
                definition,
                &port.id,
            ) else {
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
    findings
}

/// The ninth fault: a pipe whose two ends belong to different circuits.
///
/// The project-wide scan, unlike the geometry faults, does not filter by the active circuit:
/// a pipe that joins two sheets is wrong whichever sheet is in view.
fn pipes_across_circuits(document: &Document, catalog: &Catalog) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, connection) in document.connections().iter().enumerate() {
        let (Some(from), Some(to)) = (
            document.index_of(connection.from.placement),
            document.index_of(connection.to.placement),
        ) else {
            continue;
        };
        let (Some(from), Some(to)) = (
            document.placements().get(from),
            document.placements().get(to),
        ) else {
            continue;
        };
        if from.circuit == to.circuit {
            continue;
        }
        let at = pipe_path(document, catalog, index)
            .and_then(|path| path.first().copied())
            .unwrap_or_else(|| from.rect().center());
        findings.push(Finding {
            kind: FindingKind::PipeAcrossCircuits,
            at,
            component: None,
            pipe: Some(index),
        });
    }
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

/// `true` for a port that belongs to the electrical control circuit.
///
/// The check reports a pneumatic port left open, because air escaping is a fault; a
/// terminal with no wire is a drawing the student has not finished, and the reference has a
/// separate fault for a mismatched connection that this build does not implement yet.
fn is_electrical(role: PortRole) -> bool {
    matches!(role, PortRole::Electrical)
}

/// Faults about where a component sits: off the sheet, or on top of another one.
fn placement_faults(document: &Document, sheet: Rect, circuit: u32) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
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
        if placement.circuit != circuit {
            continue;
        }
        for (other, candidate) in document.placements().iter().enumerate().skip(index + 1) {
            if candidate.circuit != circuit {
                continue;
            }
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

    findings
}

/// Two actuators with one name.
///
/// The parts list would print the same actuator twice, and whoever reads the schematic
/// cannot tell which one another document refers to. Only named actuators count, and the
/// comparison ignores case, which is how a reader compares them.
fn duplicate_designations(document: &Document, catalog: &Catalog, circuit: u32) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, placement) in document.placements().iter().enumerate() {
        if placement.circuit != circuit {
            continue;
        }
        let Some(definition) = catalog.get(&placement.catalog_id) else {
            continue;
        };
        if !is_actuator(definition) {
            continue;
        }
        let Some(name) = placement.designation.as_deref() else {
            continue;
        };
        for (other_index, other) in document.placements().iter().enumerate().skip(index + 1) {
            if other.circuit != circuit {
                continue;
            }
            let Some(other_definition) = catalog.get(&other.catalog_id) else {
                continue;
            };
            if !is_actuator(other_definition) {
                continue;
            }
            if other
                .designation
                .as_deref()
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
            {
                findings.push(Finding {
                    kind: FindingKind::DuplicateDesignation,
                    at: other.box_rect().center(),
                    component: Some(other_index),
                    pipe: None,
                });
                break;
            }
        }
    }

    findings
}

/// `true` when the definition is an actuator.
///
/// The reference's duplicate-name fault is about cylinders: the parts list would print the
/// same actuator twice and a reader could not tell which one is meant. A shared name on two
/// valves is a naming habit, not a fault, so only actuators are compared.
fn is_actuator(definition: &ComponentDefinition) -> bool {
    definition.category == Category::Actuator
}

/// `true` when the port has to be wired, which a junction's never does.
///
/// A junction is a point where hoses meet, modelled as a component with three ports on one
/// spot (ADR 0006). An unused port of it is an artefact of that model, not a circuit waiting
/// to be finished, so reporting it would teach the user to ignore the check.
fn must_be_wired(definition: &ComponentDefinition, role: PortRole) -> bool {
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
    let start = port_point(
        from.rect(),
        from.orientation,
        from_definition,
        &connection.from.port,
    )?;
    let end = port_point(
        to.rect(),
        to.orientation,
        to_definition,
        &connection.to.port,
    )?;
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
mod tests;

#[cfg(test)]
mod circuit_tests;
