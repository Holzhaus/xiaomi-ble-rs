// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Service-related functions.

use crate::device::DeviceType;
use crate::error::ParseError;
use crate::mibeacon::MiBeaconServiceAdvertisement;
use crate::sensor::SensorEvent;

use uuid::Uuid;

use thiserror::Error;

/// An error that may occur while processing a service advertisement.
#[derive(Debug, Error)]
pub enum ServiceAdvertisementError {
    /// The service advertisement payload failed to parse.
    #[error("Failed to parse service advertisement: {0}")]
    ParsingFailed(#[from] ParseError),
    /// The UUID of the service advertisement is unknown/unhandled.
    #[error("Unhandled service advertisement UUID")]
    UnhandledService,
}

/// The BLE service advertisement type.
#[derive(Debug)]
pub enum ServiceType {
    /// MiBeacon
    MiBeacon,
    /// HHCCJCY10 Plant Sensor (pink version)
    HHCCJCY10,
    /// Mi Smart Scale
    Scale1,
    /// Mi Body Composition Scale
    Scale2,
}

/// A parsed BLE service advertisement.
#[derive(Debug)]
pub enum ServiceAdvertisement {
    /// A parsed MiBeacon service advertisement.
    MiBeacon(MiBeaconServiceAdvertisement),
}

impl ServiceAdvertisement {
    /// Get device type of advertisement sender.
    #[must_use]
    pub fn device_type(&self) -> Option<&'static DeviceType> {
        match &self {
            Self::MiBeacon(parsed_adverisement) => parsed_adverisement.device_type(),
        }
    }

    /// Yields a list of sensor values parsed from the objects contained in the service advertisement.
    #[must_use]
    pub fn iter_sensor_events(&self) -> Box<dyn Iterator<Item = SensorEvent> + Send + '_> {
        match &self {
            Self::MiBeacon(parsed_adverisement) => {
                Box::new(parsed_adverisement.iter_sensor_events())
            }
        }
    }
}

/// Maps a BLE service advertisement [UUID][Uuid] to a a [ServiceType].
#[must_use]
pub const fn service_uuid_to_type(uuid: &Uuid) -> Option<ServiceType> {
    const MIBEACON_UUID: Uuid = Uuid::from_bytes([
        0x00, 0x00, 0xfe, 0x95, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34,
        0xfb,
    ]);
    const HHCCJCY10_UUID: Uuid = Uuid::from_bytes([
        0x00, 0x00, 0xfd, 0x50, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34,
        0xfb,
    ]);
    const SCALE1_UUID: Uuid = Uuid::from_bytes([
        0x00, 0x00, 0x18, 0x1d, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34,
        0xfb,
    ]);
    const SCALE2_UUID: Uuid = Uuid::from_bytes([
        0x00, 0x00, 0x18, 0x1b, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34,
        0xfb,
    ]);

    match *uuid {
        MIBEACON_UUID => Some(ServiceType::MiBeacon),
        HHCCJCY10_UUID => Some(ServiceType::HHCCJCY10),
        SCALE1_UUID => Some(ServiceType::Scale1),
        SCALE2_UUID => Some(ServiceType::Scale2),
        _ => None,
    }
}

/// Parses a service advertisement payload corresponding to the given [UUID][Uuid].
pub fn parse_service_advertisement(
    uuid: &Uuid,
    payload: &[u8],
) -> Result<ServiceAdvertisement, ServiceAdvertisementError> {
    let service_type = service_uuid_to_type(uuid);
    match service_type {
        Some(ServiceType::MiBeacon) => MiBeaconServiceAdvertisement::from_slice(payload)
            .map(ServiceAdvertisement::MiBeacon)
            .map_err(ServiceAdvertisementError::ParsingFailed),
        _ => Err(ServiceAdvertisementError::UnhandledService),
    }
}
