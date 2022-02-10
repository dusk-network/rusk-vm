// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::collections::HashMap;

use crate::compiler::WasmerCompiler;
use crate::{Schedule, VMError};

use cached::cached_key_result;
use cached::TimedSizedCache;
use canonical::Store;
use dusk_abi::HostModule;
use parity_wasm::elements;
use pwasm_utils::rules::{InstructionType, Metering};
use std::collections::BTreeMap as Map;
use std::str::FromStr;
use tracing::trace;
use wasmer::Module;
use wasmparser::Validator;

pub use dusk_abi::{ContractId, ContractState};

type BoxedHostModule = Box<dyn HostModule + Send + Sync>;

/// Compiles a module with the specified bytecode or retrieves it from
pub fn compile_module(
    bytecode: &[u8],
    module_config: &ModuleConfig,
) -> Result<Module, VMError> {
    get_or_create_module(bytecode, module_config)
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct ModuleCacheKey {
    hash: [u8; 32],
    version: u32,
}

// The `cached` crate is used to generate a cache for calls to this function.
// This is done to prevent modules from being compiled over and over again,
// saving some CPU cycles.
cached_key_result! {
    COMPUTE: TimedSizedCache<ModuleCacheKey, Module>
        = TimedSizedCache::with_size_and_lifespan(2048, 86400);
    Key = {
        ModuleCacheKey{ hash: Store::hash(bytecode), version: module_config.version }
    };

    fn get_or_create_module(bytecode: &[u8], module_config: &ModuleConfig) -> Result<Module, VMError> = {
        trace!("Compiling module");
        WasmerCompiler::create_module(bytecode, module_config)
    }
}

/// A cheaply cloneable store for host modules.
#[derive(Default)]
pub struct HostModules(HashMap<ContractId, BoxedHostModule>);

impl HostModules {
    /// Insert a new module into the store.
    pub fn insert<M>(&mut self, module: M)
    where
        M: 'static + HostModule + Sync + Send,
    {
        self.0.insert(module.module_id(), Box::new(module));
    }

    // Get a reference to a particular module from the store.
    pub fn get_module<'a>(
        &'a self,
        id: &'a ContractId,
    ) -> Option<&'a BoxedHostModule> {
        self.0.get(id)
    }
}

#[derive(Debug)]
pub enum InstrumentationError {
    GasMeteringInjection,
    StackHeightInjection,
    MultipleTables,
    MaxTableSize,
    InvalidByteCode,
    InvalidInstructionType,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ModuleConfig {
    pub version: u32,
    pub has_grow_cost: bool,
    pub has_forbidden_floats: bool,
    pub has_metering: bool,
    pub has_table_size_limit: bool,
    pub max_stack_height: u32,
    pub max_table_size: u32,
    pub regular_op_cost: u32,
    pub per_type_op_cost: Map<String, u32>,
    pub grow_mem_cost: u32,
    pub max_memory_pages: u32,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        ModuleConfig::new()
    }
}

impl ModuleConfig {
    pub fn new() -> Self {
        Self {
            version: 0,
            has_grow_cost: true,
            has_forbidden_floats: true,
            has_metering: true,
            has_table_size_limit: true,
            max_stack_height: 65536,
            max_table_size: 16384,
            regular_op_cost: 1,
            per_type_op_cost: Map::new(),
            grow_mem_cost: 1,
            max_memory_pages: 16384,
        }
    }

    pub fn from_schedule(schedule: &Schedule) -> Self {
        let mut config = Self::new();
        config.version = schedule.version;
        config.regular_op_cost = schedule.regular_op_cost as u32;
        config.grow_mem_cost = schedule.grow_mem_cost as u32;
        config.max_memory_pages = schedule.max_memory_pages as u32;
        config.max_stack_height = schedule.max_stack_height;
        config.max_table_size = schedule.max_table_size;
        config.has_forbidden_floats = schedule.has_forbidden_floats;
        config.has_grow_cost = schedule.has_grow_cost;
        config.has_metering = schedule.has_metering;
        config.has_table_size_limit = schedule.has_table_size_limit;
        config.per_type_op_cost = schedule
            .per_type_op_cost
            .iter()
            .map(|(s, c)| (s.to_string(), *c))
            .collect();
        config
    }

    pub(crate) fn validate_wasm(
        wasm_code: impl AsRef<[u8]>,
    ) -> Result<(), VMError> {
        let mut validator = Validator::new();
        validator
            .validate_all(wasm_code.as_ref())
            .map_err(|e| VMError::WASMError(failure::Error::from(e)))
    }

    pub(crate) fn apply(
        &self,
        code: &[u8],
    ) -> Result<Vec<u8>, InstrumentationError> {
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

        let mut ruleset =
            pwasm_utils::rules::Set::new(self.regular_op_cost, instr_type_map);

        if self.has_grow_cost {
            ruleset = ruleset.with_grow_cost(self.grow_mem_cost as u32);
        }

        if self.has_forbidden_floats {
            ruleset = ruleset.with_forbidden_floats();
        }

        if self.has_metering {
            module = pwasm_utils::inject_gas_counter(module, &ruleset, "env")
                .or(Err(InstrumentationError::GasMeteringInjection))?;

            module = pwasm_utils::stack_height::inject_limiter(
                module,
                self.max_stack_height,
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
                    if table_type.limits().initial() > self.max_table_size {
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
