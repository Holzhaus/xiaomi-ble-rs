// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Implementation of the MiBeacon protocol data structures.
//!
//! ## References
//!
//! - <https://github.com/Bluetooth-Devices/xiaomi-ble/blob/84d79b0f7dba58472ab8fd4b3e9c27bfb838fbee/src/xiaomi_ble/parser.py>
//! - <https://github.com/Ernst79/bleparser/blob/c42ae922e1abed2720c7fac993777e1bd59c0c93/package/bleparser/xiaomi.py>

use crate::device::DeviceType;
use crate::sensor::{NumericMeasurementType, SensorEvent, UnitOfMeasurement};
use crate::util::U24;
use crate::ParseError;
use binrw::BinRead;
use std::io::Cursor;

const HHCCJCY10_DEVICE: DeviceType = DeviceType {
    name: "Plant Sensor",
    model: "HHCCJCY10",
    manufacturer: "HHCC Plant Technology Co. Ltd",
};

/// Service Advertisement in the HHCCJCY10 Plant Sensor (Pink Version) format.
#[allow(dead_code)]
#[derive(BinRead, Debug)]
#[br(little)]
pub struct HHCCJCY10ServiceAdvertisement {
    #[allow(dead_code)]
    reserved: u32,
    /// Soil Moisture (%)
    pub moisture_percent: u8,
    /// Temperature (0.1 °C)
    pub temperature_decicelsius: u16,
    /// Illuminance (lx)
    pub illuminance_lux: U24,
    /// Battery Power (%)
    pub battery_percent: u8,
    /// Soil Electrical Conductivity (µS/cm)
    pub conductivity: u16,
}

impl HHCCJCY10ServiceAdvertisement {
    /// Parses a [HHCCJCY10ServiceAdvertisement] from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self, ParseError> {
        Ok(Self::read(&mut Cursor::new(slice))?)
    }

    /// Get device type of advertisement sender.
    #[must_use]
    pub const fn device_type(&self) -> &'static DeviceType {
        &HHCCJCY10_DEVICE
    }

    /// Yields a list of sensor events parsed from the objects contained in the service advertisement.
    pub fn iter_sensor_events(&self) -> impl Iterator<Item = SensorEvent> + '_ {
        vec![
            SensorEvent::NumericMeasurement {
                measurement_type: NumericMeasurementType::Moisture,
                value: f64::from(self.moisture_percent),
                unit: UnitOfMeasurement::Percent,
            },
            SensorEvent::NumericMeasurement {
                measurement_type: NumericMeasurementType::Temperature,
                value: f64::from(self.temperature_decicelsius) / 10.0,
                unit: UnitOfMeasurement::DegreesCelsius,
            },
            SensorEvent::NumericMeasurement {
                measurement_type: NumericMeasurementType::Illuminance,
                value: f64::from(self.illuminance_lux.as_u32()),
                unit: UnitOfMeasurement::Lux,
            },
            SensorEvent::NumericMeasurement {
                measurement_type: NumericMeasurementType::BatteryPower,
                value: f64::from(self.battery_percent),
                unit: UnitOfMeasurement::Percent,
            },
        ]
        .into_iter()
    }
}
