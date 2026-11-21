// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Polyline geometry shared by the check rules.

use punchi_fluid_draw::surface::{Pt, Rect};

/// The polyline from the point `distance` millimetres along it.
pub(crate) fn trim_start(path: &[Pt], distance: f32) -> Vec<Pt> {
    if path.len() < 2 || distance <= 0.0 {
        return path.to_vec();
    }
    let mut left = distance;
    for (index, pair) in path.windows(2).enumerate() {
        let length = (pair[1].x - pair[0].x).abs() + (pair[1].y - pair[0].y).abs();
        if left >= length {
            left -= length;
            continue;
        }
        let t = if length > 0.0 { left / length } else { 0.0 };
        let cut = Pt::new(
            pair[0].x + (pair[1].x - pair[0].x) * t,
            pair[0].y + (pair[1].y - pair[0].y) * t,
        );
        let mut out = vec![cut];
        out.extend_from_slice(&path[index + 1..]);
        return out;
    }
    Vec::new()
}

/// The polyline up to the point `distance` millimetres from its end.
pub(crate) fn trim_end(path: &[Pt], distance: f32) -> Vec<Pt> {
    if path.len() < 2 || distance <= 0.0 {
        return path.to_vec();
    }
    let mut left = distance;
    for (index, pair) in path.windows(2).enumerate().rev() {
        let length = (pair[1].x - pair[0].x).abs() + (pair[1].y - pair[0].y).abs();
        if left >= length {
            left -= length;
            continue;
        }
        let t = if length > 0.0 {
            1.0 - left / length
        } else {
            0.0
        };
        let cut = Pt::new(
            pair[0].x + (pair[1].x - pair[0].x) * t,
            pair[0].y + (pair[1].y - pair[0].y) * t,
        );
        let mut out = path[..=index].to_vec();
        out.push(cut);
        return out;
    }
    Vec::new()
}

/// The box reduced by `by` millimetres on every side.
pub(crate) fn shrink(rect: Rect, by: f32) -> Rect {
    Rect::from_corners(
        Pt::new(rect.min.x + by, rect.min.y + by),
        Pt::new(rect.max.x - by, rect.max.y - by),
    )
}

/// Where a polyline enters a rectangle, if it does.
pub(crate) fn crosses(path: &[Pt], rect: Rect) -> Option<Pt> {
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
pub(crate) fn overlap(path: &[Pt], other: &[Pt]) -> Option<Pt> {
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
