// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The circuit as a network of lumped volumes joined by restrictions.
//!
//! A schematic is not a fixed three-component example: it is whatever the document
//! connects. This module takes the parts the catalog defines for each placement and the
//! pipes the document drew, and integrates the gas mass of every volume they form.
//!
//! The model is the same physics as [`crate::pneumatic`] — ideal gas, ISO 6358 orifices,
//! a piston with spring, damping and friction — applied to a graph instead of to one
//! working line. Every port is a node with a small volume, every valve position is a set
//! of restrictions between its own ports, and every pipe is a restriction between two
//! ports. A valve added as a file therefore works without a line of code changing: the
//! network reads its ports and positions from the definition.
//!
//! What the model deliberately does not do yet: double-acting cylinders (one chamber per
//! cylinder is integrated), pipe volume and length (a pipe conducts, it does not store),
//! and open working ports (they are sealed; the circuit check reports them). Each of
//! those is a capability of the reference and is recorded as such in
//! `docs/reference-parity.md`, not hidden as an assumption.

use std::collections::HashMap;

use punchi_fluid_catalog::{Actuation, Behaviour, ComponentDefinition, PortRole};

use crate::SolverError;
use crate::orifice::{self, DEFAULT_CRITICAL_RATIO};
use crate::pneumatic::{Cylinder, Telemetry};

/// Specific gas constant of dry air, `J / (kg · K)`.
const AIR_SPECIFIC_GAS_CONSTANT: f64 = 287.058;

/// Largest internal integration step, as in the fixed circuit model.
const MAX_INTERNAL_SUBSTEP: f64 = 2e-6;

/// Upper bound on the internal substeps of one advance call.
const MAX_SUBSTEPS_PER_CALL: u32 = 500_000;

/// Volume given to a single port so it can hold gas.
///
/// A port with no volume has no pressure, and a network of algebraic nodes needs a solver
/// this model does not have. A fifth of a cubic centimetre is the order of a valve's
/// internal gallery, small enough not to change the circuit and large enough to keep the
/// explicit integration comfortably stable.
const PORT_VOLUME: f64 = 2.0e-7;

/// Conductance of a hose. A short hose is not the restriction that governs a circuit; the
/// valve and the throttle are, so a hose conducts an order of magnitude better than the
/// built-in valve.
const PIPE_CONDUCTANCE: f64 = 1.0e-8;

/// Most valve positions the edge mask can hold.
const MAX_POSITIONS: usize = 63;

/// One port of a part, as the definition declares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortInfo {
    /// Identifier used by pipes, for example `"1"`.
    pub id: String,
    /// What the port is meant to carry, which decides whether an unconnected exhaust
    /// breathes to atmosphere.
    pub role: PortRole,
}

/// One internal path of a valve position.
#[derive(Debug, Clone, PartialEq)]
pub struct ValveConnection {
    /// Port on one side.
    pub from: String,
    /// Port on the other side.
    pub to: String,
    /// Sonic conductance, `m³ / (s · Pa)`.
    pub conductance: f64,
}

/// What a part contributes to the network.
#[derive(Debug, Clone, PartialEq)]
pub enum PartBehaviour {
    /// Fixed pressure source, stored as absolute pressure.
    Supply {
        /// Absolute pressure of the reservoir, `Pa`.
        pressure_pa: f64,
    },
    /// Switchable manifold: each position lists the paths it opens.
    Valve {
        /// Paths of every position, in file order.
        positions: Vec<Vec<ValveConnection>>,
        /// Names of the positions, for the interface.
        position_names: Vec<String>,
        /// Position selected before the simulation starts.
        default_position: usize,
        /// Whether releasing the actuator returns to the default position.
        spring_return: bool,
        /// How the user changes position.
        actuation: Actuation,
    },
    /// Single-acting cylinder with a return spring.
    Cylinder(Cylinder),
    /// Pressure gauge: a passive port that adds nothing to the physics.
    Gauge,
    /// Fixed restriction between two ports.
    Throttle {
        /// Sonic conductance, `m³ / (s · Pa)`.
        conductance: f64,
    },
    /// Inline restriction that reports the gas it carried (ADR 0014).
    FlowMeter {
        /// Sonic conductance, `m³ / (s · Pa)`.
        conductance: f64,
    },
    /// Hoses meet here: one node, no volume of its own beyond the ports.
    Junction,
    /// A recorder: no ports and nothing to simulate. It is a part so the document can hold
    /// it, and the session samples its source outside the network (ADR 0015).
    Recorder,
    /// A blind plug: the port is closed, so the part adds nothing (ADR 0016).
    Plug,
    /// A silencer: the exhaust leaves through a restriction to atmosphere.
    Silencer {
        /// Sonic conductance to atmosphere, `m³ / (s · Pa)`.
        conductance: f64,
    },
    /// An electrical component: drawn and checked, not yet simulated (ADR 0017).
    Electrical,
}

/// A part of the circuit: its ports and what it does.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkPart {
    /// Catalog id of the definition it came from.
    pub id: String,
    /// Ports in definition order.
    pub ports: Vec<PortInfo>,
    /// The physics it contributes.
    pub behaviour: PartBehaviour,
}

impl NetworkPart {
    /// Reads a definition into the description the network consumes.
    ///
    /// # Errors
    ///
    /// Returns the same errors the fixed circuit model reports when a definition is not
    /// the kind it claims to be.
    pub fn from_definition(definition: &ComponentDefinition) -> Result<Self, SolverError> {
        let ports = definition
            .port
            .iter()
            .map(|port| PortInfo {
                id: port.id.clone(),
                role: port.role,
            })
            .collect();
        let behaviour = match &definition.behaviour {
            Behaviour::Supply { pressure_bar } => PartBehaviour::Supply {
                pressure_pa: pressure_bar
                    .mul_add(100_000.0, punchi_fluid_core::Pressure::ATMOSPHERE.pascal()),
            },
            Behaviour::Valve {
                actuation,
                spring_return,
                default_position,
                positions,
            } => PartBehaviour::Valve {
                positions: positions
                    .iter()
                    .map(|position| {
                        position
                            .connection
                            .iter()
                            .map(|connection| ValveConnection {
                                from: connection.from.clone(),
                                to: connection.to.clone(),
                                conductance: connection.conductance,
                            })
                            .collect()
                    })
                    .collect(),
                position_names: positions
                    .iter()
                    .map(|position| position.display_name().to_owned())
                    .collect(),
                default_position: (*default_position).min(positions.len().saturating_sub(1)),
                spring_return: *spring_return,
                actuation: *actuation,
            },
            Behaviour::Cylinder { .. } => {
                PartBehaviour::Cylinder(Cylinder::from_definition(definition)?)
            }
            Behaviour::Gauge { .. } => PartBehaviour::Gauge,
            Behaviour::Throttle { conductance } => PartBehaviour::Throttle {
                conductance: *conductance,
            },
            Behaviour::FlowMeter { conductance, .. } => PartBehaviour::FlowMeter {
                conductance: *conductance,
            },
            Behaviour::Recorder { .. } => PartBehaviour::Recorder,
            Behaviour::Plug => PartBehaviour::Plug,
            Behaviour::Silencer { conductance } => PartBehaviour::Silencer {
                conductance: *conductance,
            },
            // The electrical control circuit is not simulated yet (ADR 0017): the part
            // exists so the document can hold it, and it contributes nothing to the gas.
            Behaviour::ElectricalSupply { .. }
            | Behaviour::Contact { .. }
            | Behaviour::Relay { .. }
            | Behaviour::Lamp
            | Behaviour::Buzzer => PartBehaviour::Electrical,
            Behaviour::Junction => PartBehaviour::Junction,
        };
        Ok(Self {
            id: definition.id.clone(),
            ports,
            behaviour,
        })
    }
}

/// One end of a pipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortAddress {
    /// Index of the part in the slice passed to [`Network::build`].
    pub part: usize,
    /// Port id as the definition names it.
    pub port: String,
}

impl PortAddress {
    /// A pipe end at `part`'s `port`.
    #[must_use]
    pub fn new(part: usize, id: impl Into<String>) -> Self {
        Self {
            part,
            port: id.into(),
        }
    }
}

/// What a reservoir is for, which decides how its exchange is accounted for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReservoirKind {
    Supply,
    Atmosphere,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Reservoir {
    pressure_pa: f64,
    kind: ReservoirKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Node {
    mass: f64,
    volume: f64,
    reservoir: Option<Reservoir>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Edge {
    a: usize,
    b: usize,
    conductance: f64,
    /// Index of the valve that owns this edge, if any.
    valve: Option<usize>,
    /// Bit per position: the edge is open while the valve sits on a set bit.
    mask: u64,
    /// Part the restriction belongs to, when an instrument reads the flow through it.
    part: Option<usize>,
}

/// One chamber of a cylinder and the way its pressure pushes the piston.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Chamber {
    node: usize,
    /// Piston area this chamber acts on, `m²`.
    area: f64,
    /// `+1` when the chamber extends the rod, `-1` when it retracts it.
    sign: f64,
    /// Residual volume at the chamber's retracted end, `m`.
    dead: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct Piston {
    cylinder: Cylinder,
    chambers: Vec<Chamber>,
    position: f64,
    velocity: f64,
}

impl Piston {
    /// Volume of one chamber at the piston's current position, `m³`.
    fn chamber_volume(&self, chamber: &Chamber) -> f64 {
        let travel = if chamber.sign > 0.0 {
            self.position
        } else {
            self.cylinder.stroke - self.position
        };
        chamber.area * (travel + chamber.dead) + PORT_VOLUME
    }
}

/// The visible state of one valve.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkValve {
    /// Definition this valve came from.
    pub definition_id: String,
    /// Position names, in file order.
    pub position_names: Vec<String>,
    /// Whether releasing returns the spool.
    pub spring_return: bool,
    /// How the user changes position.
    pub actuation: Actuation,
    default: usize,
    position: usize,
}

impl NetworkValve {
    /// Number of positions.
    #[must_use]
    pub fn position_count(&self) -> usize {
        self.position_names.len()
    }

    /// Position selected before the simulation starts.
    #[must_use]
    pub fn default_position(&self) -> usize {
        self.default
    }
}

/// A circuit of lumped volumes joined by restrictions, integrated together.
#[derive(Debug, Clone)]
pub struct Network {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    pistons: Vec<Piston>,
    /// Part index of each piston, parallel to `pistons`.
    piston_parts: Vec<usize>,
    /// Part index of each valve, parallel to `valves`.
    valve_parts: Vec<usize>,
    valves: Vec<NetworkValve>,
    /// Valve the interface last touched, whose position the telemetry reports.
    active_valve: usize,
    /// Node of every port, by part and port id, for the drawing and the meters.
    port_nodes: Vec<HashMap<String, usize>>,
    supply_pa: f64,
    atmosphere_pa: f64,
    temperature_kelvin: f64,
    time: f64,
    mass_in: f64,
    mass_out: f64,
    supply_flow: f64,
    working_flow: f64,
    /// Mass flow through the restrictions an instrument owns, `kg / s`, on the last
    /// substep. `None` for a part that owns none, so a missing reading never looks like
    /// a reading of zero.
    part_flow: Vec<Option<f64>>,
}

impl Network {
    /// The built-in three-component circuit, for the printed sheet and the tests.
    ///
    /// The application never calls this: it builds its network from the document, which is
    /// the whole point of a simulator that runs what is drawn. The export still prints the
    /// worked example on the sample sheet, and the tests need a circuit without a document.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::build`] and [`NetworkPart::from_definition`].
    pub fn demo(catalog: &punchi_fluid_catalog::Catalog) -> Result<Self, SolverError> {
        use crate::pneumatic::{
            DEFAULT_CYLINDER, DEFAULT_PREPARATION, DEFAULT_SUPPLY, DEFAULT_VALVE,
        };
        // The air reaches the circuit through the service unit: source, unit, valve, cylinder,
        // which is the order the diagram reads in and the order the air travels in.
        let ids = [
            DEFAULT_SUPPLY,
            DEFAULT_PREPARATION,
            DEFAULT_VALVE,
            DEFAULT_CYLINDER,
        ];
        let mut parts = Vec::with_capacity(ids.len());
        for id in ids {
            let definition = catalog
                .get(id)
                .ok_or_else(|| SolverError::MissingDefinition(id.to_owned()))?;
            parts.push(NetworkPart::from_definition(definition)?);
        }
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
            (PortAddress::new(2, "2"), PortAddress::new(3, "1")),
        ];
        Self::build(&parts, &pipes)
    }

    /// Builds the network from the parts of the document and the pipes between them.
    ///
    /// A part whose behaviour cannot be modelled is left out by the caller; the network
    /// only sees what it can simulate. Pipes that name a missing part or port are ignored
    /// here and reported by the circuit check.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::TooManyValvePositions`] when a valve declares more positions
    /// than the edge mask can hold.
    pub fn build(
        parts: &[NetworkPart],
        pipes: &[(PortAddress, PortAddress)],
    ) -> Result<Self, SolverError> {
        let atmosphere_pa = punchi_fluid_core::Pressure::ATMOSPHERE.pascal();
        let temperature_kelvin = punchi_fluid_core::Temperature::STANDARD.kelvin();
        let (mut nodes, port_node) = port_nodes(parts, atmosphere_pa, temperature_kelvin);
        let supply_pa = attach_supplies(parts, &mut nodes, &port_node);
        let (mut edges, valves, valve_parts) = valve_edges(parts, &port_node)?;
        restriction_edges(parts, &port_node, &mut edges);
        append_pipes(&mut edges, pipes, &port_node);
        close_open_exhausts(parts, &mut nodes, &port_node, &edges, atmosphere_pa);
        silencer_edges(parts, &mut nodes, &port_node, &mut edges, atmosphere_pa);
        let (pistons, piston_parts) = chambers(
            parts,
            &mut nodes,
            &port_node,
            atmosphere_pa,
            temperature_kelvin,
        );
        let mut part_flow = vec![None; parts.len()];
        for edge in &edges {
            if let Some(part) = edge.part {
                part_flow[part] = Some(0.0);
            }
        }
        Ok(Self {
            nodes,
            edges,
            pistons,
            piston_parts,
            valves,
            valve_parts,
            active_valve: 0,
            port_nodes: owned(port_node),
            supply_pa,
            atmosphere_pa,
            temperature_kelvin,
            time: 0.0,
            mass_in: 0.0,
            mass_out: 0.0,
            supply_flow: 0.0,
            working_flow: 0.0,
            part_flow,
        })
    }

    /// Supply pressure, absolute, `Pa`.
    #[must_use]
    pub fn supply_pa(&self) -> f64 {
        self.supply_pa
    }

    /// Atmospheric pressure, `Pa`.
    #[must_use]
    pub fn atmosphere_pa(&self) -> f64 {
        self.atmosphere_pa
    }

    /// The first cylinder, if the circuit has one.
    #[must_use]
    pub fn cylinder(&self) -> Option<&Cylinder> {
        self.pistons.first().map(|piston| &piston.cylinder)
    }

    /// The valve the interface last touched, or the first one.
    #[must_use]
    pub fn valve(&self) -> Option<&NetworkValve> {
        self.valves
            .get(self.active_valve)
            .or_else(|| self.valves.first())
    }

    /// Active position of that valve.
    #[must_use]
    pub fn valve_position(&self) -> usize {
        self.valve().map_or(0, |valve| valve.position)
    }

    /// Position of the valve that came from `part`, if that part is a valve.
    #[must_use]
    pub fn valve_position_at(&self, part: usize) -> Option<usize> {
        self.valve_index_of(part)
            .map(|valve| self.valves[valve].position)
    }

    /// `true` when that part is a valve with a spring returning it.
    #[must_use]
    pub fn valve_spring_return_at(&self, part: usize) -> Option<bool> {
        self.valve_index_of(part)
            .map(|valve| self.valves[valve].spring_return)
    }

    fn valve_index_of(&self, part: usize) -> Option<usize> {
        // Valves keep the order in which `build` saw their parts, and `piston_parts`
        // teaches the same lesson for cylinders: the map is in the pieces, not the
        // placements.
        self.valve_parts
            .iter()
            .position(|candidate| *candidate == part)
    }

    /// Moves a valve to `position`, clamped to the positions it has.
    pub fn set_valve_position_at(&mut self, valve: usize, position: usize) -> bool {
        let Some(target) = self.valves.get_mut(valve) else {
            return false;
        };
        self.active_valve = valve;
        let position = position.min(target.position_names.len().saturating_sub(1));
        if position == target.position {
            return false;
        }
        target.position = position;
        true
    }

    /// Moves the first valve to `position`, clamped to the positions it has.
    pub fn set_valve_position(&mut self, position: usize) -> bool {
        self.set_valve_position_at(
            self.active_valve.min(self.valves.len().saturating_sub(1)),
            position,
        )
    }

    /// Switches the active valve to the next position, wrapping around.
    pub fn toggle_valve(&mut self) -> bool {
        let count = self.valve().map_or(0, NetworkValve::position_count);
        if count < 2 {
            return false;
        }
        self.set_valve_position((self.valve_position() + 1) % count)
    }

    /// Holds a valve in the position its actuator selects.
    pub fn press_valve_at(&mut self, valve: usize) -> bool {
        let Some(target) = self.valves.get(valve) else {
            return false;
        };
        let position = usize::from(target.default == 0);
        self.set_valve_position_at(valve, position)
    }

    /// Holds the active valve in the position its actuator selects.
    pub fn press_valve(&mut self) -> bool {
        let valve = self.active_valve.min(self.valves.len().saturating_sub(1));
        self.press_valve_at(valve)
    }

    /// Releases a valve, if a spring returns it.
    pub fn release_valve_at(&mut self, valve: usize) -> bool {
        let Some(target) = self.valves.get(valve) else {
            return false;
        };
        let default = target.default;
        if target.spring_return {
            self.set_valve_position_at(valve, default)
        } else {
            false
        }
    }

    /// Releases the active valve, if a spring returns it.
    pub fn release_valve(&mut self) -> bool {
        let valve = self.active_valve.min(self.valves.len().saturating_sub(1));
        self.release_valve_at(valve)
    }

    /// Extension in `0..=1` of the cylinder that became `part`, if it is one.
    ///
    /// The drawing animates every cylinder from this, not from a list of the built-in
    /// three: a cylinder in the catalog is a cylinder on the sheet.
    #[must_use]
    pub fn extension_of(&self, part: usize) -> Option<f64> {
        let index = self
            .piston_parts
            .iter()
            .position(|candidate| *candidate == part)?;
        let piston = self.pistons.get(index)?;
        if piston.cylinder.stroke <= 0.0 {
            return Some(0.0);
        }
        Some((piston.position / piston.cylinder.stroke).clamp(0.0, 1.0))
    }

    /// Absolute pressure at a port, `Pa`, if the part and port are part of the model.
    ///
    /// This is what a pressure gauge attached to that port would read, and what colours a
    /// hose while the simulation runs.
    #[must_use]
    pub fn port_pressure(&self, part: usize, id: &str) -> Option<f64> {
        let node = *self.port_nodes.get(part)?.get(id)?;
        Some(self.node_pressure(node))
    }

    /// Pressure of the first cylinder's chamber, absolute, `Pa`.
    #[must_use]
    pub fn pressure_pa(&self) -> f64 {
        let Some(piston) = self.pistons.first() else {
            return self.atmosphere_pa;
        };
        let Some(chamber) = piston.chambers.first() else {
            return self.atmosphere_pa;
        };
        self.node_pressure(chamber.node)
    }

    /// Gas volume of the first chamber, `m³`.
    #[must_use]
    pub fn volume(&self) -> f64 {
        let Some(piston) = self.pistons.first() else {
            return 0.0;
        };
        let Some(chamber) = piston.chambers.first() else {
            return 0.0;
        };
        self.nodes[chamber.node].volume
    }

    /// Piston position, `m`.
    #[must_use]
    pub fn position(&self) -> f64 {
        self.pistons.first().map_or(0.0, |piston| piston.position)
    }

    /// Piston position in `0..=1`.
    #[must_use]
    pub fn extension(&self) -> f64 {
        let Some(piston) = self.pistons.first() else {
            return 0.0;
        };
        if piston.cylinder.stroke <= 0.0 {
            0.0
        } else {
            piston.position / piston.cylinder.stroke
        }
    }

    /// Piston speed, `m / s`.
    #[must_use]
    pub fn velocity(&self) -> f64 {
        self.pistons.first().map_or(0.0, |piston| piston.velocity)
    }

    /// Simulated time, `s`.
    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Mass flow leaving the supply, `kg / s`, on the last substep.
    #[must_use]
    pub fn supply_flow(&self) -> f64 {
        self.supply_flow
    }

    /// Net mass flow into the first chamber, `kg / s`, on the last substep.
    #[must_use]
    pub fn working_flow(&self) -> f64 {
        self.working_flow
    }

    /// Mass flow through the restriction `part` owns, `kg / s`, on the last substep.
    ///
    /// Positive means gas from port 1 to port 2 of the definition. `None` when the part
    /// owns no restriction, so an instrument that was not built reads nothing instead of a
    /// zero that would look like a measurement.
    #[must_use]
    pub fn part_flow(&self, part: usize) -> Option<f64> {
        self.part_flow.get(part).copied().flatten()
    }

    /// The same flow in normal litres per minute.
    ///
    /// The litre is the **same normal litre ISO 6358 uses for the conductance**: 0 °C and
    /// 1 013.25 hPa, where dry air is 1.293 kg/m³. Using the model's own reference means a
    /// student can read the number off the conductance — `C · p` in normal litres per
    /// second — instead of meeting a second set of conditions in the same circuit.
    #[must_use]
    pub fn part_flow_l_min(&self, part: usize) -> Option<f64> {
        let flow = self.part_flow(part)?;
        Some(flow / orifice::normal::AIR_DENSITY * 60_000.0)
    }

    /// Builds a telemetry snapshot.
    #[must_use]
    pub fn telemetry(&self) -> Telemetry {
        Telemetry {
            time: self.time,
            pressure_pa: self.pressure_pa(),
            position: self.position(),
            velocity: self.velocity(),
            valve_position: self.valve_position(),
            mass_in: self.mass_in,
            mass_out: self.mass_out,
        }
    }

    /// Total gas mass held by the network, `kg`.
    #[must_use]
    pub fn gas_mass(&self) -> f64 {
        self.nodes
            .iter()
            .filter(|node| node.reservoir.is_none())
            .map(|node| node.mass)
            .sum()
    }

    /// Total gas mass that left a supply reservoir so far, `kg`.
    #[must_use]
    pub fn mass_in(&self) -> f64 {
        self.mass_in
    }

    /// Total gas mass that reached an atmosphere reservoir so far, `kg`.
    #[must_use]
    pub fn mass_out(&self) -> f64 {
        self.mass_out
    }

    /// Advances the simulation by `dt` seconds in bounded internal substeps.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::InvalidTimeStep`] for a non-positive or non-finite step and
    /// [`SolverError::TimeStepTooLarge`] when the step would need too many substeps.
    pub fn advance(&mut self, dt: f64) -> Result<Telemetry, SolverError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(SolverError::InvalidTimeStep(dt));
        }
        let substeps = (dt / MAX_INTERNAL_SUBSTEP).ceil();
        if substeps > f64::from(MAX_SUBSTEPS_PER_CALL) {
            return Err(SolverError::TimeStepTooLarge(dt));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let count = substeps.max(1.0) as u32;
        let substep = dt / f64::from(count);
        for _ in 0..count {
            self.substep(substep);
        }
        self.time += dt;
        Ok(self.telemetry())
    }

    fn node_pressure(&self, node: usize) -> f64 {
        let node = &self.nodes[node];
        node.reservoir.map_or_else(
            || {
                node.mass * AIR_SPECIFIC_GAS_CONSTANT * self.temperature_kelvin
                    / node.volume.max(f64::MIN_POSITIVE)
            },
            |reservoir| reservoir.pressure_pa,
        )
    }

    fn substep(&mut self, dt: f64) {
        // The chambers grow and shrink with the piston, so their volumes come first.
        for piston in &self.pistons {
            for chamber in &piston.chambers {
                let volume = piston.chamber_volume(chamber);
                self.nodes[chamber.node].volume = volume;
            }
        }

        let mut delta = vec![0.0_f64; self.nodes.len()];
        // Moved out of `self` so the loop can read the network while it writes the flows.
        let mut part_flow = std::mem::take(&mut self.part_flow);
        for slot in &mut part_flow {
            *slot = slot.map(|_| 0.0);
        }
        let mut supply_flow = 0.0;
        let mut mass_out = 0.0;
        let mut working = 0.0;
        let working_node = self
            .pistons
            .first()
            .and_then(|piston| piston.chambers.first())
            .map(|chamber| chamber.node);

        for edge in &self.edges {
            if let Some(valve) = edge.valve {
                let position = self.valves[valve].position;
                if (edge.mask >> position) & 1 == 0 {
                    continue;
                }
            }
            let pressure_a = self.node_pressure(edge.a);
            let pressure_b = self.node_pressure(edge.b);
            let flow = signed_flow(
                pressure_a,
                pressure_b,
                edge.conductance,
                self.temperature_kelvin,
            );
            if let Some(part) = edge.part {
                if let Some(total) = part_flow.get_mut(part).and_then(Option::as_mut) {
                    *total += flow;
                }
            }
            if flow == 0.0 {
                continue;
            }
            let reservoir_a = self.nodes[edge.a].reservoir;
            let reservoir_b = self.nodes[edge.b].reservoir;
            if reservoir_a.is_none() {
                delta[edge.a] -= flow * dt;
            }
            if reservoir_b.is_none() {
                delta[edge.b] += flow * dt;
            }
            // The balance counts what leaves a reservoir, in either direction: gas that
            // the chamber pushes back into the supply is mass the network no longer holds,
            // and leaving it out is how a model reports a leak that is not there.
            if let Some(reservoir) = reservoir_a {
                if reservoir.kind == ReservoirKind::Supply {
                    supply_flow += flow;
                } else if reservoir.kind == ReservoirKind::Atmosphere {
                    mass_out -= flow;
                }
            }
            if let Some(reservoir) = reservoir_b {
                if reservoir.kind == ReservoirKind::Supply {
                    supply_flow -= flow;
                } else if reservoir.kind == ReservoirKind::Atmosphere {
                    mass_out += flow;
                }
            }
            if let Some(node) = working_node {
                if edge.a == node {
                    working -= flow;
                } else if edge.b == node {
                    working += flow;
                }
            }
        }

        for (node, change) in self.nodes.iter_mut().zip(&delta) {
            if node.reservoir.is_none() {
                node.mass = (node.mass + change).max(0.0);
            }
        }
        self.mass_in += supply_flow * dt;
        self.mass_out += mass_out * dt;
        self.supply_flow = supply_flow;
        self.working_flow = working;
        self.part_flow = part_flow;

        self.step_pistons(dt);
    }

    fn step_pistons(&mut self, dt: f64) {
        let forces: Vec<f64> = self
            .pistons
            .iter()
            .map(|piston| {
                piston
                    .chambers
                    .iter()
                    .map(|chamber| {
                        chamber.sign
                            * (self.node_pressure(chamber.node) - self.atmosphere_pa)
                            * chamber.area
                    })
                    .sum()
            })
            .collect();
        for (piston, pressure_force) in self.pistons.iter_mut().zip(forces) {
            let cylinder = piston.cylinder;
            let spring_force = cylinder.spring_preload + cylinder.spring_rate * piston.position;
            let net = pressure_force - spring_force;
            let friction = if piston.velocity.abs() > 1e-6 {
                cylinder.friction * piston.velocity.signum()
            } else {
                net.clamp(-cylinder.friction, cylinder.friction)
            };
            let damping = cylinder.damping * piston.velocity;
            let acceleration = (net - friction - damping) / cylinder.moving_mass;
            piston.velocity += acceleration * dt;
            piston.position += piston.velocity * dt;
            if piston.position <= 0.0 {
                piston.position = 0.0;
                piston.velocity = 0.0;
            } else if piston.position >= cylinder.stroke {
                piston.position = cylinder.stroke;
                piston.velocity = 0.0;
            }
        }
    }
}

/// One node per port, except a junction, whose ports are the same point.
fn port_nodes(
    parts: &[NetworkPart],
    atmosphere_pa: f64,
    temperature_kelvin: f64,
) -> (Vec<Node>, Vec<HashMap<&str, usize>>) {
    let mut nodes: Vec<Node> = Vec::new();
    let mut port_node: Vec<HashMap<&str, usize>> = Vec::with_capacity(parts.len());
    for part in parts {
        let mut map: HashMap<&str, usize> = HashMap::new();
        let shared = matches!(part.behaviour, PartBehaviour::Junction);
        if shared {
            let node = new_node(&mut nodes, atmosphere_pa, temperature_kelvin);
            for port in &part.ports {
                map.insert(port.id.as_str(), node);
            }
        } else {
            for port in &part.ports {
                let node = new_node(&mut nodes, atmosphere_pa, temperature_kelvin);
                map.insert(port.id.as_str(), node);
            }
        }
        port_node.push(map);
    }
    (nodes, port_node)
}

fn new_node(nodes: &mut Vec<Node>, atmosphere_pa: f64, temperature_kelvin: f64) -> usize {
    let index = nodes.len();
    nodes.push(Node {
        mass: atmosphere_pa * PORT_VOLUME / (AIR_SPECIFIC_GAS_CONSTANT * temperature_kelvin),
        volume: PORT_VOLUME,
        reservoir: None,
    });
    index
}

/// Turns the supply ports into fixed-pressure reservoirs and returns the supply pressure
/// the drawing colours against.
fn attach_supplies(
    parts: &[NetworkPart],
    nodes: &mut [Node],
    port_node: &[HashMap<&str, usize>],
) -> f64 {
    let mut supply_pa = punchi_fluid_core::Pressure::ATMOSPHERE.pascal();
    let mut seen = false;
    for (index, part) in parts.iter().enumerate() {
        let PartBehaviour::Supply { pressure_pa } = part.behaviour else {
            continue;
        };
        let Some(&node) = port_node[index].values().next() else {
            continue;
        };
        nodes[node].reservoir = Some(Reservoir {
            pressure_pa,
            kind: ReservoirKind::Supply,
        });
        if !seen {
            supply_pa = pressure_pa;
            seen = true;
        }
    }
    supply_pa
}

/// Edges, visible valves and the part each valve came from.
type ValveBuild = (Vec<Edge>, Vec<NetworkValve>, Vec<usize>);

/// One edge per connection of each valve position, with a bit per position.
fn valve_edges(
    parts: &[NetworkPart],
    port_node: &[HashMap<&str, usize>],
) -> Result<ValveBuild, SolverError> {
    let mut valves = Vec::new();
    let mut valve_parts = Vec::new();
    let mut edges = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let PartBehaviour::Valve {
            positions,
            position_names,
            default_position,
            spring_return,
            actuation,
        } = &part.behaviour
        else {
            continue;
        };
        if positions.len() > MAX_POSITIONS {
            return Err(SolverError::TooManyValvePositions {
                valve: part.id.clone(),
                count: positions.len(),
            });
        }
        let valve = valves.len();
        valve_parts.push(index);
        valves.push(NetworkValve {
            definition_id: part.id.clone(),
            position_names: position_names.clone(),
            spring_return: *spring_return,
            actuation: *actuation,
            default: *default_position,
            position: *default_position,
        });
        for (position, connections) in positions.iter().enumerate() {
            for connection in connections {
                let (Some(&a), Some(&b)) = (
                    port_node[index].get(connection.from.as_str()),
                    port_node[index].get(connection.to.as_str()),
                ) else {
                    continue;
                };
                edges.push(Edge {
                    a,
                    b,
                    conductance: connection.conductance,
                    valve: Some(valve),
                    mask: 1_u64 << position,
                    part: None,
                });
            }
        }
    }
    Ok((edges, valves, valve_parts))
}

/// A fixed restriction and a flow meter are edges between their own two ports.
///
/// A restrictor is part of the circuit and not a dead end: without this edge the air stops at
/// it instead of being slowed by it, and a service unit in series cuts the line in two. The
/// conductance comes from the definition, so a slower restrictor is a different file and not
/// a different code path. A meter is a restriction that also reports the gas it carried, so
/// its edge remembers which part owns it (ADR 0014).
fn restriction_edges(
    parts: &[NetworkPart],
    port_node: &[HashMap<&str, usize>],
    edges: &mut Vec<Edge>,
) {
    for (index, part) in parts.iter().enumerate() {
        let (conductance, meter) = match part.behaviour {
            PartBehaviour::Throttle { conductance } => (conductance, false),
            PartBehaviour::FlowMeter { conductance } => (conductance, true),
            _ => continue,
        };
        let mut nodes = part
            .ports
            .iter()
            .filter_map(|port| port_node[index].get(port.id.as_str()).copied());
        let (Some(a), Some(b)) = (nodes.next(), nodes.next()) else {
            continue;
        };
        edges.push(Edge {
            a,
            b,
            conductance,
            valve: None,
            mask: u64::MAX,
            part: meter.then_some(index),
        });
    }
}

fn append_pipes(
    edges: &mut Vec<Edge>,
    pipes: &[(PortAddress, PortAddress)],
    port_node: &[HashMap<&str, usize>],
) {
    for (from, to) in pipes {
        let (Some(&a), Some(&b)) = (
            port_node
                .get(from.part)
                .and_then(|map| map.get(from.port.as_str())),
            port_node
                .get(to.part)
                .and_then(|map| map.get(to.port.as_str())),
        ) else {
            continue;
        };
        edges.push(Edge {
            a,
            b,
            conductance: PIPE_CONDUCTANCE,
            valve: None,
            mask: u64::MAX,
            part: None,
        });
    }
}

/// An exhaust port with no hose breathes to atmosphere, which is what the reference shows
/// and what makes the built-in 3/2 valve work when its port 3 is left open.
fn close_open_exhausts(
    parts: &[NetworkPart],
    nodes: &mut [Node],
    port_node: &[HashMap<&str, usize>],
    edges: &[Edge],
    atmosphere_pa: f64,
) {
    let mut connected = vec![false; nodes.len()];
    for edge in edges {
        if edge.valve.is_none() {
            connected[edge.a] = true;
            connected[edge.b] = true;
        }
    }
    for (index, part) in parts.iter().enumerate() {
        for port in &part.ports {
            if port.role != PortRole::Exhaust {
                continue;
            }
            let Some(&node) = port_node[index].get(port.id.as_str()) else {
                continue;
            };
            if !connected[node] && nodes[node].reservoir.is_none() {
                nodes[node].reservoir = Some(Reservoir {
                    pressure_pa: atmosphere_pa,
                    kind: ReservoirKind::Atmosphere,
                });
            }
        }
    }
}

/// A silencer vents its port to atmosphere through the restriction its file declares.
///
/// The port keeps its own node — the air still crosses the hose — and the edge is what the
/// silencer adds, so a narrow one really slows the exhaust it is fitted to (ADR 0016).
fn silencer_edges(
    parts: &[NetworkPart],
    nodes: &mut Vec<Node>,
    port_node: &[HashMap<&str, usize>],
    edges: &mut Vec<Edge>,
    atmosphere_pa: f64,
) {
    for (index, part) in parts.iter().enumerate() {
        let PartBehaviour::Silencer { conductance } = part.behaviour else {
            continue;
        };
        let Some(port) = part.ports.first() else {
            continue;
        };
        let Some(&node) = port_node[index].get(port.id.as_str()) else {
            continue;
        };
        let atmosphere = nodes.len();
        nodes.push(Node {
            mass: atmosphere_pa * PORT_VOLUME
                / (AIR_SPECIFIC_GAS_CONSTANT * punchi_fluid_core::Temperature::STANDARD.kelvin()),
            volume: PORT_VOLUME,
            reservoir: Some(Reservoir {
                pressure_pa: atmosphere_pa,
                kind: ReservoirKind::Atmosphere,
            }),
        });
        edges.push(Edge {
            a: node,
            b: atmosphere,
            conductance,
            valve: None,
            mask: u64::MAX,
            part: None,
        });
    }
}

/// Each working port becomes a chamber. One chamber is a single-acting cylinder; two are
/// the extend and retract chambers of a double-acting one, in port order.
fn chambers(
    parts: &[NetworkPart],
    nodes: &mut [Node],
    port_node: &[HashMap<&str, usize>],
    atmosphere_pa: f64,
    temperature_kelvin: f64,
) -> (Vec<Piston>, Vec<usize>) {
    let mut pistons = Vec::new();
    let mut piston_parts = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let PartBehaviour::Cylinder(cylinder) = &part.behaviour else {
            continue;
        };
        let mut built: Vec<Chamber> = Vec::new();
        for port in part
            .ports
            .iter()
            .filter(|port| port.role == PortRole::Working)
            .take(2)
        {
            let Some(&node) = port_node[index].get(port.id.as_str()) else {
                continue;
            };
            built.push(Chamber {
                node,
                area: if built.is_empty() {
                    cylinder.area()
                } else {
                    cylinder.annulus_area()
                },
                sign: if built.is_empty() { 1.0 } else { -1.0 },
                dead: cylinder.dead_length,
            });
        }
        if built.is_empty() {
            continue;
        }
        let piston = Piston {
            cylinder: *cylinder,
            chambers: built,
            position: 0.0,
            velocity: 0.0,
        };
        for chamber in &piston.chambers {
            let volume = piston.chamber_volume(chamber);
            nodes[chamber.node].volume = volume;
            nodes[chamber.node].mass =
                atmosphere_pa * volume / (AIR_SPECIFIC_GAS_CONSTANT * temperature_kelvin);
        }
        piston_parts.push(index);
        pistons.push(piston);
    }
    (pistons, piston_parts)
}

/// Copies the borrowed port map into the strings the network owns.
fn owned(port_node: Vec<HashMap<&str, usize>>) -> Vec<HashMap<String, usize>> {
    port_node
        .into_iter()
        .map(|map| {
            map.into_iter()
                .map(|(port, node)| (port.to_owned(), node))
                .collect()
        })
        .collect()
}

/// Mass flow from `a` to `b`, `kg / s`, positive towards `b`.
fn signed_flow(pressure_a: f64, pressure_b: f64, conductance: f64, temperature_kelvin: f64) -> f64 {
    if (pressure_a - pressure_b).abs() <= f64::EPSILON {
        return 0.0;
    }
    if pressure_a > pressure_b {
        orifice::mass_flow(
            conductance,
            pressure_a,
            pressure_b,
            temperature_kelvin,
            DEFAULT_CRITICAL_RATIO,
        )
        .unwrap_or(0.0)
    } else {
        -orifice::mass_flow(
            conductance,
            pressure_b,
            pressure_a,
            temperature_kelvin,
            DEFAULT_CRITICAL_RATIO,
        )
        .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use punchi_fluid_catalog::Catalog;

    fn part(catalog: &Catalog, id: &str) -> NetworkPart {
        let definition = catalog.get(id).expect("definition");
        NetworkPart::from_definition(definition).expect("a part")
    }

    fn chain() -> Network {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
        ];
        Network::build(&parts, &pipes).expect("the chain builds")
    }

    #[test]
    fn a_chain_is_the_same_circuit_the_fixed_model_simulates() {
        let mut network = chain();
        let cylinder = *network.cylinder().expect("a cylinder");
        assert!(network.press_valve(), "the spring valve opens");
        for _ in 0..60 {
            network.advance(0.016).expect("a frame is accepted");
        }
        assert!(
            network.position() >= cylinder.stroke - 1e-6,
            "the piston reached {} of {} mm",
            network.position() * 1000.0,
            cylinder.stroke * 1000.0
        );
        assert!(
            network.pressure_pa() > network.atmosphere_pa() + 5.0e5,
            "the chamber is at supply pressure, not {} bar",
            network.pressure_pa() / 1e5
        );
    }

    #[test]
    fn a_port_reports_the_pressure_a_gauge_would_read() {
        let mut network = chain();
        let atmosphere = network.atmosphere_pa();
        let cylinder = network
            .port_pressure(2, "1")
            .expect("the cylinder port is in the model");
        assert!((cylinder - atmosphere).abs() < 1.0e-6);
        assert!(network.port_pressure(2, "9").is_none());
        assert!(network.port_pressure(9, "1").is_none());

        network.press_valve();
        for _ in 0..30 {
            network.advance(0.016).expect("accepted");
        }
        let cylinder = network
            .port_pressure(2, "1")
            .expect("the cylinder port is in the model");
        assert!(
            cylinder > atmosphere + 5.0e5,
            "the gauge reads {:.2} bar",
            (cylinder - atmosphere) / 1e5
        );
    }

    #[test]
    fn the_gas_that_entered_is_the_gas_the_network_holds() {
        let mut network = chain();
        let initial = network.gas_mass();
        network.press_valve();
        for _ in 0..40 {
            network.advance(0.016).expect("accepted");
        }
        let expected = initial + network.mass_in() - network.mass_out();
        let measured = network.gas_mass();

        assert!(
            (measured - expected).abs() / expected < 1e-9,
            "final {measured:e} expected {expected:e}"
        );
    }

    #[test]
    fn releasing_the_valve_lets_the_spring_return_the_rod() {
        let mut network = chain();
        network.press_valve();
        for _ in 0..10 {
            network.advance(0.016).expect("accepted");
        }
        let reached = network.position();
        for _ in 0..80 {
            network.advance(0.016).expect("accepted");
        }
        println!(
            "before release x={:.1} p={:.3} valve={}",
            network.position() * 1000.0,
            network.pressure_pa() / 1e5,
            network.valve_position()
        );
        network.release_valve();
        println!(
            "after release valve={} p={:.3}",
            network.valve_position(),
            network.pressure_pa() / 1e5
        );
        // The chamber has to vent through the valve before the spring can take over, so
        // this is a second, not a frame.
        for frame in 0..100 {
            network.advance(0.016).expect("accepted");
            if frame % 10 == 0 {
                println!(
                    "  {frame} x={:.1} p={:.3} v={:.1}",
                    network.position() * 1000.0,
                    network.pressure_pa() / 1e5,
                    network.velocity() * 1000.0
                );
            }
        }
        assert!(
            network.position() < reached,
            "with the valve at rest the working line vents and the spring returns the rod"
        );
    }

    #[test]
    fn a_junction_joins_its_three_ports_into_one_volume() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
            part(&catalog, "junction.tee"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(3, "1")),
            (PortAddress::new(3, "2"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(3, "3")),
            (PortAddress::new(3, "3"), PortAddress::new(2, "1")),
        ];
        let mut network = Network::build(&parts, &pipes).expect("the junction builds");
        network.press_valve();
        for _ in 0..60 {
            network.advance(0.016).expect("accepted");
        }
        let stroke = network.cylinder().expect("a cylinder").stroke;
        assert!(network.position() >= stroke - 1e-6);
    }

    #[test]
    fn a_gauge_reads_its_port_and_adds_nothing_to_the_physics() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
            part(&catalog, "gauge.pressure"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(3, "1")),
        ];
        let mut network = Network::build(&parts, &pipes).expect("the gauge builds");
        network.press_valve();
        for _ in 0..60 {
            network.advance(0.016).expect("accepted");
        }
        let reading = network
            .port_pressure(3, "1")
            .expect("the gauge port is in the model");
        let line = network
            .port_pressure(1, "2")
            .expect("the valve's working port is in the model");
        assert!(
            (reading - line).abs() / line < 0.01,
            "the gauge reads {:.3} bar and the working line {:.3} bar",
            (reading - network.atmosphere_pa()) / 1e5,
            (line - network.atmosphere_pa()) / 1e5
        );
        let stroke = network.cylinder().expect("a cylinder").stroke;
        assert!(
            network.position() > stroke * 0.5,
            "the gauge changed the circuit: the rod only reached {:.1} mm",
            network.position() * 1000.0
        );
    }

    #[test]
    fn a_five_port_valve_file_drives_a_double_acting_cylinder() {
        // The acceptance for F2 in one test: a valve and a cylinder that exist only as
        // files, wired the way a schematic is wired, with no branch in the solver.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.5_2.spring_return"),
            part(&catalog, "cylinder.double_acting_32_200"),
        ];
        // Port 4 is the line the valve feeds while actuated, so it goes to the cap end
        // and port 2, fed at rest, to the rod end.
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "4"), PortAddress::new(2, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "2")),
        ];
        let mut network = Network::build(&parts, &pipes).expect("the five-port chain builds");
        let stroke = network.cylinder().expect("a cylinder").stroke;
        assert_eq!(network.valve().expect("a valve").position_count(), 2);

        // Pressed, the valve feeds port 1 and vents port 2: the rod goes out.
        assert!(network.press_valve());
        for _ in 0..500 {
            network.advance(0.016).expect("accepted");
        }
        assert!(
            network.position() >= stroke - 1e-6,
            "pressed, the rod reached {:.1} of {:.1} mm",
            network.position() * 1000.0,
            stroke * 1000.0
        );

        // Released, the spring returns the spool, the supply feeds port 2 and the rod
        // comes back. Two chambers means the model has two volumes, and the pressure on
        // the annulus is what pushes the piston in.
        assert!(network.release_valve());
        for _ in 0..500 {
            network.advance(0.016).expect("accepted");
        }
        assert!(
            network.position() < 0.002,
            "released, the rod stayed at {:.1} mm",
            network.position() * 1000.0
        );
        assert!(
            network.port_pressure(2, "2").is_some(),
            "the retract port is part of the model"
        );
    }

    #[test]
    fn a_flow_meter_reads_the_flow_it_carries() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
            part(&catalog, "flow.meter"),
        ];
        // The meter sits in the line the valve feeds: port 1 towards the valve, port 2
        // towards the cylinder, which is what fixes the sign of the reading.
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(3, "1")),
            (PortAddress::new(3, "2"), PortAddress::new(2, "1")),
        ];
        let mut network = Network::build(&parts, &pipes).expect("the meter builds");
        assert_eq!(
            network.part_flow(3),
            Some(0.0),
            "a meter that has not met any air reads zero, not a guess"
        );

        // Early in the stroke, while the chamber is still filling: a meter read after the
        // piston has stopped would show nothing and the test would pass for the wrong
        // reason.
        network.press_valve();
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }
        let filling = network.part_flow(3).expect("the meter owns a restriction");
        assert!(
            filling > 0.0,
            "air enters through port 1, so the reading is positive, got {filling}"
        );
        // The quoted unit is the normal litre of ISO 6358: 0 °C and 1 013.25 hPa, where dry
        // air is 1.293 kg/m³. The density cancels in the choked branch, which is why this
        // number does not depend on which of the two normal references a data sheet picks.
        let litres = network
            .part_flow_l_min(3)
            .expect("the flow converts to litres");
        let expected = filling / 1.293 * 60_000.0;
        assert!(
            (litres - expected).abs() / expected < 0.01,
            "{litres:.1} l/min against the hand calculation {expected:.1} l/min"
        );

        // Releasing the valve exhausts the cylinder through the meter: the reading has to
        // follow the air, sign included, or it reports the size of the flow and lies about
        // its direction.
        network.release_valve();
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }
        let emptying = network.part_flow(3).expect("the meter owns a restriction");
        assert!(
            emptying < 0.0,
            "air leaves through port 1, so the reading is negative, got {emptying}"
        );

        // When the piston reaches its stop the meter reads what is still moving, which is
        // nothing: the reading has to fall by an order of magnitude, or the instrument is
        // reporting the size of the model's noise instead of the circuit.
        for _ in 0..400 {
            network.advance(0.016).expect("accepted");
        }
        let settled = network
            .part_flow_l_min(3)
            .expect("the meter owns a restriction")
            .abs();
        assert!(
            settled < litres * 0.05,
            "the settled meter reads {settled:.2} l/min against {litres:.1} while filling"
        );
    }

    #[test]
    fn a_flow_meter_matches_the_iso_6358_flow_through_its_restriction() {
        // The reading has to be the flow of the restriction the file declares, not a number
        // that merely rises and falls with the circuit. The reference is ISO 6358 evaluated
        // at the two port pressures the model reports, with the conductance and the critical
        // ratio the model itself uses.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
            part(&catalog, "flow.meter"),
        ];
        let conductance = match parts[3].behaviour {
            PartBehaviour::FlowMeter { conductance } => conductance,
            // The definition is the built-in meter, so another behaviour is a broken test.
            _ => 0.0,
        };
        assert!(conductance > 0.0, "the meter has no conductance");
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(3, "1")),
            (PortAddress::new(3, "2"), PortAddress::new(2, "1")),
        ];
        let mut network = Network::build(&parts, &pipes).expect("the meter builds");
        network.press_valve();
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }

        let one = network
            .port_pressure(3, "1")
            .expect("the meter's first port is in the model");
        let two = network
            .port_pressure(3, "2")
            .expect("the meter's second port is in the model");
        let (upstream, downstream, sign) = if one >= two {
            (one, two, 1.0)
        } else {
            (two, one, -1.0)
        };
        let expected = sign
            * orifice::mass_flow(
                conductance,
                upstream,
                downstream,
                network.temperature_kelvin,
                DEFAULT_CRITICAL_RATIO,
            )
            .expect("the pressures are physical");
        let read = network.part_flow(3).expect("the meter owns a restriction");
        assert!(
            expected.abs() > 1.0e-6,
            "the test needs a real flow, got {expected}"
        );
        assert!(
            (read - expected).abs() / expected.abs() < 0.01,
            "the meter reads {read:.3e} kg/s against the ISO 6358 flow {expected:.3e} kg/s \
             between {:.2} bar and {:.2} bar",
            (upstream - network.atmosphere_pa()) / 1e5,
            (downstream - network.atmosphere_pa()) / 1e5
        );
    }

    #[test]
    fn a_part_without_a_restriction_reports_nothing() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "throttle.fixed"),
        ];
        let pipes = [(PortAddress::new(0, "1"), PortAddress::new(1, "1"))];
        let network = Network::build(&parts, &pipes).expect("builds");
        assert_eq!(network.part_flow(0), None, "a supply owns no restriction");
        assert_eq!(network.part_flow(9), None, "there is no ninth part");
    }

    #[test]
    fn a_throttle_between_the_valve_and_the_cylinder_slows_the_fill() {
        let catalog = Catalog::builtin();
        let mut with = vec![
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
            part(&catalog, "throttle.fixed"),
        ];
        // A throttle far slower than the hose, so the difference is measurable in a frame.
        with[3].behaviour = PartBehaviour::Throttle {
            conductance: 2.0e-11,
        };
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(3, "1")),
            (PortAddress::new(3, "2"), PortAddress::new(2, "1")),
        ];
        let mut throttled = Network::build(&with, &pipes).expect("builds");
        throttled.press_valve();
        for _ in 0..10 {
            throttled.advance(0.016).expect("accepted");
        }
        let mut direct = chain();
        direct.press_valve();
        for _ in 0..10 {
            direct.advance(0.016).expect("accepted");
        }
        // A restriction slows the fill, it does not stop it: without its own edge the air
        // would stop at the restrictor, the piston would never move, and the comparison above
        // would hold for the wrong reason.
        assert!(
            throttled.position() > 0.0,
            "a restrictor conducts: the piston must still move, got {} mm",
            throttled.position() * 1000.0
        );
        assert!(
            throttled.position() < direct.position(),
            "throttled {} mm should lag direct {} mm",
            throttled.position() * 1000.0,
            direct.position() * 1000.0
        );
    }

    #[test]
    fn a_silencer_slows_the_exhaust_it_is_fitted_to() {
        // The valve vents its working line through port 3: an open port breathes at the
        // port's conductance, a silencer at the narrower one its file declares, so the rod
        // comes back later. That difference is the whole reason a silencer is a component.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
            part(&catalog, "silencer.pneumatic"),
        ];
        let direct_pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
        ];
        let silenced_pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
            (PortAddress::new(1, "3"), PortAddress::new(3, "1")),
        ];
        let mut direct =
            Network::build(&parts, &direct_pipes).expect("builds without the silencer");
        let mut silenced = Network::build(&parts, &silenced_pipes).expect("builds with it");
        for network in [&mut direct, &mut silenced] {
            network.press_valve();
            for _ in 0..80 {
                network.advance(0.016).expect("accepted");
            }
            network.release_valve();
            // Forty frames is 0.64 s: long enough for the open exhaust to let the spring
            // start the rod back, short enough that a restricted one is still holding it.
            for _ in 0..40 {
                network.advance(0.016).expect("accepted");
            }
        }
        let stroke = direct.cylinder().expect("a cylinder").stroke;
        assert!(
            direct.position() < stroke * 0.95,
            "the open exhaust did not let the rod return: {:.1} mm",
            direct.position() * 1000.0
        );
        assert!(
            silenced.position() > direct.position(),
            "silenced {:.1} mm should lag direct {:.1} mm",
            silenced.position() * 1000.0,
            direct.position() * 1000.0
        );
    }

    #[test]
    fn a_blind_plug_traps_the_air_and_so_holds_the_rod() {
        // A capped exhaust is not an open one: the cylinder can no longer vent, so it stays
        // where the pressure left it. This is what the plug means, and it is why the model
        // has no physics for it beyond closing the line.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
            part(&catalog, "plug.blind"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
            (PortAddress::new(1, "3"), PortAddress::new(3, "1")),
        ];
        let mut network = Network::build(&parts, &pipes).expect("the plug builds");
        network.press_valve();
        for _ in 0..80 {
            network.advance(0.016).expect("accepted");
        }
        let stroke = network.cylinder().expect("a cylinder").stroke;
        let extended = network.position();
        assert!(extended > stroke * 0.5, "the rod did not extend");
        network.release_valve();
        for _ in 0..40 {
            network.advance(0.016).expect("accepted");
        }
        assert!(
            (network.position() - extended).abs() < stroke * 0.05,
            "the capped cylinder moved from {:.1} mm to {:.1} mm",
            extended * 1000.0,
            network.position() * 1000.0
        );
    }

    #[test]
    fn a_valve_with_too_many_positions_is_refused_instead_of_masked() {
        let catalog = Catalog::builtin();
        let mut valve = part(&catalog, "valve.3_2.spring_return");
        if let PartBehaviour::Valve { positions, .. } = &mut valve.behaviour {
            positions.extend((0..70).map(|_| Vec::new()));
        }
        let parts = [valve];
        assert!(matches!(
            Network::build(&parts, &[]),
            Err(SolverError::TooManyValvePositions { .. })
        ));
    }

    #[test]
    fn a_pipe_to_a_missing_port_is_ignored_not_fatal() {
        let catalog = Catalog::builtin();
        let parts = [part(&catalog, "supply.compressor")];
        let pipes = [(PortAddress::new(0, "1"), PortAddress::new(7, "1"))];
        let network = Network::build(&parts, &pipes).expect("still builds");
        assert!(network.cylinder().is_none());
    }
}

#[cfg(test)]
mod electrical_tests {
    use super::*;
    use punchi_fluid_catalog::Catalog;

    fn part(catalog: &Catalog, id: &str) -> NetworkPart {
        let definition = catalog.get(id).expect("definition");
        NetworkPart::from_definition(definition).expect("a part")
    }

    #[test]
    fn an_electrical_component_adds_nothing_to_the_gas() {
        // The control circuit is drawn and checked, and the air model ignores it: the same
        // pneumatic circuit with and without a supply and a contact must behave identically.
        let catalog = Catalog::builtin();
        let base = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
        ];
        let mut with = base.to_vec();
        with.push(part(&catalog, "electrical.supply"));
        with.push(part(&catalog, "electrical.contact"));
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
        ];
        let mut pneumatic = Network::build(&base, &pipes).expect("builds");
        let mut control = Network::build(&with, &pipes).expect("builds with the control circuit");
        for network in [&mut pneumatic, &mut control] {
            network.press_valve();
            for _ in 0..40 {
                network.advance(0.016).expect("accepted");
            }
        }
        assert!(
            (control.position() - pneumatic.position()).abs() < 1e-9,
            "the control circuit changed the stroke: {:.3} mm against {:.3} mm",
            control.position() * 1000.0,
            pneumatic.position() * 1000.0
        );
        assert!(
            (control.pressure_pa() - pneumatic.pressure_pa()).abs() < 1e-6,
            "the control circuit changed the pressure"
        );
    }
}
