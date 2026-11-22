// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The live model of a circuit: the solver's network built from the document.
//!
//! The ABI still owns its own copy of this build until it retires; this crate is where
//! the window reads it, and the one place that will stay. The build follows the document:
//! a definition the solver cannot model is left out of the network, never guessed.

use punchi_fluid_catalog::{Behaviour, Catalog, ComponentDefinition, PortRole};
use punchi_fluid_doc::{Document, PlacementEdits};
use punchi_fluid_draw::placed::Reading;
use punchi_fluid_solver::network::{
    Network, NetworkPart, PartBehaviour, PortAddress, PortTermination,
};
use punchi_fluid_solver::pneumatic::Telemetry;

/// The running model of the active circuit.
#[derive(Debug)]
pub struct Model {
    network: Network,
    /// Part of the network each placement became, parallel to the document.
    part_of: Vec<Option<usize>>,
    /// Valve of the network each placement became, parallel to the document.
    valve_of: Vec<Option<usize>>,
}

impl Model {
    /// Builds the network of the active circuit from the document.
    ///
    /// # Errors
    ///
    /// Returns a message when the solver refuses the circuit, or when it has no cylinder
    /// to move: a model that cannot move anything is not a model the window can run.
    pub fn build(document: &Document, catalog: &Catalog) -> Result<Self, String> {
        let circuit = document.active_circuit();
        let mut parts: Vec<NetworkPart> = Vec::new();
        let mut part_of: Vec<Option<usize>> = vec![None; document.placements().len()];
        let mut valve_of: Vec<Option<usize>> = vec![None; document.placements().len()];
        let mut valves = 0;
        for (index, placement) in document.placements().iter().enumerate() {
            if placement.circuit != circuit {
                continue;
            }
            let Some(definition) = catalog.get(&placement.catalog_id) else {
                continue;
            };
            let mut definition = definition.clone();
            apply_edits(&mut definition, &placement.edits);
            if let Ok(part) = NetworkPart::from_definition(&definition) {
                if matches!(part.behaviour, PartBehaviour::Valve { .. }) {
                    valve_of[index] = Some(valves);
                    valves += 1;
                }
                part_of[index] = Some(parts.len());
                parts.push(part);
            }
        }
        let mut pipes = Vec::new();
        for (index, connection) in document.connections().iter().enumerate() {
            if connection.circuit != circuit {
                continue;
            }
            let (Some(from), Some(to)) = (
                document.index_of(connection.from.placement),
                document.index_of(connection.to.placement),
            ) else {
                continue;
            };
            let (Some(from), Some(to)) = (part_of[from], part_of[to]) else {
                continue;
            };
            pipes.push((
                index,
                PortAddress::new(from, connection.from.port.clone()),
                PortAddress::new(to, connection.to.port.clone()),
            ));
        }
        let pipe_count = document.connections().len();
        // A closure is a port termination, not a placement of its own. Only an exhaust gets
        // one: a working or supply port is already sealed without a pipe.
        let mut terminations = Vec::new();
        for (index, placement) in document.placements().iter().enumerate() {
            if placement.circuit != circuit || placement.port_closures.is_empty() {
                continue;
            }
            let Some(part) = part_of.get(index).copied().flatten() else {
                continue;
            };
            let Some(definition) = catalog.get(&placement.catalog_id) else {
                continue;
            };
            for (port, closure_id) in &placement.port_closures {
                let Some(role) = definition
                    .port
                    .iter()
                    .find(|candidate| &candidate.id == port)
                    .map(|candidate| candidate.role)
                else {
                    continue;
                };
                if role != PortRole::Exhaust {
                    continue;
                }
                let Some(closure) = catalog.get(closure_id) else {
                    continue;
                };
                let termination = match closure.behaviour {
                    Behaviour::Plug => PortTermination::Closed,
                    Behaviour::Silencer { conductance } => PortTermination::Vent { conductance },
                    _ => continue,
                };
                terminations.push((PortAddress::new(part, port.clone()), termination));
            }
        }
        match Network::build(&parts, &pipes, &terminations, pipe_count) {
            Ok(network) if network.cylinder().is_some() => Ok(Self {
                network,
                part_of,
                valve_of,
            }),
            Ok(_) => Err(String::from("the circuit has no cylinder to move")),
            Err(cause) => Err(cause.to_string()),
        }
    }

    /// Advances the model by `dt` seconds.
    ///
    /// # Errors
    ///
    /// Returns the solver's message when the step fails.
    pub fn advance(&mut self, dt: f64) -> Result<Telemetry, String> {
        self.network.advance(dt).map_err(|cause| cause.to_string())
    }

    /// The network the drawing reads its readings from.
    #[must_use]
    pub fn network(&self) -> &Network {
        &self.network
    }

    /// Part of the network a placement became, if it has one.
    #[must_use]
    pub fn part_of(&self, index: usize) -> Option<usize> {
        self.part_of.get(index).copied().flatten()
    }

    /// Valve of the network a placement became, if it is one.
    #[must_use]
    pub fn valve_of(&self, index: usize) -> Option<usize> {
        self.valve_of.get(index).copied().flatten()
    }

    /// Extension of the cylinder a placement is, `0..=1`, when the model has one.
    #[must_use]
    pub fn extension(&self, index: usize) -> Option<f32> {
        let part = self.part_of(index)?;
        #[allow(clippy::cast_possible_truncation)]
        self.network
            .extension_of(part)
            .map(|extension| extension as f32)
    }

    /// Active position of the valve a placement became, `0` for anything that is not one.
    #[must_use]
    pub fn valve_position(&self, index: usize) -> usize {
        self.part_of(index)
            .and_then(|part| self.network.valve_position_at(part))
            .unwrap_or(0)
    }

    /// Presses the valve a placement is, while the button is held. A momentary valve
    /// returns on [`Self::release_valve`]; a maintained one stays.
    pub fn press_valve(&mut self, index: usize) -> bool {
        let Some(valve) = self.valve_of(index) else {
            return false;
        };
        self.network.press_valve_at(valve)
    }

    /// Releases the valve a placement is, returning it when its return is a spring.
    pub fn release_valve(&mut self, index: usize) -> bool {
        let Some(valve) = self.valve_of(index) else {
            return false;
        };
        self.network.release_valve_at(valve)
    }

    /// Pressure in the hose of a connection of `document`, as a fraction of the supply
    /// span, for the drawing. A hose the model cannot resolve stays neutral.
    #[must_use]
    pub fn hose_pressure(&self, document: &Document, index: usize) -> f32 {
        let Some(connection) = document.connections().get(index) else {
            return 0.0;
        };
        let Some(from) = document.index_of(connection.from.placement) else {
            return 0.0;
        };
        let Some(part) = self.part_of(from) else {
            return 0.0;
        };
        let Some(pressure) = self.network.port_pressure(part, &connection.from.port) else {
            return 0.0;
        };
        let atmosphere = self.network.atmosphere_pa();
        let span = self.network.supply_pa() - atmosphere;
        if span <= 0.0 {
            return 0.0;
        }
        #[allow(clippy::cast_possible_truncation)]
        let ratio = ((pressure - atmosphere) / span) as f32;
        ratio.clamp(0.0, 1.0)
    }

    /// Reading of an instrument at `index`, in the unit its definition declares.
    #[must_use]
    pub fn reading(&self, index: usize, definition: &ComponentDefinition) -> Option<Reading> {
        let part = self.part_of(index)?;
        match definition.behaviour {
            Behaviour::Gauge { .. } => {
                let gauge_port = definition.port.first()?;
                let pressure = self.network.port_pressure(part, &gauge_port.id)?;
                #[allow(clippy::cast_possible_truncation)]
                let reading = ((pressure - self.network.atmosphere_pa()) / 1.0e5) as f32;
                Some(Reading::Pressure(reading))
            }
            Behaviour::FlowMeter { .. } => {
                #[allow(clippy::cast_possible_truncation)]
                let reading = self.network.part_flow_l_min(part)? as f32;
                Some(Reading::Flow(reading))
            }
            _ => None,
        }
    }
}

/// Applies the per-instance values a placement overrides, before the solver sees it.
fn apply_edits(definition: &mut ComponentDefinition, edits: &PlacementEdits) {
    match &mut definition.behaviour {
        Behaviour::Supply { pressure_bar } => {
            if let Some(value) = edits.pressure_bar {
                *pressure_bar = value;
            }
        }
        Behaviour::Valve { positions, .. } => {
            if let Some(value) = edits.conductance {
                for position in positions {
                    for connection in &mut position.connection {
                        connection.conductance = value;
                    }
                }
            }
        }
        // A throttle and a silencer are both restrictions, so the same override feeds both.
        Behaviour::Throttle { conductance } | Behaviour::Silencer { conductance } => {
            if let Some(value) = edits.conductance {
                *conductance = value;
            }
        }
        Behaviour::Cylinder {
            bore_mm,
            rod_mm,
            stroke_mm,
            moving_mass_kg,
            ..
        } => {
            if let Some(value) = edits.bore_mm {
                *bore_mm = value;
            }
            if let Some(value) = edits.rod_mm {
                *rod_mm = value;
            }
            if let Some(value) = edits.stroke_mm {
                *stroke_mm = value;
            }
            if let Some(value) = edits.moving_mass_kg {
                *moving_mass_kg = value;
            }
        }
        Behaviour::Gauge { .. }
        | Behaviour::FlowMeter { .. }
        | Behaviour::Recorder { .. }
        | Behaviour::ElectricalSupply { .. }
        | Behaviour::Contact { .. }
        | Behaviour::Relay { .. }
        | Behaviour::LimitSwitch { .. }
        | Behaviour::PressureSwitch { .. }
        | Behaviour::Lamp
        | Behaviour::Buzzer
        | Behaviour::Plug
        | Behaviour::Junction => {}
    }
}

#[cfg(test)]
mod tests {
    use punchi_fluid_catalog::Catalog;
    use punchi_fluid_doc::{Document, Endpoint, Placement};
    use punchi_fluid_draw::surface::Pt;

    use super::Model;

    fn worked_document() -> Document {
        let mut document = Document::new();
        let supply = document.place(Placement::new(
            "supply.compressor",
            Pt::new(60.0, 100.0),
            20.0,
            20.0,
        ));
        let valve = document.place(Placement::new(
            "valve.5_2.spring_return",
            Pt::new(120.0, 100.0),
            20.0,
            20.0,
        ));
        let cylinder = document.place(Placement::new(
            "cylinder.double_acting_32_200",
            Pt::new(200.0, 100.0),
            44.0,
            20.0,
        ));
        let supply_id = document.placements()[supply].id();
        let valve_id = document.placements()[valve].id();
        let cylinder_id = document.placements()[cylinder].id();
        document.connect(
            Endpoint::new(supply_id, "1"),
            Endpoint::new(valve_id, "1"),
            None,
        );
        document.connect(
            Endpoint::new(valve_id, "2"),
            Endpoint::new(cylinder_id, "1"),
            None,
        );
        document.connect(
            Endpoint::new(valve_id, "4"),
            Endpoint::new(cylinder_id, "2"),
            None,
        );
        document
    }

    #[test]
    fn a_worked_circuit_builds_and_advances() {
        let document = worked_document();
        let mut model =
            Model::build(&document, &Catalog::builtin()).expect("the worked circuit builds");
        assert_eq!(model.valve_of(1), Some(0), "the valve is the first valve");
        let before = model.network().telemetry().time;
        model.advance(0.01).expect("the model advances");
        let telemetry = model.network().telemetry();
        assert!(telemetry.time > before);
        assert!(telemetry.pressure_pa >= 0.0, "pressure never goes negative");
    }

    #[test]
    fn a_held_valve_moves_and_a_released_spring_returns_it() {
        let document = worked_document();
        let mut model =
            Model::build(&document, &Catalog::builtin()).expect("the worked circuit builds");
        assert_eq!(model.valve_position(1), 0);
        assert!(model.press_valve(1));
        assert_eq!(model.valve_position(1), 1, "holding moves the spool");
        assert!(model.release_valve(1));
        assert_eq!(model.valve_position(1), 0, "a spring returns it");
        assert!(!model.press_valve(0), "the supply is not a valve");
        assert!(
            model.hose_pressure(&document, 0) > 0.5,
            "the supply hose carries the supply pressure"
        );
    }

    #[test]
    fn a_circuit_without_a_cylinder_is_refused() {
        let mut document = Document::new();
        document.place(Placement::new(
            "supply.compressor",
            Pt::new(60.0, 100.0),
            20.0,
            20.0,
        ));
        assert!(Model::build(&document, &Catalog::builtin()).is_err());
    }
}
