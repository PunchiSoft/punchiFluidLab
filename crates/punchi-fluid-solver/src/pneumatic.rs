// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Supply, directional valve and single-acting cylinder.
//!
//! The circuit is a lumped model with one gas volume. Gas mass is the integrated
//! state and the pressure is always derived from it with the ideal gas law, so
//! pressure cannot go negative and the mass balance is exact by construction.
//! Piston motion is a second ordinary differential equation coupled to it.
//!
//! Nothing here names a particular valve: conductances, port roles and mechanical
//! data are read from the catalog definitions, so editing a definition file
//! changes the simulation without recompiling.

use punchi_fluid_catalog::{Actuation, Behaviour, Catalog, ComponentDefinition, PortRole};

use crate::SolverError;
use crate::orifice::{self, DEFAULT_CRITICAL_RATIO};

/// Specific gas constant of dry air, `J / (kg · K)`.
const AIR_SPECIFIC_GAS_CONSTANT: f64 = 287.058;

/// Largest internal integration step, chosen from the pneumatic time constant of
/// a small cylinder (tens of microseconds) so explicit Euler stays stable.
const MAX_INTERNAL_SUBSTEP: f64 = 2e-6;

/// Upper bound on the internal substeps of one [`PneumaticSystem::advance`] call,
/// to keep a single UI frame bounded.
const MAX_SUBSTEPS_PER_CALL: u32 = 500_000;

/// Identifier of the supply used by [`PneumaticSystem::from_catalog`].
pub const DEFAULT_SUPPLY: &str = "supply.workshop";

/// Identifier of the valve used by [`PneumaticSystem::from_catalog`].
pub const DEFAULT_VALVE: &str = "valve.3_2.spring_return";

/// Identifier of the cylinder used by [`PneumaticSystem::from_catalog`].
pub const DEFAULT_CYLINDER: &str = "cylinder.single_acting_20_100";

/// What one valve position does to the working line.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ValvePath {
    /// Conductance between the supply port and the working port, if open.
    pub pressurise: Option<f64>,
    /// Conductance between the working port and the exhaust port, if open.
    pub vent: Option<f64>,
}

/// Valve behaviour derived from a definition file.
#[derive(Debug, Clone, PartialEq)]
pub struct ValveModel {
    /// Identifier of the definition this model came from.
    pub definition_id: String,
    /// Name of every position, in file order.
    pub position_names: Vec<String>,
    /// How the user switches position.
    pub actuation: Actuation,
    /// Whether releasing returns to the default position.
    pub spring_return: bool,
    positions: Vec<ValvePath>,
    default_position: usize,
}

impl ValveModel {
    /// Reads the positions of a valve definition and maps its connections onto
    /// the supply / working / exhaust roles the simulation understands.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::NotAValve`] when the definition is not a valve and
    /// [`SolverError::UnmappedPort`] when a connection cannot be classified.
    pub fn from_definition(definition: &ComponentDefinition) -> Result<Self, SolverError> {
        let Behaviour::Valve {
            actuation,
            spring_return,
            default_position,
            positions,
        } = &definition.behaviour
        else {
            return Err(SolverError::NotAValve(definition.id.clone()));
        };

        let mut mapped = Vec::with_capacity(positions.len());
        for position in positions {
            let mut path = ValvePath::default();
            for connection in &position.connection {
                let from = definition
                    .port(&connection.from)
                    .map(|port| port.role)
                    .ok_or_else(|| SolverError::UnmappedPort {
                        valve: definition.id.clone(),
                        port: connection.from.clone(),
                    })?;
                let to = definition
                    .port(&connection.to)
                    .map(|port| port.role)
                    .ok_or_else(|| SolverError::UnmappedPort {
                        valve: definition.id.clone(),
                        port: connection.to.clone(),
                    })?;

                match roles(from, to) {
                    Some(Slot::Pressurise) => path.pressurise = Some(connection.conductance),
                    Some(Slot::Vent) => path.vent = Some(connection.conductance),
                    None => {
                        return Err(SolverError::UnmappedPort {
                            valve: definition.id.clone(),
                            port: format!("{}<->{}", connection.from, connection.to),
                        });
                    }
                }
            }
            mapped.push(path);
        }

        let default_position = (*default_position).min(mapped.len().saturating_sub(1));
        Ok(Self {
            definition_id: definition.id.clone(),
            position_names: positions
                .iter()
                .map(|position| position.display_name().to_owned())
                .collect(),
            actuation: *actuation,
            spring_return: *spring_return,
            positions: mapped,
            default_position,
        })
    }

    /// Number of positions.
    #[must_use]
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    /// Position selected before the simulation starts.
    #[must_use]
    pub fn default_position(&self) -> usize {
        self.default_position
    }

    /// What a position does, if it exists.
    #[must_use]
    pub fn path(&self, position: usize) -> Option<&ValvePath> {
        self.positions.get(position)
    }

    /// Largest conductance of the model, used to size the integration step.
    #[must_use]
    pub fn max_conductance(&self) -> f64 {
        self.positions
            .iter()
            .flat_map(|path| [path.pressurise, path.vent])
            .flatten()
            .fold(0.0, f64::max)
    }
}

enum Slot {
    Pressurise,
    Vent,
}

fn roles(left: PortRole, right: PortRole) -> Option<Slot> {
    match (left, right) {
        (PortRole::Supply, PortRole::Working) | (PortRole::Working, PortRole::Supply) => {
            Some(Slot::Pressurise)
        }
        (PortRole::Working, PortRole::Exhaust) | (PortRole::Exhaust, PortRole::Working) => {
            Some(Slot::Vent)
        }
        _ => None,
    }
}

/// Geometry and mechanics of a single-acting cylinder with a return spring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cylinder {
    /// Piston bore, `m`.
    pub bore: f64,
    /// Rod diameter, `m`; zero when the return chamber has the full bore.
    pub rod: f64,
    /// Usable stroke, `m`.
    pub stroke: f64,
    /// Length standing for the residual volume at `x = 0`, `m`.
    pub dead_length: f64,
    /// Moving mass, `kg`.
    pub moving_mass: f64,
    /// Return-spring stiffness, `N / m`.
    pub spring_rate: f64,
    /// Return-spring preload at `x = 0`, `N`.
    pub spring_preload: f64,
    /// Viscous damping, `N · s / m`.
    pub damping: f64,
    /// Coulomb friction, `N`.
    pub friction: f64,
}

impl Cylinder {
    /// Reads the mechanical data of a cylinder definition.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::NotACylinder`] when the definition describes
    /// something else.
    pub fn from_definition(definition: &ComponentDefinition) -> Result<Self, SolverError> {
        let Behaviour::Cylinder {
            bore_mm,
            rod_mm,
            stroke_mm,
            dead_length_mm,
            moving_mass_kg,
            spring_rate,
            spring_preload,
            damping,
            friction,
        } = &definition.behaviour
        else {
            return Err(SolverError::NotACylinder(definition.id.clone()));
        };

        Ok(Self {
            bore: bore_mm / 1000.0,
            rod: rod_mm / 1000.0,
            stroke: stroke_mm / 1000.0,
            dead_length: dead_length_mm / 1000.0,
            moving_mass: *moving_mass_kg,
            spring_rate: *spring_rate,
            spring_preload: *spring_preload,
            damping: *damping,
            friction: *friction,
        })
    }

    /// Piston area, `m²`.
    #[must_use]
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * (self.bore / 2.0) * (self.bore / 2.0)
    }

    /// Area of the chamber the rod crosses, `m²`: the bore less the rod.
    #[must_use]
    pub fn annulus_area(&self) -> f64 {
        std::f64::consts::PI
            * ((self.bore / 2.0) * (self.bore / 2.0) - (self.rod / 2.0) * (self.rod / 2.0))
    }

    /// Gas volume at piston position `x`, `m³`.
    #[must_use]
    pub fn volume_at(&self, x: f64) -> f64 {
        self.area() * (x.clamp(0.0, self.stroke) + self.dead_length)
    }

    /// Gauge pressure needed to hold the piston at `x`, `Pa`.
    #[must_use]
    pub fn balance_pressure(&self, x: f64) -> f64 {
        let force = self.spring_preload + self.spring_rate * x + self.friction;
        force / self.area()
    }
}

/// Snapshot of the simulation, safe to hand to the UI or a CSV writer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Telemetry {
    /// Simulated time, `s`.
    pub time: f64,
    /// Absolute cylinder pressure, `Pa`.
    pub pressure_pa: f64,
    /// Piston position, `m`, in `0..=stroke`.
    pub position: f64,
    /// Piston velocity, `m / s`.
    pub velocity: f64,
    /// Active valve position.
    pub valve_position: usize,
    /// Gas mass that entered the cylinder, `kg`.
    pub mass_in: f64,
    /// Gas mass that left the cylinder, `kg`.
    pub mass_out: f64,
}

/// Supply, valve and cylinder, integrated together.
#[derive(Debug, Clone)]
pub struct PneumaticSystem {
    supply_pa: f64,
    atmosphere_pa: f64,
    temperature_kelvin: f64,
    valve: ValveModel,
    cylinder: Cylinder,
    valve_position: usize,
    gas_mass: f64,
    position: f64,
    velocity: f64,
    time: f64,
    mass_in: f64,
    mass_out: f64,
    // Flow as it was on the last internal step. The drawing reads it to point an arrow
    // along each line: the direction of the air is part of the diagram, not a detail of
    // the integration.
    inflow: f64,
    outflow: f64,
}

impl PneumaticSystem {
    /// Builds the default circuit from the catalog.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::MissingDefinition`] when a required definition is
    /// absent and the errors of [`ValveModel::from_definition`] and
    /// [`Cylinder::from_definition`] when one has the wrong behaviour.
    pub fn from_catalog(catalog: &Catalog) -> Result<Self, SolverError> {
        let supply = catalog
            .get(DEFAULT_SUPPLY)
            .ok_or_else(|| SolverError::MissingDefinition(String::from(DEFAULT_SUPPLY)))?;
        let valve = catalog
            .get(DEFAULT_VALVE)
            .ok_or_else(|| SolverError::MissingDefinition(String::from(DEFAULT_VALVE)))?;
        let cylinder = catalog
            .get(DEFAULT_CYLINDER)
            .ok_or_else(|| SolverError::MissingDefinition(String::from(DEFAULT_CYLINDER)))?;

        Self::new(supply, valve, cylinder)
    }

    /// Builds a system from three definitions already in hand.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::NotASupply`] when the first definition is not a
    /// pressure source, plus the errors of [`ValveModel::from_definition`] and
    /// [`Cylinder::from_definition`].
    pub fn new(
        supply: &ComponentDefinition,
        valve: &ComponentDefinition,
        cylinder: &ComponentDefinition,
    ) -> Result<Self, SolverError> {
        let Behaviour::Supply { pressure_bar } = &supply.behaviour else {
            return Err(SolverError::NotASupply(supply.id.clone()));
        };
        let valve = ValveModel::from_definition(valve)?;
        let cylinder = Cylinder::from_definition(cylinder)?;

        let atmosphere_pa = punchi_fluid_core::Pressure::ATMOSPHERE.pascal();
        let temperature_kelvin = punchi_fluid_core::Temperature::STANDARD.kelvin();
        let gas_mass = atmosphere_pa * cylinder.volume_at(0.0)
            / (AIR_SPECIFIC_GAS_CONSTANT * temperature_kelvin);

        Ok(Self {
            // The datasheet pressure of a supply is gauge; the flowing gas sees absolute.
            supply_pa: pressure_bar.mul_add(100_000.0, atmosphere_pa),
            atmosphere_pa,
            temperature_kelvin,
            valve_position: valve.default_position(),
            valve,
            cylinder,
            gas_mass,
            position: 0.0,
            velocity: 0.0,
            time: 0.0,
            mass_in: 0.0,
            mass_out: 0.0,
            inflow: 0.0,
            outflow: 0.0,
        })
    }

    /// Supply pressure, absolute, `Pa`.
    #[must_use]
    pub fn supply_pa(&self) -> f64 {
        self.supply_pa
    }

    /// Atmospheric pressure used as the reference, `Pa`.
    #[must_use]
    pub fn atmosphere_pa(&self) -> f64 {
        self.atmosphere_pa
    }

    /// Cylinder geometry and mechanics.
    #[must_use]
    pub fn cylinder(&self) -> Cylinder {
        self.cylinder
    }

    /// Valve model read from its definition.
    #[must_use]
    pub fn valve(&self) -> &ValveModel {
        &self.valve
    }

    /// Active valve position.
    #[must_use]
    pub fn valve_position(&self) -> usize {
        self.valve_position
    }

    /// Moves the valve to `position`, clamped to the ones that exist.
    ///
    /// Returns `true` when the position changed, so callers can decide whether to
    /// record the change in a replay trace.
    pub fn set_valve_position(&mut self, position: usize) -> bool {
        let position = position.min(self.valve.position_count().saturating_sub(1));
        if position == self.valve_position {
            return false;
        }
        self.valve_position = position;
        true
    }

    /// Switches to the next position, wrapping around.
    ///
    /// Returns `true` when the position changed.
    pub fn toggle_valve(&mut self) -> bool {
        let count = self.valve.position_count().max(1);
        self.set_valve_position((self.valve_position + 1) % count)
    }

    /// Moves the valve to the position the actuator selects, away from the rest one.
    ///
    /// A spring-return valve is held actuated while the actuator is applied; this is the
    /// press half of that gesture, and [`Self::release_valve`] is the other.
    ///
    /// Returns `true` when the position changed.
    pub fn press_valve(&mut self) -> bool {
        let target = usize::from(self.valve.default_position() == 0);
        self.set_valve_position(target)
    }

    /// Returns the valve to its default position, as a spring would.
    ///
    /// Returns `true` when the position changed.
    pub fn release_valve(&mut self) -> bool {
        if self.valve.spring_return {
            self.set_valve_position(self.valve.default_position())
        } else {
            false
        }
    }

    /// Absolute cylinder pressure, `Pa`.
    #[must_use]
    pub fn pressure_pa(&self) -> f64 {
        self.gas_mass * AIR_SPECIFIC_GAS_CONSTANT * self.temperature_kelvin / self.volume()
    }

    /// Mass flow along the working line, `kg / s`, positive towards the cylinder.
    ///
    /// Filling the cylinder gives a positive number and venting a negative one, because
    /// the air leaves the cylinder and crosses the valve the other way. The drawing uses
    /// the sign to point its arrow the way the air is going.
    #[must_use]
    pub fn working_flow(&self) -> f64 {
        self.inflow - self.outflow
    }

    /// Mass flow along the supply line, `kg / s`, positive into the valve.
    #[must_use]
    pub fn supply_flow(&self) -> f64 {
        self.inflow
    }

    /// Current gas volume, `m³`.
    #[must_use]
    pub fn volume(&self) -> f64 {
        self.cylinder.volume_at(self.position)
    }

    /// Piston position, `m`.
    #[must_use]
    pub fn position(&self) -> f64 {
        self.position
    }

    /// Piston position in `0..=1`.
    #[must_use]
    pub fn extension(&self) -> f64 {
        self.position / self.cylinder.stroke
    }

    /// Piston velocity, `m / s`.
    #[must_use]
    pub fn velocity(&self) -> f64 {
        self.velocity
    }

    /// Simulated time, `s`.
    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Builds a telemetry snapshot.
    #[must_use]
    pub fn telemetry(&self) -> Telemetry {
        Telemetry {
            time: self.time,
            pressure_pa: self.pressure_pa(),
            position: self.position,
            velocity: self.velocity,
            valve_position: self.valve_position,
            mass_in: self.mass_in,
            mass_out: self.mass_out,
        }
    }

    /// Advances the simulation by `dt` seconds using stable internal substeps.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::InvalidTimeStep`] for a non-positive or non-finite
    /// step and [`SolverError::TimeStepTooLarge`] when one call would need more
    /// than [`MAX_SUBSTEPS_PER_CALL`] internal steps.
    pub fn advance(&mut self, dt: f64) -> Result<Telemetry, SolverError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(SolverError::InvalidTimeStep(dt));
        }

        let substeps = (dt / MAX_INTERNAL_SUBSTEP).ceil();
        if substeps > f64::from(MAX_SUBSTEPS_PER_CALL) {
            return Err(SolverError::TimeStepTooLarge(dt));
        }

        // Bounded by MAX_SUBSTEPS_PER_CALL above, so the cast cannot truncate.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let count = substeps.max(1.0) as u32;
        let substep = dt / f64::from(count);
        for _ in 0..count {
            self.substep(substep);
        }
        self.time += dt;
        Ok(self.telemetry())
    }

    fn substep(&mut self, dt: f64) {
        let pressure = self.pressure_pa();
        let path = self
            .valve
            .path(self.valve_position)
            .copied()
            .unwrap_or_default();
        let critical_ratio = DEFAULT_CRITICAL_RATIO;

        let inflow = path
            .pressurise
            .and_then(|conductance| {
                orifice::mass_flow(
                    conductance,
                    self.supply_pa,
                    pressure,
                    self.temperature_kelvin,
                    critical_ratio,
                )
            })
            .unwrap_or(0.0);
        let outflow = path
            .vent
            .and_then(|conductance| {
                orifice::mass_flow(
                    conductance,
                    pressure,
                    self.atmosphere_pa,
                    self.temperature_kelvin,
                    critical_ratio,
                )
            })
            .unwrap_or(0.0);

        self.inflow = inflow;
        self.outflow = outflow;
        self.gas_mass = (self.gas_mass + (inflow - outflow) * dt).max(0.0);
        self.mass_in += inflow * dt;
        self.mass_out += outflow * dt;

        let area = self.cylinder.area();
        let pressure_force = (pressure - self.atmosphere_pa) * area;
        let spring_force = self.cylinder.spring_preload + self.cylinder.spring_rate * self.position;
        let friction_force = self.friction_force(pressure_force - spring_force);
        let damping_force = self.cylinder.damping * self.velocity;
        let acceleration = (pressure_force - spring_force - friction_force - damping_force)
            / self.cylinder.moving_mass;

        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;

        if self.position <= 0.0 {
            self.position = 0.0;
            self.velocity = 0.0;
        } else if self.position >= self.cylinder.stroke {
            self.position = self.cylinder.stroke;
            self.velocity = 0.0;
        }
    }

    /// Coulomb friction with stiction: it opposes the current motion, and when the
    /// piston is at rest it only resists the net force up to its own limit.
    fn friction_force(&self, net_force: f64) -> f64 {
        const REST_SPEED: f64 = 1e-6;
        if self.velocity.abs() > REST_SPEED {
            return self.cylinder.friction * self.velocity.signum();
        }
        net_force.clamp(-self.cylinder.friction, self.cylinder.friction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Catalog {
        Catalog::builtin()
    }

    fn demo() -> PneumaticSystem {
        PneumaticSystem::from_catalog(&catalog()).expect("built-in catalog has the demo circuit")
    }

    fn run_for(system: &mut PneumaticSystem, seconds: f64, dt: f64) -> Vec<Telemetry> {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let steps = (seconds / dt).round() as u32;
        let mut out = Vec::new();
        for _ in 0..steps {
            out.push(system.advance(dt).expect("small dt is accepted"));
        }
        out
    }

    fn stroke_time(system: &mut PneumaticSystem) -> Option<f64> {
        for _ in 0..5_000 {
            system.advance(1e-3).expect("accepted");
            if system.position() >= system.cylinder().stroke - 1e-9 {
                return Some(system.time());
            }
        }
        None
    }

    #[test]
    fn valve_positions_come_from_the_definition() {
        let system = demo();
        let valve = system.valve();
        assert_eq!(valve.actuation, Actuation::Momentary);
        assert_eq!(valve.position_count(), 2);
        assert_eq!(valve.default_position(), 0);
        let rest = valve.path(0).expect("rest position");
        assert!(rest.vent.is_some(), "rest must vent the working line");
        assert!(rest.pressurise.is_none());
        let actuated = valve.path(1).expect("actuated position");
        assert!(actuated.pressurise.is_some());
        assert!(actuated.vent.is_none());
    }

    #[test]
    fn a_cylinder_definition_is_required() {
        let catalog = catalog();
        let supply = catalog.get(DEFAULT_SUPPLY).expect("supply");
        let cylinder = catalog.get(DEFAULT_CYLINDER).expect("cylinder");
        assert!(matches!(
            PneumaticSystem::new(supply, cylinder, cylinder),
            Err(SolverError::NotAValve(_))
        ));
        assert!(matches!(
            PneumaticSystem::new(cylinder, supply, cylinder),
            Err(SolverError::NotASupply(_))
        ));
    }

    #[test]
    fn missing_definition_is_reported() {
        let catalog = Catalog::new();
        assert!(matches!(
            PneumaticSystem::from_catalog(&catalog),
            Err(SolverError::MissingDefinition(_))
        ));
    }

    #[test]
    fn starts_at_rest_at_atmospheric_pressure() {
        let system = demo();
        assert!(system.position().abs() < f64::EPSILON);
        assert!((system.pressure_pa() - system.atmosphere_pa()).abs() < 1.0);
        assert_eq!(system.valve_position(), 0);
    }

    #[test]
    fn rejects_non_positive_and_oversized_steps() {
        let mut system = demo();
        assert!(matches!(
            system.advance(0.0),
            Err(SolverError::InvalidTimeStep(_))
        ));
        assert!(matches!(
            system.advance(-1e-3),
            Err(SolverError::InvalidTimeStep(_))
        ));
        assert!(matches!(
            system.advance(10.0),
            Err(SolverError::TimeStepTooLarge(_))
        ));
    }

    #[test]
    fn inflow_matches_the_choked_hand_calculation() {
        let mut system = demo();
        system.set_valve_position(1);
        let conductance = system
            .valve()
            .path(1)
            .and_then(|path| path.pressurise)
            .expect("actuated position pressurises");
        let before = system.telemetry().mass_in;
        let dt = 1e-5;
        system.advance(dt).expect("accepted");
        let expected = orifice::mass_flow(
            conductance,
            system.supply_pa(),
            system.atmosphere_pa(),
            punchi_fluid_core::Temperature::STANDARD.kelvin(),
            DEFAULT_CRITICAL_RATIO,
        )
        .expect("valid")
            * dt;
        let measured = system.telemetry().mass_in - before;
        assert!(
            (measured - expected).abs() / expected < 0.01,
            "measured {measured:e} expected {expected:e}"
        );
    }

    #[test]
    fn a_piston_that_reaches_the_stop_stays_there_with_a_held_valve() {
        // A hard stop absorbs the motion. The clamp zeroes the velocity on the frame the
        // piston arrives, so nothing may kick it back: an impact that bounces is not a stop,
        // and the front end sees a second, spurious arrival a few frames later.
        let mut system = demo();
        assert!(system.set_valve_position(1), "the valve has two positions");
        let limited = system.cylinder().stroke;
        let frames = run_for(&mut system, 1.5, 1.0 / 60.0);
        assert!(
            frames.iter().any(|frame| frame.position >= limited - 1e-6),
            "the stroke was never reached"
        );
        let settled = &frames[frames.len() - 1];
        assert!(
            (settled.position - limited).abs() < 1e-6,
            "the piston left the stop: {} mm of {} mm",
            settled.position * 1000.0,
            limited * 1000.0
        );
        assert!(
            settled.velocity.abs() < 1e-3,
            "the piston is still moving against the stop: {} mm/s",
            settled.velocity * 1000.0
        );
        // And it never went back down after arriving.
        if let Some(arrival) = frames
            .iter()
            .position(|frame| frame.position >= limited - 1e-6)
        {
            let lowest = frames[arrival..]
                .iter()
                .map(|frame| frame.position)
                .fold(f64::INFINITY, f64::min);
            assert!(
                limited - lowest < 1e-6,
                "the piston bounced back to {} mm after arriving at {} mm",
                lowest * 1000.0,
                limited * 1000.0
            );
        }
    }

    #[test]
    fn the_cylinder_settles_at_the_stop_after_the_rebound() {
        // Measured 2026-11-06: with a 16 ms frame the piston reaches the stop before the
        // chamber has charged, the return spring brings it back a few millimetres, and the
        // incoming air pushes it out again. That is the model, not a numerical crash — the
        // steady state holds it — but it is a **visible rebound** the reference does not
        // show, and the front end has to know about it so one hit is one sound.
        //
        // This test pins the behaviour so the number is not forgotten: the piston must come
        // to rest at the stop within a quarter of a second and stay.
        let mut system = demo();
        assert!(system.set_valve_position(1));
        let stroke = system.cylinder().stroke;
        let frames = run_for(&mut system, 0.25, 0.016);
        let settled = &frames[frames.len() - 1];
        assert!(
            (settled.position - stroke).abs() < 1e-9,
            "not at the stop after 0.25 s: {:.2} mm of {:.2} mm",
            settled.position * 1000.0,
            stroke * 1000.0
        );
        assert!(
            settled.velocity.abs() < 1e-6,
            "still moving at the stop: {:.1} mm/s",
            settled.velocity * 1000.0
        );
        // The rebound is what the front end has to collapse into one impact, so it is
        // asserted rather than merely observed: if a later change removes it, this test
        // fails and the impact detector can be simplified on purpose.
        let rebound = frames
            .iter()
            .map(|frame| stroke - frame.position)
            .fold(0.0_f64, f64::max);
        assert!(
            rebound > 1.0e-3,
            "the rebound disappeared; the impact detector may be simplified"
        );
    }

    #[test]
    fn stroke_time_stays_between_the_two_flow_bounds() {
        // Both bounds assume the valve delivers its choked flow the whole time,
        // which is the fastest the circuit can move gas. The piston does not wait
        // for the supply pressure, so the real time sits between the time needed
        // to store the gas at the force-balance pressure and at supply pressure.
        let mut system = demo();
        system.set_valve_position(1);
        let cylinder = system.cylinder();
        let conductance = system
            .valve()
            .path(1)
            .and_then(|path| path.pressurise)
            .expect("actuated position pressurises");
        let choked = orifice::mass_flow(
            conductance,
            system.supply_pa(),
            system.atmosphere_pa(),
            punchi_fluid_core::Temperature::STANDARD.kelvin(),
            DEFAULT_CRITICAL_RATIO,
        )
        .expect("valid");
        let gas = |pressure: f64| {
            pressure * cylinder.volume_at(cylinder.stroke)
                / (AIR_SPECIFIC_GAS_CONSTANT * punchi_fluid_core::Temperature::STANDARD.kelvin())
        };
        let stored_initial = system.pressure_pa() * system.volume()
            / (AIR_SPECIFIC_GAS_CONSTANT * punchi_fluid_core::Temperature::STANDARD.kelvin());
        let lower = (gas(system.atmosphere_pa() + cylinder.balance_pressure(cylinder.stroke))
            - stored_initial)
            / choked;
        let upper = (gas(system.supply_pa()) - stored_initial) / choked;

        let measured = stroke_time(&mut system).expect("the piston must reach the end stop");
        assert!(
            measured >= lower * 0.95 && measured <= upper,
            "stroke took {measured} s, bounds are {lower} s .. {upper} s"
        );
        assert!(system.pressure_pa() <= system.supply_pa() * 1.001);
    }

    #[test]
    fn mass_balance_is_exact() {
        let mut system = demo();
        system.set_valve_position(1);
        let initial = system.gas_mass;
        run_for(&mut system, 0.3, 1e-3);
        let telemetry = system.telemetry();
        let final_mass = system.gas_mass;
        let expected = initial + telemetry.mass_in - telemetry.mass_out;
        assert!(
            (final_mass - expected).abs() / expected < 1e-9,
            "final {final_mass:e} expected {expected:e}"
        );
    }

    #[test]
    fn releasing_the_valve_lets_the_spring_retract_the_piston() {
        let mut system = demo();
        system.set_valve_position(1);
        run_for(&mut system, 1.0, 1e-3);
        let extended = system.position();
        assert!(extended > 0.09, "piston only reached {extended} m");

        assert!(system.release_valve());
        assert_eq!(system.valve_position(), 0);
        run_for(&mut system, 1.0, 1e-3);
        assert!(
            system.position() < extended - 0.01,
            "piston should retract, stayed at {}",
            system.position()
        );
    }

    #[test]
    fn state_stays_finite_and_physical_over_a_long_run() {
        let mut system = demo();
        system.set_valve_position(1);
        let mut peak = 0.0_f64;
        for step in 0..100_000 {
            if step == 60_000 {
                system.set_valve_position(0);
            }
            let telemetry = system.advance(2e-6).expect("accepted");
            assert!(telemetry.pressure_pa.is_finite());
            assert!(telemetry.position.is_finite());
            assert!(telemetry.velocity.is_finite());
            assert!(telemetry.pressure_pa > 0.0);
            assert!((0.0..=system.cylinder().stroke).contains(&telemetry.position));
            assert!(telemetry.mass_in >= 0.0 && telemetry.mass_out >= 0.0);
            peak = peak.max(telemetry.pressure_pa);
        }
        assert!(
            peak <= system.supply_pa() * 1.001,
            "pressure overshot the supply: {peak}"
        );
    }

    #[test]
    fn a_spring_valve_only_holds_while_pressed() {
        let catalog = Catalog::builtin();
        let mut system =
            PneumaticSystem::from_catalog(&catalog).expect("the built-in circuit starts");
        let rest = system.valve_position();
        assert_eq!(rest, system.valve().default_position());

        assert!(system.press_valve(), "the press must move the spool");
        assert_ne!(system.valve_position(), rest);
        assert!(!system.press_valve(), "holding it changes nothing more");

        assert!(system.release_valve(), "the spring must return the spool");
        assert_eq!(system.valve_position(), rest);
        assert!(
            !system.release_valve(),
            "the rest position is already the rest one"
        );
    }

    #[test]
    fn the_line_flow_changes_sign_between_filling_and_venting() {
        let catalog = Catalog::builtin();
        let mut system =
            PneumaticSystem::from_catalog(&catalog).expect("the built-in circuit starts");

        // At rest the supply port is shut, so nothing enters and nothing leaves an
        // already empty cylinder: the drawing must not point an arrow at all.
        system.advance(0.01).expect("accepted");
        assert!(
            system.supply_flow().abs() < 1.0e-18,
            "a shut port carries no air"
        );
        assert!(
            system.working_flow().abs() < 1.0e-18,
            "the cylinder is already empty"
        );

        assert!(system.press_valve());
        // Long enough for the chamber to sit well above atmosphere: venting an empty
        // cylinder moves no air, and there would be no direction to draw.
        system.advance(0.3).expect("accepted");
        assert!(
            system.supply_flow() > 0.0,
            "actuated, the supply line feeds the valve"
        );
        assert!(
            system.working_flow() > 0.0,
            "actuated, the working line feeds the cylinder"
        );

        assert!(system.release_valve());
        system.advance(0.01).expect("accepted");
        assert!(
            system.supply_flow().abs() < 1.0e-18,
            "the supply port is shut again"
        );
        assert!(
            system.working_flow() < 0.0,
            "released, the air leaves the cylinder through the working line"
        );
    }

    #[test]
    fn a_spring_valve_actuated_by_hand_does_not_stay_actuated() {
        let catalog = Catalog::builtin();
        let mut system =
            PneumaticSystem::from_catalog(&catalog).expect("the built-in circuit starts");
        let rest_pressure = {
            system.advance(1e-3).expect("the step is valid");
            system.pressure_pa()
        };

        system.press_valve();
        for _ in 0..400 {
            system.advance(1e-3).expect("the step is valid");
        }
        let held_pressure = system.pressure_pa();

        system.release_valve();
        for _ in 0..2_000 {
            system.advance(1e-3).expect("the step is valid");
        }
        assert!(
            (system.pressure_pa() - rest_pressure).abs() < 0.05 * held_pressure,
            "the cylinder must vent back towards its rest pressure once the spring returns the spool"
        );
    }
}
