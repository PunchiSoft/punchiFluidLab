// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for loading, overriding and validating a definition.

use std::fmt::Write as _;

use super::*;

fn write(dir: &Path, name: &str, text: &str) {
    std::fs::write(dir.join(name), text).expect("temp dir is writable");
}

const EXTRA_VALVE: &str = r#"
schema = 2
id = "valve.5_2.test"
name = "5/2 valve"
category = "valve"
body = [0.06, 0.28, 0.44, 0.22]

[actuator]
kind = "lever"
side = "left"

[[port]]
id = "1"
role = "supply"
position = [0.28, 1.0]

[[port]]
id = "2"
role = "working"
position = [0.28, 0.0]

[behaviour]
type = "valve"
actuation = "toggle"
[[behaviour.positions]]
name = "left"
[[behaviour.positions.connection]]
from = "1"
to = "2"
conductance = 1.0e-9
[[behaviour.positions]]
name = "right"
blocked = ["1"]
"#;

#[test]
fn builtin_catalog_loads_without_problems() {
    let catalog = Catalog::builtin();
    assert!(
        catalog.problems().is_empty(),
        "built-in files must always be valid: {:?}",
        catalog.problems()
    );
    assert!(catalog.get("valve.3_2.spring_return").is_some());
    assert!(catalog.get("supply.compressor").is_some());
    assert!(catalog.get("cylinder.single_acting_20_100").is_some());
    assert!(catalog.get("flow.meter").is_some());
    assert!(catalog.len() >= 4);
}

#[test]
fn a_new_supply_is_four_bar() {
    // 4 bar gauge is the product default the owner chose on 2026-11-20. The three built-in
    // supplies are the catalog value a new placement reads, so they have to agree.
    let catalog = Catalog::builtin();
    for id in [
        "supply.compressor",
        "supply.compressor_adjustable",
        "supply.network",
    ] {
        let definition = catalog.get(id).expect("the built-in supply exists");
        assert!(
            matches!(definition.behaviour, Behaviour::Supply { .. }),
            "{id} is not a supply"
        );
        let Behaviour::Supply { pressure_bar } = definition.behaviour else {
            continue;
        };
        assert!(
            (pressure_bar - 4.0).abs() < f64::EPSILON,
            "{id} declares {pressure_bar} bar, not the 4 bar default"
        );
    }
}

#[test]
fn user_directory_adds_and_overrides_definitions() {
    let dir = tempfile::tempdir().expect("temp dir");
    write(dir.path(), "valve.toml", EXTRA_VALVE);
    write(
        dir.path(),
        "supply.toml",
        r#"
schema = 2
id = "supply.compressor"
name = "Taller 8 bar"
category = "supply"
[[port]]
id = "1"
role = "supply"
position = [0.5, 1.0]
[behaviour]
type = "supply"
pressure_bar = 8.0
"#,
    );

    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.problems().is_empty(), "{:?}", catalog.problems());
    assert!(catalog.get("valve.5_2.test").is_some());
    let supply = catalog.get("supply.compressor").expect("overridden");
    assert_eq!(supply.name, "Taller 8 bar");
    assert_eq!(supply.origin, Origin::User);
}

#[test]
fn a_plug_caps_one_port_and_a_silencer_needs_an_exhaust() {
    let dir = tempfile::tempdir().expect("temp dir");
    let two_ports = r#"
schema = 3
id = "plug.two"
name = "Plug with two ports"
category = "accessory"
[[port]]
id = "1"
role = "working"
position = [0.0, 0.5]
[[port]]
id = "2"
role = "working"
position = [1.0, 0.5]
[behaviour]
type = "plug"
"#;
    write(dir.path(), "plug.toml", two_ports);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.get("plug.two").is_none(), "a plug caps one line");
    assert!(
        catalog
            .problems()
            .iter()
            .any(|problem| problem.to_string().contains("exactly one port")),
        "{:?}",
        catalog.problems()
    );

    let working_silencer = r#"
schema = 3
id = "silencer.working"
name = "Silencer on a working port"
category = "accessory"
[[port]]
id = "1"
role = "working"
position = [0.0, 0.5]
[behaviour]
type = "silencer"
conductance = 4.0e-10
"#;
    write(dir.path(), "silencer.toml", working_silencer);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(
        catalog.get("silencer.working").is_none(),
        "a silencer vents an exhaust, not the line the circuit uses"
    );
    assert!(
        catalog
            .problems()
            .iter()
            .any(|problem| problem.to_string().contains("exhaust port")),
        "{:?}",
        catalog.problems()
    );
}

#[test]
fn a_component_with_too_many_ports_is_refused() {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut too_many = String::from(
        "schema = 3\nid = \"plug.many\"\nname = \"Many ports\"\n\
         category = \"accessory\"\n",
    );
    for index in 0..=MAX_PORTS {
        write!(
            too_many,
            "[[port]]\nid = \"p{index}\"\nrole = \"exhaust\"\nposition = [0.5, 0.5]\n"
        )
        .expect("writing to a String cannot fail");
    }
    too_many.push_str("[behaviour]\ntype = \"plug\"\n");
    write(dir.path(), "many.toml", &too_many);

    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.get("plug.many").is_none());
    let expected = format!("at most {MAX_PORTS} ports");
    assert!(
        catalog
            .problems()
            .iter()
            .any(|problem| problem.to_string().contains(&expected)),
        "{:?}",
        catalog.problems()
    );
}

#[test]
fn a_recorder_has_no_ports_and_a_bounded_window() {
    let dir = tempfile::tempdir().expect("temp dir");
    // A recorder is the one component that observes instead of connecting.
    write(
        dir.path(),
        "chart.toml",
        r#"
schema = 3
id = "chart.recorder"
name = "Strip chart"
category = "sensor"
[behaviour]
type = "recorder"
"#,
    );
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.problems().is_empty(), "{:?}", catalog.problems());
    let definition = catalog.get("chart.recorder").expect("the file loads");
    let Behaviour::Recorder { span_s, rate_hz } = definition.behaviour else {
        return;
    };
    assert!(
        (span_s - 60.0).abs() < f64::EPSILON,
        "the window defaults to a minute"
    );
    assert!(
        (rate_hz - 10.0).abs() < f64::EPSILON,
        "the rate defaults to ten hertz"
    );

    // A port would promise a connection the model does not use.
    let mut with_port = String::from(
        r#"
schema = 3
id = "chart.recorder.port"
name = "Chart with a port"
category = "sensor"
[[port]]
id = "1"
role = "working"
position = [0.0, 0.5]
[behaviour]
type = "recorder"
"#,
    );
    write(dir.path(), "chart_port.toml", &with_port);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.get("chart.recorder.port").is_none());
    assert!(
        catalog
            .problems()
            .iter()
            .any(|problem| problem.to_string().contains("a recorder has no ports")),
        "{:?}",
        catalog.problems()
    );

    // The product of window and rate is what the buffer costs, so it is bounded.
    with_port = with_port
        .replace("chart.recorder.port", "chart.recorder.long")
        .replace(
            "type = \"recorder\"",
            "type = \"recorder\"\nspan_s = 600.0\nrate_hz = 50.0",
        );
    let without_port = with_port.replace(
        "[[port]]\nid = \"1\"\nrole = \"working\"\nposition = [0.0, 0.5]\n",
        "",
    );
    write(dir.path(), "chart_long.toml", &without_port);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.get("chart.recorder.long").is_none());
    assert!(
        catalog
            .problems()
            .iter()
            .any(|problem| problem.to_string().contains("at most")),
        "{:?}",
        catalog.problems()
    );
}

#[test]
fn a_flow_meter_needs_two_working_ports_a_conductance_and_a_scale() {
    let dir = tempfile::tempdir().expect("temp dir");
    let one_port = r#"
schema = 3
id = "flow.meter.one"
name = "Meter with one port"
category = "sensor"
[[port]]
id = "1"
role = "working"
position = [0.0, 0.5]
[behaviour]
type = "flow_meter"
conductance = 2.0e-9
nominal_l_min = 100.0
"#;
    write(dir.path(), "one.toml", one_port);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(
        catalog.get("flow.meter.one").is_none(),
        "a meter in the line needs a port to leave by"
    );
    assert!(
        catalog
            .problems()
            .iter()
            .any(|problem| problem.to_string().contains("two working ports")),
        "{:?}",
        catalog.problems()
    );

    let no_scale = one_port
            .replace("flow.meter.one", "flow.meter.two")
            .replace("nominal_l_min = 100.0", "nominal_l_min = 0.0")
            .replace(
                "[[port]]\nid = \"1\"\nrole = \"working\"\nposition = [0.0, 0.5]",
                "[[port]]\nid = \"1\"\nrole = \"working\"\nposition = [0.0, 0.5]\n\n[[port]]\nid = \"2\"\nrole = \"working\"\nposition = [1.0, 0.5]",
            );
    write(dir.path(), "two.toml", &no_scale);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.get("flow.meter.two").is_none());
    assert!(
        catalog.problems().iter().any(|problem| problem
            .to_string()
            .contains("nominal flow must be positive")),
        "{:?}",
        catalog.problems()
    );
}

#[test]
fn a_disabled_file_hides_the_builtin_with_the_same_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    write(
        dir.path(),
        "hide.toml",
        r#"
schema = 2
id = "throttle.fixed"
name = "hidden"
category = "throttle"
disabled = true
[[port]]
id = "1"
role = "working"
position = [0.0, 0.5]
[behaviour]
type = "throttle"
conductance = 1.0e-9
"#,
    );

    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(catalog.problems().is_empty(), "{:?}", catalog.problems());
    assert!(catalog.get("throttle.fixed").is_none());
    assert!(catalog.hidden().contains("throttle.fixed"));
}

#[test]
fn a_broken_file_is_reported_and_the_rest_still_loads() {
    let dir = tempfile::tempdir().expect("temp dir");
    write(dir.path(), "bad-toml.toml", "schema = 1\nid = [");
    write(
        dir.path(),
        "bad-connection.toml",
        r#"
schema = 2
id = "valve.broken"
name = "broken"
category = "valve"
body = [0.06, 0.28, 0.44, 0.22]
[actuator]
kind = "lever"
side = "left"
[[port]]
id = "1"
role = "supply"
position = [0.28, 1.0]
[behaviour]
type = "valve"
actuation = "toggle"
[[behaviour.positions]]
name = "a"
[[behaviour.positions.connection]]
from = "1"
to = "9"
conductance = 1.0e-9
[[behaviour.positions]]
name = "b"
blocked = ["1"]
"#,
    );
    write(dir.path(), "good.toml", EXTRA_VALVE);

    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert_eq!(catalog.problems().len(), 2, "{:?}", catalog.problems());
    assert!(catalog.get("valve.5_2.test").is_some());
    assert!(catalog.get("valve.broken").is_none());
}

#[test]
fn a_typo_in_a_key_is_rejected_instead_of_ignored() {
    let dir = tempfile::tempdir().expect("temp dir");
    write(
        dir.path(),
        "typo.toml",
        r#"
schema = 1
id = "valve.typo"
name = "typo"
category = "valve"
sumary = "misspelled"
[[port]]
id = "1"
role = "supply"
position = [0.5, 1.0]
[behaviour]
type = "valve"
actuation = "toggle"
[[behaviour.positions]]
name = "a"
[[behaviour.positions]]
name = "b"
"#,
    );

    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert_eq!(catalog.problems().len(), 1);
    assert!(matches!(catalog.problems()[0], CatalogError::Parse { .. }));
}

#[test]
fn unsupported_schema_is_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    write(
        dir.path(),
        "future.toml",
        r#"
schema = 99
id = "valve.future"
name = "future"
category = "valve"
[[port]]
id = "1"
role = "supply"
position = [0.5, 1.0]
[behaviour]
type = "supply"
pressure_bar = 6.0
"#,
    );

    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(matches!(
        catalog.problems()[0],
        CatalogError::UnsupportedSchema { found: 99, .. }
    ));
}

#[test]
fn missing_directory_is_not_an_error() {
    let catalog = Catalog::builtin().with_user_dir(Path::new("/nonexistent/catalog"));
    assert!(catalog.problems().is_empty());
    assert!(!catalog.is_empty());
}

#[test]
fn a_source_triangle_points_at_its_connection() {
    // A supply delivers through the point of its triangle, and the port sits on that point.
    // A triangle standing on its base reads as a source that delivers upwards, away from
    // the hose: the owner caught exactly that on the sheet.
    let catalog = Catalog::builtin();
    for id in ["supply.network", "supply.compressor"] {
        let definition = catalog.get(id).expect("the built-in supply exists");
        let port = definition.port.first().expect("a supply has a port");
        let triangle = definition
            .symbol
            .iter()
            .find(|primitive| matches!(primitive.kind, PrimitiveKind::Polygon))
            .expect("the supply draws a triangle");
        // The apex is the vertex furthest from the base on the vertical axis; the reference
        // draws it at the top, with the stem going up to the connection.
        let ys: Vec<f64> = triangle
            .points
            .chunks(2)
            .filter_map(|pair| pair.get(1).copied())
            .collect();
        let apex = ys
            .iter()
            .copied()
            .reduce(f64::min)
            .expect("the triangle has points");
        let apex_y = ys
            .iter()
            .copied()
            .find(|value| (value - apex).abs() < f64::EPSILON)
            .unwrap_or(apex);
        assert!(
            (apex_y - port.position[1]).abs() < 0.35,
            "{id}: the apex is at {apex_y:.2} and the port at {:.2}",
            port.position[1]
        );
        assert!(
            apex_y < 0.5,
            "{id}: the apex must be at the top, on the stem, not at the bottom"
        );
    }
}

#[test]
fn a_contact_needs_two_electrical_terminals() {
    let dir = tempfile::tempdir().expect("temp dir");
    let with_a_working_port = r#"
schema = 3
id = "electrical.contact.bad"
name = "Contact with a pneumatic port"
category = "sensor"
[[port]]
id = "1"
role = "electrical"
position = [0.0, 0.5]
[[port]]
id = "2"
role = "working"
position = [1.0, 0.5]
[behaviour]
type = "contact"
"#;
    write(dir.path(), "contact.toml", with_a_working_port);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(
        catalog.get("electrical.contact.bad").is_none(),
        "a contact is an electrical part, not a pneumatic one"
    );
    assert!(
        catalog
            .problems()
            .iter()
            .any(|problem| problem.to_string().contains("two electrical terminals")),
        "{:?}",
        catalog.problems()
    );
}

#[test]
fn a_relay_names_two_coil_and_two_contact_terminals() {
    let valid = r#"
schema = 3
id = "electrical.relay.test"
name = "Relay"
category = "sensor"
[[port]]
id = "A1"
role = "electrical"
position = [0.0, 0.30]
[[port]]
id = "A2"
role = "electrical"
position = [0.0, 0.70]
[[port]]
id = "11"
role = "electrical"
position = [1.0, 0.30]
[[port]]
id = "14"
role = "electrical"
position = [1.0, 0.70]
[behaviour]
type = "relay"
coil = ["A1", "A2"]
contact = ["11", "14"]
"#;
    let dir = tempfile::tempdir().expect("temp dir");
    write(dir.path(), "relay.toml", valid);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(
        catalog.get("electrical.relay.test").is_some(),
        "{:?}",
        catalog.problems()
    );

    // A terminal used by both pairs would make the net ambiguous about what the coil
    // closes, so the file is refused instead of guessed.
    let overlapping = valid.replace("contact = [\"11\", \"14\"]", "contact = [\"11\", \"A1\"]");
    let dir = tempfile::tempdir().expect("temp dir");
    write(dir.path(), "relay.toml", &overlapping);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(
        catalog.get("electrical.relay.test").is_none(),
        "a relay may not share a terminal between coil and contact"
    );
    assert!(
        catalog
            .problems()
            .iter()
            .any(|problem| problem.to_string().contains("used twice")),
        "{:?}",
        catalog.problems()
    );
}

#[test]
fn a_timer_declares_a_bounded_delay_and_an_instant_relay_does_not() {
    let relay = r#"
schema = 3
id = "electrical.timer.test"
name = "Timer"
category = "sensor"
[[port]]
id = "A1"
role = "electrical"
position = [0.0, 0.30]
[[port]]
id = "A2"
role = "electrical"
position = [0.0, 0.70]
[[port]]
id = "11"
role = "electrical"
position = [1.0, 0.30]
[[port]]
id = "14"
role = "electrical"
position = [1.0, 0.70]
[behaviour]
type = "relay"
coil = ["A1", "A2"]
contact = ["11", "14"]
kind = "ton"
delay_s = 2.0
"#;
    let dir = tempfile::tempdir().expect("temp dir");
    write(dir.path(), "timer.toml", relay);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(
        catalog.get("electrical.timer.test").is_some(),
        "{:?}",
        catalog.problems()
    );

    let instant_with_delay = relay.replace("kind = \"ton\"", "kind = \"instant\"");
    let dir = tempfile::tempdir().expect("temp dir");
    write(dir.path(), "timer.toml", &instant_with_delay);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(
        catalog.get("electrical.timer.test").is_none(),
        "an instant relay must not carry a delay"
    );

    let too_long = relay.replace("delay_s = 2.0", "delay_s = 7200.0");
    let dir = tempfile::tempdir().expect("temp dir");
    write(dir.path(), "timer.toml", &too_long);
    let catalog = Catalog::builtin().with_user_dir(dir.path());
    assert!(
        catalog.get("electrical.timer.test").is_none(),
        "a delay above one hour is refused"
    );
}

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
