// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! `symbol lint`: the automated half of the ISO 1219 conformance review.
//!
//! A definition that breaks one of these rules is skipped like any other malformed
//! file, with the rule named in the report. The rules exist because a diagram that
//! looks right and is not is the one outcome this project cannot afford, and because
//! a reviewer cannot be relied on to see a valve with two unequal squares.
//!
//! The contract these rules enforce is [docs/symbology.md](../../../../docs/symbology.md).

use thiserror::Error;

use crate::schema::{Actuation, Behaviour, ComponentDefinition, Primitive, PrimitiveKind};

/// Position inside the symbol box where a drawn element may live.
const BOX_MIN: f64 = 0.0;
/// Upper bound of the symbol box.
const BOX_MAX: f64 = 1.0;
/// A square narrower than this is illegible at palette size.
pub const MIN_SQUARE_SIDE: f64 = 0.15;
/// Thinnest stroke a definition may ask for, as a multiple of the base width.
pub const MIN_STROKE: f64 = 0.5;
/// Tolerance for geometry that is meant to touch an edge.
const EPSILON: f64 = 1.0e-6;

/// A rule of the symbol lint that a definition failed.
#[derive(Debug, Error, PartialEq)]
pub enum LintError {
    /// The valve body is missing, ragged, or too small to read.
    #[error("body: {0}")]
    Body(String),

    /// A position of a valve says nothing: no path and no closed port.
    #[error("position `{position}`: it neither opens nor closes anything")]
    EmptyPosition {
        /// Name of the offending position.
        position: String,
    },

    /// A port is drawn but no position refers to it, so it means nothing.
    #[error("port `{0}`: no position opens or closes it")]
    UnusedPort(String),

    /// A valve that returns by itself without declaring what returns it.
    #[error("return: the valve returns by `{0}` but declares no `[return]` part")]
    MissingReturn(&'static str),

    /// A return declared on a valve that does not return by itself.
    #[error("return: `{0}` is declared but `spring_return` is not set")]
    UnexpectedReturn(&'static str),

    /// A detent declared on a valve that the file says springs back.
    #[error("return: `detent` holds the spool, so `spring_return` must be false")]
    DetentWithReturn,

    /// Two spellings of the same part, which is one meaning too many.
    #[error("return: `[spring]` is the short form of `[return] kind = \"spring\"`, declare one")]
    TwoReturns,

    /// A spring-return valve declared as a toggle: it would stay actuated with nothing
    /// pushing the spool back.
    #[error("spring: a valve that returns by spring cannot be a `toggle`, use `momentary`")]
    ToggleWithSpring,

    /// A valve that moves but declares no actuator.
    #[error("actuator: `actuation = {0}` needs a declared `[actuator]` part")]
    MissingActuator(&'static str),

    /// An actuator declared on a component that cannot be actuated.
    #[error("actuator: declared on a component with `actuation = none`")]
    UnexpectedActuator,

    /// A primitive outside the symbol box, or degenerate.
    #[error("geometry: {0}")]
    Geometry(String),

    /// A line without a class, so the renderer has to guess its pattern.
    #[error("line class: a {0} declares no `line_class`")]
    MissingLineClass(&'static str),

    /// A base primitive drawn inside the body the renderer owns.
    #[error("body: a base primitive lies inside the body, which the renderer draws")]
    BodyIntrusion,
}

/// Runs every rule over one definition.
///
/// Checks the whole list before returning, so a file with several defects reports the
/// first one in rule order rather than whichever happened to be visited first.
///
/// # Errors
///
/// Returns the first [`LintError`] the definition breaks, or `Ok(())` when it passes
/// every rule.
pub fn lint(definition: &ComponentDefinition) -> Result<(), LintError> {
    check_geometry(definition)?;
    check_line_classes(definition)?;

    let positions = match &definition.behaviour {
        Behaviour::Valve { positions, .. } => positions.as_slice(),
        _ => {
            return check_parts(definition);
        }
    };

    let body = check_body(definition, positions.len())?;
    check_positions(definition, positions)?;
    check_parts(definition)?;
    check_intrusion(definition, body)?;
    Ok(())
}

/// Rule 1: a valve declares a body, wide enough for equal squares of readable size.
fn check_body(
    definition: &ComponentDefinition,
    position_count: usize,
) -> Result<[f64; 4], LintError> {
    let Some(body) = definition.body else {
        return Err(LintError::Body(String::from(
            "a valve must declare `body = [x, y, width, height]`",
        )));
    };
    let [x, y, width, height] = body;
    let values = [x, y, width, height];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(LintError::Body(String::from("the body must be finite")));
    }
    if x < BOX_MIN || y < BOX_MIN || x + width > BOX_MAX || y + height > BOX_MAX {
        return Err(LintError::Body(String::from(
            "the body must lie inside the symbol box, 0..=1",
        )));
    }
    if width <= 0.0 || height <= 0.0 {
        return Err(LintError::Body(String::from(
            "the body must have a positive width and height",
        )));
    }
    if height < MIN_SQUARE_SIDE {
        return Err(LintError::Body(format!(
            "a square {height:.3} tall is illegible, the floor is {MIN_SQUARE_SIDE}"
        )));
    }
    #[allow(clippy::cast_precision_loss)]
    let expected = height * position_count as f64;
    if (width - expected).abs() > EPSILON.max(expected * 0.02) {
        return Err(LintError::Body(format!(
            "{position_count} positions need a body {expected:.3} wide for equal squares, found {width:.3}"
        )));
    }
    Ok(body)
}

/// Rule 2 and 3: every position says something, every port is used.
fn check_positions(
    definition: &ComponentDefinition,
    positions: &[crate::schema::ValvePosition],
) -> Result<(), LintError> {
    for position in positions {
        let opens = !position.connection.is_empty();
        let closes = !position.blocked.is_empty();
        if !opens && !closes {
            return Err(LintError::EmptyPosition {
                position: position.name.clone(),
            });
        }
        for id in &position.blocked {
            if definition.port(id).is_none() {
                return Err(LintError::UnusedPort(id.clone()));
            }
        }
    }

    for port in &definition.port {
        // An electrical terminal feeds the coil and no valve position opens or closes it;
        // it is not an orphan, it is the other domain (ADR 0018).
        if port.role == crate::schema::PortRole::Electrical {
            continue;
        }
        let used = positions.iter().any(|position| {
            position.blocked.contains(&port.id)
                || position
                    .connection
                    .iter()
                    .any(|connection| connection.from == port.id || connection.to == port.id)
        });
        if !used {
            return Err(LintError::UnusedPort(port.id.clone()));
        }
    }
    Ok(())
}

/// Rules 4 and 5: declared parts match the behaviour that needs them.
fn check_parts(definition: &ComponentDefinition) -> Result<(), LintError> {
    let Behaviour::Valve {
        spring_return,
        actuation,
        ..
    } = &definition.behaviour
    else {
        // A cylinder has a return spring of its own, so a spring part is legal
        // outside a valve; an actuator outside a valve is not.
        return if definition.actuator.is_some() {
            Err(LintError::UnexpectedActuator)
        } else {
            Ok(())
        };
    };
    let (spring_return, actuation) = (*spring_return, *actuation);

    if definition.declares_two_returns() {
        return Err(LintError::TwoReturns);
    }

    // `[spring]` is the short form of `[return] kind = "spring"`, so the rules read the
    // resolved part and hold for either spelling.
    let declared = definition.resolved_return();
    match (spring_return, declared) {
        (true, None) => return Err(LintError::MissingReturn("spring_return = true")),
        (false, Some(part)) => {
            return Err(LintError::UnexpectedReturn(part.kind.as_str()));
        }
        (true, Some(part)) if !part.kind.pushes_back() => {
            return Err(LintError::DetentWithReturn);
        }
        _ => {}
    }

    if spring_return && actuation == Actuation::Toggle {
        return Err(LintError::ToggleWithSpring);
    }

    match (actuation, definition.actuator) {
        (Actuation::None, Some(_)) => return Err(LintError::UnexpectedActuator),
        (Actuation::Toggle, None) => return Err(LintError::MissingActuator("toggle")),
        (Actuation::Momentary, None) => return Err(LintError::MissingActuator("momentary")),
        _ => {}
    }
    Ok(())
}

/// Rule 6: no primitive outside the box, none degenerate.
fn check_geometry(definition: &ComponentDefinition) -> Result<(), LintError> {
    for primitive in all_primitives(definition) {
        check_primitive(primitive)?;
    }
    Ok(())
}

fn all_primitives(definition: &ComponentDefinition) -> Vec<&Primitive> {
    let mut primitives: Vec<&Primitive> = definition.symbol.iter().collect();
    if let Behaviour::Valve { positions, .. } = &definition.behaviour {
        for position in positions {
            primitives.extend(position.symbol.iter());
        }
    }
    primitives
}

fn check_primitive(primitive: &Primitive) -> Result<(), LintError> {
    let points = &primitive.points;
    for value in points {
        if !value.is_finite() {
            return Err(LintError::Geometry(String::from(
                "a coordinate is not a number",
            )));
        }
        if !(BOX_MIN - EPSILON..=BOX_MAX + EPSILON).contains(value) {
            return Err(LintError::Geometry(format!(
                "a {:?} coordinate {value} leaves the symbol box, 0..=1",
                primitive.kind
            )));
        }
    }

    match primitive.kind {
        PrimitiveKind::Line | PrimitiveKind::Polyline | PrimitiveKind::Polygon => {
            if points.len() < 4 || points.len() % 2 != 0 {
                return Err(LintError::Geometry(format!(
                    "a {:?} needs at least two points and an even coordinate count",
                    primitive.kind
                )));
            }
            let pairs: Vec<(f64, f64)> = points
                .chunks_exact(2)
                .map(|pair| (pair[0], pair[1]))
                .collect();
            let degenerate = pairs.windows(2).all(|window| {
                (window[0].0 - window[1].0).abs() < EPSILON
                    && (window[0].1 - window[1].1).abs() < EPSILON
            });
            if degenerate {
                return Err(LintError::Geometry(String::from("a line has zero length")));
            }
        }
        PrimitiveKind::Rect => {
            if points.len() != 4 {
                return Err(LintError::Geometry(String::from(
                    "a rect needs `[x, y, width, height]`",
                )));
            }
            if points[2] <= 0.0 || points[3] <= 0.0 {
                return Err(LintError::Geometry(String::from(
                    "a rect needs a positive width and height",
                )));
            }
        }
        PrimitiveKind::Circle => {
            if points.len() != 3 || points[2] <= 0.0 {
                return Err(LintError::Geometry(String::from(
                    "a circle needs `[cx, cy, radius]` with a positive radius",
                )));
            }
        }
        PrimitiveKind::Arc => {
            if points.len() != 5 || points[2] <= 0.0 {
                return Err(LintError::Geometry(String::from(
                    "an arc needs `[cx, cy, radius, start, end]` with a positive radius",
                )));
            }
        }
    }

    if primitive.stroke < MIN_STROKE {
        return Err(LintError::Geometry(format!(
            "a stroke of {} is below the legibility floor {MIN_STROKE}",
            primitive.stroke
        )));
    }
    Ok(())
}

/// Rule 7: every line declares its class.
fn check_line_classes(definition: &ComponentDefinition) -> Result<(), LintError> {
    for primitive in all_primitives(definition) {
        let draws_a_line = matches!(
            primitive.kind,
            PrimitiveKind::Line | PrimitiveKind::Polyline
        );
        if draws_a_line && primitive.line_class.is_none() {
            return Err(LintError::MissingLineClass(match primitive.kind {
                PrimitiveKind::Line => "line",
                _ => "polyline",
            }));
        }
    }
    Ok(())
}

/// Rule 8: the renderer owns the body, so no base primitive may draw inside it.
fn check_intrusion(definition: &ComponentDefinition, body: [f64; 4]) -> Result<(), LintError> {
    let [bx, by, bw, bh] = body;
    for primitive in &definition.symbol {
        let Some(bbox) = bounding_box(primitive) else {
            continue;
        };
        let inside = bbox.0 > bx + EPSILON
            && bbox.1 > by + EPSILON
            && bbox.2 < bx + bw - EPSILON
            && bbox.3 < by + bh - EPSILON;
        if inside {
            return Err(LintError::BodyIntrusion);
        }
    }
    Ok(())
}

fn bounding_box(primitive: &Primitive) -> Option<(f64, f64, f64, f64)> {
    let points = &primitive.points;
    match primitive.kind {
        PrimitiveKind::Line | PrimitiveKind::Polyline | PrimitiveKind::Polygon => {
            let xs: Vec<f64> = points.chunks_exact(2).map(|pair| pair[0]).collect();
            let ys: Vec<f64> = points.chunks_exact(2).map(|pair| pair[1]).collect();
            Some((
                xs.iter().copied().fold(f64::INFINITY, f64::min),
                ys.iter().copied().fold(f64::INFINITY, f64::min),
                xs.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                ys.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ))
        }
        PrimitiveKind::Rect => {
            let [x, y, width, height] = <[f64; 4]>::try_from(points.as_slice()).ok()?;
            Some((x, y, x + width, y + height))
        }
        PrimitiveKind::Circle | PrimitiveKind::Arc => {
            let [cx, cy, radius] = <[f64; 3]>::try_from(&points[..3]).ok()?;
            Some((cx - radius, cy - radius, cx + radius, cy + radius))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest conformant two-position valve: every rule passes on it, and each
    /// test below breaks exactly one thing.
    const GOOD_VALVE: &str = r#"
schema = 2
id = "valve.test"
name = "test valve"
category = "valve"
body = [0.06, 0.28, 0.88, 0.44]

[spring]
side = "right"

[actuator]
kind = "lever"
side = "left"

[[port]]
id = "1"
role = "supply"
position = [0.22, 1.0]

[[port]]
id = "2"
role = "working"
position = [0.22, 0.0]

[[port]]
id = "3"
role = "exhaust"
position = [0.78, 1.0]

[behaviour]
type = "valve"
actuation = "momentary"
spring_return = true
default_position = 0

[[behaviour.positions]]
name = "rest"
blocked = ["1"]

[[behaviour.positions.connection]]
from = "2"
to = "3"
conductance = 1.0e-9

[[behaviour.positions.symbol]]
kind = "polyline"
points = [0.78, 0.32, 0.78, 0.60]
line_class = "working"

[[behaviour.positions]]
name = "actuated"
blocked = ["3"]

[[behaviour.positions.connection]]
from = "1"
to = "2"
conductance = 1.0e-9
"#;

    fn parse(text: &str) -> ComponentDefinition {
        toml::from_str(text).expect("the fixture parses")
    }

    fn broken(from: &str, to: &str) -> LintError {
        let text = GOOD_VALVE.replace(from, to);
        assert_ne!(text, GOOD_VALVE, "the fixture did not contain the fragment");
        lint(&parse(&text)).expect_err("the lint must reject this definition")
    }

    #[test]
    fn every_builtin_definition_passes_the_lint() {
        for (name, text) in crate::builtin::FILES {
            let definition = parse(text);
            assert_eq!(
                lint(&definition),
                Ok(()),
                "{name} must pass the symbol lint"
            );
        }
    }

    #[test]
    fn the_fixture_itself_passes() {
        assert_eq!(lint(&parse(GOOD_VALVE)), Ok(()));
    }

    #[test]
    fn rule_1_a_body_that_does_not_fit_its_squares_is_rejected() {
        assert!(matches!(
            broken(
                "body = [0.06, 0.28, 0.88, 0.44]",
                "body = [0.06, 0.28, 0.70, 0.44]"
            ),
            LintError::Body(_)
        ));
        assert!(matches!(
            broken(
                "body = [0.06, 0.28, 0.88, 0.44]",
                "body = [0.06, 0.28, 0.88, 0.10]"
            ),
            LintError::Body(_)
        ));
        assert!(matches!(
            broken(
                "body = [0.06, 0.28, 0.88, 0.44]",
                "body = [0.06, 0.70, 0.88, 0.44]"
            ),
            LintError::Body(_)
        ));
    }

    #[test]
    fn rule_1_a_valve_without_a_body_is_rejected() {
        assert!(matches!(
            broken("body = [0.06, 0.28, 0.88, 0.44]\n", "\n"),
            LintError::Body(_)
        ));
    }

    #[test]
    fn rule_2_a_position_that_neither_opens_nor_closes_is_rejected() {
        let text = GOOD_VALVE.replace(
            "name = \"actuated\"\nblocked = [\"3\"]\n",
            "name = \"actuated\"\n",
        );
        let definition = parse(&text);
        let Behaviour::Valve { positions, .. } = &definition.behaviour else {
            unreachable!("the fixture is a valve")
        };
        assert!(!positions[1].connection.is_empty());
        let text = text.replace(
            "\n[[behaviour.positions.connection]]\nfrom = \"1\"\nto = \"2\"\nconductance = 1.0e-9\n",
            "",
        );
        assert!(matches!(
            lint(&parse(&text)),
            Err(LintError::EmptyPosition { .. })
        ));
    }

    #[test]
    fn rule_3_a_port_no_position_refers_to_is_rejected() {
        let text = GOOD_VALVE
            .replace("from = \"2\"\nto = \"3\"", "from = \"2\"\nto = \"1\"")
            .replace("blocked = [\"3\"]", "blocked = [\"1\"]");
        assert_eq!(
            lint(&parse(&text)),
            Err(LintError::UnusedPort(String::from("3")))
        );
    }

    #[test]
    fn rule_3_a_blocked_port_must_exist() {
        assert_eq!(
            broken("blocked = [\"1\"]", "blocked = [\"9\"]"),
            LintError::UnusedPort(String::from("9"))
        );
    }

    #[test]
    fn rule_4_a_spring_return_valve_needs_a_spring() {
        assert_eq!(
            broken("\n[spring]\nside = \"right\"\n", "\n"),
            LintError::MissingReturn("spring_return = true")
        );
    }

    #[test]
    fn rule_4_a_spring_without_spring_return_is_rejected() {
        assert_eq!(
            broken("spring_return = true", "spring_return = false"),
            LintError::UnexpectedReturn("spring")
        );
    }

    #[test]
    fn rule_4_a_pressure_return_without_spring_return_is_rejected() {
        // The file says the valve holds, but it declares a part that pushes it back.
        let text = GOOD_VALVE
            .replace("spring_return = true", "spring_return = false")
            .replace(
                "\n[spring]\nside = \"right\"\n",
                "\n[return]\nkind = \"pneumatic\"\nside = \"right\"\n",
            );
        assert_eq!(
            lint(&parse(&text)),
            Err(LintError::UnexpectedReturn("pneumatic"))
        );
    }

    #[test]
    fn rule_4_a_pressure_return_replaces_the_spring() {
        // A valve that returns by air declares the return it has, not a spring.
        let text = GOOD_VALVE.replace(
            "\n[spring]\nside = \"right\"\n",
            "\n[return]\nkind = \"pneumatic\"\nside = \"right\"\n",
        );
        assert_eq!(lint(&parse(&text)), Ok(()));
    }

    #[test]
    fn rule_4_a_detent_cannot_promise_a_spring_return() {
        let text = GOOD_VALVE.replace(
            "\n[spring]\nside = \"right\"\n",
            "\n[return]\nkind = \"detent\"\nside = \"right\"\n",
        );
        assert_eq!(lint(&parse(&text)), Err(LintError::DetentWithReturn));
    }

    #[test]
    fn rule_4_the_two_spellings_of_a_spring_are_rejected_together() {
        let text = GOOD_VALVE.replace(
            "\n[spring]\nside = \"right\"\n",
            "\n[spring]\nside = \"right\"\n\n[return]\nkind = \"spring\"\nside = \"right\"\n",
        );
        assert_eq!(lint(&parse(&text)), Err(LintError::TwoReturns));
    }

    #[test]
    fn rule_5_an_actuated_valve_needs_its_actuator() {
        assert_eq!(
            broken("\n[actuator]\nkind = \"lever\"\nside = \"left\"\n", "\n"),
            LintError::MissingActuator("momentary")
        );
    }

    #[test]
    fn rule_4_a_spring_valve_cannot_be_a_toggle() {
        assert_eq!(
            broken("actuation = \"momentary\"", "actuation = \"toggle\""),
            LintError::ToggleWithSpring
        );
    }

    #[test]
    fn rule_5_an_actuator_on_a_fixed_component_is_rejected() {
        let text = GOOD_VALVE
            .replace("actuation = \"momentary\"", "actuation = \"none\"")
            .replace("spring_return = true", "spring_return = false")
            .replace("\n[spring]\nside = \"right\"\n", "\n");
        assert_eq!(lint(&parse(&text)), Err(LintError::UnexpectedActuator));
    }

    #[test]
    fn rule_6_a_primitive_outside_the_box_is_rejected() {
        assert!(matches!(
            broken(
                "points = [0.78, 0.32, 0.78, 0.60]",
                "points = [0.78, 0.32, 0.78, 1.20]"
            ),
            LintError::Geometry(_)
        ));
    }

    #[test]
    fn rule_6_a_degenerate_primitive_is_rejected() {
        assert!(matches!(
            broken(
                "points = [0.78, 0.32, 0.78, 0.60]",
                "points = [0.78, 0.32, 0.78, 0.32]"
            ),
            LintError::Geometry(_)
        ));
        assert!(matches!(
            broken(
                "points = [0.78, 0.32, 0.78, 0.60]\nline_class = \"working\"",
                "points = [0.78, 0.32, 0.78, 0.60]\nline_class = \"working\"\nstroke = 0.2"
            ),
            LintError::Geometry(_)
        ));
    }

    #[test]
    fn rule_7_a_line_without_a_class_is_rejected() {
        assert_eq!(
            broken(
                "points = [0.78, 0.32, 0.78, 0.60]\nline_class = \"working\"",
                "points = [0.78, 0.32, 0.78, 0.60]"
            ),
            LintError::MissingLineClass("polyline")
        );
    }

    #[test]
    fn rule_8_a_base_primitive_inside_the_body_is_rejected() {
        let text = GOOD_VALVE.replace(
            "[behaviour]",
            "[[symbol]]\nkind = \"line\"\npoints = [0.30, 0.40, 0.40, 0.40]\nline_class = \"working\"\n\n[behaviour]",
        );
        assert_eq!(lint(&parse(&text)), Err(LintError::BodyIntrusion));
    }

    #[test]
    fn a_cylinder_may_declare_its_return_spring() {
        let text = r#"
schema = 2
id = "cylinder.test"
name = "test cylinder"
category = "actuator"
kinematics = "linear_actuator"

[spring]
side = "right"

[[port]]
id = "1"
role = "working"
position = [0.0, 0.5]

[behaviour]
type = "cylinder"
bore_mm = 20.0
stroke_mm = 100.0
dead_length_mm = 5.0
moving_mass_kg = 0.5
spring_rate = 250.0
spring_preload = 5.0
damping = 2.0
friction = 1.5

[[symbol]]
kind = "rect"
points = [0.08, 0.28, 0.72, 0.44]

[[symbol]]
kind = "line"
points = [0.0, 0.5, 0.08, 0.5]
line_class = "working"
"#;
        assert_eq!(lint(&parse(text)), Ok(()));
    }
}
