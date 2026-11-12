// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for loading, overriding and validating a definition.

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
