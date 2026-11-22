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

use std::collections::{HashMap, HashSet};

use punchi_fluid_catalog::{Actuation, Behaviour, ComponentDefinition, PortRole, PressureKind};

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
/// The shuttle's ball is pushed by a live inlet; below this the inlet is spent.
const SHUTTLE_LIVE_BAND_PA: f64 = 5_000.0;
/// Two live inlets this close hold the ball centered, closing both seats.
const SHUTTLE_EQUAL_BAND_PA: f64 = 2_000.0;

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
    /// Passive pressure element: a shuttle or a two-pressure valve (ADR 0029). The rule
    /// is the next slice; the network refuses it for now instead of guessing.
    Pressure {
        /// Which pressure rule it follows.
        kind: PressureKind,
    },
}

/// One passive pressure element, resolved to its three nodes (ADR 0029).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PressureElement {
    kind: PressureKind,
    /// Part of the circuit this element resolves, for the live state of its symbol.
    part: usize,
    inlet_a: usize,
    inlet_b: usize,
    outlet: usize,
}

/// Seat of a shuttle's ball in its three positions (ADR 0041).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BallSeat {
    /// Seated on the left seat: the left inlet is closed and the right one feeds.
    Left,
    /// Centred: both live inlets push it, both seats close and the outlet is isolated.
    Centre,
    /// Seated on the right seat: the right inlet is closed and the left one feeds.
    Right,
}

fn shuttle_ball(a: f64, b: f64, atmosphere: f64) -> BallSeat {
    let live_a = a > atmosphere + SHUTTLE_LIVE_BAND_PA;
    let live_b = b > atmosphere + SHUTTLE_LIVE_BAND_PA;
    if live_a && live_b && (a - b).abs() <= SHUTTLE_EQUAL_BAND_PA {
        BallSeat::Centre
    } else if a >= b {
        BallSeat::Right
    } else {
        BallSeat::Left
    }
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
            Behaviour::Pressure { kind } => PartBehaviour::Pressure { kind: *kind },
            // The electrical control circuit is not simulated yet (ADR 0017): the part
            // exists so the document can hold it, and it contributes nothing to the gas.
            Behaviour::ElectricalSupply { .. }
            | Behaviour::Contact { .. }
            | Behaviour::Relay { .. }
            | Behaviour::LimitSwitch { .. }
            | Behaviour::PressureSwitch { .. }
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

/// How a closure terminates a port in the gas network.
///
/// Only an exhaust receives one: a supply or working port with no pipe is already sealed,
/// so a plug there changes the drawing and the parts list, not the graph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortTermination {
    /// A blind plug: the port is sealed, with no atmosphere boundary and no edge.
    Closed,
    /// A silencer: the port vents to atmosphere through this conductance.
    Vent {
        /// Sonic conductance, `m³ / (s · Pa)`.
        conductance: f64,
    },
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
    /// Document connection this edge models, when it is a hose and not an internal path.
    ///
    /// The index addresses the document's connection list, not the slice passed to
    /// [`Network::build`], so a connection the caller could not model keeps its slot.
    pipe: Option<usize>,
    /// Exhaust reading this edge feeds, when it is the vent of a closed port.
    ///
    /// A free exhaust has no edge at all: the port node itself is the atmosphere, and the
    /// internal path that feeds it carries the reading.
    vent: Option<usize>,
}

/// One exhaust port with an atmosphere boundary and the reading that belongs to it.
///
/// A free exhaust vents through the port node itself (the atmosphere reservoir); a silencer
/// vents through its own edge. Both carry a reading, because a student must see the air
/// leaving the valve, not only entering the hose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExhaustPort {
    /// Index of the part in the slice passed to [`Network::build`].
    pub part: usize,
    /// Port identifier as the definition names it.
    pub port: String,
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
    /// Signed flow of every document connection, in normal litres per minute, averaged over
    /// the substeps of the last call. Indexed by the document connection list; `None` for a
    /// connection the model could not represent, `Some(0.0)` for a modelled hose that
    /// carried nothing.
    pipe_flows: Vec<Option<f64>>,
    /// Running sum for [`Self::pipe_flows`], one slot per document connection.
    ///
    /// A reading is the **net** flow of the frame, not the sign of one substep: a dead-end
    /// line chatters between equal and opposite flows every substep, and reporting the last
    /// one made the sheet show air running backwards on a line that was still.
    pipe_flow_sums: Vec<f64>,
    /// Running sum for [`Self::part_flow`], one slot per part.
    part_flow_sums: Vec<f64>,
    /// Every exhaust port with an atmosphere boundary, free or closed by a silencer.
    exhausts: Vec<ExhaustPort>,
    /// Signed flow leaving each exhaust to the atmosphere on the last frame, normal litres
    /// per minute. `Some(0.0)` is a port that vents nothing right now.
    exhaust_flows: Vec<Option<f64>>,
    /// Running sum for [`Self::exhaust_flows`].
    exhaust_sums: Vec<f64>,
    /// Port node to exhaust reading index, for the free vents whose node is the atmosphere.
    exhaust_node: HashMap<usize, usize>,
    /// Passive pressure elements, resolved to their three nodes (ADR 0029).
    pressure: Vec<PressureElement>,
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
            (0, PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (1, PortAddress::new(1, "2"), PortAddress::new(2, "1")),
            (2, PortAddress::new(2, "2"), PortAddress::new(3, "1")),
        ];
        Self::build(&parts, &pipes, &[], pipes.len())
    }

    /// Builds the network from the parts of the document, the pipes between them and the
    /// closures chosen on their exhaust ports.
    ///
    /// A part whose behaviour cannot be modelled is left out by the caller; the network
    /// only sees what it can simulate. Pipes that name a missing part or port are ignored
    /// here and reported by the circuit check. Terminations are resolved by the caller from
    /// the catalog and only apply to an exhaust port: a `Closed` or `Vent` on any other
    /// port is not a change to the graph.
    ///
    /// Each pipe carries the index of its connection in the **document's** list before its
    /// two ends, and `pipe_count` is that whole list's length. The count sizes the flow
    /// vector, so a connection this build could not represent keeps its slot as `None`
    /// instead of letting the next hose inherit its index.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::TooManyValvePositions`] when a valve declares more positions
    /// than the edge mask can hold.
    pub fn build(
        parts: &[NetworkPart],
        pipes: &[(usize, PortAddress, PortAddress)],
        terminations: &[(PortAddress, PortTermination)],
        pipe_count: usize,
    ) -> Result<Self, SolverError> {
        let atmosphere_pa = punchi_fluid_core::Pressure::ATMOSPHERE.pascal();
        let temperature_kelvin = punchi_fluid_core::Temperature::STANDARD.kelvin();
        let (mut nodes, port_node) = port_nodes(parts, atmosphere_pa, temperature_kelvin);
        let pressure = pressure_elements(parts, &port_node)?;
        let supply_pa = attach_supplies(parts, &mut nodes, &port_node);
        let (mut edges, valves, valve_parts) = valve_edges(parts, &port_node)?;
        restriction_edges(parts, &port_node, &mut edges);
        append_pipes(&mut edges, pipes, &port_node);
        // The closed exhausts are known before the free-vent pass: a plug must not receive
        // the atmosphere reservoir that pass would give it, and a vent replaces it with
        // its own edge. Collecting them first keeps the order out of the result.
        let terminated = terminated_exhausts(parts, terminations, &port_node);
        close_open_exhausts(
            parts,
            &mut nodes,
            &port_node,
            &edges,
            atmosphere_pa,
            &terminated,
        );
        let vent_edge_start = edges.len();
        vent_edges(
            parts,
            terminations,
            &mut nodes,
            &port_node,
            &mut edges,
            atmosphere_pa,
            temperature_kelvin,
        );
        silencer_edges(parts, &mut nodes, &port_node, &mut edges, atmosphere_pa);
        // Every exhaust with an atmosphere boundary carries a reading: a free vent is the
        // port node itself, and a silencer is the vent edge that leaves it. A plugged port
        // is collected too, with a zero that never draws anything.
        let ExhaustReadings {
            ports: exhausts,
            node: exhaust_node,
            flows: exhaust_flows,
            sums: exhaust_sums,
        } = collect_exhausts(
            parts,
            &nodes,
            &port_node,
            &terminated,
            &mut edges,
            vent_edge_start,
        );
        let (pistons, piston_parts) = chambers(
            parts,
            &mut nodes,
            &port_node,
            atmosphere_pa,
            temperature_kelvin,
        );
        let mut part_flow = vec![None; parts.len()];
        let mut pipe_flows = vec![None; pipe_count];
        let pipe_flow_sums = vec![0.0; pipe_count];
        let part_flow_sums = vec![0.0; parts.len()];
        for edge in &edges {
            if let Some(part) = edge.part {
                part_flow[part] = Some(0.0);
            }
            // A modelled hose reads zero until the first substep moves air; an
            // unrepresentable connection is never an edge and stays `None`.
            if let Some(pipe) = edge.pipe {
                if let Some(slot) = pipe_flows.get_mut(pipe) {
                    *slot = Some(0.0);
                }
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
            pipe_flows,
            pipe_flow_sums,
            part_flow_sums,
            exhausts,
            exhaust_flows,
            exhaust_sums,
            exhaust_node,
            pressure,
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

    /// Signed flow through each document connection over the last call's substeps, normal
    /// litres per minute.
    ///
    /// Indexed by the **document's connection list**, not by the slice handed to
    /// [`Self::build`]: a connection the model could not represent is `None` and keeps its
    /// slot, so one hose never reports another's air. `Some(0.0)` is a modelled hose that
    /// carried nothing. Positive means from the connection's `from` to its `to`, the order
    /// the document enumerated.
    ///
    /// Borrowed on purpose: the frame path reads the vector without building one of its own.
    #[must_use]
    pub fn pipe_flows(&self) -> &[Option<f64>] {
        &self.pipe_flows
    }

    /// Signed flow leaving an exhaust port to the atmosphere on the last frame, normal
    /// litres per minute. `None` for a port that is not an exhaust with an atmosphere
    /// boundary, so a caller never reads a hose's air as the valve's exhaust.
    #[must_use]
    pub fn exhaust_flow(&self, component: usize, port_id: &str) -> Option<f64> {
        let index = self
            .exhausts
            .iter()
            .position(|entry| entry.part == component && entry.port == port_id)?;
        self.exhaust_flows.get(index).copied().flatten()
    }

    /// The exhaust ports that carry a reading, in build order.
    #[must_use]
    pub fn exhausts(&self) -> &[ExhaustPort] {
        &self.exhausts
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
        self.pipe_flow_sums.fill(0.0);
        self.part_flow_sums.fill(0.0);
        self.exhaust_sums.fill(0.0);
        for _ in 0..count {
            self.substep(substep);
            let flows = &self.pipe_flows;
            for (sum, reading) in self.pipe_flow_sums.iter_mut().zip(flows) {
                if let Some(value) = reading {
                    *sum += value;
                }
            }
            let part_flows = &self.part_flow;
            for (sum, reading) in self.part_flow_sums.iter_mut().zip(part_flows) {
                if let Some(value) = reading {
                    *sum += value;
                }
            }
            let exhaust_readings = &self.exhaust_flows;
            for (sum, reading) in self.exhaust_sums.iter_mut().zip(exhaust_readings) {
                if let Some(value) = reading {
                    *sum += value;
                }
            }
        }
        // The frame's reading is the net flow: a still line that chatters cancels itself,
        // and a hose that truly carries air keeps its sign and its magnitude.
        let divisor = f64::from(count);
        let sums = &self.pipe_flow_sums;
        for (reading, sum) in self.pipe_flows.iter_mut().zip(sums) {
            if let Some(value) = reading {
                *value = sum / divisor;
            }
        }
        let sums = &self.part_flow_sums;
        for (reading, sum) in self.part_flow.iter_mut().zip(sums) {
            if let Some(value) = reading {
                *value = sum / divisor;
            }
        }
        let sums = &self.exhaust_sums;
        for (reading, sum) in self.exhaust_flows.iter_mut().zip(sums) {
            if let Some(value) = reading {
                *value = sum / divisor;
            }
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
        let mut pipe_flows = std::mem::take(&mut self.pipe_flows);
        let mut exhaust_flows = std::mem::take(&mut self.exhaust_flows);
        for slot in &mut part_flow {
            *slot = slot.map(|_| 0.0);
        }
        for slot in &mut exhaust_flows {
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
            // Stored before the zero-flow `continue`: a hose that settles at exactly zero
            // still reports that zero, and its slot never keeps the reading of an earlier
            // substep. No reset pass is needed because every modelled hose is assigned here.
            if let Some(pipe) = edge.pipe {
                if let Some(reading) = pipe_flows.get_mut(pipe).and_then(Option::as_mut) {
                    *reading = flow / orifice::normal::AIR_DENSITY * 60_000.0;
                }
            }
            if let Some(part) = edge.part {
                if let Some(total) = part_flow.get_mut(part).and_then(Option::as_mut) {
                    *total += flow;
                }
            }
            add_exhaust_flow(edge, &self.exhaust_node, &mut exhaust_flows, flow);
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

        self.apply_pressure(&mut delta, &mut mass_out);

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
        self.pipe_flows = pipe_flows;
        self.exhaust_flows = exhaust_flows;

        self.step_pistons(dt);
    }

    /// Moves each pressure element's outlet toward its rule by transferring mass from the
    /// inlet that feeds it (ADR 0029, ADR 0039).
    ///
    /// The transfer is capped by the mass the inlet holds, so the projection never invents
    /// gas. The shuttle holds its outlet when both inlets fall (the seats close); the
    /// two-pressure valve vents it, through the lower inlet or to atmosphere.
    fn apply_pressure(&self, delta: &mut [f64], mass_out: &mut f64) {
        let gas = AIR_SPECIFIC_GAS_CONSTANT * self.temperature_kelvin;
        for element in &self.pressure {
            let a = self.node_pressure(element.inlet_a);
            let b = self.node_pressure(element.inlet_b);
            let out = self.node_pressure(element.outlet);
            // The shuttle feeds from the higher inlet; the two-pressure valve from the
            // lower one while both carry pressure, and vents when either falls to
            // atmosphere (ADR 0029).
            let feed = match element.kind {
                PressureKind::Shuttle => match shuttle_ball(a, b, self.atmosphere_pa) {
                    // Both signals push the ball: it centres and closes both seats.
                    BallSeat::Centre => None,
                    BallSeat::Right => Some(element.inlet_a),
                    BallSeat::Left => Some(element.inlet_b),
                },
                PressureKind::TwoPressure => {
                    if a > self.atmosphere_pa && b > self.atmosphere_pa {
                        if a <= b {
                            Some(element.inlet_a)
                        } else {
                            Some(element.inlet_b)
                        }
                    } else {
                        None
                    }
                }
            };
            let volume = self.nodes[element.outlet].volume;
            let Some(feed) = feed else {
                // The two-pressure valve vents its outlet to atmosphere; the centered
                // shuttle closes both seats and its outlet stays where it is.
                if matches!(element.kind, PressureKind::TwoPressure) && out > self.atmosphere_pa {
                    let dm = ((out - self.atmosphere_pa) * volume / gas)
                        .min(self.nodes[element.outlet].mass.max(0.0));
                    if dm > 0.0 {
                        delta[element.outlet] -= dm;
                        *mass_out += dm;
                    }
                }
                continue;
            };
            // Two volumes joined at the element: the mass that equalises them keeps the
            // projection stable instead of draining one tiny node into the other.
            let feed_volume = self.nodes[feed].volume;
            let dm = (self.node_pressure(feed) - out) * (feed_volume * volume)
                / ((feed_volume + volume) * gas);
            if dm > 0.0 {
                let dm = dm.min(self.nodes[feed].mass.max(0.0));
                if dm > 0.0 {
                    delta[feed] -= dm;
                    delta[element.outlet] += dm;
                }
            } else if dm < 0.0 {
                // Both kinds let the outlet fall through the feed: the outlet follows the
                // selected inlet down and vents through it when both inlets fall. A seat
                // that held the outlet would trap a single-acting cylinder against its
                // return spring.
                let dm = (-dm).min(self.nodes[element.outlet].mass.max(0.0));
                if dm > 0.0 {
                    delta[element.outlet] -= dm;
                    delta[feed] += dm;
                }
            }
        }
    }

    /// Seat of the shuttle's ball for the part, when the part is one.
    ///
    /// The drawing places the ball from this, so the screen and the model agree on which
    /// inlet is open without the symbol carrying a second copy of the rule.
    #[must_use]
    pub fn shuttle_seat(&self, part: usize) -> Option<BallSeat> {
        let element = self.pressure.iter().find(|element| element.part == part)?;
        let a = self.node_pressure(element.inlet_a);
        let b = self.node_pressure(element.inlet_b);
        Some(shuttle_ball(a, b, self.atmosphere_pa))
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
fn pressure_elements(
    parts: &[NetworkPart],
    port_node: &[HashMap<&str, usize>],
) -> Result<Vec<PressureElement>, SolverError> {
    let mut elements = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let PartBehaviour::Pressure { kind } = &part.behaviour else {
            continue;
        };
        let mut inlets = [0_usize; 2];
        let mut count = 0;
        let mut outlet = None;
        for port in &part.ports {
            let Some(&node) = port_node[index].get(port.id.as_str()) else {
                continue;
            };
            match port.role {
                PortRole::Supply if count < inlets.len() => {
                    inlets[count] = node;
                    count += 1;
                }
                PortRole::Working => outlet = Some(node),
                _ => {}
            }
        }
        let Some(outlet) = outlet else {
            return Err(SolverError::UnmappedPort {
                valve: part.id.clone(),
                port: String::from("2"),
            });
        };
        if count != inlets.len() {
            return Err(SolverError::UnmappedPort {
                valve: part.id.clone(),
                port: String::from("1"),
            });
        }
        elements.push(PressureElement {
            kind: *kind,
            part: index,
            inlet_a: inlets[0],
            inlet_b: inlets[1],
            outlet,
        });
    }
    Ok(elements)
}

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
                    pipe: None,
                    vent: None,
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
            pipe: None,
            vent: None,
        });
    }
}

/// Appends one edge per hose, remembering the document connection it came from.
///
/// The index and the ends are the caller's: the document enumerated the connections before
/// filtering them, so an entry that reaches here keeps its place in the flow vector.
fn append_pipes(
    edges: &mut Vec<Edge>,
    pipes: &[(usize, PortAddress, PortAddress)],
    port_node: &[HashMap<&str, usize>],
) {
    for (pipe, from, to) in pipes {
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
            pipe: Some(*pipe),
            vent: None,
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
    terminated: &HashSet<usize>,
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
            if !connected[node] && nodes[node].reservoir.is_none() && !terminated.contains(&node) {
                nodes[node].reservoir = Some(Reservoir {
                    pressure_pa: atmosphere_pa,
                    kind: ReservoirKind::Atmosphere,
                });
            }
        }
    }
}

/// The exhaust nodes a closure terminates, so the free-vent pass leaves them alone.
fn terminated_exhausts(
    parts: &[NetworkPart],
    terminations: &[(PortAddress, PortTermination)],
    port_node: &[HashMap<&str, usize>],
) -> HashSet<usize> {
    terminations
        .iter()
        .filter_map(|(address, _)| exhaust_node(parts, address, port_node))
        .collect()
}

/// The node of an addressed port, when the port exists and is an exhaust.
///
/// A closure on a supply, working, mechanical or electrical port is not a termination: the
/// graph already seals a working line with no pipe, and the document refuses a pipe on a
/// closed port. Filtering here keeps the solver from inventing physics for the no-op.
fn exhaust_node(
    parts: &[NetworkPart],
    address: &PortAddress,
    port_node: &[HashMap<&str, usize>],
) -> Option<usize> {
    let component = parts.get(address.part)?;
    let port = component
        .ports
        .iter()
        .find(|candidate| candidate.id == address.port)?;
    if port.role != PortRole::Exhaust {
        return None;
    }
    port_node
        .get(address.part)?
        .get(address.port.as_str())
        .copied()
}

/// A port closure at an exhaust adds exactly one atmosphere edge through its conductance.
///
/// The free-vent pass has already skipped the terminated node, so a silencer replaces the
/// open boundary instead of joining it. A `Closed` port adds nothing and keeps its volume:
/// the air is trapped and its pressure does not decay.
/// Collects every exhaust with an atmosphere boundary and marks its vent edge.
///
/// A free exhaust is the port node itself; a silencer is the edge that leaves it. A plugged
/// port is collected too, with a zero that never draws anything.
/// The exhaust readings a build starts with: the ports, the node map and their slots.
struct ExhaustReadings {
    ports: Vec<ExhaustPort>,
    node: HashMap<usize, usize>,
    flows: Vec<Option<f64>>,
    sums: Vec<f64>,
}

fn collect_exhausts(
    parts: &[NetworkPart],
    nodes: &[Node],
    port_node: &[HashMap<&str, usize>],
    terminated: &HashSet<usize>,
    edges: &mut [Edge],
    vent_edge_start: usize,
) -> ExhaustReadings {
    let mut exhausts = Vec::new();
    let mut exhaust_node = HashMap::new();
    for (index, part) in parts.iter().enumerate() {
        for port in &part.ports {
            if port.role != PortRole::Exhaust {
                continue;
            }
            let Some(&node) = port_node[index].get(port.id.as_str()) else {
                continue;
            };
            let free = nodes
                .get(node)
                .and_then(|node| node.reservoir.as_ref())
                .is_some_and(|reservoir| reservoir.kind == ReservoirKind::Atmosphere);
            if free || terminated.contains(&node) {
                exhaust_node.insert(node, exhausts.len());
                exhausts.push(ExhaustPort {
                    part: index,
                    port: port.id.clone(),
                });
            }
        }
    }
    for edge in &mut edges[vent_edge_start..] {
        if let Some(&index) = exhaust_node.get(&edge.a) {
            edge.vent = Some(index);
        }
    }
    let flows = vec![Some(0.0); exhausts.len()];
    let sums = vec![0.0; exhausts.len()];
    ExhaustReadings {
        ports: exhausts,
        node: exhaust_node,
        flows,
        sums,
    }
}

fn vent_edges(
    parts: &[NetworkPart],
    terminations: &[(PortAddress, PortTermination)],
    nodes: &mut Vec<Node>,
    port_node: &[HashMap<&str, usize>],
    edges: &mut Vec<Edge>,
    atmosphere_pa: f64,
    temperature_kelvin: f64,
) {
    for (address, termination) in terminations {
        let PortTermination::Vent { conductance } = termination else {
            continue;
        };
        let Some(node) = exhaust_node(parts, address, port_node) else {
            continue;
        };
        let atmosphere = nodes.len();
        nodes.push(Node {
            mass: atmosphere_pa * PORT_VOLUME / (AIR_SPECIFIC_GAS_CONSTANT * temperature_kelvin),
            volume: PORT_VOLUME,
            reservoir: Some(Reservoir {
                pressure_pa: atmosphere_pa,
                kind: ReservoirKind::Atmosphere,
            }),
        });
        edges.push(Edge {
            a: node,
            b: atmosphere,
            conductance: *conductance,
            valve: None,
            mask: u64::MAX,
            part: None,
            pipe: None,
            vent: None,
        });
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
            pipe: None,
            vent: None,
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
/// Adds one edge's flow to the exhaust reading it feeds, if any.
///
/// A silencer or a plug vents through its own edge; a free exhaust is its port node, fed by
/// the internal path. A hose never counts: its air goes into the pipe, not the atmosphere.
fn add_exhaust_flow(
    edge: &Edge,
    exhaust_node: &HashMap<usize, usize>,
    readings: &mut [Option<f64>],
    flow: f64,
) {
    let vent = edge.vent.or_else(|| {
        if edge.pipe.is_none() {
            exhaust_node.get(&edge.b).copied()
        } else {
            None
        }
    });
    if let Some(vent) = vent {
        if let Some(reading) = readings.get_mut(vent).and_then(Option::as_mut) {
            *reading += flow / orifice::normal::AIR_DENSITY * 60_000.0;
        }
    }
}

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

    /// Builds a test network from pipes addressed in order.
    ///
    /// The solver sizes its flow vector by the document connection list; these tests have no
    /// document, so the position in the slice is the connection index and its length is the
    /// document's connection count.
    pub(super) fn build(
        parts: &[NetworkPart],
        pipes: &[(PortAddress, PortAddress)],
        terminations: &[(PortAddress, PortTermination)],
    ) -> Result<Network, SolverError> {
        let indexed: Vec<(usize, PortAddress, PortAddress)> = pipes
            .iter()
            .enumerate()
            .map(|(index, (from, to))| (index, from.clone(), to.clone()))
            .collect();
        Network::build(parts, &indexed, terminations, pipes.len())
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
        build(&parts, &pipes, &[]).expect("the chain builds")
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
        // The built-in compressor is the 4 bar catalog default, so the chamber settles at
        // 4 bar gauge: absolute pressure is atmosphere plus 4e5 Pa.
        assert!(
            network.pressure_pa() > network.atmosphere_pa() + 3.9e5,
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
        // The same 4 bar supply reaches the cylinder port, so the gauge reads near 4 bar.
        assert!(
            cylinder > atmosphere + 3.9e5,
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
        let mut network = build(&parts, &pipes, &[]).expect("the junction builds");
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
        let mut network = build(&parts, &pipes, &[]).expect("the gauge builds");
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
        let mut network = build(&parts, &pipes, &[]).expect("the five-port chain builds");
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
        let mut network = build(&parts, &pipes, &[]).expect("the meter builds");
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
        let mut network = build(&parts, &pipes, &[]).expect("the meter builds");
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
        let network = build(&parts, &pipes, &[]).expect("builds");
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
        let mut throttled = build(&with, &pipes, &[]).expect("builds");
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
        let mut direct = build(&parts, &direct_pipes, &[]).expect("builds without the silencer");
        let mut silenced = build(&parts, &silenced_pipes, &[]).expect("builds with it");
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
        let mut network = build(&parts, &pipes, &[]).expect("the plug builds");
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
    fn a_plug_on_an_exhaust_holds_the_air_and_the_rod_stays() {
        // A closure is not a placed plug: the port has no hose and no second component.
        // The plug seals the exhaust node itself, so the air has nowhere to go.
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
        let terminations = [(PortAddress::new(1, "3"), PortTermination::Closed)];
        let mut network =
            build(&parts, &pipes, &terminations).expect("the plug on the port builds");
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
        let settled = network.position();
        let pressure = network
            .port_pressure(1, "3")
            .expect("the exhaust port is in the model");
        for _ in 0..40 {
            network.advance(0.016).expect("accepted");
        }
        assert!(
            (network.position() - settled).abs() < stroke * 0.01,
            "the capped cylinder moved from {:.1} mm to {:.1} mm",
            settled * 1000.0,
            network.position() * 1000.0
        );
        let later = network
            .port_pressure(1, "3")
            .expect("the exhaust port is in the model");
        assert!(
            (later - pressure).abs() / pressure < 0.01,
            "the trapped air decayed from {:.3} bar to {:.3} bar",
            (pressure - network.atmosphere_pa()) / 1e5,
            (later - network.atmosphere_pa()) / 1e5
        );
        assert!(
            network.mass_out().abs() < 1e-12,
            "nothing crossed the plug: {} kg left",
            network.mass_out()
        );
    }

    #[test]
    fn a_plug_on_a_working_port_adds_no_solver_edge() {
        // A working port with no pipe is already sealed. The plug's effect is the pipe
        // refusal, the drawing and the parts row; the graph must not grow an edge for it.
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
        let terminations = [(PortAddress::new(1, "2"), PortTermination::Closed)];
        let without = build(&parts, &pipes, &[]).expect("builds");
        let with = build(&parts, &pipes, &terminations).expect("still builds");
        assert_eq!(
            with.edges.len(),
            without.edges.len(),
            "a working plug is a no-op, not closure physics"
        );
        assert_eq!(with.nodes.len(), without.nodes.len());
    }

    #[test]
    fn a_silencer_on_an_exhaust_vents_through_its_conductance() {
        // The closure replaces the free atmosphere boundary with the silencer's own
        // restriction: one edge to one atmosphere reservoir, never two.
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
        let silencer = part(&catalog, "silencer.pneumatic");
        let conductance = match silencer.behaviour {
            PartBehaviour::Silencer { conductance } => conductance,
            _ => 0.0,
        };
        assert!(
            conductance > 0.0,
            "the built-in silencer has no conductance"
        );
        let terminations = [(
            PortAddress::new(1, "3"),
            PortTermination::Vent { conductance },
        )];
        let mut open = build(&parts, &pipes, &[]).expect("the open exhaust builds");
        let mut vented = build(&parts, &pipes, &terminations).expect("the silencer closure builds");

        let node = vented.port_nodes[1]["3"];
        assert!(
            vented.nodes[node].reservoir.is_none(),
            "a vented port keeps its own node and adds an edge, not a reservoir"
        );
        assert_eq!(
            vented.edges.len(),
            open.edges.len() + 1,
            "the silencer adds exactly one restriction"
        );

        for network in [&mut open, &mut vented] {
            network.press_valve();
            for _ in 0..80 {
                network.advance(0.016).expect("accepted");
            }
            network.release_valve();
            for _ in 0..40 {
                network.advance(0.016).expect("accepted");
            }
        }
        let stroke = open.cylinder().expect("a cylinder").stroke;
        assert!(
            open.position() < stroke * 0.95,
            "the open exhaust did not let the rod return: {:.1} mm",
            open.position() * 1000.0
        );
        assert!(
            vented.position() > open.position(),
            "silenced {:.1} mm should lag open {:.1} mm",
            vented.position() * 1000.0,
            open.position() * 1000.0
        );
    }

    #[test]
    fn an_exhaust_without_a_closure_still_vents_freely() {
        let network = chain();
        let node = network.port_nodes[1]["3"];
        let reservoir = network.nodes[node]
            .reservoir
            .expect("an open exhaust is a free atmosphere boundary");
        assert_eq!(reservoir.kind, ReservoirKind::Atmosphere);
    }

    #[test]
    fn an_old_placed_silencer_and_a_closure_elsewhere_vent_once() {
        // The two paths are the placed silencer and the per-port closure. A circuit with a
        // placed silencer on one exhaust and a closure on the other must give each port one
        // atmosphere connection, not two.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.5_2.spring_return"),
            part(&catalog, "cylinder.double_acting_32_200"),
            part(&catalog, "silencer.pneumatic"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(2, "1")),
            (PortAddress::new(1, "4"), PortAddress::new(2, "2")),
            (PortAddress::new(1, "3"), PortAddress::new(3, "1")),
        ];
        let silencer = part(&catalog, "silencer.pneumatic");
        let conductance = match silencer.behaviour {
            PartBehaviour::Silencer { conductance } => conductance,
            _ => 0.0,
        };
        let terminations = [(
            PortAddress::new(1, "5"),
            PortTermination::Vent { conductance },
        )];
        let network = build(&parts, &pipes, &terminations).expect("both exhausts build");
        for port in ["3", "5"] {
            let node = network.port_nodes[1][port];
            assert!(
                network.nodes[node].reservoir.is_none(),
                "exhaust {port} received a free reservoir behind its closure"
            );
        }
        // The placed silencer vents from its own port node, not from the host exhaust: the
        // hose carries the air from the valve to the silencer first. Count the atmosphere
        // boundaries of the whole network instead, one per exhaust path, so a duplicate
        // free vent anywhere in a path breaks the invariant.
        let atmosphere: Vec<usize> = network
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                matches!(
                    node.reservoir,
                    Some(Reservoir {
                        kind: ReservoirKind::Atmosphere,
                        ..
                    })
                )
            })
            .map(|(index, _)| index)
            .collect();
        assert_eq!(
            atmosphere.len(),
            2,
            "one atmosphere vent for each exhaust, not a second free one"
        );
        let vents = |node: usize| {
            network
                .edges
                .iter()
                .filter(|edge| edge.a == node && atmosphere.contains(&edge.b))
                .count()
        };
        assert_eq!(
            vents(network.port_nodes[3]["1"]),
            1,
            "the placed silencer vents its own port exactly once"
        );
        assert_eq!(
            vents(network.port_nodes[1]["5"]),
            1,
            "the closure vents its host exhaust exactly once"
        );
        assert_eq!(
            vents(network.port_nodes[1]["3"]),
            0,
            "the host exhaust must not add a free vent on top of the silencer path"
        );
        assert_eq!(
            network
                .edges
                .iter()
                .filter(|edge| atmosphere.contains(&edge.b))
                .count(),
            atmosphere.len(),
            "each atmosphere reservoir is fed by one edge"
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
            build(&parts, &[], &[]),
            Err(SolverError::TooManyValvePositions { .. })
        ));
    }

    #[test]
    fn a_pipe_to_a_missing_port_is_ignored_not_fatal() {
        let catalog = Catalog::builtin();
        let parts = [part(&catalog, "supply.compressor")];
        let pipes = [(PortAddress::new(0, "1"), PortAddress::new(7, "1"))];
        let network = build(&parts, &pipes, &[]).expect("still builds");
        assert!(network.cylinder().is_none());
    }

    #[test]
    fn a_pipe_reports_the_flow_through_it() {
        // The hose from the supply to the valve is the first document connection. Its
        // reading must be the ISO 6358 flow through the hose's own conductance, between the
        // pressures the model reports at its two ends, positive from `from` to `to`.
        let mut network = chain();
        network.press_valve();
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }
        let flows = network.pipe_flows();
        assert_eq!(flows.len(), 2, "one reading per hose of the document");
        let read = flows[0].expect("the supply hose is modelled");
        assert!(
            read > 0.0,
            "air runs from the supply to the valve, so the reading is positive, got {read}"
        );

        let supply = network
            .port_pressure(0, "1")
            .expect("the supply port is in the model");
        let valve = network
            .port_pressure(1, "1")
            .expect("the valve's supply port is in the model");
        let (upstream, downstream, sign) = if supply >= valve {
            (supply, valve, 1.0)
        } else {
            (valve, supply, -1.0)
        };
        let reference = orifice::mass_flow(
            PIPE_CONDUCTANCE,
            upstream,
            downstream,
            network.temperature_kelvin,
            DEFAULT_CRITICAL_RATIO,
        )
        .expect("the pressures are physical");
        let expected = sign * reference / orifice::normal::AIR_DENSITY * 60_000.0;
        assert!(
            expected.abs() > 0.01,
            "the test needs a real flow, got {expected} l/min"
        );
        assert!(
            (read - expected).abs() / expected.abs() < 0.01,
            "the hose reads {read:.2} l/min against the ISO 6358 flow {expected:.2} l/min \
             between {:.2} bar and {:.2} bar",
            (upstream - network.atmosphere_pa()) / 1e5,
            (downstream - network.atmosphere_pa()) / 1e5
        );

        // Releasing the valve exhausts the chamber back through the second hose, whose
        // `from` end is the valve: the reading has to turn negative.
        network.release_valve();
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }
        let returning = network.pipe_flows()[1].expect("the working hose is modelled");
        assert!(
            returning < 0.0,
            "air returns from the cylinder to the valve, so the reading is negative, got \
             {returning}"
        );
    }

    #[test]
    fn a_zero_flow_pipe_reports_zero_after_a_step() {
        // The gauge is a dead end: its hose carries nothing. A substep whose flow is exactly
        // zero still stores the zero, because the store sits before the zero-flow `continue`.
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
        let mut network = build(&parts, &pipes, &[]).expect("the gauge builds");
        // A reading from an earlier substep: the next substep has to overwrite it even
        // though its flow is zero, or the hose keeps a stale pulse.
        network.pipe_flows[2] = Some(0.75);
        network.advance(0.016).expect("a frame is accepted");
        assert_eq!(
            network.pipe_flows()[2],
            Some(0.0),
            "the dead-end hose carried nothing, so its stored reading is zero, not the old one"
        );
    }

    #[test]
    fn the_exhaust_port_reports_the_air_it_releases() {
        // A 3/2 valve with no silencer vents its exhaust port to the atmosphere through the
        // port node itself. While the valve is held, port 3 is blocked and reads nothing;
        // when it returns, the chamber empties through that port and the reading is positive.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "cylinder.single_acting_20_100"),
        ];
        let pipes = [
            (0, PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (1, PortAddress::new(1, "2"), PortAddress::new(2, "1")),
        ];
        let mut network = Network::build(&parts, &pipes, &[], pipes.len()).expect("it builds");
        assert!(
            network
                .exhausts()
                .iter()
                .any(|entry| entry.part == 1 && entry.port == "3")
        );

        assert!(network.press_valve(), "the actuator is in the model");
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }
        let filling = network.exhaust_flow(1, "3").expect("port 3 is an exhaust");

        network.release_valve();
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }
        let venting = network.exhaust_flow(1, "3").expect("port 3 is an exhaust");
        assert!(
            venting > 0.0,
            "the chamber empties through the exhaust, so the reading is positive: {venting:.3}"
        );
        assert!(
            venting > filling.abs(),
            "the blocked port reads less than the venting one: {filling:.3} against {venting:.3}"
        );
        assert_eq!(
            network.exhaust_flow(1, "1"),
            None,
            "a supply port is not an exhaust"
        );
    }

    #[test]
    fn a_dead_end_line_does_not_report_a_reversing_flow() {
        // A supply line that ends at a closed valve port chatters: the explicit substep
        // alternates between equal and opposite flows every 2 us. The reading is the
        // frame's net, so a still line reports ~0 instead of an arrow that flips direction.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
        ];
        let pipes = [(0, PortAddress::new(0, "1"), PortAddress::new(1, "1"))];
        let mut network = Network::build(&parts, &pipes, &[], 1).expect("it builds");
        assert!(network.press_valve(), "the actuator is in the model");
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }
        network.release_valve();
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
        }
        for _ in 0..5 {
            network.advance(0.016).expect("accepted");
            let reading = network.pipe_flows()[0].expect("the supply hose is modelled");
            assert!(
                reading.abs() < 0.01,
                "a sealed line is still: it read {reading:.4} l/min, so the sheet would \
                 draw air running backwards"
            );
        }
    }

    #[test]
    fn a_skipped_pipe_reports_no_flow() {
        // The second connection names a part the model does not have, so it contributes no
        // edge. Its slot is `None`, not a stale or borrowed zero from the hose beside it.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(1, "3"), PortAddress::new(9, "1")),
        ];
        let mut network = build(&parts, &pipes, &[]).expect("the modelled hose still builds");
        assert_eq!(network.pipe_flows().len(), 2);
        assert_eq!(network.pipe_flows()[0], Some(0.0));
        assert_eq!(network.pipe_flows()[1], None);

        network.press_valve();
        network.advance(0.016).expect("accepted");
        let flows = network.pipe_flows();
        assert_eq!(flows.len(), 2, "the skipped connection keeps its slot");
        assert_eq!(
            flows[1], None,
            "a connection the model cannot represent has no reading"
        );
        assert!(
            flows[0].is_some(),
            "the hose beside it still reports its flow"
        );
    }
}

#[cfg(test)]
mod electrical_tests {
    use super::tests::build;
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
        let mut pneumatic = build(&base, &pipes, &[]).expect("builds");
        let mut control = build(&with, &pipes, &[]).expect("builds with the control circuit");
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

#[cfg(test)]
mod pressure_tests {
    use super::tests::build;
    use super::*;
    use punchi_fluid_catalog::Catalog;

    fn part(catalog: &Catalog, id: &str) -> NetworkPart {
        let definition = catalog.get(id).expect("definition");
        NetworkPart::from_definition(definition).expect("a part")
    }

    fn supply_at(gauge_bar: f64) -> NetworkPart {
        NetworkPart {
            id: format!("supply.test.{gauge_bar}"),
            ports: vec![PortInfo {
                id: "1".to_owned(),
                role: PortRole::Supply,
            }],
            behaviour: PartBehaviour::Supply {
                pressure_pa: gauge_bar * 1.0e5 + punchi_fluid_core::Pressure::ATMOSPHERE.pascal(),
            },
        }
    }

    fn outlet_after(
        parts: &[NetworkPart],
        pipes: &[(PortAddress, PortAddress)],
        element: usize,
    ) -> f64 {
        let mut network = build(parts, pipes, &[]).expect("builds");
        for _ in 0..200 {
            network.advance(0.001).expect("accepted");
        }
        network
            .port_pressure(element, "2")
            .expect("the pressure element has an outlet")
    }

    fn atmosphere() -> f64 {
        punchi_fluid_core::Pressure::ATMOSPHERE.pascal()
    }

    #[test]
    fn a_shuttle_passes_the_higher_inlet() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            supply_at(2.0),
            part(&catalog, "valve.shuttle"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(2, "1a")),
            (PortAddress::new(1, "1"), PortAddress::new(2, "1b")),
        ];
        let outlet = outlet_after(&parts, &pipes, 2);
        assert!(
            (outlet - (4.0e5 + atmosphere())).abs() < 0.3e5,
            "the shuttle did not pass the 4 bar inlet: {outlet} Pa"
        );
    }

    #[test]
    fn a_shuttle_passes_its_only_live_inlet() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.shuttle"),
        ];
        let pipes = [(PortAddress::new(0, "1"), PortAddress::new(1, "1a"))];
        let outlet = outlet_after(&parts, &pipes, 1);
        assert!(
            (outlet - (4.0e5 + atmosphere())).abs() < 0.3e5,
            "the shuttle did not follow its live inlet: {outlet} Pa"
        );
    }

    #[test]
    fn a_two_pressure_valve_passes_the_lower_inlet() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            supply_at(2.0),
            part(&catalog, "valve.two_pressure"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(2, "1a")),
            (PortAddress::new(1, "1"), PortAddress::new(2, "1b")),
        ];
        let outlet = outlet_after(&parts, &pipes, 2);
        assert!(
            (outlet - (2.0e5 + atmosphere())).abs() < 0.3e5,
            "the two-pressure valve did not pass the 2 bar inlet: {outlet} Pa"
        );
    }

    #[test]
    fn a_two_pressure_valve_vents_when_one_inlet_is_at_atmosphere() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.two_pressure"),
        ];
        let pipes = [(PortAddress::new(0, "1"), PortAddress::new(1, "1a"))];
        let outlet = outlet_after(&parts, &pipes, 1);
        assert!(
            (outlet - atmosphere()).abs() < 0.1e5,
            "the two-pressure valve did not vent: {outlet} Pa"
        );
    }

    #[test]
    fn a_pressure_element_keeps_the_mass_balance() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            supply_at(2.0),
            part(&catalog, "valve.two_pressure"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(2, "1a")),
            (PortAddress::new(1, "1"), PortAddress::new(2, "1b")),
        ];
        let mut network = build(&parts, &pipes, &[]).expect("builds");
        let initial = network.gas_mass();
        for _ in 0..100 {
            network.advance(0.001).expect("accepted");
        }
        let expected = initial + network.mass_in() - network.mass_out();
        let measured = network.gas_mass();
        assert!(
            (measured - expected).abs() / expected < 1e-9,
            "final {measured:e} expected {expected:e}"
        );
    }

    #[test]
    fn a_pressure_element_is_deterministic() {
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            supply_at(2.0),
            part(&catalog, "valve.shuttle"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(2, "1a")),
            (PortAddress::new(1, "1"), PortAddress::new(2, "1b")),
        ];
        let first = outlet_after(&parts, &pipes, 2);
        let second = outlet_after(&parts, &pipes, 2);
        assert_eq!(
            first.to_bits(),
            second.to_bits(),
            "the same circuit gave two outlets"
        );
    }

    #[test]
    fn the_shuttle_feeds_and_vents_a_single_acting_cylinder() {
        // The circuit the owner drew: one supply, two 3/2 valves, their working ports to
        // the two inlets of the shuttle and the shuttle outlet to a single-acting cylinder.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "valve.shuttle"),
            part(&catalog, "cylinder.single_acting_20_100"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(0, "1"), PortAddress::new(2, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(3, "1a")),
            (PortAddress::new(2, "2"), PortAddress::new(3, "1b")),
            (PortAddress::new(3, "2"), PortAddress::new(4, "1")),
        ];
        let mut network = build(&parts, &pipes, &[]).expect("the shuttle chain builds");
        let stroke = network.cylinder().expect("a cylinder").stroke;

        // Pressed, 1a feeds the shuttle, 1b is at atmosphere and the rod goes out.
        assert!(network.press_valve_at(0));
        for _ in 0..100 {
            network.advance(0.016).expect("accepted");
        }
        let inlet_b = network.port_pressure(3, "1b").expect("an inlet");
        assert!((inlet_b - network.atmosphere_pa).abs() < 1_000.0);
        assert_eq!(network.shuttle_seat(3), Some(BallSeat::Right));
        assert!(
            network.position() > stroke * 0.5,
            "the cylinder did not extend: {:.1} of {:.1} mm",
            network.position() * 1000.0,
            stroke * 1000.0
        );

        // Released, both inlets fall and the outlet vents through the selected one, so the
        // spring returns the rod. A seat that held the air would leave it out.
        assert!(network.release_valve_at(0));
        for _ in 0..100 {
            network.advance(0.016).expect("accepted");
        }
        let outlet = network.port_pressure(3, "2").expect("an outlet");
        assert!(
            (outlet - network.atmosphere_pa).abs() < 2_000.0,
            "the shuttle held its outlet at {outlet:.0} Pa"
        );
        assert!(
            network.position() < stroke * 0.1,
            "the rod stayed out at {:.1} of {:.1} mm",
            network.position() * 1000.0,
            stroke * 1000.0
        );
    }

    #[test]
    fn the_shuttle_centers_and_blocks_the_outlet_when_both_inlets_are_alive() {
        // Two signals at once push the ball to the centre: both seats close and the
        // outlet 2 is isolated, so the cylinder does not move.
        let catalog = Catalog::builtin();
        let parts = [
            part(&catalog, "supply.compressor"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "valve.3_2.spring_return"),
            part(&catalog, "valve.shuttle"),
            part(&catalog, "cylinder.single_acting_20_100"),
        ];
        let pipes = [
            (PortAddress::new(0, "1"), PortAddress::new(1, "1")),
            (PortAddress::new(0, "1"), PortAddress::new(2, "1")),
            (PortAddress::new(1, "2"), PortAddress::new(3, "1a")),
            (PortAddress::new(2, "2"), PortAddress::new(3, "1b")),
            (PortAddress::new(3, "2"), PortAddress::new(4, "1")),
        ];
        let mut network = build(&parts, &pipes, &[]).expect("the shuttle chain builds");
        assert!(network.press_valve_at(0));
        assert!(network.press_valve_at(1));
        for _ in 0..100 {
            network.advance(0.016).expect("accepted");
        }
        let inlet_a = network.port_pressure(3, "1a").expect("an inlet");
        let inlet_b = network.port_pressure(3, "1b").expect("an inlet");
        assert!(
            inlet_a > 4.0e5 && inlet_b > 4.0e5,
            "both inlets are live: {inlet_a:.0} / {inlet_b:.0} Pa"
        );
        assert_eq!(network.shuttle_seat(3), Some(BallSeat::Centre));
        let outlet = network.port_pressure(3, "2").expect("an outlet");
        assert!(
            (outlet - network.atmosphere_pa).abs() < 2_000.0,
            "the centred ball closed port 2 at {outlet:.0} Pa"
        );
        assert!(
            network.position() < 1e-6,
            "the cylinder moved with both signals: {:.3} mm",
            network.position() * 1000.0
        );
    }
}
