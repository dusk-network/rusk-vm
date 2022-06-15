// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Configuration of the virtual machine.

use crate::Gas;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub static DEFAULT_CONFIG: Config = Config::new();

/// Parameters used to configure the virtual machine.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Config {
    /// Gas cost of a regular operation
    pub regular_op_cost: Gas,

    /// Maximum allowed size of a declared table
    pub max_table_size: u32,

    /// Maximum number of memory pages
    pub max_memory_pages: u32,

    /// Is metering on
    pub has_metering: bool,

    /// Cost per instruction type
    pub op_costs: OpCosts,
}

impl Config {
    /// Creates a new [`Config`] with default values
    pub const fn new() -> Self {
        Self {
            regular_op_cost: 1,
            max_table_size: 16384,
            max_memory_pages: 16384,
            has_metering: true,
            op_costs: OpCosts::new(),
        }
    }
}

pub fn config_hash(config: &Config) -> u64 {
    let mut hasher = DefaultHasher::new();
    config.hash(&mut hasher);
    hasher.finish()
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Costs of particular operations
#[allow(missing_docs)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OpCosts {
    pub bit: Gas,
    pub add: Gas,
    pub mul: Gas,
    pub div: Gas,
    pub load: Gas,
    pub store: Gas,
    pub const_decl: Gas,
    pub local: Gas,
    pub global: Gas,
    pub flow: Gas,
    pub integer_comp: Gas,
    pub float_comp: Gas,
    pub float: Gas,
    pub conversion: Gas,
    pub float_conversion: Gas,
    pub reinterpret: Gas,
    pub unreachable: Gas,
    pub nop: Gas,
    pub current_mem: Gas,
    pub grow_mem: Gas,
}

impl OpCosts {
    /// Creates a new [`OpCosts`] with default values
    pub const fn new() -> Self {
        Self {
            bit: 1,
            add: 1,
            mul: 1,
            div: 1,
            load: 1,
            store: 1,
            const_decl: 1,
            local: 1,
            global: 1,
            flow: 1,
            integer_comp: 1,
            float_comp: 1,
            float: 1,
            conversion: 1,
            float_conversion: 1,
            reinterpret: 1,
            unreachable: 1,
            nop: 1,
            current_mem: 1,
            grow_mem: 1,
        }
    }
}

impl Default for OpCosts {
    fn default() -> Self {
        Self::new()
    }
}
