// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Information about BLE devices.

use phf::phf_map;

/// The BLE device type.
#[derive(Debug)]
pub struct DeviceType {
    /// Name of the device.
    pub name: &'static str,
    /// Model Number of the device.
    pub model: &'static str,
}

static DEVICE_TYPES: phf::Map<u16, DeviceType> = phf_map! {
    0x0C3Cu16 => DeviceType { name: "Alarm Clock", model: "CGC1" },
    0x0576u16 => DeviceType { name: "3-in-1 Alarm Clock", model: "CGD1" },
    0x066Fu16 => DeviceType { name: "Temperature/Humidity Sensor", model: "CGDK2" },
    0x0347u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "CGG1" },
    0x0B48u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "CGG1-ENCRYPTED" },
    0x03D6u16 => DeviceType { name: "Door/Window Sensor", model: "CGH1" },
    0x0A83u16 => DeviceType { name: "Motion/Light Sensor", model: "CGPR1" },
    0x03BCu16 => DeviceType { name: "Grow Care Garden", model: "GCLS002" },
    0x0098u16 => DeviceType { name: "Plant Sensor", model: "HHCCJCY01" },
    0x015Du16 => DeviceType { name: "Smart Flower Pot", model: "HHCCPOT002" },
    0x02DFu16 => DeviceType { name: "Formaldehyde Sensor", model: "JQJCY01YM" },
    0x0997u16 => DeviceType { name: "Smoke Detector", model: "JTYJGD03MI" },
    0x1568u16 => DeviceType { name: "Switch (single button)", model: "K9B-1BTN" },
    0x1569u16 => DeviceType { name: "Switch (double button)", model: "K9B-2BTN" },
    0x0DFDu16 => DeviceType { name: "Switch (triple button)", model: "K9B-3BTN" },
    0x1C10u16 => DeviceType { name: "Switch (single button)", model: "K9BB-1BTN" },
    0x1889u16 => DeviceType { name: "Door/Window Sensor", model: "MS1BB(MI)" },
    0x2AEBu16 => DeviceType { name: "Motion Sensor", model: "HS1BB(MI)" },
    0x3F0Fu16 => DeviceType { name: "Flood and Rain Sensor", model: "RS1BB(MI)" },
    0x01AAu16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSDCGQ" },
    0x045Bu16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSD02" },
    0x16E4u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSD02MMC" },
    0x2542u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSD02MMC" },
    0x055Bu16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSD03MMC" },
    0x2832u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "MJWSD05MMC" },
    0x098Bu16 => DeviceType { name: "Door/Window Sensor", model: "MCCGQ02HL" },
    0x06D3u16 => DeviceType { name: "Alarm Clock", model: "MHO-C303" },
    0x0387u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "MHO-C401" },
    0x07F6u16 => DeviceType { name: "Nightlight", model: "MJYD02YL" },
    0x04E9u16 => DeviceType { name: "Door Lock", model: "MJZNMSQ01YD" },
    0x00DBu16 => DeviceType { name: "Baby Thermometer", model: "MMC-T201-1" },
    0x0391u16 => DeviceType { name: "Body Thermometer", model: "MMC-W505" },
    0x03DDu16 => DeviceType { name: "Nightlight", model: "MUE4094RT" },
    0x0489u16 => DeviceType { name: "Smart Toothbrush", model: "M1S-T500" },
    0x0806u16 => DeviceType { name: "Smart Toothbrush", model: "T700" },
    0x1790u16 => DeviceType { name: "Smart Toothbrush", model: "T700" },
    0x0A8Du16 => DeviceType { name: "Motion Sensor", model: "RTCGQ02LM" },
    0x3531u16 => DeviceType { name: "Motion Sensor", model: "XMPIRO2SXS" },
    0x0863u16 => DeviceType { name: "Flood Detector", model: "SJWS01LM" },
    0x045Cu16 => DeviceType { name: "Smart Kettle", model: "V-SK152" },
    0x040Au16 => DeviceType { name: "Mosquito Repellent", model: "WX08ZM" },
    0x04E1u16 => DeviceType { name: "Magic Cube", model: "XMMF01JQD" },
    0x1203u16 => DeviceType { name: "Thermometer", model: "XMWSDJ04MMC" },
    0x1949u16 => DeviceType { name: "Switch (double button)", model: "XMWXKG01YL" },
    0x2387u16 => DeviceType { name: "Button", model: "XMWXKG01LM" },
    0x098Cu16 => DeviceType { name: "Door Lock", model: "XMZNMST02YD" },
    0x0784u16 => DeviceType { name: "Door Lock", model: "XMZNMS04LM" },
    0x0E39u16 => DeviceType { name: "Door Lock", model: "XMZNMS08LM" },
    0x07BFu16 => DeviceType { name: "Wireless Switch", model: "YLAI003" },
    0x38BBu16 => DeviceType { name: "Wireless Switch", model: "PTX_YK1_QMIMB" },
    0x0153u16 => DeviceType { name: "Remote Control", model: "YLYK01YL" },
    0x068Eu16 => DeviceType { name: "Fan Remote Control", model: "YLYK01YL-FANCL" },
    0x04E6u16 => DeviceType { name: "Ventilator Fan Remote Control", model: "YLYK01YL-VENFAN" },
    0x03BFu16 => DeviceType { name: "Bathroom Heater Remote", model: "YLYB01YL-BHFRC" },
    0x03B6u16 => DeviceType { name: "Dimmer Switch", model: "YLKG07YL/YLKG08YL" },
    0x0083u16 => DeviceType { name: "Smart Kettle", model: "YM-K1501" },
    0x0113u16 => DeviceType { name: "Smart Kettle", model: "YM-K1501EU" },
    0x069Eu16 => DeviceType { name: "Door Lock", model: "ZNMS16LM" },
    0x069Fu16 => DeviceType { name: "Door Lock", model: "ZNMS17LM" },
    0x0380u16 => DeviceType { name: "Door Lock", model: "DSL-C08" },
    0x11C2u16 => DeviceType { name: "Door Lock", model: "Lockin-SV40" },
    0x0DE7u16 => DeviceType { name: "Odor Eliminator", model: "SU001-T" },
};

/// Maps a Xiamo device ID to a [DeviceType].
#[must_use]
pub fn device_id_to_type(device_id: u16) -> Option<&'static DeviceType> {
    DEVICE_TYPES.get(&device_id)
}
