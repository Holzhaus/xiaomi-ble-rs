// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! The sensor abstraction are used to access data regardless of the underlying protocol that is
//! used.

use core::fmt;

/// Represents a measured sensor value.
#[derive(Debug, Copy, Clone)]
pub enum SensorEvent {
    /// A binary measurement (true/false).
    BinaryMeasurement {
        /// The type of the measurement.
        measurement_type: BinaryMeasurementType,
        /// The measured value.
        value: bool,
    },
    /// A numeric measurement (i.e. [`f64`]).
    NumericMeasurement {
        /// The type of the measurement.
        measurement_type: NumericMeasurementType,
        /// The measured value.
        value: f64,
        /// The unit of the value.
        unit: UnitOfMeasurement,
    },
}

impl fmt::Display for SensorEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::BinaryMeasurement {
                measurement_type,
                value,
            } => write!(f, "{} = {}", measurement_type, value),
            Self::NumericMeasurement {
                measurement_type,
                value,
                unit,
            } => write!(f, "{} = {} {}", measurement_type, value, unit),
        }
    }
}

/// Measurement type for binary sensors.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum BinaryMeasurementType {
    /// Power State.
    Power,
    /// Sleep State.
    Sleep,
    /// Binding State.
    Binding,
    /// Switch State.
    Switch,
    /// Water Immersion State.
    WaterImmersion,
    /// Gas Leakage State.
    GasLeak,
    /// Light State.
    Light,
}

impl BinaryMeasurementType {
    /// Get the lowercase name of this type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match &self {
            Self::Power => "power",
            Self::Sleep => "sleep",
            Self::Binding => "binding",
            Self::Switch => "switch",
            Self::WaterImmersion => "water_immersion",
            Self::GasLeak => "gas_leak",
            Self::Light => "light",
        }
    }
}

impl fmt::Display for BinaryMeasurementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Measurement type for numeric sensors.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum NumericMeasurementType {
    /// A temperature measurement.
    Temperature,
    /// A humidity measurement.
    Humidity,
    /// An illuminance measurement.
    Illuminance,
    /// An moisture measurement.
    Moisture,
    /// An electrical conductivity measurement.
    Conductivity,
    /// A Formaldehyde concentration measurement.
    FormaldehydeConcentration,
    /// A measurement of remaining supplies.
    RemainingSupplies,
    /// A battery power measurement.
    BatteryPower,
    /// A weight measurement.
    Weight,
    /// An impedance measurement.
    Impedance,
}

impl NumericMeasurementType {
    /// Get the lowercase name of this type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match &self {
            Self::Temperature => "temperature",
            Self::Humidity => "humidity",
            Self::Illuminance => "illuminance",
            Self::Moisture => "moisture",
            Self::Conductivity => "conductivity",
            Self::FormaldehydeConcentration => "formaldehyde_concentration",
            Self::RemainingSupplies => "remaining_supplies",
            Self::BatteryPower => "battery_power",
            Self::Weight => "weight",
            Self::Impedance => "impedance",
        }
    }
}

impl fmt::Display for NumericMeasurementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The unit of measurement.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum UnitOfMeasurement {
    /// Degrees Celsius (°C)
    DegreesCelsius,
    /// Percent (%)
    Percent,
    /// Lux (lx)
    Lux,
    /// Microsiemens per Centimeter (µS/cm)
    MicrosiemensPerCentimeter,
    /// Milligram per Cubic Meter (mg/m³)
    MilligramPerCubicMeter,
    /// Seconds (s)
    Seconds,
    /// Kilogram (kg)
    Kilogram,
    /// Ohm (Ω)
    Ohm,
}

impl UnitOfMeasurement {
    /// Get the unit as [`str`].
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match &self {
            Self::DegreesCelsius => "°C",
            Self::Percent => "%",
            Self::Lux => "lx",
            Self::MicrosiemensPerCentimeter => "µS/cm",
            Self::MilligramPerCubicMeter => "mg/m³",
            Self::Seconds => "s",
            Self::Kilogram => "kg",
            Self::Ohm => "Ω",
        }
    }
}

impl fmt::Display for UnitOfMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
