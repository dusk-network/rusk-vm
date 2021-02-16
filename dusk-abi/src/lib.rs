// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! #Dusk ABI
//!
//! ABI functionality for communicating with the host
#![warn(missing_docs)]
#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

mod types;
pub use types::contract::{ContractId, ContractState};
pub use types::query::Query;
pub use types::return_value::ReturnValue;
pub use types::transaction::Transaction;

#[doc(hidden)]
mod canon_to_vec;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        // re-export WeeAlloc
        pub use wee_alloc::WeeAlloc;

        #[doc(hidden)]
        pub mod hosted;
        pub use hosted::*;
    } else {
        #[doc(hidden)]
        pub mod host;
        pub use host::*;
    }
}
