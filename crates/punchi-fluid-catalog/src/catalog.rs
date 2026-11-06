// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Collection of definitions, built from directories of files.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::error::CatalogError;
use crate::schema::{
    Behaviour, Category, ComponentDefinition, Origin, Primitive, PrimitiveKind, SCHEMA_MIN,
    SCHEMA_VERSION,
};

/// Every definition this build can use, indexed by identifier.
///
/// Building the catalog never fails as a whole: a file with a mistake is skipped
/// and reported through [`Catalog::problems`], so one bad valve cannot stop the
/// application from starting.
#[derive(Debug, Default)]
pub struct Catalog {
    entries: BTreeMap<String, ComponentDefinition>,
    problems: Vec<CatalogError>,
    hidden: BTreeSet<String>,
}

impl Catalog {
    /// Empty catalog.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Catalog with the definitions compiled into the binary.
    #[must_use]
    pub fn builtin() -> Self {
        let mut catalog = Self::new();
        for (name, text) in crate::builtin::FILES {
            catalog.add_text(Origin::Builtin, Path::new("builtin").join(name), text);
        }
        catalog
    }

    /// Catalog with the built-in definitions plus the ones in `dir`.
    #[must_use]
    pub fn with_user_dir(mut self, dir: &Path) -> Self {
        self.load_dir(dir);
        self
    }

    /// Reads every `.toml` file of `dir`, in file-name order.
    ///
    /// A missing directory is not a problem: it only means the user has not
    /// created any custom component yet.
    pub fn load_dir(&mut self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file()
                    && path
                        .extension()
                        .is_some_and(|extension| extension == "toml")
            })
            .collect();
        paths.sort();

        for path in paths {
            match std::fs::read_to_string(&path) {
                Ok(text) => self.add_text(Origin::User, &path, &text),
                Err(source) => self.problems.push(CatalogError::Read { path, source }),
            }
        }
    }

    /// Adds one definition from `text`, reporting any problem it has.
    ///
    /// Later definitions override earlier ones with the same identifier, which is
    /// how a user file replaces a built-in one.
    pub fn add_text(&mut self, origin: Origin, path: impl Into<PathBuf>, text: &str) {
        let path = path.into();
        let mut definition: ComponentDefinition = match toml::from_str(text) {
            Ok(definition) => definition,
            Err(source) => {
                self.problems.push(CatalogError::Parse {
                    path,
                    source: Box::new(source),
                });
                return;
            }
        };
        definition.source = Some(path.clone());
        definition.origin = origin;

        if !(SCHEMA_MIN..=SCHEMA_VERSION).contains(&definition.schema) {
            self.problems.push(CatalogError::UnsupportedSchema {
                path,
                found: definition.schema,
            });
            return;
        }

        if let Err(reason) = validate(&definition, &path) {
            self.problems.push(reason);
            return;
        }

        if definition.disabled {
            self.hidden.insert(definition.id.clone());
            self.entries.remove(&definition.id);
            return;
        }

        self.hidden.remove(&definition.id);
        self.entries.insert(definition.id.clone(), definition);
    }

    /// Definition with this identifier, if it is enabled.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ComponentDefinition> {
        self.entries.get(id)
    }

    /// Every enabled definition, ordered by identifier.
    pub fn iter(&self) -> impl Iterator<Item = &ComponentDefinition> {
        self.entries.values()
    }

    /// Definitions of one family.
    pub fn by_category(&self, category: Category) -> impl Iterator<Item = &ComponentDefinition> {
        self.entries
            .values()
            .filter(move |definition| definition.category == category)
    }

    /// Identifiers that a user file marked as `disabled`.
    #[must_use]
    pub fn hidden(&self) -> &BTreeSet<String> {
        &self.hidden
    }

    /// Files that were skipped, with the reason.
    #[must_use]
    pub fn problems(&self) -> &[CatalogError] {
        &self.problems
    }

    /// Number of enabled definitions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when nothing loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Checks that a parsed file describes a component that can be simulated.
fn validate(definition: &ComponentDefinition, path: &Path) -> Result<(), CatalogError> {
    check_identity(definition, path)?;
    check_ports(definition, path)?;

    for primitive in &definition.symbol {
        check_primitive(primitive).map_err(|reason| invalid(path, &reason))?;
    }

    crate::lint::lint(definition).map_err(|error| invalid(path, &error.to_string()))?;

    match &definition.behaviour {
        Behaviour::Supply { pressure_bar } => {
            if !pressure_bar.is_finite() || *pressure_bar <= 0.0 {
                return Err(invalid(path, "supply pressure must be positive"));
            }
            Ok(())
        }
        Behaviour::Throttle { conductance } => {
            if !conductance.is_finite() || *conductance <= 0.0 {
                return Err(invalid(path, "throttle conductance must be positive"));
            }
            Ok(())
        }
        Behaviour::Cylinder {
            bore_mm,
            stroke_mm,
            dead_length_mm,
            moving_mass_kg,
            spring_rate,
            spring_preload,
            damping,
            friction,
        } => {
            let positive = [
                ("bore_mm", *bore_mm),
                ("stroke_mm", *stroke_mm),
                ("moving_mass_kg", *moving_mass_kg),
            ];
            if let Some((field, value)) = positive
                .into_iter()
                .find(|(_, value)| !value.is_finite() || *value <= 0.0)
            {
                return Err(invalid(
                    path,
                    &format!("{field} must be positive, found {value}"),
                ));
            }
            let non_negative = [
                ("dead_length_mm", *dead_length_mm),
                ("spring_rate", *spring_rate),
                ("spring_preload", *spring_preload),
                ("damping", *damping),
                ("friction", *friction),
            ];
            if let Some((field, value)) = non_negative
                .into_iter()
                .find(|(_, value)| !value.is_finite() || *value < 0.0)
            {
                return Err(invalid(
                    path,
                    &format!("{field} must be zero or positive, found {value}"),
                ));
            }
            Ok(())
        }
        Behaviour::Valve {
            positions,
            default_position,
            ..
        } => check_valve(definition, path, positions, *default_position),
    }
}

fn invalid(path: &Path, reason: &str) -> CatalogError {
    CatalogError::Invalid {
        path: path.to_path_buf(),
        reason: reason.to_owned(),
    }
}

fn check_identity(definition: &ComponentDefinition, path: &Path) -> Result<(), CatalogError> {
    if definition.id.trim().is_empty() {
        return Err(invalid(path, "id must not be empty"));
    }
    if !definition.id.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || "._-".contains(character)
    }) {
        return Err(invalid(
            path,
            &format!(
                "id `{}` may only use lowercase letters, digits, dot, underscore and dash",
                definition.id
            ),
        ));
    }
    if definition.name.trim().is_empty() {
        return Err(invalid(path, "name must not be empty"));
    }
    Ok(())
}

fn check_ports(definition: &ComponentDefinition, path: &Path) -> Result<(), CatalogError> {
    if definition.port.is_empty() {
        return Err(invalid(path, "a component needs at least one port"));
    }

    let mut seen = BTreeSet::new();
    for port in &definition.port {
        if port.id.trim().is_empty() {
            return Err(invalid(path, "port id must not be empty"));
        }
        if !seen.insert(port.id.as_str()) {
            return Err(invalid(
                path,
                &format!("port `{}` is defined twice", port.id),
            ));
        }
        let inside = port
            .position
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value));
        if !inside {
            return Err(invalid(
                path,
                &format!(
                    "port `{}` position must be inside the symbol box, 0..=1",
                    port.id
                ),
            ));
        }
    }
    Ok(())
}

fn check_valve(
    definition: &ComponentDefinition,
    path: &Path,
    positions: &[crate::schema::ValvePosition],
    default_position: usize,
) -> Result<(), CatalogError> {
    if positions.len() < 2 {
        return Err(invalid(
            path,
            &format!(
                "a valve needs at least two positions, found {}",
                positions.len()
            ),
        ));
    }
    if default_position >= positions.len() {
        return Err(invalid(
            path,
            &format!(
                "default_position {default_position} does not exist, the valve has {}",
                positions.len()
            ),
        ));
    }

    for (index, position) in positions.iter().enumerate() {
        if position.name.trim().is_empty() {
            return Err(invalid(path, &format!("position {index} needs a name")));
        }
        for connection in &position.connection {
            if definition.port(&connection.from).is_none() {
                return Err(invalid(
                    path,
                    &format!(
                        "position `{}` connects unknown port `{}`",
                        position.name, connection.from
                    ),
                ));
            }
            if definition.port(&connection.to).is_none() {
                return Err(invalid(
                    path,
                    &format!(
                        "position `{}` connects unknown port `{}`",
                        position.name, connection.to
                    ),
                ));
            }
            if connection.from == connection.to {
                return Err(invalid(
                    path,
                    &format!(
                        "position `{}` connects port `{}` to itself",
                        position.name, connection.from
                    ),
                ));
            }
            if !connection.conductance.is_finite() || connection.conductance <= 0.0 {
                return Err(invalid(
                    path,
                    &format!("position `{}` needs a positive conductance", position.name),
                ));
            }
        }
        for primitive in &position.symbol {
            check_primitive(primitive).map_err(|reason| invalid(path, &reason))?;
        }
    }
    Ok(())
}

/// Checks the coordinate count each primitive kind requires.
fn check_primitive(primitive: &Primitive) -> Result<(), String> {
    let points = &primitive.points;
    if !points.iter().all(|value| value.is_finite()) {
        return Err(String::from("symbol coordinates must be finite numbers"));
    }
    if !primitive.stroke.is_finite() || primitive.stroke <= 0.0 {
        return Err(String::from("symbol stroke must be positive"));
    }

    let valid = match primitive.kind {
        PrimitiveKind::Line => points.len() == 4,
        PrimitiveKind::Polyline => points.len() >= 4 && points.len() % 2 == 0,
        PrimitiveKind::Polygon => points.len() >= 6 && points.len() % 2 == 0,
        PrimitiveKind::Rect => points.len() == 4 && points[2].abs() > 0.0 && points[3].abs() > 0.0,
        PrimitiveKind::Circle => points.len() == 3 && points[2] > 0.0,
        PrimitiveKind::Arc => points.len() == 5 && points[2] > 0.0,
    };

    if valid {
        Ok(())
    } else {
        Err(format!(
            "symbol primitive `{:?}` has {} values, which is not valid for that kind",
            primitive.kind,
            points.len()
        ))
    }
}

#[cfg(test)]
mod tests {
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
        assert!(catalog.get("supply.workshop").is_some());
        assert!(catalog.get("cylinder.single_acting_20_100").is_some());
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
id = "supply.workshop"
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
        let supply = catalog.get("supply.workshop").expect("overridden");
        assert_eq!(supply.name, "Taller 8 bar");
        assert_eq!(supply.origin, Origin::User);
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
}
