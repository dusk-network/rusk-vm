// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::VMError;

use parity_wasm::elements;
use pwasm_utils::rules::{InstructionType, Metering};
use wasmparser::Validator;
use std::fs;
use std::path::Path;
use serde::Deserialize;
#[cfg(not(features = "std"))]
use std::collections::BTreeMap as Map;
#[cfg(features = "std")]
use std::collections::HashMap as Map;
pub use dusk_abi::{ContractId, ContractState};
use std::str::FromStr;

#[derive(Debug)]
pub enum InstrumentationError {
    GasMeteringInjection,
    StackHeightInjection,
    MultipleTables,
    MaxTableSize,
    InvalidByteCode,
    InvalidConfigPath, // todo
    InvalidConfig, // todo
    InvalidInstructionType,
}

#[derive(Deserialize, Clone)]
pub(crate) struct ModuleConfig {
    has_grow_cost: bool,
    has_forbidden_floats: bool,
    has_metering: bool,
    has_table_size_limit: bool,
    max_stack_height: u32,
    max_table_size: u32,
    regular_op_cost: u32,
    pub per_type_op_cost: Map<String, u32>,
    grow_mem_cost: u32,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        ModuleConfig::new()
    }
}

impl ModuleConfig {
    const CONFIG_FILE: &'static str = "config.toml";

    pub fn new() -> Self {
        Self {
            has_grow_cost: true,
            has_forbidden_floats: true,
            has_metering: true,
            has_table_size_limit: true,
            max_stack_height: 65536,
            max_table_size: 16384,
            regular_op_cost: 1,
            per_type_op_cost: Map::new(),
            grow_mem_cost: 1
        }
    }

    pub fn with_file(file_path: Option<String>) -> Result<Self, VMError> {
        let path_string = file_path.unwrap_or_else(||ModuleConfig::CONFIG_FILE.to_string());
        let config_file_path = Path::new(&path_string);
        let config_string = fs::read_to_string(config_file_path)
            .map_err(VMError::ConfigurationFileError)?;
        let config: ModuleConfig = toml::from_str(&config_string)
            .map_err(VMError::ConfigurationError)?;
        Ok(config)
    }

    pub fn validate_wasm(wasm_code: impl AsRef<[u8]>) -> Result<(), VMError> {
        let mut validator = Validator::new();
        validator
            .validate_all(wasm_code.as_ref())
            .map_err(|e| VMError::WASMError(failure::Error::from(e)))
    }

    pub fn apply(&self, code: &[u8]) -> Result<Vec<u8>, InstrumentationError> {
        let mut module: parity_wasm::elements::Module =
            elements::deserialize_buffer(code)
                .or(Err(InstrumentationError::InvalidByteCode))?;

        let mut instr_type_map: Map<InstructionType, Metering> = Map::new();
        for (instr_type, value) in self.per_type_op_cost.iter() {
            instr_type_map.insert(
                InstructionType::from_str(instr_type)
                    .or(Err(InstrumentationError::InvalidInstructionType))?,
                Metering::Fixed(*value),
            );
        }

        let schedule = crate::Schedule::default();
        let mut ruleset = pwasm_utils::rules::Set::new(
            schedule.regular_op_cost as u32,
            instr_type_map,
        );

        if self.has_grow_cost {
            ruleset = ruleset.with_grow_cost(schedule.grow_mem_cost as u32);
        }

        if self.has_forbidden_floats {
            ruleset = ruleset.with_forbidden_floats();
        }

        if self.has_metering {
            module = pwasm_utils::inject_gas_counter(module, &ruleset, "env")
                .or(Err(InstrumentationError::GasMeteringInjection))?;

            module = pwasm_utils::stack_height::inject_limiter(
                module,
                schedule.max_stack_height,
            )
            .or(Err(InstrumentationError::StackHeightInjection))?;
        }

        if self.has_table_size_limit {
            if let Some(table_section) = module.table_section() {
                // In Wasm MVP spec, there may be at most one table declared.
                // Double check this explicitly just in case the
                // Wasm version changes.
                if table_section.entries().len() > 1 {
                    return Err(InstrumentationError::MultipleTables);
                }

                if let Some(table_type) = table_section.entries().first() {
                    // Check the table's initial size as there is no instruction
                    // or environment function capable of
                    // growing the table.
                    if table_type.limits().initial() > schedule.max_table_size {
                        return Err(InstrumentationError::MaxTableSize);
                    }
                }
            }
        }

        let code_bytes = module
            .to_bytes()
            .or(Err(InstrumentationError::InvalidByteCode))?;

        ModuleConfig::validate_wasm(&code_bytes)
            .or(Err(InstrumentationError::InvalidByteCode))?;

        Ok(code_bytes)
    }
}
