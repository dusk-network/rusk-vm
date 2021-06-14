// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use parity_wasm::elements;
use wasmi_validation::{validate_module, PlainValidator};

pub use dusk_abi::{ContractId, ContractState};

#[derive(Debug)]
pub enum InstrumentalizationError {
    GasMeteringInjection,
    StackHeightInjection,
    MultipleTables,
    MaxTableSize,
    InvalidByteCode,
}

#[derive(Default)]
pub(crate) struct ModuleConfig {
    has_grow_cost: bool,
    has_forbidden_floats: bool,
    has_metering: bool,
    has_table_size_limit: bool,
}

impl ModuleConfig {
    pub fn new() -> Self {
        Self {
            has_grow_cost: false,
            has_forbidden_floats: false,
            has_metering: false,
            has_table_size_limit: false,
        }
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

    pub fn apply(
        &self,
        code: &[u8],
    ) -> Result<Vec<u8>, InstrumentalizationError> {
        let mut module = elements::deserialize_buffer(code)
            .or(Err(InstrumentalizationError::InvalidByteCode))?;

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
                .or(Err(InstrumentalizationError::GasMeteringInjection))?;

            module = pwasm_utils::stack_height::inject_limiter(
                module,
                schedule.max_stack_height,
            )
            .or(Err(InstrumentalizationError::StackHeightInjection))?;
        }

        if self.has_table_size_limit {
            if let Some(table_section) = module.table_section() {
                // In Wasm MVP spec, there may be at most one table declared.
                // Double check this explicitly just in case the
                // Wasm version changes.
                if table_section.entries().len() > 1 {
                    return Err(InstrumentalizationError::MultipleTables);
                }

                if let Some(table_type) = table_section.entries().first() {
                    // Check the table's initial size as there is no instruction
                    // or environment function capable of
                    // growing the table.
                    if table_type.limits().initial() > schedule.max_table_size {
                        return Err(InstrumentalizationError::MaxTableSize);
                    }
                }
            }
        }

        validate_module::<PlainValidator>(&module)
            .or(Err(InstrumentalizationError::InvalidByteCode))?;

        module
            .to_bytes()
            .or(Err(InstrumentalizationError::InvalidByteCode))
    }
}
