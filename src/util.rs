// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Utilities in multiple modules within this crate.

use binrw::BinRead;
use core::fmt;
use thiserror::Error;

/// Raised when error occurs while parsing a data structure.
#[derive(Debug, Error)]
#[error("Parsing failed: {source}")]
pub struct ParseError {
    // The underlying `binrw` error.
    #[from]
    source: binrw::Error,
}

/// Unsigned integer consisting of 3 bytes.
#[derive(BinRead)]
#[br(little)]
pub struct U24([u8; 3]);

impl U24 {
    /// Get the number as [`u32`].
    pub fn as_u32(&self) -> u32 {
        u32::from(self.0[2]) << 16 | u32::from(self.0[1]) << 8 | u32::from(self.0[0])
    }
}

impl fmt::Debug for U24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}
