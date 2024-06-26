// Copyright (c) 2024 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Library for parsing Xiamo BLE data structures.

#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::all)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::must_use_candidate)]
#![cfg_attr(not(debug_assertions), warn(clippy::used_underscore_binding))]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]

pub mod device;
pub mod hhccjcy10;
pub mod mibeacon;
pub mod miscale;
pub mod sensor;
pub mod service;
mod util;

pub use service::parse_service_advertisement;
pub use util::ParseError;
