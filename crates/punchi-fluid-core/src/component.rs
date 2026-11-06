// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Catalogue of circuit components and the ports they expose.

use crate::units::{Pressure, Volume};

/// Stable identifier of a port inside its component, for example `"1"` on a
/// directional valve.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PortId(pub String);

impl PortId {
    /// Builds a port id from anything string-like.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrows the id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Direction a port is designed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortKind {
    /// Supply connection, normally the compressed-air inlet.
    Supply,
    /// Working connection, such as the ports that drive a cylinder.
    Working,
    /// Exhaust or return to the atmosphere or to the reservoir.
    Exhaust,
    /// Electrical control signal.
    Signal,
}

/// A single connection point on a component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    /// Identifier used by connections and by the file format.
    pub id: PortId,
    /// What the port is meant to carry.
    pub kind: PortKind,
    /// Human-readable label from the symbol, for example `"P"` or `"A"`.
    pub label: String,
}

/// The kinds of component the first document format understands.
///
/// This is the instance layer of the placeholder document, not the component
/// library: behaviour and symbol live in the catalog definitions described by
/// [ADR 0002](../../../docs/adr/0002-components-are-files.md). Adding a valve
/// never means adding a variant here. Until the editor lands (v0.3) a placed
/// component names a kind instead of referencing a catalog id, and this enum is
/// what the file format stores.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentKind {
    /// Compressed-air supply with a regulated working pressure.
    Supply {
        /// Regulated outlet pressure.
        pressure: Pressure,
    },
    /// Single-acting cylinder pushed by air and returned by a spring.
    CylinderSingleActing {
        /// Piston area.
        area: f64,
        /// Stroke length.
        stroke: f64,
    },
    /// 3/2 directional valve, two positions and three ports.
    Valve3_2,
    /// Fixed restriction used to model a throttle or a nozzle.
    Throttle {
        /// Orifice area.
        area: f64,
    },
    /// Lumped volume, such as a receiver or the dead volume of a line.
    Reservoir {
        /// Geometric volume.
        volume: Volume,
    },
}

impl ComponentKind {
    /// Ports the component exposes, in symbol order.
    #[must_use]
    pub fn ports(&self) -> Vec<Port> {
        match self {
            Self::Supply { .. } => vec![Port {
                id: PortId::new("1"),
                kind: PortKind::Supply,
                label: String::from("P"),
            }],
            Self::CylinderSingleActing { .. } => vec![
                Port {
                    id: PortId::new("1"),
                    kind: PortKind::Working,
                    label: String::from("A"),
                },
                Port {
                    id: PortId::new("2"),
                    kind: PortKind::Exhaust,
                    label: String::from("R"),
                },
            ],
            Self::Valve3_2 => ["1", "2", "3"]
                .into_iter()
                .map(|id| Port {
                    id: PortId::new(id),
                    kind: match id {
                        "1" => PortKind::Supply,
                        "2" => PortKind::Working,
                        _ => PortKind::Exhaust,
                    },
                    label: String::from(id),
                })
                .collect(),
            Self::Throttle { .. } => vec![
                Port {
                    id: PortId::new("1"),
                    kind: PortKind::Working,
                    label: String::from("1"),
                },
                Port {
                    id: PortId::new("2"),
                    kind: PortKind::Working,
                    label: String::from("2"),
                },
            ],
            Self::Reservoir { .. } => vec![Port {
                id: PortId::new("1"),
                kind: PortKind::Working,
                label: String::from("1"),
            }],
        }
    }
}

/// A component placed in a circuit.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    /// Identifier unique inside its circuit.
    pub id: String,
    /// Visual label shown next to the symbol.
    pub name: String,
    /// Physical behaviour and port layout.
    pub kind: ComponentKind,
}
