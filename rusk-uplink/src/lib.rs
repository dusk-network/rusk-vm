// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]
#![feature(lang_items)]

/// Store backend over FFI
#[cfg(not(feature = "host"))]
mod ffi_store;
#[cfg(not(feature = "host"))]
pub use ffi_store::*;

pub mod definitions;
pub use definitions::*;

pub mod hosted;
pub use hosted::*;

pub mod helpers;
pub use helpers::*;

pub mod bufwriter;
pub use bufwriter::*;

pub mod debug;
pub use debug::*;

pub mod framing;
pub use framing::*;

#[cfg(not(feature = "host"))]
mod items;
