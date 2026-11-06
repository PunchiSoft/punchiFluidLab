// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Lumped-parameter pneumatic solver.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

//!
//! Pneumatics is not computational fluid dynamics: a circuit is a network of
//! volumes joined by restrictive elements, described by the ideal gas law and an
//! orifice flow model. The solver is deliberately separated from the domain model
//! and from any UI, so it can be validated headlessly and replaced without
//! touching the rest of the application.

pub mod orifice;
pub mod pneumatic;

use punchi_fluid_core::{Circuit, Pressure};

/// Physical constants used by the pneumatic model.
pub mod constants {
    /// Universal gas constant, `J / (mol K)`.
    pub const R: f64 = 8.314_462_618;

    /// Molar mass of dry air, `kg / mol`.
    pub const AIR_MOLAR_MASS: f64 = 0.028_964_4;

    /// Specific heat ratio of dry air.
    pub const AIR_GAMMA: f64 = 1.4;
}

/// State of one lumped volume during a simulation step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeState {
    /// Absolute pressure inside the volume.
    pub pressure: Pressure,
    /// Temperature inside the volume.
    pub temperature_kelvin: f64,
}

/// Result of a simulation step for the whole circuit.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StepResult {
    /// New state of every volume, in the order the solver discovered them.
    pub volumes: Vec<VolumeState>,
}

/// Everything the solver can refuse to do.
#[derive(Debug, thiserror::Error)]
pub enum SolverError {
    /// The circuit has no supply, so nothing can be driven.
    #[error("the circuit has no supply component")]
    MissingSupply,

    /// The step size is zero, negative or not finite.
    #[error("invalid time step: {0}")]
    InvalidTimeStep(f64),

    /// The step is so large that one call would need too many internal substeps.
    #[error("time step too large for a bounded number of substeps: {0} s")]
    TimeStepTooLarge(f64),

    /// A definition the circuit needs is not in the catalog.
    #[error("the catalog has no definition `{0}`")]
    MissingDefinition(String),

    /// The definition used as a supply does not source pressure.
    #[error("`{0}` is not a supply")]
    NotASupply(String),

    /// The definition used as a valve has no positions.
    #[error("`{0}` is not a valve")]
    NotAValve(String),

    /// The definition used as a cylinder has no mechanics.
    #[error("`{0}` is not a cylinder")]
    NotACylinder(String),

    /// A valve connection cannot be mapped onto supply, working or exhaust.
    #[error("valve `{valve}` has a connection that cannot be simulated: {port}")]
    UnmappedPort {
        /// Valve definition being converted.
        valve: String,
        /// Port or port pair that could not be classified.
        port: String,
    },
}

/// Solver facade: owns the analysis state and advances the simulation.
#[derive(Debug, Default)]
pub struct Solver {
    circuit: Option<Circuit>,
}

impl Solver {
    /// Prepares a circuit for simulation.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::MissingSupply`] when no supply component exists.
    pub fn load(&mut self, circuit: Circuit) -> Result<(), SolverError> {
        let has_supply = circuit.components.iter().any(|component| {
            matches!(
                component.kind,
                punchi_fluid_core::ComponentKind::Supply { .. }
            )
        });
        if !has_supply {
            return Err(SolverError::MissingSupply);
        }
        self.circuit = Some(circuit);
        Ok(())
    }

    /// Advances the simulation by `dt` seconds.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::InvalidTimeStep`] for a non-positive or non-finite
    /// step, and [`SolverError::MissingSupply`] when no circuit is loaded.
    pub fn step(&mut self, dt: f64) -> Result<StepResult, SolverError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(SolverError::InvalidTimeStep(dt));
        }
        let circuit = self.circuit.as_ref().ok_or(SolverError::MissingSupply)?;

        // First version: the volumes settle at the supply pressure. The
        // integrate step is where the orifice model and the gas law land.
        let supply = circuit
            .components
            .iter()
            .find_map(|component| match component.kind {
                punchi_fluid_core::ComponentKind::Supply { pressure } => Some(pressure),
                _ => None,
            })
            .ok_or(SolverError::MissingSupply)?;

        Ok(StepResult {
            volumes: vec![VolumeState {
                pressure: supply,
                temperature_kelvin: punchi_fluid_core::Temperature::STANDARD.kelvin(),
            }],
        })
    }

    /// Releases the loaded circuit.
    pub fn unload(&mut self) {
        self.circuit = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use punchi_fluid_core::{Component, ComponentKind};

    fn circuit_with_supply() -> Circuit {
        let mut circuit = Circuit::default();
        circuit.add_component(Component {
            id: String::from("p"),
            name: String::from("Supply"),
            kind: ComponentKind::Supply {
                pressure: Pressure::from_bar(6.0).expect("valid"),
            },
        });
        circuit
    }

    #[test]
    fn rejects_a_circuit_without_supply() {
        let mut solver = Solver::default();
        assert!(matches!(
            solver.load(Circuit::default()),
            Err(SolverError::MissingSupply)
        ));
    }

    #[test]
    fn rejects_invalid_time_step() {
        let mut solver = Solver::default();
        solver.load(circuit_with_supply()).expect("has supply");
        assert!(matches!(
            solver.step(0.0),
            Err(SolverError::InvalidTimeStep(_))
        ));
        assert!(matches!(
            solver.step(-1.0),
            Err(SolverError::InvalidTimeStep(_))
        ));
    }

    #[test]
    fn first_step_reports_the_supply_pressure() {
        let mut solver = Solver::default();
        solver.load(circuit_with_supply()).expect("has supply");
        let result = solver.step(0.001).expect("valid step");
        assert_eq!(result.volumes.len(), 1);
        assert!((result.volumes[0].pressure.bar() - 6.0).abs() < 1e-9);
    }
}
