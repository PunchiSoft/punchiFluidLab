// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Physical quantities in SI units, with the conversions the simulator needs.

use crate::CoreError;

/// Absolute or gauge pressure, stored in pascal.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pressure(f64);

impl Pressure {
    /// Standard atmospheric pressure, `101 325 Pa`.
    pub const ATMOSPHERE: Self = Self(101_325.0);

    /// Builds a pressure from pascal, rejecting negative absolute values.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::OutOfRange`] when `pascal` is negative or not finite.
    pub fn from_pascal(pascal: f64) -> Result<Self, CoreError> {
        if !pascal.is_finite() || pascal < 0.0 {
            return Err(CoreError::OutOfRange {
                quantity: "pressure",
                value: pascal,
            });
        }
        Ok(Self(pascal))
    }

    /// Builds a pressure from bar, the unit used on pneumatic datasheets.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::OutOfRange`] when the value is negative or not finite.
    pub fn from_bar(bar: f64) -> Result<Self, CoreError> {
        Self::from_pascal(bar * 100_000.0)
    }

    /// Value in pascal.
    #[must_use]
    pub fn pascal(self) -> f64 {
        self.0
    }

    /// Value in bar.
    #[must_use]
    pub fn bar(self) -> f64 {
        self.0 / 100_000.0
    }
}

/// Thermodynamic temperature, stored in kelvin.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature(f64);

impl Temperature {
    /// Standard reference temperature, `293.15 K`.
    pub const STANDARD: Self = Self(293.15);

    /// Builds a temperature from kelvin, rejecting values at or below zero.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::OutOfRange`] for a non-positive or non-finite value.
    pub fn from_kelvin(kelvin: f64) -> Result<Self, CoreError> {
        if !kelvin.is_finite() || kelvin <= 0.0 {
            return Err(CoreError::OutOfRange {
                quantity: "temperature",
                value: kelvin,
            });
        }
        Ok(Self(kelvin))
    }

    /// Value in kelvin.
    #[must_use]
    pub fn kelvin(self) -> f64 {
        self.0
    }
}

/// Volume, stored in cubic metre.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Volume(f64);

impl Volume {
    /// Empty volume, used when a stored value cannot be trusted.
    pub const ZERO: Self = Self(0.0);

    /// Builds a volume from cubic metre, rejecting negative values.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::OutOfRange`] for a negative or non-finite value.
    pub fn from_cubic_metre(cubic_metre: f64) -> Result<Self, CoreError> {
        if !cubic_metre.is_finite() || cubic_metre < 0.0 {
            return Err(CoreError::OutOfRange {
                quantity: "volume",
                value: cubic_metre,
            });
        }
        Ok(Self(cubic_metre))
    }

    /// Builds a volume from litre, which is how cylinder bores are specified.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::OutOfRange`] for a negative or non-finite value.
    pub fn from_litre(litre: f64) -> Result<Self, CoreError> {
        Self::from_cubic_metre(litre / 1000.0)
    }

    /// Value in cubic metre.
    #[must_use]
    pub fn cubic_metre(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_round_trips_through_pascal() {
        let pressure = Pressure::from_bar(6.0).expect("6 bar is valid");
        assert!((pressure.pascal() - 600_000.0).abs() < f64::EPSILON);
        assert!((pressure.bar() - 6.0).abs() < 1e-12);
    }

    #[test]
    fn negative_pressure_is_rejected() {
        assert!(Pressure::from_pascal(-1.0).is_err());
    }

    #[test]
    fn litre_converts_to_cubic_metre() {
        let volume = Volume::from_litre(2.5).expect("2.5 l is valid");
        assert!((volume.cubic_metre() - 0.0025).abs() < 1e-12);
    }

    #[test]
    fn zero_kelvin_is_rejected() {
        assert!(Temperature::from_kelvin(0.0).is_err());
    }
}
