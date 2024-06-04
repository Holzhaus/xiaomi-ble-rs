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
//! - <https://iot.mi.com/new/doc/accesses/direct-access/embedded-development/ble/ble-mibeacon>
//! - <https://iot.mi.com/new/doc/accesses/direct-access/embedded-development/ble/object-definition>
//! - <https://home-is-where-you-hang-your-hack.github.io/ble_monitor/MiBeacon_protocol>
//! - <https://github.com/Bluetooth-Devices/xiaomi-ble/blob/84d79b0f7dba58472ab8fd4b3e9c27bfb838fbee/src/xiaomi_ble/parser.py>
//! - <https://github.com/Ernst79/bleparser/blob/c42ae922e1abed2720c7fac993777e1bd59c0c93/package/bleparser/xiaomi.py>

// FIXME: This lint is incompatible with `modular-bitfield` crate.
#![allow(clippy::must_use_candidate)]

use crate::device::DeviceType;
use crate::sensor::{NumericMeasurementType, SensorEvent, UnitOfMeasurement};
use crate::util::U24;
use crate::ParseError;
use binrw::{binread, helpers::until_eof, BinRead};
use core::fmt;
use log::warn;
use modular_bitfield::prelude::*;
use phf::phf_map;
use std::io::Cursor;

static DEVICE_TYPES: phf::Map<u16, DeviceType> = phf_map! {
    0x0C3Cu16 => DeviceType { name: "Alarm Clock", model: "CGC1", manufacturer: "Xiaomi" },
    0x0576u16 => DeviceType { name: "3-in-1 Alarm Clock", model: "CGD1", manufacturer: "Xiaomi" },
    0x066Fu16 => DeviceType { name: "Temperature/Humidity Sensor", model: "CGDK2", manufacturer: "Xiaomi" },
    0x0347u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "CGG1", manufacturer: "Xiaomi" },
    0x0B48u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "CGG1-ENCRYPTED", manufacturer: "Xiaomi" },
    0x03D6u16 => DeviceType { name: "Door/Window Sensor", model: "CGH1", manufacturer: "Xiaomi" },
    0x0A83u16 => DeviceType { name: "Motion/Light Sensor", model: "CGPR1", manufacturer: "Xiaomi" },
    0x03BCu16 => DeviceType { name: "Grow Care Garden", model: "GCLS002", manufacturer: "Xiaomi" },
    0x0098u16 => DeviceType { name: "Plant Sensor", model: "HHCCJCY01", manufacturer: "Xiaomi" },
    0x015Du16 => DeviceType { name: "Smart Flower Pot", model: "HHCCPOT002", manufacturer: "Xiaomi" },
    0x02DFu16 => DeviceType { name: "Formaldehyde Sensor", model: "JQJCY01YM", manufacturer: "Xiaomi" },
    0x0997u16 => DeviceType { name: "Smoke Detector", model: "JTYJGD03MI", manufacturer: "Xiaomi" },
    0x1568u16 => DeviceType { name: "Switch (single button)", model: "K9B-1BTN", manufacturer: "Xiaomi" },
    0x1569u16 => DeviceType { name: "Switch (double button)", model: "K9B-2BTN", manufacturer: "Xiaomi" },
    0x0DFDu16 => DeviceType { name: "Switch (triple button)", model: "K9B-3BTN", manufacturer: "Xiaomi" },
    0x1C10u16 => DeviceType { name: "Switch (single button)", model: "K9BB-1BTN", manufacturer: "Xiaomi" },
    0x1889u16 => DeviceType { name: "Door/Window Sensor", model: "MS1BB(MI)", manufacturer: "Xiaomi" },
    0x2AEBu16 => DeviceType { name: "Motion Sensor", model: "HS1BB(MI)", manufacturer: "Xiaomi" },
    0x3F0Fu16 => DeviceType { name: "Flood and Rain Sensor", model: "RS1BB(MI)", manufacturer: "Xiaomi" },
    0x01AAu16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSDCGQ", manufacturer: "Xiaomi" },
    0x045Bu16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSD02", manufacturer: "Xiaomi" },
    0x16E4u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSD02MMC", manufacturer: "Xiaomi" },
    0x2542u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSD02MMC", manufacturer: "Xiaomi" },
    0x055Bu16 => DeviceType { name: "Temperature/Humidity Sensor", model: "LYWSD03MMC", manufacturer: "Xiaomi" },
    0x2832u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "MJWSD05MMC", manufacturer: "Xiaomi" },
    0x098Bu16 => DeviceType { name: "Door/Window Sensor", model: "MCCGQ02HL", manufacturer: "Xiaomi" },
    0x06D3u16 => DeviceType { name: "Alarm Clock", model: "MHO-C303", manufacturer: "Xiaomi" },
    0x0387u16 => DeviceType { name: "Temperature/Humidity Sensor", model: "MHO-C401", manufacturer: "Xiaomi" },
    0x07F6u16 => DeviceType { name: "Nightlight", model: "MJYD02YL", manufacturer: "Xiaomi" },
    0x04E9u16 => DeviceType { name: "Door Lock", model: "MJZNMSQ01YD", manufacturer: "Xiaomi" },
    0x00DBu16 => DeviceType { name: "Baby Thermometer", model: "MMC-T201-1", manufacturer: "Xiaomi" },
    0x0391u16 => DeviceType { name: "Body Thermometer", model: "MMC-W505", manufacturer: "Xiaomi" },
    0x03DDu16 => DeviceType { name: "Nightlight", model: "MUE4094RT", manufacturer: "Xiaomi" },
    0x0489u16 => DeviceType { name: "Smart Toothbrush", model: "M1S-T500", manufacturer: "Xiaomi" },
    0x0806u16 => DeviceType { name: "Smart Toothbrush", model: "T700", manufacturer: "Xiaomi" },
    0x1790u16 => DeviceType { name: "Smart Toothbrush", model: "T700", manufacturer: "Xiaomi" },
    0x0A8Du16 => DeviceType { name: "Motion Sensor", model: "RTCGQ02LM", manufacturer: "Xiaomi" },
    0x3531u16 => DeviceType { name: "Motion Sensor", model: "XMPIRO2SXS", manufacturer: "Xiaomi" },
    0x0863u16 => DeviceType { name: "Flood Detector", model: "SJWS01LM", manufacturer: "Xiaomi" },
    0x045Cu16 => DeviceType { name: "Smart Kettle", model: "V-SK152", manufacturer: "Xiaomi" },
    0x040Au16 => DeviceType { name: "Mosquito Repellent", model: "WX08ZM", manufacturer: "Xiaomi" },
    0x04E1u16 => DeviceType { name: "Magic Cube", model: "XMMF01JQD", manufacturer: "Xiaomi" },
    0x1203u16 => DeviceType { name: "Thermometer", model: "XMWSDJ04MMC", manufacturer: "Xiaomi" },
    0x1949u16 => DeviceType { name: "Switch (double button)", model: "XMWXKG01YL", manufacturer: "Xiaomi" },
    0x2387u16 => DeviceType { name: "Button", model: "XMWXKG01LM", manufacturer: "Xiaomi" },
    0x098Cu16 => DeviceType { name: "Door Lock", model: "XMZNMST02YD", manufacturer: "Xiaomi" },
    0x0784u16 => DeviceType { name: "Door Lock", model: "XMZNMS04LM", manufacturer: "Xiaomi" },
    0x0E39u16 => DeviceType { name: "Door Lock", model: "XMZNMS08LM", manufacturer: "Xiaomi" },
    0x07BFu16 => DeviceType { name: "Wireless Switch", model: "YLAI003", manufacturer: "Xiaomi" },
    0x38BBu16 => DeviceType { name: "Wireless Switch", model: "PTX_YK1_QMIMB", manufacturer: "Xiaomi" },
    0x0153u16 => DeviceType { name: "Remote Control", model: "YLYK01YL", manufacturer: "Xiaomi" },
    0x068Eu16 => DeviceType { name: "Fan Remote Control", model: "YLYK01YL-FANCL", manufacturer: "Xiaomi" },
    0x04E6u16 => DeviceType { name: "Ventilator Fan Remote Control", model: "YLYK01YL-VENFAN", manufacturer: "Xiaomi" },
    0x03BFu16 => DeviceType { name: "Bathroom Heater Remote", model: "YLYB01YL-BHFRC", manufacturer: "Xiaomi" },
    0x03B6u16 => DeviceType { name: "Dimmer Switch", model: "YLKG07YL/YLKG08YL", manufacturer: "Xiaomi" },
    0x0083u16 => DeviceType { name: "Smart Kettle", model: "YM-K1501", manufacturer: "Xiaomi" },
    0x0113u16 => DeviceType { name: "Smart Kettle", model: "YM-K1501EU", manufacturer: "Xiaomi" },
    0x069Eu16 => DeviceType { name: "Door Lock", model: "ZNMS16LM", manufacturer: "Xiaomi" },
    0x069Fu16 => DeviceType { name: "Door Lock", model: "ZNMS17LM", manufacturer: "Xiaomi" },
    0x0380u16 => DeviceType { name: "Door Lock", model: "DSL-C08", manufacturer: "Xiaomi" },
    0x11C2u16 => DeviceType { name: "Door Lock", model: "Lockin-SV40", manufacturer: "Xiaomi" },
    0x0DE7u16 => DeviceType { name: "Odor Eliminator", model: "SU001-T", manufacturer: "Xiaomi" },
};

/// Maps a MiBeacon device ID to a [DeviceType].
#[must_use]
pub fn device_id_to_type(device_id: u16) -> Option<&'static DeviceType> {
    DEVICE_TYPES.get(&device_id)
}

/// MAC Address of a device.
#[derive(BinRead)]
#[br(little)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Get a reference to the underlying byte array.
    pub fn as_slice(&self) -> &[u8; 6] {
        &self.0
    }

    /// Get the underlying byte array.
    pub fn into_inner(self) -> [u8; 6] {
        self.0
    }
}

impl fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// Frame Control Structure
#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct FrameControl {
    /// Reserved
    #[allow(dead_code)]
    reserved: B3,
    /// 0: The packet is not encrypted; 1: The packet is encrypted
    pub is_encrypted: bool,
    /// 0: does not include MAC address; 1: includes a fixed MAC address (the MAC address is included so that iOS can recognize this device and connect)
    pub mac_included: bool,
    /// 0: does not include Capability; 1: includes Capability. Before the device is bound, this bit is forced to be 1
    pub capabilities_included: bool,
    /// 0: does not contain [`MiBeaconObjectPayload`] objects; 1: contains the object
    pub objects_included: bool,
    /// 0: Not including Mesh; 1: including Mesh. For standard BLE access products and high security level access, this item is mandatory to be 0. For Mesh access, this item is mandatory to be 1.
    pub mesh: bool,
    /// 0: The device is not bound; 1: The device is registered and bound. This item is used to indicate whether the device has been reset.
    pub registered: bool,
    /// 0: No operation; 1: Request APP to register and bind. It is only effective when the user selects the device to confirm the pairing on the developer platform, otherwise it is set to 0. The original name of this item is bindingCfm, and it has been renamed solicited to "actively request, solicit" APP to register and bind.
    pub solicited: bool,
    /// 0: Old version certification; 1: Security certification; 2: Standard certification; 3: Reserved
    pub auth_mode: B2,
    /// Version number (currently v5)
    pub version: B4,
}

/// Key ID of a Fingerprint Event
#[derive(BinRead, Debug)]
#[br(little)]
pub enum FingerprintEventKeyId {
    /// Lock Administrator (0x00000000)
    #[br(magic(0x00000000u32))]
    LockAdministrator,
    /// Unknown Operator (0xFFFFFFFF)
    #[br(magic(0xFFFFFFFFu32))]
    UnknownOperator,
    /// Key ID
    KeyId(u32),
}

/// Matching Result of a Fingerprint Event
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum FingerprintEventMatchingResult {
    /// Matching successful
    MatchingSuccessful = 0x00,
    /// Matching failed
    MatchingFailed = 0x01,
    /// Timeout and no entry
    Timeout = 0x02,
    /// Low quality (too light, blurry)
    LowQuality = 0x03,
    /// Insufficient area
    InsufficientArea = 0x04,
    /// Skin is too dry
    SkinTooDry = 0x05,
    /// Skin is too wet
    SkinTooWet = 0x06,
}

/// Door Event status
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum DoorEvent {
    /// Open the door
    DoorOpened = 0x00,
    /// Close the door
    DoorClosed = 0x01,
    /// Timeout not closed
    DoorCloseTimeout = 0x02,
    /// Knocking on the door
    KnockingOnTheDoor = 0x03,
    /// Prying the door
    PryingTheDoorOpen = 0x04,
    /// Door stuck
    DoorStuck = 0x05,
}

/// Arming Event status field.
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum ArmingEventStatus {
    /// Armed
    Armed = 0x00,
    /// Disarmed
    Disarmed = 0x01,
}

/// Gesture Type of a Gesture Event.
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u16))]
pub enum Gesture {
    /// Shake
    Shake = 0x0001,
    /// Flip 90 degrees
    FlipNinetyDegrees = 0x0002,
    /// Flip 180 degrees
    FlipOneHundredEightyDegrees = 0x0003,
    /// Plane rotation
    PlaneRotation = 0x0004,
    /// Knock
    Knock = 0x0005,
    /// Nudge
    Nudge = 0x0006,
}

/// Operation field of a Lock Event.
#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct LockEventOperation {
    /// The lower 4 bits of the operation field represent the action, which is divided into the following categories:
    ///
    /// - 0000b: Unlock the door from outside
    /// - 0001b: Locked (If it is impossible to distinguish whether the door is locked from inside or outside, this type of report is used)
    /// - 0010b: Enable anti-lock
    /// - 0011b: Unlock
    /// - 0100b: Unlock the door from inside
    /// - 0101b: Door locked from inside
    /// - 0110b: Open the child lock
    /// - 0111b: Turn off the child lock
    /// - 1000b: Door locked from outside
    /// - 1111b: Abnormal
    pub operation_action: B4,
    /// The high 4 bits of the operation field represent the method, which is divided into the following categories:
    ///
    /// - 0000b: Bluetooth mode
    /// - 0001b: Password mode
    /// - 0010b: Biometrics (fingerprints, faces, human veins, palm prints, etc.)
    /// - 0011b: Key mode
    /// - 0100b: Turntable
    /// - 0101b: NFC method
    /// - 0110b: One-time password
    /// - 0111b: Two-factor authentication
    /// - 1001b：Homekit
    /// - 1000b: Coercion
    /// - 1010b: Artificial
    /// - 1011b: Automatic
    /// - 1111b: Abnormal
    pub operation_method: B4,
}

/// Lock Event Payload
#[derive(BinRead, Debug)]
#[br(little)]
pub struct LockEvent {
    /// Lock Event Operation.
    pub operation: LockEventOperation,
    /// Key IDs are divided into the following categories:
    ///
    /// - 0x00000000: Lock administrator
    /// - 0xFFFFFFFF: unknown operator
    /// - 0xDEADBEEF: Invalid operator
    /// - 0x00000000 - 0x7FFFFFFF: Bluetooth (up to 2147483647)
    /// - 0x80010000 - 0x8001FFFF: Biometrics - Fingerprint (up to 65536)
    /// - 0x80020000 - 0x8002FFFF: Password (up to 65536)
    /// - 0x80030000 - 0x8003FFFF: Keys (up to 65536)
    /// - 0x80040000 - 0x8004FFFF: NFC (up to 65536)
    /// - 0x80050000 - 0x8005FFFF: Two-factor authentication (up to 65536)
    /// - 0x80060000 - 0x8006FFFF: Biometrics - Face (up to 65536)
    /// - 0x80070000 - 0x8007FFFF: Biometrics - Finger vein (up to 65536)
    /// - 0x80080000 - 0x8008FFFF: Biometrics - Palmprint (up to 65536)
    ///
    /// IDs starting with 0xC0DE indicate exceptions. Externally triggered exceptions include:
    ///
    /// - 0xC0DE0000: Frequent unlocking with incorrect password
    /// - 0xC0DE0001: Frequent unlocking with incorrect fingerprints
    /// - 0xC0DE0002: Operation timeout (password input timeout)
    /// - 0xC0DE0003: Lock picking
    /// - 0xC0DE0004: Reset button pressed
    /// - 0xC0DE0005: Frequent unlocking with wrong key
    /// - 0xC0DE0006: Foreign object in keyhole
    /// - 0xC0DE0007: Key not removed
    /// - 0xC0DE0008: Error NFC frequent unlocking
    /// - 0xC0DE0009: The lock was not locked as required after the timeout.
    /// - 0xC0DE000A: Multiple unlocking methods frequently fail
    /// - 0xC0DE000B: Frequent face unlocking failures
    /// - 0xC0DE000C: Frequent vein unlocking failures
    /// - 0xC0DE000D: Hijacking alarm
    /// - 0xC0DE000E: Door unlocked after arming
    /// - 0xC0DE000F: Frequent palm print unlocking failures
    /// - 0xC0DE0010: The safe has been moved
    ///
    /// Internally triggered exceptions include:
    ///
    /// - 0xC0DE1000: Battery level is less than 10%
    /// - 0xC0DE1001: Battery level is less than 5%
    /// - 0xC0DE1002: Fingerprint sensor abnormality
    /// - 0xC0DE1003: Accessory battery is low
    /// - 0xC0DE1004: Mechanical failure
    /// - 0xC0DE1005: Lock sensor failure
    pub key_id: u32,
    /// Timestamp (UTC)
    pub timestamp: u32,
}

/// Flooding Alarm Event Payload
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum FloodingAlarmEvent {
    /// Alarm Cleared
    AlarmCleared = 0x00,
    /// Alarm
    Alarm = 0x01,
}

/// Smoke Alarm Event Payload
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum SmokeAlarmEvent {
    /// Normal Monitoring
    Normal = 0x00,
    /// Fire Alarm
    FireAlarm = 0x01,
    /// Equipment Failure
    EquipmentFailure = 0x02,
    /// Equipment Self-Test
    EquipmentSelfTest = 0x03,
    /// Analog Alarm
    AnalogAlarm = 0x04,
}

/// Gas Alarm Event Payload
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum GasAlarmEvent {
    /// Normal Monitoring
    Normal = 0x00,
    /// Gas Leak Alarm
    GasLeakAlarm = 0x01,
    /// Equipment Failure
    EquipmentFailure = 0x02,
    /// Sensor Life Expiration
    SensorLifeExpiration = 0x03,
    /// Sensor Preheating
    SensorPreheating = 0x04,
    /// Equipment Self-Test
    EquipmentSelfTest = 0x05,
    /// Analog Alarm
    AnalogAlarm = 0x06,
}

/// Toothbrush Event Type
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum ToothbrushEventType {
    /// Brushing Started
    BrushingStarted,
    /// Brushing Ended
    BrushingEnded,
}

/// Maoyan Doorbell Camera Event Payload
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum DoorbellCameraEvent {
    /// Someone is staying
    SomeoneIsStaying = 0x00,
    /// Someone is passing by
    SomeoneIsPassingBy = 0x01,
    /// Someone is ringing the bell
    SomeoneIsRingingTheBell = 0x02,
    /// Someone is leaving a message
    SomeoneIsLeavingAMessage = 0x03,
    /// Equipment Damage
    EquipmentDamage = 0x04,
    /// Duress Alarm
    DuressAlarm = 0x05,
    /// Abnormal Unlocking
    AbnormalUnlocking = 0x06,
}

/// Weighing Event Type
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum WeighingEventType {
    /// Current Weight
    CurrentWeight = 0x00,
    /// Reduced Weight
    ReducedWeight = 0x01,
    /// Increased Weight
    IncreasedWeight = 0x02,
}

/// Button Event Type
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum ButtonEventType {
    /// Single Click
    SingleClick = 0x00,
    /// Double Click
    DoubleClick = 0x01,
    /// Long Press
    LongPress = 0x02,
    /// Triple Click
    TripleClick = 0x03,
}

/// Sleep State
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum SleepState {
    /// Not Sleeping
    NotSleeping = 0x00,
    /// Asleep
    Asleep = 0x01,
}

/// Lock State
///
/// **Normal Combinations:**
/// - 0x00: unlocked (all lock tongues retracted)
/// - 0x04: Lock tongue pops up (tilted tongue pops up)
/// - 0x05: Locked + Lock tongue pops out (square tongue, oblique tongue pops out)
/// - 0x06: Anti-lock + Lock tongue pops up (dead tongue, oblique tongue pops up)
/// - 0x07: All bolts pop out (square bolt, dead bolt, and oblique bolt pop out)
#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct LockState {
    /// Tongue ejected or retracted
    pub tongue_ejected: bool,
    /// Dead Tongue ejected or retracted
    pub dead_tongue_ejected: bool,
    /// Latch ejected or retracted
    pub latch_ejected: bool,
    /// Child lock ejected or retracted
    pub child_lock_ejected: bool,
    /// Reserved Bits
    #[allow(dead_code)]
    reserved: B4,
}

/// Door State
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum DoorState {
    /// Door is open.
    Open = 0x00,
    /// Door is closed.
    Closed = 0x01,
    /// Abnormal State
    Abnormal = 0xFF,
}

/// Binding State
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum BindingState {
    /// Unbound
    Unbound = 0x00,
    /// Bound
    Bound = 0x01,
}

/// Switch State
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum SwitchState {
    /// Disabled
    Disabled = 0x00,
    /// Enabled
    Enabled = 0x01,
}

/// Water Immersion State
#[derive(BinRead, Debug)]
#[br(little)]
#[br(repr(u8))]
pub enum WaterImmersionState {
    /// Not Submerged
    NotSubmerged = 0x00,
    /// Submerged
    Submerged = 0x01,
}

/// Smoke Detection State
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum SmokeDetectionState {
    /// Normal monitoring
    Normal = 0x00,
    /// Fire Alarm
    FireAlarm = 0x01,
    /// Equipment Failure
    EquipmentFailure = 0x02,
}

/// Gas Leakage Detection State
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum GasLeakageDetectionState {
    /// Leakage
    Leakage = 0x00,
    /// No Leakage
    NoLeakage = 0x01,
}

/// Light Intensity State
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum LightIntensityState {
    /// Weak Light
    Dark = 0x00,
    /// Strong Light
    Light = 0x01,
}

/// Door Sensor State
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum DoorSensorState {
    /// Door Open
    DoorOpen = 0x00,
    /// Door Closed
    DoorClosed = 0x01,
    /// Timeout
    Timeout = 0x02,
    /// Device Reset
    DeviceReset = 0x03,
}

/// Movement Detection State
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum MovementDetectionState {
    /// Movement detected in configured timeframe
    MovementDetectedWithinTimeframe = 0x00,
    /// No movement detected in configured timeframe
    NoMovementDetectedWithinTimeframe = 0x01,
}

/// Smart Pillow State
#[derive(BinRead, Debug)]
pub enum SmartPillowState {
    /// Out of Bed
    #[br(magic(0x00u8))]
    OutOfBed,
    /// In Bed
    #[br(magic(0x01u8))]
    InBed,
    /// Reserved (0x02 - 0xFF)
    Reserved(u8),
}

/// Huami Mi Band Sleep State
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum MiBandSleepState {
    /// No Sleep Event occurred
    None = 0x00,
    /// User fell asleep
    FallAsleep = 0x01,
    /// User woke up
    WakeUp = 0x02,
}

/// Roidmi Vacuum Cleaner State
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum RoidmiVacuumCleanerState {
    /// Vacuum Cleaner is charging
    Charging = 0x00,
    /// Vacuum Cleaner is in standby mode
    Standby = 0x01,
    /// Is cleaning in standard mode
    Standard = 0x02,
    /// Is cleaning in strong mode
    Strong = 0x03,
    /// Abnormal state
    Abnormal = 0xFF,
}

/// Flower and Grass Detector Event
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum FlowerAndGrassDetectorEvent {
    /// Normal
    Normal = 0x00,
    /// Unplugged
    Unplugged = 0x01,
}

/// Quingping Sensor Location Event
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum QuingpingSensorLocationEvent {
    /// Separated From Base
    SeparatedFromBase = 0x00,
    /// Connected
    Connected = 0x01,
}

/// Quingping Pomodoro Event
#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum QuingpingPomodoroEvent {
    /// Start of Pomodoro
    Start = 0x00,
    /// End of Pomodoro
    End = 0x01,
    /// Start of Break
    StartOfBreak = 0x02,
    /// End of Break
    EndOfBreak = 0x03,
}

/// Parsed payload of a MiBeacon object.
#[derive(BinRead, Debug)]
#[br(import(id: u16, length: u8))]
pub enum MiBeaconObjectPayload {
    // Common Events (0x0000 - 0x1001)
    /// Connect Event
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    /// - **Value:** Object ID to be paired, such as key event (0x1001).
    #[br(pre_assert(id == 0x0001))]
    #[br(assert(length == 2))]
    ConnectEvent(u16),

    /// Simple Pairing Event
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    /// - **Value:** Object ID to be paired, such as key event (0x1001).
    #[br(pre_assert(id == 0x0002))]
    #[br(assert(length == 2))]
    SimplePairingEvent(u16),

    /// Proximity Event
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0003))]
    #[br(assert(length == 2))]
    ProximityEvent(u16),

    /// Keep Away Event
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0004))]
    #[br(assert(length == 2))]
    KeepAwayEvent(u16),

    /// Lock Event (deprecated)
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0005))]
    #[br(assert(length == 2))]
    LockEventDeprecated(u16),

    /// Fingerprint Event
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0006))]
    #[br(assert(length == 5))]
    FingerprintEvent {
        /// Key ID
        key_id: FingerprintEventKeyId,
        /// Fingerprint Matching Result
        matching_result: FingerprintEventMatchingResult,
    },

    /// Door Event
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0007))]
    #[br(assert(length == 1))]
    DoorEvent(DoorEvent),

    /// Arming
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0008))]
    #[br(assert(length == 1 || length == 5))]
    ArmingEvent {
        /// Status
        status: ArmingEventStatus,
        /// Timestamp (UTC time, optional)
        #[br(if(length == 5))]
        timestamp: Option<u32>,
    },

    /// Gesture Event
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0009))]
    #[br(assert(length == 2))]
    GestureEvent(Gesture),

    /// Body Temperature (0.01 °C) Event
    ///
    /// - **Time Interval:** 60 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x000A))]
    #[br(assert(length == 2))]
    BodyTemperatureEvent(i16),

    /// Lock Event
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x000B))]
    #[br(assert(length == 9))]
    LockEvent(LockEvent),

    /// Flooding Alarm Event
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x000C))]
    #[br(assert(length == 1))]
    FloodingAlarmEvent(FloodingAlarmEvent),

    /// Smoke Detector Alarm
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x000D))]
    #[br(assert(length == 1))]
    SmokeAlarmEvent(SmokeAlarmEvent),

    /// Gas Leak Alarm
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x000E))]
    #[br(assert(length == 1))]
    GasAlarmEvent(GasAlarmEvent),

    /// Movement Detector Alarm (with Illuminance)
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x000F))]
    #[br(assert(length == 3))]
    MovementAlarmWithIlluminanceEvent(U24),

    /// Toothbrush Event
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0010))]
    #[br(assert(length == 1 || length == 2))]
    ToothbrushEvent {
        /// 0 = Start Brushing, 1 = End Brushing
        event_type: ToothbrushEventType,
        /// Score (optional)
        ///
        /// - **Range:** 0-100
        #[br(if(length == 2))]
        scope: Option<u8>,
    },

    /// Maoyan Doorbell Camera Event
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0011))]
    #[br(assert(length == 1))]
    DoorbellCameraEvent(DoorbellCameraEvent),

    /// Weighing Event
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0012))]
    #[br(assert(length == 3))]
    WeighingEvent {
        /// Value (g)
        weight: u16,
        /// 0 = Current Weight, 1 = Reduced Weight, 2 = Increased Weight
        weighing_type: WeighingEventType,
    },

    /// Button
    ///
    /// - **Time Interval:** 0
    /// - **Change:** 0
    #[br(pre_assert(id == 0x0012))]
    #[br(assert(length == 3))]
    ButtonEvent {
        /// Button Number (0-9)
        index: u16,
        /// Single click (0x00), double click (0x01), long press (0x02), triple click (0x03)
        event_type: ButtonEventType,
    },

    // Common Attributes (0x1002 - 0x1FFF)
    /// Sleep (on/off)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x1002))]
    #[br(assert(length == 1))]
    Sleep(SleepState),

    /// RSSI
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 10
    #[br(pre_assert(id == 0x1003))]
    #[br(assert(length == 1))]
    Rssi(u8),

    /// Temperature (degrees decicelsius)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x1004))]
    #[br(assert(length == 2))]
    Temperature(i16),

    /// Power (on/off) and Temperature (°C)
    #[br(pre_assert(id == 0x1005))]
    #[br(assert(length == 2))]
    PowerAndTemperature {
        /// Power (on/off)
        power: u8,
        /// Temperature (°C)
        temperature: u8,
    },

    /// Humidity (‰)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    /// - **Range:** 0-1000
    #[br(pre_assert(id == 0x1006))]
    #[br(assert(length == 2))]
    Humidity(u16),

    /// Illuminance (lx)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    /// - **Range:** 0-120000
    #[br(pre_assert(id == 0x1007))]
    #[br(assert(length == 3))]
    Illuminance(U24),

    /// Soil Moisture (%)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    /// - **Range:** 0-100
    #[br(pre_assert(id == 0x1008))]
    #[br(assert(length == 1))]
    Moisture(u8),

    /// Soil Electrical Conductivity (µS/cm)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    /// - **Range:** 0-5000
    #[br(pre_assert(id == 0x1009))]
    #[br(assert(length == 2))]
    Conductivity(u16),

    /// Battery Power (%)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    /// - **Range:** 0-100
    #[br(pre_assert(id == 0x100A))]
    #[br(assert(length == 1))]
    BatteryPower(u8),

    /// Lock Sensor
    ///
    /// - **Time Interval:** 60 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x100E))]
    #[br(assert(length == 1))]
    Lock(LockState),

    /// Door
    ///
    /// - **Time Interval:** 60 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x100F))]
    #[br(assert(length == 1))]
    Door(DoorState),

    /// Formaldehyde Concentration (0.01 * mg/m³)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x1010))]
    #[br(assert(length == 2))]
    FormaldehydeConcentration(u16),

    /// Binding
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x1011))]
    #[br(assert(length == 2))]
    Binding(BindingState),

    /// Switch state (on/off)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x1012))]
    #[br(assert(length == 1))]
    Switch(SwitchState),

    /// Remaining Consumable Supplies (%)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    /// - **Range:** 0-100
    #[br(pre_assert(id == 0x1013))]
    #[br(assert(length == 1))]
    RemainingSupplies(u8),

    /// Water Immersion (yes/no)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x1014))]
    #[br(assert(length == 1))]
    WaterImmersion(WaterImmersionState),

    /// Smoke Detection (on/off)
    ///
    /// - **Time Interval:** 1 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x1015))]
    #[br(assert(length == 1))]
    SmokeDetection(SmokeDetectionState),

    /// Gas Leakage Detection
    ///
    /// - **Time Interval:** 1 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x1016))]
    #[br(assert(length == 1))]
    GasLeakageDetection(GasLeakageDetectionState),

    /// Time without Motion (s)
    ///
    /// - **Time Interval:** 1 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x1017))]
    #[br(assert(length == 4))]
    TimeWithoutMotion(u32),

    /// Light Intensity
    ///
    /// - **Time Interval:** 1 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x1018))]
    #[br(assert(length == 2))]
    LightIntensity(LightIntensityState),

    /// Door Sensor
    ///
    /// - **Time Interval:** 1 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x1019))]
    #[br(assert(length == 2))]
    DoorSensor(DoorSensorState),

    /// Weight (g)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x101A))]
    #[br(assert(length == 2))]
    Weight(u16),

    /// Movement Detection in Timeframe
    ///
    /// - **Time Interval:** 1 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x101B))]
    #[br(assert(length == 1))]
    MovementDetection(MovementDetectionState),

    /// Smart Pillow
    ///
    /// - **Time Interval:** 60 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x101C))]
    #[br(assert(length == 1))]
    SmartPillow(SmartPillowState),

    /// Formaldehyde Concentration (0.001 * mg/m³)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x101C))]
    #[br(assert(length == 2))]
    FormaldehydeConcentrationNew(u16),

    // Manufacturer Custom Attributes (0x2000 - 0x2FFF)
    /// Body Temperature (measures in Seconds)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x2000))]
    #[br(assert(length == 5))]
    BodyTemperature {
        /// Skin Temperature (0.01 °C)
        skin_temperature: u16,
        /// PCB Temperature (0.01 °C)
        pcb_temperature: u16,
        /// Battery Power (%)
        battery_power: u8,
    },

    /// Xiaomi Mi Band (Huami)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x2001))]
    #[br(assert(length == 4))]
    MiBand {
        /// Current Step Count
        step_count: u16,
        /// Sleep (0x01 = Fall Asleep, 0x02 = Wake Up)
        sleep: MiBandSleepState,
        /// RSSI (Current Signal Strength)
        rssi: u8,
    },

    /// Vacuum Cleaner (ROIDMI)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x2002))]
    #[br(assert(length == 2))]
    RoidmiVacuumCleaner {
        /// Vacuum State
        status: RoidmiVacuumCleanerState,
        /// Current Standard Gear
        gear: u8,
    },

    /// Black Plus Bracelet (As One)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x2003))]
    #[br(assert(length == 4))]
    BlackPlusBracelet {
        /// Steps per day
        step_count: u16,
        /// Last Heart Rate
        heart_rate: u8,
        /// Current Activity Status
        state: u8,
    },

    // Manufacturer Custom Events (0x3000 - 0x3FFF)
    /// Flower And Grass Detector Event (Flowers and Plants)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 1
    #[br(pre_assert(id == 0x3000))]
    #[br(assert(length == 1))]
    FlowerAndGrassDetectorEvent(FlowerAndGrassDetectorEvent),

    /// Sensor Location (Quingping)
    ///
    /// - **Time Interval:** 600 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x3001))]
    #[br(assert(length == 1))]
    QuingpingSensorLocationEvent(QuingpingSensorLocationEvent),

    /// Pomodoro Event (Quingping)
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x3002))]
    #[br(assert(length == 1))]
    QuingpingPomodoroEvent(QuingpingPomodoroEvent),

    /// Xiaobel Toothbrush Incident (Qinghe Xiaobel)
    ///
    /// - **Time Interval:** 0 s
    /// - **Change:** 0
    #[br(pre_assert(id == 0x3003))]
    #[br(assert(length == 5 || length == 6))]
    XiaobelToothbrushEvent {
        /// 0 = Start Brushing, 1 = end brushing
        event_type: ToothbrushEventType,
        /// UTC time
        timestamp: u32,
        /// Tooth Brushing Score (optional, range 0-100)
        #[br(if(length == 6))]
        score: Option<u8>,
    },

    /// Unknown Payload
    Unknown(#[br(count = usize::from(length))] Vec<u8>),
}

/// MiBeacon Object
#[allow(dead_code)]
#[binread]
#[br(little)]
#[derive(Debug)]
pub struct MiBeaconObject {
    /// Object ID
    pub id: u16,
    /// Object Length
    pub length: u8,
    /// Object Payload
    #[br(args(id, length))]
    pub payload: MiBeaconObjectPayload,
}

/// Base I/O capability types can be divided into two categories: Input and Output.
///
/// *Note:* This field is used only in high security level access.
#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct MiBeaconBaseIoCapabilities {
    // Basic Input Capabilities
    /// Whether the device can input 6 digits
    pub can_input_6_digits: bool,
    /// Whether the device can input 6 characters
    pub can_input_6_characters: bool,
    /// Whether the device can read NFC tags
    pub can_read_nfc_tags: bool,
    /// Whether the device can recognize QR codes
    pub can_read_qr_codes: bool,
    // Basic Output Capabilities
    /// Whether the device can output 6 digits
    pub can_output_6_digits: bool,
    /// Whether the device can output 6 characters
    pub can_output_6_characters: bool,
    /// Whether the device can generate NFC tags
    pub can_generate_nfc_tags: bool,
    /// Whether the device can generate QR codes
    pub can_generate_qr_code: bool,
}

/// I/O Capabilities
#[allow(dead_code)]
#[binread]
#[br(little)]
#[derive(Debug)]
pub struct MiBeaconIoCapabilities {
    /// Base I/O Capabilities
    pub base_io_capabilities: MiBeaconBaseIoCapabilities,
    /// Reserved
    #[allow(dead_code)]
    reserved: u8,
}

/// Capabilities
#[bitfield]
#[derive(BinRead, Debug)]
#[br(map = Self::from_bytes)]
pub struct MiBeaconCapabilities {
    /// Connectable (temporarily unused)
    pub connectable: bool,
    /// Centralable (temporarily unused)
    pub centralable: bool,
    /// Encryptable (temporarily unused)
    pub encryptable: bool,
    /// The Bond Ability field indicates how to determine which device to bond to when there are multiple identical devices nearby.
    ///
    /// 0: No binding: APP selects pairing, RSSI meets the pairing requirements;
    /// 1: Pre-binding: The device confirms pairing, which requires scanning first. The connection is established after the device sends a confirmation packet (solicited in Frame Control);
    /// 2: Post-binding: Directly connect after scanning, and the device confirms by vibration or other means;
    /// 3: Combo: Only chips that support Combo are available (this binding method needs to be selected in the Xiaomi IoT Developer Platform and is consistent with this)
    pub bond_ability: B2,
    /// Whether the message includes the I/O Capabilities field
    pub io: bool,
    /// Reserved
    #[allow(dead_code)]
    reserved: B2,
}

/// Service Advertisement in the MiBeacon format.
#[allow(dead_code)]
#[binread]
#[br(little)]
#[derive(Debug)]
pub struct MiBeaconServiceAdvertisement {
    /// Frame Control Header
    #[br(big)]
    pub frame_control: FrameControl,
    /// Xiaomi Device ID
    pub device_id: u16,
    /// Packet ID
    pub packet_id: u8,
    /// MAC Address (only included if [FrameControl::mac_included()] is `true`)
    #[br(if(frame_control.mac_included()))]
    pub mac_address: Option<MacAddress>,
    /// Capabilities (only included if [FrameControl::capabilities_included()] is `true`)
    #[br(if(frame_control.capabilities_included()))]
    pub capabilities: Option<MiBeaconCapabilities>,
    /// I/O Capabilities
    #[br(if(capabilities.as_ref().is_some_and(|cap| cap.io())))]
    pub io_capabilities: Option<MiBeaconIoCapabilities>,
    /// Objects (only included if [FrameControl::objects_included()] is `true`)
    #[br(if(frame_control.objects_included()))]
    #[br(parse_with = until_eof)]
    pub objects: Vec<MiBeaconObject>,
}

impl MiBeaconObjectPayload {
    /// Map this [`MiBeaconObjectPayload`] to one or more [`Sensor Value`] objects.
    fn to_sensor_events(&self) -> Vec<SensorEvent> {
        match &self {
            MiBeaconObjectPayload::Temperature(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::Temperature,
                    value: f64::from(*value) / 10.0,
                    unit: UnitOfMeasurement::DegreesCelsius,
                }]
            }
            MiBeaconObjectPayload::Humidity(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::Humidity,
                    value: f64::from(*value) / 10.0,
                    unit: UnitOfMeasurement::Percent,
                }]
            }
            MiBeaconObjectPayload::Illuminance(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::Illuminance,
                    value: f64::from(value.as_u32()),
                    unit: UnitOfMeasurement::Lux,
                }]
            }
            MiBeaconObjectPayload::Moisture(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::Moisture,
                    value: f64::from(*value),
                    unit: UnitOfMeasurement::Percent,
                }]
            }
            MiBeaconObjectPayload::Conductivity(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::Conductivity,
                    value: f64::from(*value),
                    unit: UnitOfMeasurement::MicrosiemensPerCentimeter,
                }]
            }
            MiBeaconObjectPayload::FormaldehydeConcentration(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::FormaldehydeConcentration,
                    value: f64::from(*value) / 100.0,
                    unit: UnitOfMeasurement::MilligramPerCubicMeter,
                }]
            }
            MiBeaconObjectPayload::FormaldehydeConcentrationNew(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::FormaldehydeConcentration,
                    value: f64::from(*value) / 1000.0,
                    unit: UnitOfMeasurement::MilligramPerCubicMeter,
                }]
            }
            MiBeaconObjectPayload::BatteryPower(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::BatteryPower,
                    value: f64::from(*value),
                    unit: UnitOfMeasurement::Percent,
                }]
            }
            MiBeaconObjectPayload::RemainingSupplies(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::RemainingSupplies,
                    value: f64::from(*value),
                    unit: UnitOfMeasurement::Percent,
                }]
            }
            MiBeaconObjectPayload::Weight(value) => {
                vec![SensorEvent::NumericMeasurement {
                    measurement_type: NumericMeasurementType::Weight,
                    value: f64::from(*value),
                    unit: UnitOfMeasurement::Kilogram,
                }]
            }
            _ => {
                warn!("Ignoring unhandled MiBeacon object payload: {:?}", &self);
                vec![]
            }
        }
    }
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

    /// Yields the object paylads for the service advertisement.
    pub fn iter_sensor_events(&self) -> impl Iterator<Item = SensorEvent> + '_ {
        self.objects
            .iter()
            .flat_map(|obj| obj.payload.to_sensor_events().into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::MiBeaconServiceAdvertisement;

    const HHCCJCY01_TEMPERATURE_READING: [u8; 17] = [
        0x71, 0x20, 0x98, 0x00, 0xB1, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x0D, 0x04, 0x10, 0x02,
        0xEC, 0x00,
    ];
    const HHCCJCY01_ILLUMINANCE_READING: [u8; 18] = [
        0x71, 0x20, 0x98, 0x00, 0x36, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x0D, 0x07, 0x10, 0x03,
        0x73, 0x00, 0x00,
    ];
    const HHCCJCY01_CONDUCTIVITY_READING: [u8; 17] = [
        0x71, 0x20, 0x98, 0x00, 0xBC, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x0D, 0x09, 0x10, 0x02,
        0x2E, 0x00,
    ];
    const HHCCJCY01_MOISTURE_READING: [u8; 16] = [
        0x71, 0x20, 0x98, 0x00, 0xD7, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x0D, 0x08, 0x10, 0x01,
        0x17,
    ];

    #[test]
    fn parse_hhccjcy01_temperature_reading() {
        let message = MiBeaconServiceAdvertisement::from_slice(&HHCCJCY01_TEMPERATURE_READING);
        assert!(message.is_ok())
    }

    #[test]
    fn parse_hhccjcy01_illuminance_reading() {
        let message = MiBeaconServiceAdvertisement::from_slice(&HHCCJCY01_ILLUMINANCE_READING);
        assert!(message.is_ok())
    }

    #[test]
    fn parse_hhccjcy01_conductivity_reading() {
        let message = MiBeaconServiceAdvertisement::from_slice(&HHCCJCY01_CONDUCTIVITY_READING);
        assert!(message.is_ok())
    }

    #[test]
    fn parse_hhccjcy01_moisture_reading() {
        let message = MiBeaconServiceAdvertisement::from_slice(&HHCCJCY01_MOISTURE_READING);
        assert!(message.is_ok())
    }
}
