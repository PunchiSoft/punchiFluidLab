// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Versioned circuit file format.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

//!
//! A saved circuit carries a `format` version so a future release can migrate
//! old documents instead of refusing to open them. The on-disk representation is
//! deliberately separate from [`punchi_fluid_core`]: the domain can evolve
//! without breaking files a user already has.

pub mod sheet;

use punchi_fluid_core::{Circuit, Component, ComponentKind, PortId};
use serde::{Deserialize, Serialize};

/// Format version written by this build.
pub const FORMAT_VERSION: u32 = 1;

/// Everything that can go wrong while reading or writing a circuit file.
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    /// The document is not valid JSON.
    #[error("invalid circuit document: {0}")]
    Parse(#[from] serde_json::Error),

    /// The document was written by a newer, unknown format version.
    #[error("unsupported format version {found}, this build understands up to {supported}")]
    UnsupportedVersion {
        /// Version found in the document.
        found: u32,
        /// Highest version this build understands.
        supported: u32,
    },
}

/// A circuit document as stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Format version of the document.
    pub format: u32,
    /// Human-readable document title.
    #[serde(default)]
    pub title: String,
    /// Components in the circuit.
    #[serde(default)]
    pub components: Vec<StoredComponent>,
    /// Connections between components.
    #[serde(default)]
    pub connections: Vec<StoredConnection>,
}

/// Stored form of a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredComponent {
    /// Identifier unique inside the document.
    pub id: String,
    /// Label shown next to the symbol.
    pub name: String,
    /// Component kind and its parameters.
    pub kind: StoredKind,
}

/// Stored component kinds, mirroring the domain catalogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StoredKind {
    /// Supply with a regulated pressure, in bar.
    Supply {
        /// Regulated pressure.
        pressure_bar: f64,
    },
    /// Single-acting cylinder; areas in square millimetre, stroke in millimetre.
    CylinderSingleActing {
        /// Piston area.
        area_mm2: f64,
        /// Stroke.
        stroke_mm: f64,
    },
    /// 3/2 directional valve.
    Valve3_2,
    /// Throttle with an orifice area in square millimetre.
    Throttle {
        /// Orifice area.
        area_mm2: f64,
    },
    /// Lumped volume in litre.
    Reservoir {
        /// Volume.
        volume_l: f64,
    },
}

/// Stored form of a connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConnection {
    /// Source component id.
    pub from_component: String,
    /// Source port id.
    pub from_port: String,
    /// Target component id.
    pub to_component: String,
    /// Target port id.
    pub to_port: String,
}

impl Document {
    /// Parses a document from JSON text.
    ///
    /// # Errors
    ///
    /// Returns [`FileError::Parse`] when the text is not a valid document.
    pub fn parse(text: &str) -> Result<Self, FileError> {
        Ok(serde_json::from_str(text)?)
    }

    /// Serialises a circuit into the current format.
    ///
    /// # Errors
    ///
    /// Returns [`FileError::Parse`] if the document cannot be serialised, which
    /// would be a bug in this crate rather than a user error.
    pub fn from_circuit(title: &str, circuit: &Circuit) -> Result<String, FileError> {
        let document = Self {
            format: FORMAT_VERSION,
            title: title.to_owned(),
            components: circuit
                .components
                .iter()
                .map(|component| StoredComponent {
                    id: component.id.clone(),
                    name: component.name.clone(),
                    kind: StoredKind::from(&component.kind),
                })
                .collect(),
            connections: circuit
                .connections
                .iter()
                .map(|connection| StoredConnection {
                    from_component: connection.from.0.clone(),
                    from_port: connection.from.1.as_str().to_owned(),
                    to_component: connection.to.0.clone(),
                    to_port: connection.to.1.as_str().to_owned(),
                })
                .collect(),
        };
        Ok(serde_json::to_string_pretty(&document)?)
    }

    /// Parses a document and converts it into a domain circuit.
    ///
    /// # Errors
    ///
    /// Returns [`FileError::UnsupportedVersion`] when the document is newer than
    /// this build, and [`FileError::Parse`] when it is not valid JSON.
    pub fn into_circuit(self) -> Result<Circuit, FileError> {
        if self.format > FORMAT_VERSION {
            return Err(FileError::UnsupportedVersion {
                found: self.format,
                supported: FORMAT_VERSION,
            });
        }

        let mut circuit = Circuit::default();
        for stored in self.components {
            circuit.add_component(Component {
                id: stored.id,
                name: stored.name,
                kind: stored.kind.into(),
            });
        }
        for stored in self.connections {
            circuit
                .connect(
                    (stored.from_component, PortId::new(stored.from_port)),
                    (stored.to_component, PortId::new(stored.to_port)),
                )
                .ok();
        }
        Ok(circuit)
    }
}

impl From<&ComponentKind> for StoredKind {
    fn from(kind: &ComponentKind) -> Self {
        match kind {
            ComponentKind::Supply { pressure } => Self::Supply {
                pressure_bar: pressure.bar(),
            },
            ComponentKind::CylinderSingleActing { area, stroke } => Self::CylinderSingleActing {
                area_mm2: *area,
                stroke_mm: *stroke,
            },
            ComponentKind::Valve3_2 => Self::Valve3_2,
            ComponentKind::Throttle { area } => Self::Throttle { area_mm2: *area },
            ComponentKind::Reservoir { volume } => Self::Reservoir {
                volume_l: volume.cubic_metre() * 1000.0,
            },
        }
    }
}

impl From<StoredKind> for ComponentKind {
    fn from(kind: StoredKind) -> Self {
        match kind {
            StoredKind::Supply { pressure_bar } => Self::Supply {
                // A stored value that cannot be trusted falls back to atmosphere
                // so the model stays total instead of panicking.
                pressure: punchi_fluid_core::Pressure::from_bar(pressure_bar)
                    .unwrap_or(punchi_fluid_core::Pressure::ATMOSPHERE),
            },
            StoredKind::CylinderSingleActing {
                area_mm2,
                stroke_mm,
            } => Self::CylinderSingleActing {
                area: area_mm2,
                stroke: stroke_mm,
            },
            StoredKind::Valve3_2 => Self::Valve3_2,
            StoredKind::Throttle { area_mm2 } => Self::Throttle { area: area_mm2 },
            StoredKind::Reservoir { volume_l } => Self::Reservoir {
                volume: punchi_fluid_core::Volume::from_litre(volume_l)
                    .unwrap_or(punchi_fluid_core::Volume::ZERO),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_circuit() {
        let mut circuit = Circuit::default();
        circuit.add_component(Component {
            id: String::from("p"),
            name: String::from("Air supply"),
            kind: ComponentKind::Supply {
                pressure: punchi_fluid_core::Pressure::from_bar(6.0).expect("valid"),
            },
        });
        circuit.add_component(Component {
            id: String::from("r"),
            name: String::from("Receiver"),
            kind: ComponentKind::Reservoir {
                volume: punchi_fluid_core::Volume::from_litre(1.0).expect("valid"),
            },
        });
        circuit
            .connect(
                (String::from("p"), PortId::new("1")),
                (String::from("r"), PortId::new("1")),
            )
            .expect("ports exist");

        let json = Document::from_circuit("Prueba", &circuit).expect("serialisable");
        let parsed = Document::parse(&json).expect("valid json");
        let restored = parsed.into_circuit().expect("compatible version");

        assert_eq!(restored.components.len(), 2);
        assert_eq!(restored.connections.len(), 1);
    }

    #[test]
    fn rejects_newer_format() {
        let document = Document {
            format: FORMAT_VERSION + 1,
            title: String::new(),
            components: Vec::new(),
            connections: Vec::new(),
        };

        assert!(matches!(
            document.into_circuit(),
            Err(FileError::UnsupportedVersion { .. })
        ));
    }

    #[test]
    fn empty_document_is_valid() {
        let document = Document::parse(r#"{"format":1}"#).expect("valid json");
        let circuit = document.into_circuit().expect("compatible version");
        assert!(circuit.components.is_empty());
    }
}
