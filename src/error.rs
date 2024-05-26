// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Errors used in multiple modules within this crate.

use thiserror::Error;

/// Raised when error occurs while parsing a data structure.
#[derive(Debug, Error)]
#[error("Parsing failed: {source}")]
pub struct ParseError {
    // The underlying `binrw` error.
    #[from]
    source: binrw::Error,
}
