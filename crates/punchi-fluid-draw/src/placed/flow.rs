// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The air's mark on a hose: a stream of solid arrowheads that follows the model's flow.
//!
//! The front end owns the clock and the solver owns the flow; this module only maps the
//! phase onto a route the drawing already built. It allocates nothing: a mark is one run of
//! points on the stack and the head is one triangle.
//!
//! One mark alone cannot work: it has to leave the hose and be born again at the source, and
//! that restart is a visible jump. A stream spaced along the route hides it — an arrowhead
//! enters while another leaves — and no frame is ever empty while the air moves.

use crate::surface::{Colour, Pt, Stroke, Surface};
use crate::symbol;

use super::Pipe;

/// Length of the solid arrowhead at a mark's leading end, in millimetres.
pub(crate) const PULSE_HEAD_MM: f32 = 1.2;
/// Length of a mark's stroke, in millimetres; its head sits at the leading end.
pub(crate) const MARK_MM: f32 = 4.0;
/// How much wider than the hose a mark draws, so the air reads against the line.
pub(crate) const PULSE_WIDTH_FACTOR: f32 = 1.7;
/// Distance a mark keeps from the port it starts or ends at, in millimetres.
pub(crate) const PULSE_CLEAR_MM: f32 = 2.0;
/// Most marks a hose carries at once.
pub(crate) const MAX_MARKS: usize = 8;
/// Preferred distance between marks on a long hose, in millimetres.
pub(crate) const MARK_SPACING_MM: f32 = 14.0;
/// Pause between one mark leaving and the next entering, in millimetres of route.
pub(crate) const RESTART_GAP_MM: f32 = 8.0;
/// Slowest mark, in millimetres per simulated second: a trickle still moves.
pub(crate) const MIN_SPEED: f32 = 6.0;
/// Fastest mark, in millimetres per simulated second: a burst is still readable.
pub(crate) const MAX_SPEED: f32 = 45.0;
/// Millimetres per simulated second for each normal litre per minute.
///
/// The air moves with the flow, which is what the sheet is teaching: a strong current runs,
/// a weak one drifts. The scale is a readability constant — real air in a hose runs at metres
/// per second — but the *ordering* is the model's, and a hose long enough takes longer to
/// cross, as it should.
pub(crate) const SPEED_PER_LPM: f32 = 0.9;
/// Flow at or below which the hose is still, in normal litres per minute.
pub(crate) const FLOW_FLOOR: f32 = 0.1;
/// Most vertices one mark takes on the stack; a 4 mm stroke crosses at most a couple of bends.
const PULSE_POINTS: usize = 8;

/// How fast the marks travel, in millimetres per simulated second.
///
/// Speed follows `|flow|` between a floor and a cap: a still hose has no speed at all, a
/// trickle drifts, and a strong current runs. The values are a readability scale, not the
/// physical air speed, and they are recorded as a debt in ADR 0034; what is honest is the
/// ordering and the direction, both of which come from the model.
pub(crate) fn pulse_speed(flow_l_min: f32) -> f32 {
    (flow_l_min.abs() * SPEED_PER_LPM).clamp(MIN_SPEED, MAX_SPEED)
}

/// The point and the unit direction at `distance` along a polyline.
fn along_route(route: &[Pt], distance: f32) -> Option<(Pt, Pt)> {
    let mut walked = 0.0_f32;
    for pair in route.windows(2) {
        let (dx, dy) = pair[0].to(pair[1]);
        let length = dx.hypot(dy);
        if length <= f32::EPSILON {
            continue;
        }
        let segment_end = walked + length;
        if distance <= segment_end {
            let travelled = (distance - walked).clamp(0.0, length);
            let unit = Pt::new(dx / length, dy / length);
            return Some((
                Pt::new(
                    pair[0].x + unit.x * travelled,
                    pair[0].y + unit.y * travelled,
                ),
                unit,
            ));
        }
        walked = segment_end;
    }
    None
}

/// Where the marks sit on a hose, in millimetres from its `from` end, ordered.
///
/// A still hose, a route too short for a mark, and an electrical pipe have none. With motion
/// reduced or on paper there is one mark in the middle: the head alone carries the direction.
///
/// The stream is spaced so an arrowhead is always on the hose while the air moves: the gap
/// at the ends is shorter than the spacing, so a mark entering covers the one that left.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "a hose carries at most MAX_MARKS marks, far below the precision of f32"
)]
pub(crate) fn mark_positions(
    route_len: f32,
    flow_l_min: f32,
    phase_s: f32,
    animate: bool,
) -> ([f32; MAX_MARKS], usize) {
    let mut out = [0.0_f32; MAX_MARKS];
    if flow_l_min.abs() <= FLOW_FLOOR || route_len < MARK_MM + PULSE_CLEAR_MM * 2.0 {
        return (out, 0);
    }
    if !animate {
        out[0] = route_len * 0.5;
        return (out, 1);
    }
    // The stream is a lattice, not a list of offsets: the cycle is divided into a whole
    // number of equal steps, so two marks can never fall three millimetres apart when the
    // travel wraps. A short hose carries two, a long one up to the cap.
    let marks_wanted = (route_len / MARK_SPACING_MM).round() as usize;
    let marks_wanted = marks_wanted.clamp(2, MAX_MARKS);
    let gap = RESTART_GAP_MM.min(route_len * 0.05);
    let cycle = route_len + gap;
    let spacing = cycle / marks_wanted as f32;
    let travel = (phase_s * pulse_speed(flow_l_min)) % cycle;
    let mut count = 0;
    for index in 0..marks_wanted {
        let start = (travel + index as f32 * spacing) % cycle;
        // A mark exists only while its stroke is inside the clipped interior of the hose;
        // the rest of the cycle is the pause where it leaves and the next one enters.
        if start < PULSE_CLEAR_MM || start + MARK_MM > route_len - PULSE_CLEAR_MM {
            continue;
        }
        // Keep the stream ordered so a test or a reader sees it from the source outwards.
        let mut slot = count;
        while slot > 0 && out[slot - 1] > start {
            out[slot] = out[slot - 1];
            slot -= 1;
        }
        out[slot] = start;
        count += 1;
    }
    (out, count)
}

/// The polyline of one mark between two distances along the route.
fn stroke_between(route: &[Pt], from: f32, to: f32) -> ([Pt; PULSE_POINTS], usize) {
    let mut points = [Pt::new(0.0, 0.0); PULSE_POINTS];
    let mut count = 0;
    let mut walked = 0.0_f32;
    for pair in route.windows(2) {
        let (dx, dy) = pair[0].to(pair[1]);
        let length = dx.hypot(dy);
        if length <= f32::EPSILON {
            continue;
        }
        let segment_end = walked + length;
        let low = from.max(walked);
        let high = to.min(segment_end);
        if high > low {
            let unit = (dx / length, dy / length);
            let at = |distance: f32| {
                Pt::new(
                    pair[0].x + unit.0 * (distance - walked),
                    pair[0].y + unit.1 * (distance - walked),
                )
            };
            let first = at(low);
            if count == 0 || points[count - 1] != first {
                if count == PULSE_POINTS {
                    break;
                }
                points[count] = first;
                count += 1;
            }
            if count == PULSE_POINTS {
                break;
            }
            points[count] = at(high);
            count += 1;
            if high >= to {
                break;
            }
        }
        walked = segment_end;
    }
    (points, count)
}

/// Draws the air's marks over a pipe's base line.
///
/// The stroke says the air moves and the head says which way: ISO 1219-1 shows direction with
/// an arrowhead, so the head is never optional, and it is the same primitive the symbols and
/// the fixed example use. The marks take the hose's own colour, drawn a little thicker than
/// it, and never the accent, which means selection on this sheet.
pub(crate) fn draw_flow_marks(
    surface: &mut dyn Surface,
    route: &[Pt],
    pipe: &Pipe<'_>,
    width: f32,
    colour: Colour,
) {
    let route_len: f32 = route
        .windows(2)
        .map(|pair| {
            let (dx, dy) = pair[0].to(pair[1]);
            dx.hypot(dy)
        })
        .sum();
    let (marks, count) = mark_positions(route_len, pipe.flow_l_min, pipe.phase_s, pipe.animate);
    if count == 0 {
        return;
    }
    let sign = if pipe.flow_l_min >= 0.0 { 1.0 } else { -1.0 };
    // From the source's point of view: `u` is the distance the air has travelled, and the
    // same `u` maps to the opposite end of the route when the flow reverses.
    let at_u = |u: f32| {
        let distance = if sign > 0.0 { u } else { route_len - u };
        let distance = distance.clamp(PULSE_HEAD_MM, route_len - PULSE_HEAD_MM);
        along_route(route, distance).map(|(at, unit)| (at, Pt::new(unit.x * sign, unit.y * sign)))
    };
    for start in &marks[..count] {
        let head_u = start + MARK_MM;
        let (from_u, to_u) = if sign > 0.0 {
            (*start, head_u)
        } else {
            (route_len - *start - MARK_MM, route_len - *start)
        };
        let (points, points_len) = stroke_between(route, from_u, to_u);
        let mut ordered = points;
        if sign < 0.0 {
            ordered[..points_len].reverse();
        }
        if points_len >= 2 {
            surface.line(
                &ordered[..points_len],
                Stroke::new(width * PULSE_WIDTH_FACTOR, colour),
            );
        }
        if let Some((at, direction)) = at_u(head_u) {
            symbol::draw_flow_arrow(surface, at, direction, PULSE_HEAD_MM, colour);
        }
    }
}
