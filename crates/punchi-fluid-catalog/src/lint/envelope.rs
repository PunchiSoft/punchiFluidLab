// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rule 9: the declared envelope and the port-numeral clearance.
//!
//! Split from `lint.rs` to keep the rule and its tests under the architecture
//! budget. `bounding_box` stays in the parent because rule 6 uses it too.

use super::{BOX_MAX, BOX_MIN, EPSILON, LintError, all_primitives, bounding_box};
use crate::schema::{ComponentDefinition, Primitive, PrimitiveKind};

/// Rule 9: a declared envelope is the symbol's outer boundary and agrees with the drawing.
///
/// A symbol without an envelope is fine: the renderer falls back to the body and then to
/// the whole box. A declared envelope has to be true, because a port numeral is placed
/// against it: inside `0..=1`, around the body and every port, and around any frame the
/// file draws itself.
pub(super) fn check_envelope(definition: &ComponentDefinition) -> Result<(), LintError> {
    let Some([x1, y1, x2, y2]) = definition.envelope else {
        return Ok(());
    };
    let values = [x1, y1, x2, y2];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(LintError::Envelope(String::from(
            "the envelope must be finite",
        )));
    }
    if x1 < BOX_MIN || y1 < BOX_MIN || x2 > BOX_MAX + EPSILON || y2 > BOX_MAX + EPSILON {
        return Err(LintError::Envelope(String::from(
            "the envelope must lie inside the symbol box, 0..=1",
        )));
    }
    if x2 <= x1 || y2 <= y1 {
        return Err(LintError::Envelope(String::from(
            "the envelope needs two corners in increasing order",
        )));
    }

    if let Some([bx, by, bw, bh]) = definition.body {
        if bx < x1 - EPSILON
            || by < y1 - EPSILON
            || bx + bw > x2 + EPSILON
            || by + bh > y2 + EPSILON
        {
            return Err(LintError::Envelope(String::from(
                "the envelope must contain the body",
            )));
        }
    }

    for port in &definition.port {
        let [px, py] = port.position;
        if px < x1 - EPSILON || py < y1 - EPSILON || px > x2 + EPSILON || py > y2 + EPSILON {
            return Err(LintError::Envelope(format!(
                "the envelope must contain port `{}`",
                port.id
            )));
        }
    }

    // A frame is a boundary the file draws itself: a rectangle or a closed polyline. It
    // has to agree with the envelope, or the numeral is placed against a line that is not
    // where the file says the boundary is.
    for primitive in all_primitives(definition) {
        if !draws_a_frame(primitive) {
            continue;
        }
        let Some((min_x, min_y, max_x, max_y)) = bounding_box(primitive) else {
            continue;
        };
        if min_x < x1 - EPSILON
            || min_y < y1 - EPSILON
            || max_x > x2 + EPSILON
            || max_y > y2 + EPSILON
        {
            return Err(LintError::Envelope(String::from(
                "the envelope must contain the frame primitive",
            )));
        }
    }

    Ok(())
}

/// Whether a primitive draws an outer frame: a rectangle or a polyline closed on itself.
fn draws_a_frame(primitive: &Primitive) -> bool {
    match primitive.kind {
        PrimitiveKind::Rect => true,
        PrimitiveKind::Polyline => {
            let points = &primitive.points;
            points.len() >= 4
                && points.len() % 2 == 0
                && (points[0] - points[points.len() - 2]).abs() < EPSILON
                && (points[1] - points[points.len() - 1]).abs() < EPSILON
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::super::lint;
    use super::super::tests::{GOOD_VALVE, broken, parse};
    use crate::lint::LintError;

    #[test]
    fn rule_9_an_envelope_outside_the_box_is_rejected() {
        assert!(matches!(
            broken(
                "envelope = [0.0, 0.0, 1.0, 1.0]",
                "envelope = [0.0, 0.0, 1.0, 1.2]"
            ),
            LintError::Envelope(_)
        ));
        assert!(matches!(
            broken(
                "envelope = [0.0, 0.0, 1.0, 1.0]",
                "envelope = [0.0, 0.0, 0.0, 1.0]"
            ),
            LintError::Envelope(_)
        ));
    }

    #[test]
    fn rule_9_an_envelope_that_misses_the_body_or_a_port_is_rejected() {
        assert!(matches!(
            broken(
                "envelope = [0.0, 0.0, 1.0, 1.0]",
                "envelope = [0.10, 0.0, 0.90, 1.0]"
            ),
            LintError::Envelope(_)
        ));
        assert_eq!(
            broken(
                "envelope = [0.0, 0.0, 1.0, 1.0]",
                "envelope = [0.0, 0.10, 1.0, 1.0]"
            ),
            LintError::Envelope(String::from("the envelope must contain port `2`"))
        );
    }

    #[test]
    fn rule_9_an_envelope_that_contradicts_a_frame_is_rejected() {
        let mut frame = parse(&GOOD_VALVE.replace(
            "[behaviour]",
            "[[symbol]]\nkind = \"polyline\"\npoints = [0.02, 0.10, 0.98, 0.10, 0.98, 0.90, 0.02, 0.90, 0.02, 0.10]\nline_class = \"drain\"\n\n[behaviour]",
        ));
        frame.envelope = Some([0.02, 0.0, 0.96, 1.0]);
        assert_eq!(
            lint(&frame),
            Err(LintError::Envelope(String::from(
                "the envelope must contain the frame primitive"
            )))
        );
    }

    #[test]
    fn rule_9_a_missing_envelope_is_allowed() {
        let mut definition = parse(GOOD_VALVE);
        definition.envelope = None;
        assert_eq!(lint(&definition), Ok(()));
    }
}
