// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! The sensor abstraction are used to access data regardless of the underlying protocol that is
//! used.

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
