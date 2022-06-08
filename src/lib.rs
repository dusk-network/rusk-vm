// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! #Rusk-VM
//!
//! The main engine for executing WASM on the network state
#![warn(missing_docs)]

use std::collections::HashMap;

mod call_context;
mod compiler;
mod compiler_config;
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

pub use contract::{Contract, ContractId, ContractRef};
pub use error::VMError;
pub use gas::{Gas, GasMeter};
pub use state::NetworkState;

/// Definition of the cost schedule and other parameterizations for wasm vm.
#[derive(Clone, PartialEq, Eq)]
pub struct Schedule {
    /// Version of the schedule.
    pub version: u32,

    /// Gas cost of a regular operation.
    pub regular_op_cost: Gas,

    /// Gas cost of a growing memory by single page.
    pub grow_mem_cost: Gas,

    /// Maximum allowed stack height.
    ///
    /// See `<https://wiki.parity.io/WebAssembly-StackHeight>` to find out
    /// how the stack frame cost is calculated.
    pub max_stack_height: u32,

    /// Maximum allowed size of a declared table.
    pub max_table_size: u32,

    /// Maximum number of memory pages.
    pub max_memory_pages: u32,

    /// Floats are forbidden
    pub has_forbidden_floats: bool,

    /// Cost of memory growth
    pub has_grow_cost: bool,

    /// Is metering on
    pub has_metering: bool,

    /// Is table size limit on
    pub has_table_size_limit: bool,

    /// Op cost bit
    pub per_type_op_cost: HashMap<String, u32>,
}

impl Default for Schedule {
    fn default() -> Schedule {
        let per_type_op_cost: HashMap<String, u32> = [
            ("bit", 1),
            ("add", 1),
            ("mul", 1),
            ("div", 1),
            ("load", 1),
            ("store", 1),
            ("const", 1),
            ("local", 1),
            ("global", 1),
            ("flow", 1),
            ("integer_comp", 1),
            ("float_comp", 1),
            ("float", 1),
            ("conversion", 1),
            ("float_conversion", 1),
            ("reinterpret", 1),
            ("unreachable", 1),
            ("nop", 1),
            ("current_mem", 1),
            ("grow_mem", 1),
        ]
        .iter()
        .map(|(s, c)| (s.to_string(), *c))
        .collect();
        Schedule {
            version: 0,
            regular_op_cost: 1,
            grow_mem_cost: 1,
            max_stack_height: 65536,
            max_table_size: 16384,
            max_memory_pages: 16384,
            has_forbidden_floats: true,
            has_grow_cost: true,
            has_metering: true,
            has_table_size_limit: true,
            per_type_op_cost,
        }
    }
}

impl Schedule {
    /// Create schedule with version
    pub fn with_version(version: u32) -> Self {
        Self {
            version,
            ..Self::default()
        }
    }
}
