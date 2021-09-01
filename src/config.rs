// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use pwasm_utils::rules::{InstructionType, Metering};
use serde::Deserialize;
#[cfg(not(features = "std"))]
use std::collections::BTreeMap as Map;
#[cfg(features = "std")]
use std::collections::HashMap as Map;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use wasmi_validation::{validate_module, PlainValidator};

pub use dusk_abi::{ContractId, ContractState};

#[derive(Debug)]
pub enum InstrumentationError {
    GasMeteringInjection,
    StackHeightInjection,
    MultipleTables,
    MaxTableSize,
    InvalidByteCode,
    InvalidConfigPath,
    InvalidConfig,
    InvalidInstructionType,
}

#[derive(Clone, Deserialize)]
pub(crate) struct Config {
    regular_op_cost: u32,
    per_type_op_cost: Map<String, u32>,
    grow_mem_cost: u32,
    max_stack_height: u32,
    max_table_size: u32,
    has_forbidden_floats: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            regular_op_cost: 1,
            grow_mem_cost: 1,
            max_stack_height: 65536,
            max_table_size: 16384,
            has_forbidden_floats: true,
            per_type_op_cost: Map::new(),
        }
    }
}

impl Config {
    pub fn new<P: AsRef<Path>>(
        config_file: P,
    ) -> Result<Self, InstrumentationError> {
        let config_string = fs::read_to_string(&config_file)
            .or(Err(InstrumentationError::InvalidConfigPath))?;

        let config = toml::from_str(&config_string)
            .or(Err(InstrumentationError::InvalidConfig))?;
        Ok(config)
    }

    pub fn apply(&self, code: &[u8]) -> Result<Vec<u8>, InstrumentationError> {
        let mut module = parity_wasm::elements::deserialize_buffer(code)
            .or(Err(InstrumentationError::InvalidByteCode))?;

        let mut instr_type_map: Map<InstructionType, Metering> = Map::new();
        for (instr_type, value) in self.per_type_op_cost.iter() {
            instr_type_map.insert(
                InstructionType::from_str(instr_type)
                    .or(Err(InstrumentationError::InvalidInstructionType))?,
                Metering::Fixed(*value),
            );
        }

        let mut ruleset =
            pwasm_utils::rules::Set::new(self.regular_op_cost, instr_type_map)
                .with_grow_cost(self.grow_mem_cost);

        if self.has_forbidden_floats {
            ruleset = ruleset.with_forbidden_floats();
        }

        module = pwasm_utils::inject_gas_counter(module, &ruleset, "env")
            .or(Err(InstrumentationError::GasMeteringInjection))?;
        module = pwasm_utils::stack_height::inject_limiter(
            module,
            self.max_stack_height,
        )
        .or(Err(InstrumentationError::StackHeightInjection))?;

        if let Some(table_section) = module.table_section() {
            // In Wasm MVP spec, there may be at most one table declared.
            // Double check this explicitly just in case the Wasm version
            // changes.
            if table_section.entries().len() > 1 {
                return Err(InstrumentationError::MultipleTables);
            }

            if let Some(table_type) = table_section.entries().first() {
                // Check the table's initial size as there is no instruction or
                // environment function capable of growing the table.
                if table_type.limits().initial() > self.max_table_size {
                    return Err(InstrumentationError::MaxTableSize);
                }
            }
        }

        validate_module::<PlainValidator>(&&module)
            .or(Err(InstrumentationError::InvalidByteCode))?;
        Ok(module
            .to_bytes()
            .or(Err(InstrumentationError::InvalidByteCode))?)
    }
}
