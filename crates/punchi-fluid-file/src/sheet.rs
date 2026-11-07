// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The saved schematic: placed components, their pipes and their per-instance values.
//!
//! This is a *sheet* format, not the legacy circuit format in [`crate`]: it stores what
//! the editor edits. A file is untrusted input, so every collection is bounded, every
//! number is checked, unknown keys are rejected, and a document that fails validation
//! never leaves the parser. A future release migrates older sheets instead of refusing
//! them.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Format version written by this build.
///
/// 2 added the orientation of a placement — the quarter turns and the mirror. 3 added the
/// designation, the name the user gives a component. An older file reads because both
/// additions have an obvious default: an unturned symbol and an unnamed component.
pub const SHEET_VERSION: u32 = 3;

/// Oldest format version this build still reads.
pub const SHEET_MIN: u32 = 1;

/// Most components a sheet may hold.
pub const MAX_PLACEMENTS: usize = 2_000;

/// Most pipes a sheet may hold.
pub const MAX_PIPES: usize = 4_000;

/// Largest file the reader will load, in bytes.
pub const MAX_BYTES: u64 = 8 * 1024 * 1024;

/// Longest accepted catalog id or port id.
pub const MAX_ID: usize = 128;

/// Largest accepted symbol box side, in millimetres.
pub const MAX_SIDE_MM: f32 = 2_000.0;

/// Longest accepted designation, in characters.
///
/// It matches `punchi_fluid_doc::MAX_DESIGNATION`; the format crate does not depend on the
/// document crate, so the number is stated in both and tested here.
pub const MAX_DESIGNATION: usize = 40;

/// One component, as the sheet stores it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredPlacement {
    /// Identity the pipes use. Unique and non-zero.
    pub id: u64,
    /// Catalog id of the definition.
    pub catalog: String,
    /// Centre x, millimetres.
    pub x: f32,
    /// Centre y, millimetres.
    pub y: f32,
    /// Symbol box width, millimetres.
    pub width: f32,
    /// Symbol box height, millimetres.
    pub height: f32,
    /// Per-instance values, when they differ from the definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit: Option<StoredEdits>,
    /// Clockwise quarter turns, `0..=3`; absent when the symbol is not turned.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub quarter_turns: u8,
    /// `true` when the symbol is reflected on its own vertical axis.
    #[serde(default, skip_serializing_if = "is_false")]
    pub mirrored: bool,
    /// Name the user gave this component; absent when it has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub designation: Option<String>,
}

/// Skips a field that carries the default, so an unturned sheet looks as it always did.
// serde hands the field to the predicate by reference, whatever its size.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(value: &u8) -> bool {
    *value == 0
}

/// Skips a field that carries the default.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !*value
}

/// The fields of a placement that override its definition.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredEdits {
    /// Supply pressure, gauge bar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressure_bar: Option<f64>,
    /// Conductance, `m³ / (s · Pa)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conductance: Option<f64>,
    /// Cylinder stroke, millimetres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_mm: Option<f64>,
    /// Cylinder bore, millimetres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bore_mm: Option<f64>,
    /// Cylinder rod diameter, millimetres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rod_mm: Option<f64>,
    /// Cylinder moving mass, kilograms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moving_mass_kg: Option<f64>,
}

/// One pipe between two ports, named by placement identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredPipe {
    /// Placement at one end.
    pub from: u64,
    /// Its port id.
    pub from_port: String,
    /// Placement at the other end.
    pub to: u64,
    /// Its port id.
    pub to_port: String,
    /// Middle-segment offset from the automatic route, tenths of a millimetre.
    #[serde(default)]
    pub offset_tenths: i32,
}

/// A saved schematic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sheet {
    /// Format version of the document.
    pub format: u32,
    /// Document title shown in the window.
    #[serde(default)]
    pub title: String,
    /// Components in placement order.
    #[serde(default)]
    pub placement: Vec<StoredPlacement>,
    /// Pipes in drawing order.
    #[serde(default)]
    pub pipe: Vec<StoredPipe>,
}

/// Everything that can go wrong while reading or writing a sheet.
#[derive(Debug, thiserror::Error)]
pub enum SheetError {
    /// The document is not valid JSON.
    #[error("invalid sheet: {0}")]
    Parse(#[from] serde_json::Error),

    /// The document was written by a newer, unknown version.
    #[error("sheet format {found} is newer than this build understands ({supported})")]
    UnsupportedVersion {
        /// Version found in the file.
        found: u32,
        /// Highest version this build understands.
        supported: u32,
    },

    /// A rule the document breaks, with the field that breaks it.
    #[error("invalid sheet: {0}")]
    Invalid(String),

    /// The file is larger than the reader will load.
    #[error("the sheet is larger than {MAX_BYTES} bytes")]
    TooLarge,

    /// The file could not be read or written.
    #[error("sheet I/O failed: {0}")]
    Io(#[from] std::io::Error),
}

impl Sheet {
    /// An empty sheet in the current format.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            format: SHEET_VERSION,
            title: title.into(),
            placement: Vec::new(),
            pipe: Vec::new(),
        }
    }

    /// Checks every bound and every number before the caller trusts the document.
    ///
    /// # Errors
    ///
    /// Returns [`SheetError::UnsupportedVersion`] for a future format and
    /// [`SheetError::Invalid`] for a value outside its range or a duplicate identity.
    pub fn validate(&self) -> Result<(), SheetError> {
        if self.format < SHEET_MIN || self.format > SHEET_VERSION {
            return Err(SheetError::UnsupportedVersion {
                found: self.format,
                supported: SHEET_VERSION,
            });
        }
        if self.placement.len() > MAX_PLACEMENTS {
            return Err(SheetError::Invalid(format!(
                "{} components, at most {MAX_PLACEMENTS}",
                self.placement.len()
            )));
        }
        if self.pipe.len() > MAX_PIPES {
            return Err(SheetError::Invalid(format!(
                "{} pipes, at most {MAX_PIPES}",
                self.pipe.len()
            )));
        }
        let ids = self.validate_placements()?;
        for pipe in &self.pipe {
            if !ids.contains(&pipe.from) || !ids.contains(&pipe.to) {
                return Err(SheetError::Invalid(String::from(
                    "a pipe names a component that is not on the sheet",
                )));
            }
            if pipe.from == pipe.to {
                return Err(SheetError::Invalid(String::from(
                    "a pipe joins a component to itself",
                )));
            }
            for (name, port) in [("from_port", &pipe.from_port), ("to_port", &pipe.to_port)] {
                if port.is_empty() || port.len() > MAX_ID {
                    return Err(SheetError::Invalid(format!("{name} is empty or too long")));
                }
            }
            if pipe.offset_tenths.unsigned_abs() > 10_000 {
                return Err(SheetError::Invalid(String::from(
                    "a pipe is routed more than a metre from the automatic elbow",
                )));
            }
        }
        Ok(())
    }

    /// Checks every placement, returning the identities a pipe may name.
    ///
    /// # Errors
    ///
    /// Returns [`SheetError::Invalid`] for the first placement that breaks a bound.
    fn validate_placements(&self) -> Result<Vec<u64>, SheetError> {
        let mut ids: Vec<u64> = Vec::with_capacity(self.placement.len());
        for placement in &self.placement {
            if placement.id == 0 {
                return Err(SheetError::Invalid(String::from("a component has id 0")));
            }
            if ids.contains(&placement.id) {
                return Err(SheetError::Invalid(format!(
                    "two components share id {}",
                    placement.id
                )));
            }
            ids.push(placement.id);
            if let Some(designation) = &placement.designation {
                if designation.is_empty()
                    || designation.chars().count() > MAX_DESIGNATION
                    || designation.chars().any(char::is_control)
                    || designation.trim() != designation
                {
                    return Err(SheetError::Invalid(String::from(
                        "a component has an unusable designation",
                    )));
                }
            }
            if placement.quarter_turns > 3 {
                return Err(SheetError::Invalid(format!(
                    "a component is turned {} quarters",
                    placement.quarter_turns
                )));
            }
            if placement.catalog.is_empty()
                || placement.catalog.len() > MAX_ID
                || !placement.catalog.is_ascii()
            {
                return Err(SheetError::Invalid(String::from(
                    "a component has an unusable catalog id",
                )));
            }
            for (name, value) in [
                ("x", placement.x),
                ("y", placement.y),
                ("width", placement.width),
                ("height", placement.height),
            ] {
                if !value.is_finite() {
                    return Err(SheetError::Invalid(format!("{name} is not finite")));
                }
            }
            if placement.width <= 0.0
                || placement.height <= 0.0
                || placement.width > MAX_SIDE_MM
                || placement.height > MAX_SIDE_MM
            {
                return Err(SheetError::Invalid(String::from(
                    "a component box is outside 0..=2000 mm",
                )));
            }
            if let Some(edit) = placement.edit {
                edit.validate()?;
            }
        }
        Ok(ids)
    }

    /// Reads a sheet from a file.
    ///
    /// # Errors
    ///
    /// Returns [`SheetError::TooLarge`] before reading when the file exceeds
    /// [`MAX_BYTES`], plus the errors of [`Self::parse`].
    pub fn read(path: &Path) -> Result<Self, SheetError> {
        let size = fs::metadata(path)?.len();
        if size > MAX_BYTES {
            return Err(SheetError::TooLarge);
        }
        Self::parse(&fs::read_to_string(path)?)
    }

    /// Parses and validates a sheet from text.
    ///
    /// # Errors
    ///
    /// Returns [`SheetError::Parse`] for invalid JSON and the errors of
    /// [`Self::validate`].
    pub fn parse(text: &str) -> Result<Self, SheetError> {
        let mut sheet: Self = serde_json::from_str(text)?;
        sheet.validate()?;
        // An older file is upgraded by reading it: every field it does not have is what
        // the older release meant, so the document in memory is this format and saving it
        // writes the fields the reader just filled in.
        sheet.format = SHEET_VERSION;
        Ok(sheet)
    }

    /// Serialises the sheet, validating it first.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::validate`] or [`SheetError::Parse`].
    pub fn to_json(&self) -> Result<String, SheetError> {
        self.validate()?;
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Writes the sheet atomically: a temporary file in the same folder, then a rename.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::to_json`] and [`SheetError::Io`].
    pub fn write(&self, path: &Path) -> Result<(), SheetError> {
        let text = self.to_json()?;
        let temporary = path.with_extension("pfluid.tmp");
        fs::write(&temporary, text)?;
        fs::rename(&temporary, path)?;
        Ok(())
    }
}

impl StoredEdits {
    /// Checks the overrides that have physical ranges.
    ///
    /// # Errors
    ///
    /// Returns [`SheetError::Invalid`] when a value is not finite or outside its range.
    pub fn validate(self) -> Result<(), SheetError> {
        let non_negative = [("pressure_bar", self.pressure_bar), ("rod_mm", self.rod_mm)];
        for (name, value) in non_negative {
            if let Some(value) = value
                && (!value.is_finite() || value < 0.0)
            {
                return Err(SheetError::Invalid(format!(
                    "{name} must be zero or positive"
                )));
            }
        }
        let positive = [
            ("conductance", self.conductance),
            ("stroke_mm", self.stroke_mm),
            ("bore_mm", self.bore_mm),
            ("moving_mass_kg", self.moving_mass_kg),
        ];
        for (name, value) in positive {
            if let Some(value) = value
                && (!value.is_finite() || value <= 0.0)
            {
                return Err(SheetError::Invalid(format!("{name} must be positive")));
            }
        }
        if let (Some(rod), Some(bore)) = (self.rod_mm, self.bore_mm)
            && rod >= bore
        {
            return Err(SheetError::Invalid(String::from(
                "the rod is not smaller than the bore",
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet() -> Sheet {
        Sheet {
            format: SHEET_VERSION,
            title: String::from("Prueba"),
            placement: vec![
                StoredPlacement {
                    id: 1,
                    catalog: String::from("supply.workshop"),
                    x: 10.0,
                    y: 20.0,
                    width: 26.0,
                    height: 26.0,
                    edit: None,
                    quarter_turns: 0,
                    mirrored: false,
                    designation: None,
                },
                StoredPlacement {
                    id: 2,
                    catalog: String::from("cylinder.double_acting_32_200"),
                    x: 120.0,
                    y: 80.0,
                    width: 62.0,
                    height: 26.0,
                    edit: Some(StoredEdits {
                        stroke_mm: Some(150.0),
                        ..StoredEdits::default()
                    }),
                    quarter_turns: 1,
                    mirrored: true,
                    designation: Some(String::from("1A1")),
                },
            ],
            pipe: vec![StoredPipe {
                from: 1,
                from_port: String::from("1"),
                to: 2,
                to_port: String::from("2"),
                offset_tenths: 25,
            }],
        }
    }

    #[test]
    fn a_sheet_round_trips_byte_identically() {
        let sheet = sheet();
        let first = sheet.to_json().expect("valid");
        let parsed = Sheet::parse(&first).expect("parses");
        assert_eq!(parsed, sheet);
        assert_eq!(parsed.to_json().expect("valid"), first);
    }

    #[test]
    fn an_unknown_key_is_rejected_instead_of_ignored() {
        let text = r#"{"format":1,"placement":[],"pipe":[],"surprise":true}"#;
        assert!(matches!(Sheet::parse(text), Err(SheetError::Parse(_))));
    }

    #[test]
    fn a_future_format_is_refused() {
        let text = r#"{"format":99,"placement":[],"pipe":[]}"#;
        assert!(matches!(
            Sheet::parse(text),
            Err(SheetError::UnsupportedVersion { found: 99, .. })
        ));
    }

    #[test]
    fn bounds_are_checked_before_the_document_is_trusted() {
        let mut sheet = sheet();
        sheet.placement[0].width = 0.0;
        assert!(sheet.validate().is_err());
        sheet.placement[0].width = 26.0;
        sheet.placement[1].id = 1;
        assert!(sheet.validate().is_err(), "identities must be unique");
        sheet.placement[1].id = 2;
        sheet.pipe[0].from = 99;
        assert!(sheet.validate().is_err());
        sheet.pipe[0].from = 1;
        sheet.pipe[0].offset_tenths = 200_000;
        assert!(sheet.validate().is_err());
        sheet.pipe[0].offset_tenths = 0;
        sheet.placement[1].edit = Some(StoredEdits {
            rod_mm: Some(40.0),
            bore_mm: Some(32.0),
            ..StoredEdits::default()
        });
        assert!(sheet.validate().is_err(), "the rod must fit the bore");
    }

    #[test]
    fn an_unturned_symbol_writes_no_orientation_at_all() {
        let mut sheet = sheet();
        sheet.placement[1].quarter_turns = 0;
        sheet.placement[1].mirrored = false;
        let text = sheet.to_json().expect("valid");
        assert!(!text.contains("quarter_turns"), "{text}");
        assert!(!text.contains("mirrored"), "{text}");
    }

    #[test]
    fn a_version_two_sheet_still_reads_and_gains_an_unnamed_component() {
        // Exactly what the previous release wrote: orientation but no designation.
        let text = r#"{
          "format": 2,
          "title": "Anterior",
          "placement": [
            {"id": 1, "catalog": "valve.3_2.spring_return", "x": 10.0, "y": 20.0,
             "width": 30.0, "height": 15.0, "quarter_turns": 1, "mirrored": true}
          ],
          "pipe": []
        }"#;
        let sheet = Sheet::parse(text).expect("a version 2 sheet reads");
        assert_eq!(sheet.format, SHEET_VERSION);
        assert_eq!(sheet.placement[0].quarter_turns, 1);
        assert!(sheet.placement[0].mirrored);
        assert_eq!(sheet.placement[0].designation, None);
        assert!(sheet.to_json().expect("valid").contains(r#""format": 3"#));
    }

    #[test]
    fn a_designation_that_cannot_be_read_is_refused() {
        let mut sheet = sheet();
        sheet.placement[0].designation = Some(String::from("  "));
        assert!(sheet.validate().is_err(), "whitespace is not a name");
        sheet.placement[0].designation = Some("x".repeat(MAX_DESIGNATION + 1));
        assert!(sheet.validate().is_err(), "too long");
        sheet.placement[0].designation = Some(String::from("1A1"));
        assert!(sheet.validate().is_ok());
        // An unnamed component writes nothing at all, so a sheet without names looks as it
        // did before the field existed.
        for placement in &mut sheet.placement {
            placement.designation = None;
        }
        assert!(!sheet.to_json().expect("valid").contains("designation"));
    }

    #[test]
    fn a_version_one_sheet_still_reads_and_gains_an_unturned_orientation() {
        // Exactly what the previous release wrote: no orientation field, format 1.
        let text = r#"{
          "format": 1,
          "title": "Viejo",
          "placement": [
            {"id": 1, "catalog": "supply.workshop", "x": 10.0, "y": 20.0,
             "width": 26.0, "height": 26.0}
          ],
          "pipe": []
        }"#;
        let sheet = Sheet::parse(text).expect("a version 1 sheet reads");
        assert_eq!(sheet.format, SHEET_VERSION, "reading it upgrades it");
        assert_eq!(sheet.placement[0].quarter_turns, 0);
        assert!(!sheet.placement[0].mirrored);
        // Saving it writes the format this build understands, with the fields it added.
        let saved = sheet.to_json().expect("valid");
        assert!(saved.contains(r#""format": 3"#), "{saved}");
        assert_eq!(sheet.placement[0].designation, None, "and no name either");
    }

    #[test]
    fn a_turn_that_is_not_one_of_the_four_is_refused() {
        let mut sheet = sheet();
        sheet.placement[0].quarter_turns = 4;
        assert!(sheet.validate().is_err());
    }

    #[test]
    fn a_sheet_is_written_atomically_and_read_back() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let path = directory.path().join("circuito.pfluid");
        let sheet = sheet();
        sheet.write(&path).expect("writes");
        assert!(!path.with_extension("pfluid.tmp").exists());
        assert_eq!(Sheet::read(&path).expect("reads"), sheet);
    }
}
