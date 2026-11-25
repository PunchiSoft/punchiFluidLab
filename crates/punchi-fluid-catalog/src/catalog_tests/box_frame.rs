// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Box, frame and family tests for the conditioning files.
//!
//! Split from `catalog_tests.rs` to keep both files under the architecture budget; the
//! helpers (`write`, `Catalog`, `tempfile`) come from the parent test module.

use super::*;
use crate::schema::LineClass;

const BOXED_THROTTLE: &str = r#"
schema = 3
id = "throttle.boxed"
name = "Boxed throttle"
category = "throttle"
box = [46, 25]

[[port]]
id = "1"
role = "working"
position = [0.0, 0.5]

[[port]]
id = "2"
role = "working"
position = [1.0, 0.5]

[[symbol]]
kind = "line"
points = [0.0, 0.5, 1.0, 0.5]
line_class = "working"

[behaviour]
type = "throttle"
conductance = 1.0e-9
"#;

#[test]
fn a_declared_box_is_read_from_the_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    write(dir.path(), "throttle.toml", BOXED_THROTTLE);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.problems().is_empty(), "{:?}", catalog.problems());
    let definition = catalog.get("throttle.boxed").expect("the file loads");
    assert_eq!(definition.declared_box_mm(), Some([46.0, 25.0]));
    // A definition without the key keeps the canonical square, not a made-up size.
    let valve = catalog
        .get("valve.3_2.spring_return")
        .expect("the built-in valve");
    assert_eq!(valve.declared_box_mm(), None);
}

#[test]
fn a_box_past_its_bounds_is_rejected_with_the_limit_named() {
    for (value, expected) in [
        ("0.9, 20", "between 1 and 200"),
        ("20, 0.9", "between 1 and 200"),
        ("200.1, 20", "between 1 and 200"),
        ("20, 200.1", "between 1 and 200"),
        ("0, 20", "finite, positive"),
        ("20, 0", "finite, positive"),
    ] {
        let text = BOXED_THROTTLE.replace("46, 25", value);
        let dir = tempfile::tempdir().expect("temp dir");
        write(dir.path(), "throttle.toml", &text);
        let catalog = Catalog::builtin().with_user_dir(dir.path());
        assert!(
            catalog.get("throttle.boxed").is_none(),
            "box [{value}] must be refused"
        );
        let problem = format!("{:?}", catalog.problems());
        assert!(
            problem.contains("`box`") && problem.contains(expected),
            "box [{value}] should name its limit, got {problem}"
        );
    }
}

#[test]
fn a_frame_must_be_a_real_rectangle_inside_the_box() {
    for (value, expected) in [
        ("0.1, 0.1, 1.1, 0.9", "inside the symbol box"),
        ("0.1, 0.1, 0.9, 0.0", "increasing order"),
        ("0.9, 0.1, 0.1, 0.9", "increasing order"),
    ] {
        let text = BOXED_THROTTLE.replace(
            "box = [46, 25]",
            &format!("box = [46, 25]\nframe = [{value}]"),
        );
        let dir = tempfile::tempdir().expect("temp dir");
        write(dir.path(), "throttle.toml", &text);
        let catalog = Catalog::builtin().with_user_dir(dir.path());
        assert!(
            catalog.get("throttle.boxed").is_none(),
            "frame [{value}] must be refused"
        );
        let problem = format!("{:?}", catalog.problems());
        assert!(
            problem.contains("`frame`") && problem.contains(expected),
            "frame [{value}] should name its rule, got {problem}"
        );
    }
}

#[test]
fn the_two_service_units_and_the_cylinders_declare_their_box() {
    let catalog = Catalog::builtin();
    let cases = [
        ("preparation.service_unit", [46.0, 25.0]),
        ("preparation.service_unit_simple", [40.0, 20.0]),
        ("preparation.filter_regulator_simple", [40.0, 20.0]),
        ("cylinder.single_acting_20_100", [44.0, 20.0]),
        ("cylinder.double_acting_32_200", [44.0, 20.0]),
    ];
    for (id, expected) in cases {
        let definition = catalog.get(id).expect("the built-in exists");
        assert_eq!(
            definition.declared_box_mm(),
            Some(expected),
            "{id} must declare its box in its own file"
        );
    }
}

#[test]
fn the_filter_regulator_is_the_filter_and_regulator_without_a_lubricator() {
    let catalog = Catalog::builtin();
    let definition = catalog
        .get("preparation.filter_regulator")
        .expect("the filter-regulator is a built-in");
    assert_eq!(definition.declared_box_mm(), Some([46.0, 25.0]));
    assert_eq!(definition.port.len(), 2);
    // The filter's diamond is the one unfilled four-sided polygon; the vent is a triangle
    // and the two facing arrows are filled. A lubricator diamond would make two, and this
    // form is the filter-regulator, not the FRL.
    let diamonds = definition
        .symbol
        .iter()
        .filter(|primitive| {
            primitive.kind == PrimitiveKind::Polygon
                && primitive.points.len() == 8
                && !primitive.filled
        })
        .count();
    assert_eq!(diamonds, 1, "one filter diamond and no lubricator diamond");
    // The gauge and its tap are the two circles.
    let circles = definition
        .symbol
        .iter()
        .filter(|primitive| primitive.kind == PrimitiveKind::Circle)
        .count();
    assert_eq!(circles, 2);
    // The frame is the chain line, not a drain dash.
    assert!(
        definition
            .symbol
            .iter()
            .any(|primitive| primitive.line_class == Some(LineClass::Boundary)),
        "the filter-regulator must draw its chain-line frame"
    );
}

#[test]
fn the_two_drain_filters_are_separate_files() {
    let catalog = Catalog::builtin();
    let manual = catalog
        .get("filter.air_manual_drain")
        .expect("the manual drain filter");
    let auto = catalog
        .get("filter.air_auto_drain")
        .expect("the automatic drain filter");
    assert_ne!(manual.id, auto.id);
    assert_ne!(manual.name, auto.name);
    assert_eq!(manual.port.len(), 2);
    assert_eq!(auto.port.len(), 2);
    // Both carry the bowl chord and the stem from the bottom vertex; the automatic one is
    // its own file, so a user chooses the drain by choosing the component.
    for definition in [manual, auto] {
        assert!(
            definition
                .symbol
                .iter()
                .any(|primitive| primitive.kind == PrimitiveKind::Line
                    && primitive.points == [0.5, 0.72, 0.5, 0.9]),
            "{} has no drain stem",
            definition.id
        );
        assert!(
            definition
                .symbol
                .iter()
                .filter(|primitive| {
                    primitive.kind == PrimitiveKind::Line
                        && primitive.points.len() == 4
                        && (primitive.points[1] - 0.642).abs() < 1.0e-9
                })
                .count()
                >= 1,
            "{} has no bowl chord",
            definition.id
        );
    }
}
