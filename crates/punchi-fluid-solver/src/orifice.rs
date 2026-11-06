// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Compressible flow through a restriction, following ISO 6358.
//!
//! ISO 6358 describes a pneumatic component with two numbers: the sonic
//! conductance `C` and the critical pressure ratio `b`. Below `b` the flow is
//! choked and stops growing with the pressure drop; above it the flow falls
//! along a quarter ellipse. Every valve and throttle in the circuit is one of
//! these, so the model is tested on its own before any network exists.

/// Normal conditions used by ISO 6358: `0 °C` and `1.01325 bar`.
pub mod normal {
    /// Reference temperature, `273.15 K`.
    pub const TEMPERATURE_KELVIN: f64 = 273.15;

    /// Dry air density at normal conditions, `kg / m³`.
    pub const AIR_DENSITY: f64 = 1.293;
}

/// Critical pressure ratio of a typical pneumatic valve.
pub const DEFAULT_CRITICAL_RATIO: f64 = 0.3;

/// Mass flow through a restriction, in `kg / s`.
///
/// `conductance` is the sonic conductance `C` in `m³ / (s · Pa)`, `upstream_pa`
/// and `downstream_pa` are absolute pressures, `temperature_kelvin` is the
/// upstream stagnation temperature and `critical_ratio` is `b`.
///
/// # Errors
///
/// Returns `None` when an input is outside its physical range, so the caller
/// reports a validation error instead of propagating a NaN into the pressure.
#[must_use]
pub fn mass_flow(
    conductance: f64,
    upstream_pa: f64,
    downstream_pa: f64,
    temperature_kelvin: f64,
    critical_ratio: f64,
) -> Option<f64> {
    if !conductance.is_finite()
        || conductance < 0.0
        || !upstream_pa.is_finite()
        || upstream_pa <= 0.0
        || !downstream_pa.is_finite()
        || downstream_pa < 0.0
        || !temperature_kelvin.is_finite()
        || temperature_kelvin <= 0.0
        || !critical_ratio.is_finite()
        || !(0.0..1.0).contains(&critical_ratio)
    {
        return None;
    }

    let ratio = (downstream_pa / upstream_pa).clamp(0.0, 1.0);
    let reference = conductance
        * upstream_pa
        * normal::AIR_DENSITY
        * (normal::TEMPERATURE_KELVIN / temperature_kelvin).sqrt();

    let flow = if ratio <= critical_ratio {
        reference
    } else {
        let normalised = (ratio - critical_ratio) / (1.0 - critical_ratio);
        reference * (1.0 - normalised * normalised).max(0.0).sqrt()
    };

    Some(flow)
}

#[cfg(test)]
mod tests {
    use super::*;

    const C: f64 = 1e-8;
    const T: f64 = 293.15;

    #[test]
    fn rejects_invalid_inputs() {
        assert!(mass_flow(-1.0, 700_000.0, 100_000.0, T, 0.3).is_none());
        assert!(mass_flow(C, 0.0, 100_000.0, T, 0.3).is_none());
        assert!(mass_flow(C, 700_000.0, 100_000.0, 0.0, 0.3).is_none());
        assert!(mass_flow(C, 700_000.0, 100_000.0, T, 1.5).is_none());
    }

    #[test]
    fn choked_flow_matches_the_hand_calculation() {
        // m = C * p1 * rho_n * sqrt(T_n / T1)
        let expected = C * 700_000.0 * normal::AIR_DENSITY * (273.15_f64 / T).sqrt();
        let flow = mass_flow(C, 700_000.0, 100_000.0, T, DEFAULT_CRITICAL_RATIO).expect("valid");
        assert!((flow - expected).abs() < 1e-15);
    }

    #[test]
    fn choked_flow_stops_growing() {
        let at_threshold = mass_flow(C, 700_000.0, 700_000.0 * 0.3, T, 0.3).expect("valid");
        let far_below = mass_flow(C, 700_000.0, 10_000.0, T, 0.3).expect("valid");
        assert!((at_threshold - far_below).abs() < 1e-15);
    }

    #[test]
    fn subcritical_flow_is_smaller_than_choked() {
        let choked = mass_flow(C, 700_000.0, 150_000.0, T, 0.3).expect("valid");
        let subcritical = mass_flow(C, 700_000.0, 600_000.0, T, 0.3).expect("valid");
        assert!(subcritical < choked);
    }

    #[test]
    fn equal_pressures_give_no_flow() {
        let flow = mass_flow(C, 700_000.0, 700_000.0, T, 0.3).expect("valid");
        assert!(flow.abs() < 1e-18);
    }
}
