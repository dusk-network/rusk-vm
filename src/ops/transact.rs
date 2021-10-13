// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::ops::AbiCall;
use crate::VMError;

use canonical::{Canon, Sink, Source};
use dusk_abi::{ContractId, ContractState, Transaction};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};
use crate::resolver::Env;
use crate::NetworkState;

pub struct ApplyTransaction;

impl AbiCall for ApplyTransaction {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(contract_id_ofs), RuntimeValue::I32(transaction_ofs)] =
            *args.as_ref()
        {
            let contract_id_ofs = contract_id_ofs as usize;
            let transaction_ofs = transaction_ofs as usize;

            let (contract_id, state, transaction) = context
                .memory(|m| {
                    let contract_id = ContractId::from(
                        &m[contract_id_ofs..contract_id_ofs + 32],
                    );

                    let mut source = Source::new(&m[transaction_ofs..]);

                    let state = ContractState::decode(&mut source)?;
                    let transaction = Transaction::decode(&mut source)?;

                    Ok((contract_id, state, transaction))
                })
                .map_err(VMError::from_store_error)?;

            let callee = *context.callee();
            *context.state_mut().get_contract_mut(&callee)?.state_mut() = state;

            let (state, result) = context.transact(contract_id, transaction)?;

            context
                .memory_mut(|m| {
                    // write back the return value
                    let mut sink = Sink::new(&mut m[transaction_ofs..]);
                    state.encode(&mut sink);
                    result.encode(&mut sink);
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

impl ApplyTransaction {
    pub fn transact(env: &Env, contract_id_ofs: u32, transaction_ofs: u32) -> Result<(), VMError> {
        let contract_id_ofs = contract_id_ofs as usize;
        let transaction_ofs = transaction_ofs as usize;
        let mut network_state = NetworkState::with_block_height(env.height).restore(env.persisted_id.clone())?;
        let mut context = CallContext::new(&mut network_state, env.gas_meter.clone());

        let (contract_id, state, transaction) = context
            .memory(|m| {
                let contract_id = ContractId::from(
                    &m[contract_id_ofs..contract_id_ofs + 32],
                );

                let mut source = Source::new(&m[transaction_ofs..]);

                let state = ContractState::decode(&mut source)?;
                let transaction = Transaction::decode(&mut source)?;

                Ok((contract_id, state, transaction))
            })
            .map_err(VMError::from_store_error)?;

        let callee = *context.callee();
        *context.state_mut().get_contract_mut(&callee)?.state_mut() = state;

        let (state, result) = context.transact(contract_id, transaction)?;

        context
            .memory_mut(|m| {
                // write back the return value
                let mut sink = Sink::new(&mut m[transaction_ofs..]);
                state.encode(&mut sink);
                result.encode(&mut sink);
                Ok(())
            })
            .map_err(VMError::from_store_error)
    }
}
