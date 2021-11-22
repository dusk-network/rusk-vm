// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::VMError;

use parity_wasm::elements;
use wasmparser::Validator;
use std::fs;
use std::path::Path;
use serde::Deserialize;

pub use dusk_abi::{ContractId, ContractState};

#[derive(Debug)]
pub enum InstrumentationError {
    GasMeteringInjection,
    StackHeightInjection,
    MultipleTables,
    MaxTableSize,
    InvalidByteCode,
}

#[derive(Default, Deserialize)]
pub(crate) struct ModuleConfig {
    has_grow_cost: bool,
    has_forbidden_floats: bool,
    has_metering: bool,
    has_table_size_limit: bool,
}

impl ModuleConfig {
    const CONFIG_FILE: &'static Path = Path::new("config.toml");

    pub fn new() -> Self {
        Self {
            has_grow_cost: false,
            has_forbidden_floats: false,
            has_metering: false,
            has_table_size_limit: false,
        }
    }

    pub fn with_file() -> Result<Self, VMError> {
        let config_string = fs::read_to_string(ModuleConfig::CONFIG_FILE)
            .or(Err(VMError::ConfigurationError("could not read configuration file: ".to_string() + ModuleConfig::CONFIG_FILE.to_str().unwrap_or_default())))?;

        let config = toml::from_str(&config_string)
            .or(Err(VMError::ConfigurationError("error when parsing configuration file".to_string())))?;
        Ok(config)
    }

    pub fn with_grow_cost(&mut self) -> &mut Self {
        self.has_grow_cost = true;
        self
    }

    pub fn with_forbidden_floats(&mut self) -> &mut Self {
        self.has_forbidden_floats = true;
        self
    }

    pub fn with_metering(&mut self) -> &mut Self {
        self.has_metering = true;
        self
    }

    pub fn with_table_size_limit(&mut self) -> &mut Self {
        self.has_table_size_limit = true;
        self
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

        let schedule = crate::Schedule::default();
        let mut ruleset = pwasm_utils::rules::Set::new(
            schedule.regular_op_cost as u32,
            Default::default(),
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
