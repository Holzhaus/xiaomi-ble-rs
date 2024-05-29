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
//! - <https://home-is-where-you-hang-your-hack.github.io/ble_monitor/MiBeacon_protocol>
//! - <https://github.com/Bluetooth-Devices/xiaomi-ble/blob/84d79b0f7dba58472ab8fd4b3e9c27bfb838fbee/src/xiaomi_ble/parser.py>
//! - <https://github.com/Ernst79/bleparser/blob/c42ae922e1abed2720c7fac993777e1bd59c0c93/package/bleparser/xiaomi.py>

// FIXME: This lint is incompatible with `modular-bitfield` crate.
#![allow(clippy::must_use_candidate)]
// We parse a bunch of fields that are not actually used.
#![allow(dead_code)]

use crate::device::{device_id_to_type, DeviceType};
use crate::error::ParseError;
use crate::sensor::SensorValue;
use binrw::{binread, helpers::until_eof, BinRead};
use log::warn;
use modular_bitfield::prelude::*;
use std::io::Cursor;

#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
struct FrameControl {
    request_timing: bool,
    unknown: B2,
    is_encrypted: bool,
    has_mac_address: bool,
    has_capabilities: bool,
    has_objects: bool,
    is_mesh_device: bool,
    is_registered: bool,
    is_solicited: bool,
    auth_mode: B2,
    version: B4,
}

/// Parsed payload of a MiBeacon object.
#[binread]
#[br(little)]
#[br(import(id: u16, length: u8))]
#[derive(Debug)]
pub enum MiBeaconObjectPayload {
    /// Temperature (degrees dezicelsius)
    #[br(pre_assert(id == 0x1004 && length == 2))]
    Temperature(i16),
    /// Power (on/off) and Temperature (°C)
    #[br(pre_assert(id == 0x1005 && length == 2))]
    PowerAndTemperature {
        /// Power (on/off)
        power: u8,
        /// Temperature (°C)
        temperature: u8,
    },
    /// Humidity (%)
    #[br(pre_assert(id == 0x1006 && length == 2))]
    Humidity(u16),
    /// Illuminance (lx)
    #[br(pre_assert(id == 0x1007 && length == 3))]
    Illuminance([u8; 3]),
    /// Moisture (%)
    #[br(pre_assert(id == 0x1008 && length == 1))]
    Moisture(u8),
    /// Conductivity (µS/cm)
    #[br(pre_assert(id == 0x1009 && length == 2))]
    Conductivity(u16),
    /// Formaldehyde Concentration (mg/m³)
    #[br(pre_assert(id == 0x1010 && length == 2))]
    FormaldehydeConcentration(u16),
    /// Power (on/off)
    #[br(pre_assert(id == 0x1012 && length == 1))]
    Power(u8),
    /// Consumable (%)
    #[br(pre_assert(id == 0x1013 && length == 1))]
    Consumable(u8),
    /// Moisture Detection (on/off)
    #[br(pre_assert(id == 0x1014 && length == 1))]
    MoistureDetected(u8),
    /// Smoke Detection (on/off)
    #[br(pre_assert(id == 0x1015 && length == 1))]
    SmokeDetected(u8),
    /// Time without motion (s)
    #[br(pre_assert(id == 0x1017 && length == 4))]
    TimeWithoutMotion(u8),
    /// Unknown Payload
    Unknown(#[br(count = usize::from(length))] Vec<u8>),
}

impl MiBeaconObjectPayload {
    /// Map this [`MiBeaconObjectPayload`] to one or more [`Sensor Value`] objects.
    fn to_sensor_values(&self) -> Vec<SensorValue> {
        match &self {
            MiBeaconObjectPayload::Temperature(temperature_decicelsius) => {
                vec![SensorValue::Temperature(
                    f64::from(*temperature_decicelsius) / 10.0,
                )]
            }
            MiBeaconObjectPayload::PowerAndTemperature { power, temperature } => vec![
                SensorValue::Power(*power != 0),
                SensorValue::Temperature(f64::from(*temperature)),
            ],
            MiBeaconObjectPayload::Humidity(humidity) => vec![SensorValue::Humidity(*humidity)],
            MiBeaconObjectPayload::Illuminance(illuminance) => vec![SensorValue::Illuminance(
                u32::from(illuminance[2]) << 16
                    | u32::from(illuminance[1]) << 8
                    | u32::from(illuminance[0]),
            )],
            MiBeaconObjectPayload::Moisture(moisture) => vec![SensorValue::Moisture(*moisture)],
            MiBeaconObjectPayload::Conductivity(conductivity) => {
                vec![SensorValue::Conductivity(*conductivity)]
            }
            MiBeaconObjectPayload::FormaldehydeConcentration(concentration) => {
                vec![SensorValue::FormaldehydeConcentration(*concentration)]
            }
            MiBeaconObjectPayload::Power(value) => vec![SensorValue::Power(*value > 0)],
            MiBeaconObjectPayload::Consumable(consumable) => {
                vec![SensorValue::Consumable(*consumable)]
            }
            MiBeaconObjectPayload::MoistureDetected(value) => {
                vec![SensorValue::MoistureDetected(*value > 0)]
            }
            MiBeaconObjectPayload::SmokeDetected(value) => {
                vec![SensorValue::SmokeDetected(*value > 0)]
            }
            MiBeaconObjectPayload::TimeWithoutMotion(seconds) => {
                vec![SensorValue::TimeWithoutMotion(*seconds)]
            }
            _ => {
                warn!("Ignoring unhandled MiBeacon object payload: {:?}", &self);
                vec![]
            }
        }
    }
}

#[allow(dead_code)]
#[binread]
#[br(little)]
#[derive(Debug)]
struct MiBeaconObject {
    id: u16,
    length: u8,
    #[br(args(id, length))]
    payload: MiBeaconObjectPayload,
}

#[allow(dead_code)]
#[binread]
#[br(little)]
#[derive(Debug)]
struct MiBeaconCapabilities {
    types: u8,
    #[br(if(types & 0x20 != 0))]
    io: Option<u8>,
}

/// Service Advertisement in the MiBeacon format.
#[allow(dead_code)]
#[binread]
#[br(little)]
#[derive(Debug)]
pub struct MiBeaconServiceAdvertisement {
    /// Frame Control Header
    #[br(big)]
    frame_control: FrameControl,
    /// Xiaomi Device ID
    device_id: u16,
    /// Device Type (Depends on Device ID)
    #[br(calc(device_id_to_type(device_id)))]
    device_type: Option<&'static DeviceType>,
    /// Packet ID
    packet_id: u8,
    /// MAC Address (only included if [FrameControl::has_mac_address()] is `true`)
    #[br(if(frame_control.has_mac_address()))]
    mac_address: Option<[u8; 6]>,
    /// Capabilities (only included if [FrameControl::has_capabilities()] is `true`)
    #[br(if(frame_control.has_capabilities()))]
    capabilities: Option<MiBeaconCapabilities>,
    /// Objects (only included if [FrameControl::has_objects()] is `true`)
    #[br(if(frame_control.has_objects()))]
    #[br(parse_with = until_eof)]
    objects: Vec<MiBeaconObject>,
}

impl MiBeaconServiceAdvertisement {
    /// Parses a [MiBeaconServiceAdvertisement] from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self, ParseError> {
        Ok(Self::read(&mut Cursor::new(slice))?)
    }

    /// Get device type of advertisement sender.
    pub fn device_type(&self) -> Option<&'static DeviceType> {
        device_id_to_type(self.device_id)
    }

    /// Yields a list of sensor values parsed from the objects contained in the service
    /// advertisement.
    pub fn iter_sensor_values(&self) -> impl Iterator<Item = SensorValue> + '_ {
        self.objects
            .iter()
            .flat_map(|obj| obj.payload.to_sensor_values().into_iter())
    }
}
