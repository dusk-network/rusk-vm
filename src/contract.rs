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
use parity_wasm::elements::{self, Module};
use wasmi_validation::{validate_module, PlainValidator};

/// A representation of a contract with a state and bytecode
#[derive(Clone, Canon)]
pub struct Contract<S: Store> {
    state: ContractState,
    pub(crate) code: Vec<u8>,
    #[doc(hidden)]
    _marker: PhantomData<S>,
}

impl<S: Store> Contract<S> {
    /// Create a new Contract with initial state and code
    pub fn new<State, Code>(
        state: State,
        code: Code,
        store: &S,
    ) -> Result<Self, S::Error>
    where
        State: Canon<S>,
        Code: Into<Vec<u8>>,
    {
        Ok(Contract {
            state: ContractState::from_canon(&state, &store)?,
            code: code.into(),
            _marker: PhantomData::<S>,
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

    /// Creates a new [`ContractInstrumenter`] instance from a mutable reference
    /// to a [`Contract`] and a [`Schedule`] config.
    pub(crate) fn instrument(mut self) -> Result<Self, VMError<S>> {
        let module = elements::deserialize_buffer(self.code.as_slice())
            .map_err(|_| VMError::InvalidWASMModule)?;

        // Apply instrumentalization & Update the Contract instance bytecode.
        self.code = apply_module_config(module)?;

        Ok(self)
    }
}

/// Given a [`Contract`] instance, it applies the
/// instrumentalization to the bytecode and returning the newly
/// generated one after checking that it is a valid WASM module.
fn apply_module_config<S: Store>(
    mut module: Module,
) -> Result<Vec<u8>, VMError<S>> {
    let schedule = crate::Schedule::default();

    let ruleset = pwasm_utils::rules::Set::new(
        schedule.regular_op_cost as u32,
        Default::default(),
    )
    .with_grow_cost(schedule.grow_mem_cost as u32)
    .with_forbidden_floats();

    module = inject_gas_metering(module, ruleset)?;
    module = inject_stack_height_metering(module, &schedule)?;
    ensure_table_size_limit(&module, &schedule)?;
    validate_mod(module)
}

/// Check the validity of a [`Module`] pertaining to a
/// [`Contract`] and in case it is, return it serialized as
/// bytecode.
fn validate_mod<S: Store>(module: Module) -> Result<Vec<u8>, VMError<S>> {
    let _ = validate_module::<PlainValidator>(&module)
        .map_err(|_| VMError::InvalidWASMModule)?;

    module.to_bytes().map_err(|_| {
        VMError::InstrumentalizationError(
            "Failed to serialize the WASM Module back to plain bytecode"
                .to_string(),
        )
    })
}

/// Ensures that tables declared in the module are not too big.
fn ensure_table_size_limit<S: Store>(
    module: &Module,
    schedule: &Schedule,
) -> Result<(), VMError<S>> {
    if let Some(table_section) = module.table_section() {
        // In Wasm MVP spec, there may be at most one table declared. Double
        // check this explicitly just in case the Wasm version
        // changes.
        if table_section.entries().len() > 1 {
            return Err(VMError::InstrumentalizationError(
                "multiple tables declared".to_string(),
            ));
        }
        if let Some(table_type) = table_section.entries().first() {
            // Check the table's initial size as there is no instruction or
            // environment function capable of growing the
            // table.
            if table_type.limits().initial() > schedule.max_table_size {
                return Err(VMError::InstrumentalizationError(
                    "table exceeds maximum size allowed".to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Injects gas metering instrumentation into the Module.
fn inject_gas_metering<T, S: Store>(
    module: Module,
    ruleset: T,
) -> Result<Module, VMError<S>>
where
    T: pwasm_utils::rules::Rules,
{
    pwasm_utils::inject_gas_counter(module, &ruleset, "env").map_err(|_| {
        VMError::InstrumentalizationError(
            "gas instrumentation injection failed".to_string(),
        )
    })
}

/// Injects stack height metering instrumentation into the Module.
fn inject_stack_height_metering<S: Store>(
    module: Module,
    schedule: &Schedule,
) -> Result<Module, VMError<S>> {
    pwasm_utils::stack_height::inject_limiter(module, schedule.max_stack_height)
        .map_err(|_| {
            VMError::InstrumentalizationError(
                "stack height instrumentation injection failed".to_string(),
            )
        })
}
