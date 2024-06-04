// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Implementation of the Xiaomi Mi Scale (v1/v2) protocol data structures.
//!
//! ## References
//!
//! - <https://github.com/Bluetooth-Devices/xiaomi-ble/blob/84d79b0f7dba58472ab8fd4b3e9c27bfb838fbee/src/xiaomi_ble/parser.py>
//! - <https://github.com/Ernst79/bleparser/blob/c42ae922e1abed2720c7fac993777e1bd59c0c93/package/bleparser/miscale.py>

// FIXME: This lint is incompatible with `modular-bitfield` crate.
#![allow(clippy::must_use_candidate)]

use crate::device::DeviceType;
use crate::sensor::{NumericMeasurementType, SensorEvent, UnitOfMeasurement};
use crate::ParseError;
use binrw::{binread, BinRead};
use modular_bitfield::prelude::*;
use phf::phf_map;
use std::io::Cursor;

static DEVICE_TYPES: phf::Map<u16, DeviceType> = phf_map! {
    0x181Du16 => DeviceType { name: "Mi Smart Scale", model: "XMTZC01HM/XMTZC04HM", manufacturer: "Xiaomi" },
    0x181Bu16 => DeviceType { name: "Mi Body Composition Scale", model: "XMTZC02HM/XMTZC05HM/NUN4049CN", manufacturer: "Xiaomi" },
};

enum WeightUnit {
    OneHundredPounds,
    OneHundredCatty,
    TwoHundredKilograms,
}

/// Mi Scale Packet Header (v1 protocol variant)
#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct PacketHeaderV1 {
    /// This is [`true`] if the weight unit is pounds (lbs).
    pub weight_unit_is_pounds: bool,
    #[allow(dead_code)]
    reserved: B3,
    /// This is [`true`] if the weight unit is chinese (market) catty (jin).
    pub weight_unit_is_catty: bool,
    /// This is [`true`] if the weight measurement is stabilized.
    pub weight_stabilized: bool,
    #[allow(dead_code)]
    reserved2: bool,
    /// This is [`true`] if the weight measurement is not present.
    pub weight_removed: bool,
}

/// Mi Scale Packet (v1 protocol variant)
#[binread]
#[br(little)]
#[derive(Debug)]
pub struct PacketV1 {
    /// Packet Header
    pub header: PacketHeaderV1,
    /// The measured weight value.
    pub weight: u16,
    #[allow(dead_code)]
    reserved: [u8; 7],
}

impl PacketV1 {
    /// Get weight and unit from the packet (if present).
    fn weight(&self) -> Option<(u16, WeightUnit)> {
        if self.header.weight_removed() {
            return None;
        }

        let unit = if self.header.weight_unit_is_pounds() {
            WeightUnit::OneHundredPounds
        } else if self.header.weight_unit_is_catty() {
            WeightUnit::OneHundredCatty
        } else {
            WeightUnit::TwoHundredKilograms
        };

        Some((self.weight, unit))
    }
}

/// Mi Scale Packet Header (v2 protocol variant)
#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct PacketHeaderV2 {
    #[allow(dead_code)]
    reserved: B7,
    /// This is [`true`] if the weight unit is pounds (lbs).
    pub weight_unit_is_pounds: bool,
    /// This is [`true`] if the weight measurement is not present.
    pub weight_removed: bool,
    /// This is [`true`] if the weight unit is chinese (market) catty (jin).
    pub weight_unit_is_catty: bool,
    pub weight_stabilized: bool,
    #[allow(dead_code)]
    reserved2: B3,
    pub impedance_stabilized: bool,
    #[allow(dead_code)]
    reserved3: bool,
}

/// Mi Scale Packet (v2 protocol variant)
#[binread]
#[br(little)]
#[derive(Debug)]
pub struct PacketV2 {
    /// Packet Header
    pub header: PacketHeaderV2,
    #[allow(dead_code)]
    reserved: [u8; 7],
    /// Measured body impedance.
    pub impedance: u16,
    /// Measured weight.
    pub weight: u16,
}

impl PacketV2 {
    /// Get weight and unit from the packet (if present).
    fn weight(&self) -> Option<(u16, WeightUnit)> {
        if self.header.weight_removed() {
            return None;
        }

        let unit = if self.header.weight_unit_is_pounds() {
            WeightUnit::OneHundredPounds
        } else if self.header.weight_unit_is_catty() {
            WeightUnit::OneHundredCatty
        } else {
            WeightUnit::TwoHundredKilograms
        };

        Some((self.weight, unit))
    }
}

/// Mi Scale Packet (either v1 or v2 protocol variant).
#[binread]
#[br(little)]
#[derive(Debug)]
#[br(import(device_id: u16))]
pub enum MiScalePacket {
    /// This is a v1 protocol packet.
    #[br(pre_assert(device_id == 0x181D))]
    MiScaleV1(PacketV1),
    /// This is a v2 protocol packet.
    #[br(pre_assert(device_id == 0x181B))]
    MiScaleV2(PacketV2),
}

impl MiScalePacket {
    /// Get the body impedance (Ohm) from the packet (v2 only).
    fn impedance(&self) -> Option<u16> {
        match &self {
            Self::MiScaleV1(_) => None,
            Self::MiScaleV2(payload) => Some(payload.impedance),
        }
    }

    /// Get the weight and unit from the packet (may be omitted in v2).
    fn weight(&self) -> Option<(u16, WeightUnit)> {
        match &self {
            Self::MiScaleV1(payload) => payload.weight(),
            Self::MiScaleV2(payload) => payload.weight(),
        }
    }

    /// Get the weight from the packet (normalized to kg, may be omitted in v2).
    fn weight_kilograms(&self) -> Option<f64> {
        self.weight().map(|(weight, unit)| match unit {
            WeightUnit::TwoHundredKilograms => f64::from(weight) * 0.005,
            WeightUnit::OneHundredPounds => f64::from(weight) * 0.0045359237,
            WeightUnit::OneHundredCatty => f64::from(weight) * 0.01,
        })
    }
}

/// Service Advertisement in the HHCCJCY10 Plant Sensor (Pink Version) format.
#[binread]
#[br(little)]
#[derive(Debug)]
pub struct MiScaleServiceAdvertisement {
    #[allow(dead_code)]
    reserved: u16,
    /// Device ID.
    pub device_id: u16,
    /// Packet Payload.
    #[br(args(device_id))]
    pub payload: MiScalePacket,
}

impl MiScaleServiceAdvertisement {
    /// Parses a [MiScaleServiceAdvertisement] from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self, ParseError> {
        Ok(Self::read(&mut Cursor::new(slice))?)
    }

    /// Get device type of advertisement sender.
    #[must_use]
    pub fn device_type(&self) -> Option<&'static DeviceType> {
        DEVICE_TYPES.get(&self.device_id)
    }

    /// Yields a list of sensor events parsed from the objects contained in the service advertisement.
    pub fn iter_sensor_events(&self) -> impl Iterator<Item = SensorEvent> + '_ {
        let mut events = Vec::with_capacity(2);
        if let Some(weight) = self.payload.weight_kilograms() {
            events.push(SensorEvent::NumericMeasurement {
                measurement_type: NumericMeasurementType::Weight,
                value: weight,
                unit: UnitOfMeasurement::Kilogram,
            });
        }
        if let Some(impedance) = self.payload.impedance() {
            events.push(SensorEvent::NumericMeasurement {
                measurement_type: NumericMeasurementType::Impedance,
                value: f64::from(impedance),
                unit: UnitOfMeasurement::Ohm,
            });
        }
        events.into_iter()
    }
}
