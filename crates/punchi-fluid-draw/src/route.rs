// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The path a hose takes between two ports.
//!
//! A hose is orthogonal, leaves and enters each port along its edge, and keeps clear of
//! the symbols it joins. The plain route is an L or a Z; when that would cross one of the
//! two boxes, [`route_between_boxes`] searches a small visibility grid around them for a
//! path that goes round instead. The check and the drawing both call this one function, so
//! what is inspected is what is printed.

use crate::placed::PortEdge;
use crate::surface::{Pt, Rect};

/// Distance a hose keeps from a symbol box, in millimetres.
///
/// The port sits on or just inside the box; the lead from the port to the edge is the only
/// part allowed to touch it.
const CLEARANCE_MM: f32 = 2.0;

/// What a bend costs in the search, in millimetres of straight run.
///
/// A bend is worth eight millimetres: enough to prefer a clean L over a staircase of almost
/// the same length, small enough never to take a detour through the sheet to avoid one.
const BEND_COST_MM: f32 = 8.0;

/// Tolerance for two coordinates being the same point or line, in millimetres.
const EPS: f32 = 1e-3;

/// The route between two ports, leaving each one along its own edge.
///
/// Two ports on perpendicular edges are joined with **one bend**: a hose leaving a unit
/// horizontally and entering a valve from below turns once. The old router always drew a Z,
/// which is what made a connection look like a staircase when the two ports were not
/// aligned. Two ports on the same axis keep the Z, because an orthogonal route between them
/// needs the jog; there the `offset` is what the user drags.
///
/// This is the box-less shape: tests and the fallback use it. The drawing asks for
/// [`route_between_boxes`], which can go round the symbols.
#[must_use]
pub fn route_between(
    start: Pt,
    start_edge: PortEdge,
    end: Pt,
    end_edge: PortEdge,
    offset: f32,
) -> Vec<Pt> {
    match (start_edge.horizontal(), end_edge.horizontal()) {
        (true, false) => {
            // Out horizontally, then straight into the end's edge.
            if (end.x - start.x).abs() < 0.5 {
                return vec![start, end];
            }
            vec![start, Pt::new(end.x, start.y), end]
        }
        (false, true) => {
            if (end.y - start.y).abs() < 0.5 {
                return vec![start, end];
            }
            vec![start, Pt::new(start.x, end.y), end]
        }
        // Parallel edges: the Z of two jogs, displaced by the drag.
        _ => route_offset(start, end, offset),
    }
}

/// The polyline between two points: straight, or an elbow with one jog.
#[must_use]
pub fn route(start: Pt, end: Pt) -> Vec<Pt> {
    route_offset(start, end, 0.0)
}

/// The polyline with its middle segment displaced by `offset` millimetres.
///
/// The offset applies along the axis the elbow jogs in: horizontally when the ports are
/// mostly side by side, vertically when they are mostly one above the other. That keeps
/// the stored value meaningful after a component moves, and it is the segment a user
/// drags in the reference.
#[must_use]
pub fn route_offset(start: Pt, end: Pt, offset: f32) -> Vec<Pt> {
    let horizontal = (end.x - start.x).abs() >= (end.y - start.y).abs();
    if horizontal && (end.y - start.y).abs() < 0.5 {
        return vec![start, end];
    }
    if !horizontal && (end.x - start.x).abs() < 0.5 {
        return vec![start, end];
    }
    if horizontal {
        let middle = f32::midpoint(start.x, end.x) + offset;
        vec![start, Pt::new(middle, start.y), Pt::new(middle, end.y), end]
    } else {
        let middle = f32::midpoint(start.y, end.y) + offset;
        vec![start, Pt::new(start.x, middle), Pt::new(end.x, middle), end]
    }
}

/// The route that keeps both symbols clear.
///
/// A hose between perpendicular edges used to take its bend at the departure's height, so a
/// port on the side of a unit and a port on the top of the compressor beside it produced a
/// line straight through the compressor. This route leaves the departure edge, goes round
/// both boxes with [`CLEARANCE_MM`] to spare, and only then enters the arrival edge.
///
/// A stored `offset` is the user's own drag: the route keeps that shape, because the user
/// moved the jog on purpose. The check still measures it, so a dragged hose that crosses a
/// symbol is reported.
#[must_use]
pub fn route_between_boxes(
    start: Pt,
    start_edge: PortEdge,
    start_box: Rect,
    end: Pt,
    end_edge: PortEdge,
    end_box: Rect,
    offset: f32,
) -> Vec<Pt> {
    if offset.abs() > 0.01 {
        return route_between(start, start_edge, end, end_edge, offset);
    }
    route_around(start, start_edge, start_box, end, end_edge, end_box).unwrap_or_else(|| {
        // Two boxes that overlap, or a port enclosed by the other: there is no clean path.
        // The plain shape is still drawn, and the check reports the crossing.
        route_between(start, start_edge, end, end_edge, 0.0)
    })
}

/// The point where a hose leaves its box on the way out: on the box edge plus the clearance.
fn stub(port: Pt, edge: PortEdge, rect: Rect, clearance: f32) -> Pt {
    match edge {
        PortEdge::Left => Pt::new(rect.min.x - clearance, port.y),
        PortEdge::Right => Pt::new(rect.max.x + clearance, port.y),
        PortEdge::Top => Pt::new(port.x, rect.min.y - clearance),
        PortEdge::Bottom => Pt::new(port.x, rect.max.y + clearance),
    }
}

/// The box grown by `by` millimetres on every side.
fn grow(rect: Rect, by: f32) -> Rect {
    Rect::from_corners(
        Pt::new(rect.min.x - by, rect.min.y - by),
        Pt::new(rect.max.x + by, rect.max.y + by),
    )
}

/// `true` when the point is strictly inside, so a path may run along the boundary.
fn strictly_inside(point: Pt, rect: Rect) -> bool {
    point.x > rect.min.x + EPS
        && point.x < rect.max.x - EPS
        && point.y > rect.min.y + EPS
        && point.y < rect.max.y - EPS
}

/// `true` when an orthogonal segment enters the interior of the rectangle.
fn segment_hits(start: Pt, end: Pt, rect: Rect) -> bool {
    if (start.y - end.y).abs() < EPS {
        if start.y <= rect.min.y + EPS || start.y >= rect.max.y - EPS {
            return false;
        }
        let low = start.x.min(end.x);
        let high = start.x.max(end.x);
        return high > rect.min.x + EPS && low < rect.max.x - EPS;
    }
    if (start.x - end.x).abs() < EPS {
        if start.x <= rect.min.x + EPS || start.x >= rect.max.x - EPS {
            return false;
        }
        let low = start.y.min(end.y);
        let high = start.y.max(end.y);
        return high > rect.min.y + EPS && low < rect.max.y - EPS;
    }
    false
}

/// Adds a coordinate once, so a grid line at the same place is not searched twice.
fn add_coordinate(values: &mut Vec<f32>, value: f32) {
    if !values
        .iter()
        .any(|candidate| (candidate - value).abs() < 0.01)
    {
        values.push(value);
    }
}

/// The position of a coordinate in a sorted list.
fn coordinate_index(values: &[f32], value: f32) -> Option<usize> {
    values
        .iter()
        .position(|candidate| (candidate - value).abs() < 0.01)
}

/// A path around the two boxes, or `None` when their clearance leaves no room.
fn route_around(
    start: Pt,
    start_edge: PortEdge,
    start_box: Rect,
    end: Pt,
    end_edge: PortEdge,
    end_box: Rect,
) -> Option<Vec<Pt>> {
    let start_stub = stub(start, start_edge, start_box, CLEARANCE_MM);
    let end_stub = stub(end, end_edge, end_box, CLEARANCE_MM);
    let blocked_a = grow(start_box, CLEARANCE_MM);
    let blocked_b = grow(end_box, CLEARANCE_MM);

    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for point in [start, end, start_stub, end_stub] {
        add_coordinate(&mut xs, point.x);
        add_coordinate(&mut ys, point.y);
    }
    for rect in [blocked_a, blocked_b] {
        add_coordinate(&mut xs, rect.min.x);
        add_coordinate(&mut xs, rect.max.x);
        add_coordinate(&mut ys, rect.min.y);
        add_coordinate(&mut ys, rect.max.y);
    }
    xs.sort_by(f32::total_cmp);
    ys.sort_by(f32::total_cmp);

    let count = xs.len() * ys.len();
    let point = |index: usize| Pt::new(xs[index / ys.len()], ys[index % ys.len()]);
    let valid: Vec<bool> = (0..count)
        .map(|index| {
            let at = point(index);
            !strictly_inside(at, blocked_a) && !strictly_inside(at, blocked_b)
        })
        .collect();
    let start_node =
        coordinate_index(&xs, start_stub.x)? * ys.len() + coordinate_index(&ys, start_stub.y)?;
    let end_node =
        coordinate_index(&xs, end_stub.x)? * ys.len() + coordinate_index(&ys, end_stub.y)?;
    if !valid[start_node] || !valid[end_node] {
        return None;
    }
    let chain = shortest_path(&xs, &ys, &valid, blocked_a, blocked_b, start_node, end_node)?;
    let mut points = Vec::with_capacity(chain.len() + 2);
    points.push(start);
    points.extend(chain.into_iter().map(point));
    points.push(end);
    Some(simplify(points))
}

/// The shortest orthogonal node chain between two grid nodes, bends included in the cost.
fn shortest_path(
    xs: &[f32],
    ys: &[f32],
    valid: &[bool],
    blocked_a: Rect,
    blocked_b: Rect,
    start_node: usize,
    end_node: usize,
) -> Option<Vec<usize>> {
    let point = |index: usize| Pt::new(xs[index / ys.len()], ys[index % ys.len()]);
    let count = xs.len() * ys.len();
    // Direction 0 is "not moving yet"; 1..=4 are east, west, south and north.
    let states = count * 5;
    let mut best = vec![f32::INFINITY; states];
    let mut previous: Vec<Option<usize>> = vec![None; states];
    let mut visited = vec![false; states];
    best[start_node * 5] = 0.0;
    let mut end_state = None;
    loop {
        let current = (0..states)
            .filter(|state| !visited[*state] && best[*state].is_finite())
            .min_by(|left, right| best[*left].total_cmp(&best[*right]));
        let Some(state) = current else {
            break;
        };
        visited[state] = true;
        let node_index = state / 5;
        if node_index == end_node {
            end_state = Some(state);
            break;
        }
        let direction = state % 5;
        let x_index = node_index / ys.len();
        let y_index = node_index % ys.len();
        for (dx, dy, next_direction) in [
            (1_isize, 0_isize, 1_usize),
            (-1, 0, 2),
            (0, 1, 3),
            (0, -1, 4),
        ] {
            let (Some(next_x), Some(next_y)) = (
                x_index.checked_add_signed(dx),
                y_index.checked_add_signed(dy),
            ) else {
                continue;
            };
            if next_x >= xs.len() || next_y >= ys.len() {
                continue;
            }
            let next_node = next_x * ys.len() + next_y;
            if !valid[next_node] {
                continue;
            }
            let from = point(node_index);
            let to = point(next_node);
            if segment_hits(from, to, blocked_a) || segment_hits(from, to, blocked_b) {
                continue;
            }
            let length = (from.x - to.x).abs() + (from.y - to.y).abs();
            let bend = if direction != 0 && direction != next_direction {
                BEND_COST_MM
            } else {
                0.0
            };
            let next_state = next_node * 5 + next_direction;
            let cost = best[state] + length + bend;
            if cost < best[next_state] {
                best[next_state] = cost;
                previous[next_state] = Some(state);
            }
        }
    }

    let mut state = end_state?;
    let mut chain = Vec::new();
    loop {
        chain.push(state / 5);
        match previous[state] {
            Some(prior) => state = prior,
            None => break,
        }
    }
    chain.reverse();
    Some(chain)
}

/// Drops repeated points and the middle one of three collinear points.
fn simplify(points: Vec<Pt>) -> Vec<Pt> {
    let mut out: Vec<Pt> = Vec::with_capacity(points.len());
    for point in points {
        if out
            .last()
            .is_some_and(|last| (last.x - point.x).abs() < EPS && (last.y - point.y).abs() < EPS)
        {
            continue;
        }
        out.push(point);
        while out.len() >= 3 {
            let a = out[out.len() - 3];
            let b = out[out.len() - 2];
            let c = out[out.len() - 1];
            let vertical = (a.x - b.x).abs() < EPS && (b.x - c.x).abs() < EPS;
            let horizontal = (a.y - b.y).abs() < EPS && (b.y - c.y).abs() < EPS;
            if vertical || horizontal {
                out.remove(out.len() - 2);
            } else {
                break;
            }
        }
    }
    out
}
