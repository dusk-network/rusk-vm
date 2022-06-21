// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! #Rusk-VM
//!
//! The main engine for executing WASM on the network state
#![warn(missing_docs)]

mod call_context;
mod compiler;
mod compiler_config;
mod config;
mod contract;
mod env;
mod error;
mod gas;
mod memory;
mod modules;
mod ops;
mod resolver;
mod state;

pub use rusk_uplink;
pub use state::persist::NetworkStateId;

pub use config::{Config, OpCosts};
pub use contract::{Contract, ContractId};
pub use error::VMError;
pub use gas::{Gas, GasMeter};
pub use state::NetworkState;
