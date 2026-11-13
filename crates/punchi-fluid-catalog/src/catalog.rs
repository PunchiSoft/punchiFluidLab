// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Collection of definitions, built from directories of files.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::error::CatalogError;
use crate::schema::{
    Behaviour, Category, ComponentDefinition, Origin, PortRole, Primitive, PrimitiveKind,
    RelayKind, SCHEMA_MIN, SCHEMA_VERSION,
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

/// Most samples a recorder may keep, the product of its window and its rate.
///
/// The bound belongs to the catalog so a file cannot ask the session for an unbounded
/// buffer; 6 000 samples is 60 s at 100 Hz, or a minute at ten times the default rate.
pub const MAX_RECORDER_SAMPLES: usize = 6_000;

/// Checks that a parsed file describes a component that can be simulated.
fn validate(definition: &ComponentDefinition, path: &Path) -> Result<(), CatalogError> {
    check_identity(definition, path)?;
    check_ports(definition, path)?;
    if matches!(definition.behaviour, Behaviour::Gauge { .. }) {
        let working = definition
            .port
            .iter()
            .filter(|port| port.role == PortRole::Working)
            .count();
        if working != 1 {
            return Err(invalid(path, "a gauge needs exactly one working port"));
        }
    }
    if matches!(definition.behaviour, Behaviour::Recorder { .. }) && !definition.port.is_empty() {
        // A recorder observes a named component; a port on it would promise a connection
        // the model does not use.
        return Err(invalid(path, "a recorder has no ports"));
    }
    if matches!(
        definition.behaviour,
        Behaviour::Plug | Behaviour::Silencer { .. }
    ) {
        let exhaust = definition
            .port
            .iter()
            .filter(|port| port.role == PortRole::Exhaust)
            .count();
        if matches!(definition.behaviour, Behaviour::Plug) {
            if definition.port.len() != 1 {
                return Err(invalid(path, "a blind plug needs exactly one port"));
            }
        } else if definition.port.len() != 1 || exhaust != 1 {
            // A silencer sits on an exhaust and nowhere else: a silencer on a working port
            // would vent the line the circuit is trying to use.
            return Err(invalid(path, "a silencer needs exactly one exhaust port"));
        }
    }
    if matches!(definition.behaviour, Behaviour::FlowMeter { .. }) {
        // A meter sits in the line: one port to arrive and one to leave, and nothing else
        // to connect. An exhaust or a pilot port on a meter would have no meaning.
        let working = definition
            .port
            .iter()
            .filter(|port| port.role == PortRole::Working)
            .count();
        if definition.port.len() != 2 || working != 2 {
            return Err(invalid(
                path,
                "a flow meter needs exactly two working ports",
            ));
        }
    }

    for primitive in &definition.symbol {
        check_primitive(primitive).map_err(|reason| invalid(path, &reason))?;
    }

    crate::lint::lint(definition).map_err(|error| invalid(path, &error.to_string()))?;

    check_behaviour(definition, path)
}

/// Checks the numbers of the behaviour this build knows how to simulate.
fn check_behaviour(definition: &ComponentDefinition, path: &Path) -> Result<(), CatalogError> {
    match &definition.behaviour {
        Behaviour::Supply { pressure_bar } => {
            if !pressure_bar.is_finite() || *pressure_bar <= 0.0 {
                return Err(invalid(path, "supply pressure must be positive"));
            }
            Ok(())
        }
        Behaviour::Gauge { min_bar, max_bar } => {
            if !min_bar.is_finite() || !max_bar.is_finite() || *max_bar <= *min_bar {
                return Err(invalid(
                    path,
                    "gauge scale needs finite bounds with max_bar above min_bar",
                ));
            }
            Ok(())
        }
        Behaviour::Throttle { conductance } => {
            if !conductance.is_finite() || *conductance <= 0.0 {
                return Err(invalid(path, "throttle conductance must be positive"));
            }
            Ok(())
        }
        Behaviour::Recorder { span_s, rate_hz } => check_recorder(path, *span_s, *rate_hz),
        Behaviour::ElectricalSupply { voltage_v } => check_electrical_supply(path, *voltage_v),
        Behaviour::Contact { .. } | Behaviour::LimitSwitch { .. } => {
            check_contact(definition, path)
        }
        Behaviour::Relay {
            coil,
            contact,
            kind,
            delay_s,
            ..
        } => check_relay(definition, path, coil, contact, *kind, *delay_s),
        Behaviour::Lamp | Behaviour::Buzzer => check_indicator(definition, path),
        Behaviour::Silencer { conductance } => {
            if !conductance.is_finite() || *conductance <= 0.0 {
                return Err(invalid(path, "silencer conductance must be positive"));
            }
            Ok(())
        }
        Behaviour::FlowMeter {
            conductance,
            nominal_l_min,
        } => {
            if !conductance.is_finite() || *conductance <= 0.0 {
                return Err(invalid(path, "flow meter conductance must be positive"));
            }
            if !nominal_l_min.is_finite() || *nominal_l_min <= 0.0 {
                return Err(invalid(path, "flow meter nominal flow must be positive"));
            }
            Ok(())
        }
        Behaviour::Cylinder {
            bore_mm,
            rod_mm,
            stroke_mm,
            dead_length_mm,
            moving_mass_kg,
            spring_rate,
            spring_preload,
            damping,
            friction,
        } => check_cylinder(
            path,
            *bore_mm,
            *rod_mm,
            *stroke_mm,
            *dead_length_mm,
            *moving_mass_kg,
            *spring_rate,
            *spring_preload,
            *damping,
            *friction,
        ),
        Behaviour::Valve {
            positions,
            default_position,
            ..
        } => check_valve(definition, path, positions, *default_position),
        // A plug closes a port and a junction joins three: neither declares a number.
        Behaviour::Plug | Behaviour::Junction => Ok(()),
    }
}

/// Checks the window and the rate of a recorder, and the buffer their product asks for.
fn check_recorder(path: &Path, span_s: f64, rate_hz: f64) -> Result<(), CatalogError> {
    if !span_s.is_finite() || !(1.0..=600.0).contains(&span_s) {
        return Err(invalid(path, "a recorder window must be 1 to 600 seconds"));
    }
    if !rate_hz.is_finite() || !(1.0..=50.0).contains(&rate_hz) {
        return Err(invalid(
            path,
            "a recorder rate must be 1 to 50 samples per second",
        ));
    }
    // The count is a small integer, so the comparison is exact in `f64`.
    #[allow(clippy::cast_precision_loss)]
    let wanted = (span_s * rate_hz).round();
    #[allow(clippy::cast_precision_loss)]
    let allowed = MAX_RECORDER_SAMPLES as f64;
    if wanted > allowed {
        return Err(invalid(
            path,
            &format!("a recorder may keep at most {MAX_RECORDER_SAMPLES} samples"),
        ));
    }
    Ok(())
}

/// Checks the numbers of a cylinder: the ones that must be positive and the ones that may be
/// zero, with the rod smaller than the bore it runs in.
#[allow(clippy::too_many_arguments)]
fn check_cylinder(
    path: &Path,
    bore_mm: f64,
    rod_mm: f64,
    stroke_mm: f64,
    dead_length_mm: f64,
    moving_mass_kg: f64,
    spring_rate: f64,
    spring_preload: f64,
    damping: f64,
    friction: f64,
) -> Result<(), CatalogError> {
    let positive = [
        ("bore_mm", bore_mm),
        ("stroke_mm", stroke_mm),
        ("moving_mass_kg", moving_mass_kg),
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
    if !rod_mm.is_finite() || rod_mm < 0.0 || rod_mm >= bore_mm {
        return Err(invalid(
            path,
            &format!("rod_mm must be zero or positive and smaller than the bore, found {rod_mm}"),
        ));
    }
    let non_negative = [
        ("dead_length_mm", dead_length_mm),
        ("spring_rate", spring_rate),
        ("spring_preload", spring_preload),
        ("damping", damping),
        ("friction", friction),
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

/// Checks the rail of the control circuit.
fn check_electrical_supply(path: &Path, voltage_v: f64) -> Result<(), CatalogError> {
    if !voltage_v.is_finite() || voltage_v <= 0.0 {
        return Err(invalid(
            path,
            "an electrical supply needs a positive voltage",
        ));
    }
    Ok(())
}

/// Checks that a contact is a two-terminal electrical part.
fn check_contact(definition: &ComponentDefinition, path: &Path) -> Result<(), CatalogError> {
    let terminals = definition
        .port
        .iter()
        .filter(|port| port.role == PortRole::Electrical)
        .count();
    if terminals != 2 {
        return Err(invalid(
            path,
            "a contact needs exactly two electrical terminals",
        ));
    }
    Ok(())
}

/// A relay names two coil terminals and two contact terminals, and nothing else.
///
/// The pair a terminal belongs to is behaviour: a coil and a contact share the same role,
/// so the file has to say which is which instead of the code guessing from a port number.
fn check_relay(
    definition: &ComponentDefinition,
    path: &Path,
    coil: &[String; 2],
    contact: &[String; 2],
    kind: RelayKind,
    delay_s: f64,
) -> Result<(), CatalogError> {
    if coil[0] == coil[1] || contact[0] == contact[1] {
        return Err(invalid(
            path,
            "a relay coil and its contact each need two distinct terminals",
        ));
    }
    if definition.port.len() != 4 {
        return Err(invalid(
            path,
            "a relay needs exactly four electrical terminals",
        ));
    }
    let mut seen = BTreeSet::new();
    for id in coil.iter().chain(contact.iter()) {
        let electrical = definition
            .port
            .iter()
            .any(|port| port.id == *id && port.role == PortRole::Electrical);
        if !electrical {
            return Err(invalid(
                path,
                &format!("relay terminal `{id}` is missing or is not electrical"),
            ));
        }
        if !seen.insert(id.as_str()) {
            return Err(invalid(
                path,
                &format!("relay terminal `{id}` is used twice"),
            ));
        }
    }
    if !delay_s.is_finite() || !(0.0..=3600.0).contains(&delay_s) {
        return Err(invalid(
            path,
            "a relay delay must be between 0 and 3600 seconds",
        ));
    }
    if kind == RelayKind::Instant && delay_s != 0.0 {
        return Err(invalid(
            path,
            "an instant relay must leave its delay at zero",
        ));
    }
    Ok(())
}

/// An indicator shows one line: one electrical terminal and nothing else.
fn check_indicator(definition: &ComponentDefinition, path: &Path) -> Result<(), CatalogError> {
    if definition.port.len() != 1 || definition.port[0].role != PortRole::Electrical {
        return Err(invalid(
            path,
            "an indicator needs exactly one electrical terminal",
        ));
    }
    Ok(())
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
    if definition.port.is_empty() && !matches!(definition.behaviour, Behaviour::Recorder { .. }) {
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
#[path = "catalog_tests.rs"]
mod tests;
