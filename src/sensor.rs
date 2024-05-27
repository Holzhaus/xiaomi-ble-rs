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

/// Represents a sensor value.
#[derive(Debug, Copy, Clone)]
pub enum SensorValue {
    /// Power (on/off)
    Power(bool),
    /// Temperature (°C)
    Temperature(f64),
    /// Humidity (%)
    Humidity(u16),
    /// Illuminance (lx)
    Illuminance(u32),
    /// Moisture (%)
    Moisture(u8),
    /// Conductivity (µS/cm)
    Conductivity(u16),
    /// Formaldehyde Concentration (mg/m³)
    FormaldehydeConcentration(u16),
    /// Consumable (%)
    Consumable(u8),
    /// Moisture Detection (on/off)
    MoistureDetected(bool),
    /// Smoke Detection (on/off)
    SmokeDetected(bool),
    /// Time without motion (s)
    TimeWithoutMotion(u8),
}

impl fmt::Display for SensorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Power(value) => write!(f, "Power = {}", if *value { "on" } else { "off" }),
            Self::Temperature(value) => write!(f, "Temperature = {} °C", value),
            Self::Humidity(value) => write!(f, "Humidity = {} %", value),
            Self::Illuminance(value) => write!(f, "Illuminance = {} lx", value),
            Self::Moisture(value) => write!(f, "Moisture = {} %", value),
            Self::Conductivity(value) => write!(f, "Conductivity = {} µS/cm", value),
            Self::FormaldehydeConcentration(value) => {
                write!(f, "Formaldehyde Concentration = {} mg/m³", value)
            }
            Self::Consumable(value) => write!(f, "Consumable = {} %", value),
            Self::MoistureDetected(value) => write!(
                f,
                "Moisture detected = {}",
                if *value { "yes" } else { "no" }
            ),
            Self::SmokeDetected(value) => {
                write!(f, "Smoke detected = {}", if *value { "yes" } else { "no" })
            }
            Self::TimeWithoutMotion(value) => write!(f, "Time without motion = {} s", value),
        }
    }
}
