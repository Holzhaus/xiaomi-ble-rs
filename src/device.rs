// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Information about BLE devices.

/// The BLE device type.
#[derive(Debug)]
pub struct DeviceType {
    /// Device Name
    pub name: &'static str,
    /// Model number
    pub model: &'static str,
    /// Device Manufacturer
    pub manufacturer: &'static str,
}
