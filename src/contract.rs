// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Schedule, VMError};
use canonical::{Canon, Store};
use canonical_derive::Canon;
use core::marker::PhantomData;
pub use dusk_abi::{ContractId, ContractState};
use parity_wasm::elements::{self, Module, Type, ValueType};
use wasmi_validation::{validate_module, PlainValidator};

/// A representation of a contract with a state and bytecode
#[derive(Canon, Clone)]
pub struct Contract {
    state: ContractState,
    code: Vec<u8>,
}

impl Contract {
    /// Create a new Contract with initial state and code
    pub fn new<State, Code, S>(
        state: State,
        code: Code,
        store: &S,
    ) -> Result<Self, S::Error>
    where
        State: Canon<S>,
        Code: Into<Vec<u8>>,
        S: Store,
    {
        Ok(Contract {
            state: ContractState::from_canon(&state, &store)?,
            code: code.into(),
        })
    }

    /// Returns a reference to the contract bytecode
    pub fn bytecode(&self) -> &[u8] {
        &self.code
    }

    /// Returns a reference to the contract state
    pub fn state(&self) -> &ContractState {
        &self.state
    }

    /// Returns a mutable reference to the contract state
    pub fn state_mut(&mut self) -> &mut ContractState {
        &mut self.state
    }
}

pub struct ContractInstrumenter<'a, S: Store> {
    module: Module,
    schedule: &'a crate::Schedule,
    #[doc(hidden)]
    _marker: PhantomData<S>,
}

impl<'a, S: Store> ContractInstrumenter<'a, S> {
    /// Creates a new [`ContractInstrumenter`] instance from a bytecode source
    /// and a Schedule config.
    pub fn new(
        bytecode: &[u8],
        schedule: &'a Schedule,
    ) -> Result<ContractInstrumenter<'a, S>, VMError<S>> {
        Ok(Self {
            module: elements::deserialize_buffer(bytecode)
                .map_err(|_| VMError::InvalidWASMModule)?,
            schedule,
            _marker: PhantomData::<S>,
        })
    }

    /// Serializes the [`ContractInstrumenter`] bytecode returning an
    /// error if anything in the Module is wrongly configured.
    pub fn bytecode(&self) -> Result<Vec<u8>, VMError<S>> {
        self.module
            .clone()
            .to_bytes()
            .map_err(|_| VMError::InvalidWASMModule)
    }

    /// Applies all of the checks and instrumentation injections to a contract
    /// bytecode.
    /// It also validates the module after the instrumentation has been applied.
    pub fn apply_module_config(&mut self) -> Result<(), VMError<S>> {
        self.ensure_no_floating_types()?;
        self.ensure_table_size_limit(&crate::Schedule::default())?;
        self.inject_gas_metering()?;
        self.inject_stack_height_metering()?;
        self.validate_module()
    }

    pub fn validate_module(&self) -> Result<(), VMError<S>> {
        validate_module::<PlainValidator>(&self.module)
            .map_err(|_| VMError::InvalidWASMModule)
    }

    /// Filter that ensures that there's no floating point usage inside of the bytecode
    /// of the ContractInstrumenter module.
    fn ensure_no_floating_types(&self) -> Result<(), VMError<S>> {
        // TODO: Check wether the type section contains the `ValueType`s used across the other
        // sections too. Don't think so. But we should check.
        if let Some(global_section) = self.module.global_section() {
            for global in global_section.entries() {
                match global.global_type().content_type() {
                    ValueType::F32 | ValueType::F64 => return Err(VMError::InstrumentationError(
                        "use of floating point type in globals is forbidden".to_string(),
                    )),
                    _ => {}
                }
            }
        }

        if let Some(code_section) = self.module.code_section() {
            for func_body in code_section.bodies() {
                for local in func_body.locals() {
                    match local.value_type() {
                        ValueType::F32 | ValueType::F64 => return Err(
                            VMError::InstrumentationError("use of floating point type in locals is forbidden".to_string()),
                        ),
                        _ => {}
                    }
                }
            }
        }

        if let Some(type_section) = self.module.type_section() {
            for wasm_type in type_section.types() {
                match wasm_type {
                    Type::Function(func_type) => {
                        let return_type = func_type.return_type();
                        for value_type in
                            func_type.params().iter().chain(return_type.iter())
                        {
                            match value_type {
								ValueType::F32 | ValueType::F64 => {
									return Err(
										VMError::InstrumentationError("use of floating point type in function types is forbidden".to_string()),
									)
								}
								_ => {}
							}
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Ensures that tables declared in the module are not too big.
    fn ensure_table_size_limit(
        &self,
        schedule: &Schedule,
    ) -> Result<(), VMError<S>> {
        if let Some(table_section) = self.module.table_section() {
            // In Wasm MVP spec, there may be at most one table declared. Double check this
            // explicitly just in case the Wasm version changes.
            if table_section.entries().len() > 1 {
                return Err(VMError::InstrumentationError(
                    "multiple tables declared".to_string(),
                ));
            }
            if let Some(table_type) = table_section.entries().first() {
                // Check the table's initial size as there is no instruction or environment function
                // capable of growing the table.
                if table_type.limits().initial() > schedule.max_table_size {
                    return Err(VMError::InstrumentationError(
                        "table exceeds maximum size allowed".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Injects gas metering instrumentation into the Module.
    fn inject_gas_metering(&mut self) -> Result<(), VMError<S>> {
        let gas_rules = pwasm_utils::rules::Set::new(
            self.schedule.regular_op_cost as u32,
            Default::default(),
        )
        .with_grow_cost(self.schedule.grow_mem_cost as u32)
        .with_forbidden_floats();

        self.module = pwasm_utils::inject_gas_counter(
            self.module.clone(),
            &gas_rules,
            "gas_metered",
        )
        .map_err(|_| {
            VMError::InstrumentationError(
                "gas instrumentation injection failed".to_string(),
            )
        })?;

        Ok(())
    }

    /// Injects stack height metering instrumentation into the Module.
    fn inject_stack_height_metering(&mut self) -> Result<(), VMError<S>> {
        self.module = pwasm_utils::stack_height::inject_limiter(
            self.module.clone(),
            self.schedule.max_stack_height,
        )
        .map_err(|_| {
            VMError::InstrumentationError(
                "stack height instrumentation injection failed".to_string(),
            )
        })?;
        Ok(())
    }
}
