// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use crate::env::Env;
use canonical::{Canon, Sink, Source};
use core::mem::size_of;
use dusk_abi::{ContractId, ContractState, Transaction};

pub struct ApplyTransaction;

impl ApplyTransaction {
    pub fn transact(
        env: &Env,
        contract_id_offset: i32,
        transaction_offset: i32,
    ) -> Result<(), VMError> {
        let contract_id_offset = contract_id_offset as u64;
        let transaction_offset = transaction_offset as u64;
        let context: &mut CallContext =
            unsafe { &mut *(env.context.0 as *mut CallContext) };

        let contract_id_memory =
            context.read_memory(contract_id_offset, size_of::<ContractId>())?;
        let contract_id = ContractId::from(&contract_id_memory);
        let transaction_memory =
            context.read_memory_from(transaction_offset)?;
        let mut source = Source::new(transaction_memory);
        let state = ContractState::decode(&mut source)
            .map_err(VMError::from_store_error)?;
        let transaction = Transaction::decode(&mut source)
            .map_err(VMError::from_store_error)?;

        let callee = *context.callee();
        *context.state_mut().get_contract_mut(&callee)?.state_mut() = state;

        let (state, result) = context.transact(contract_id, transaction)?;

        let state_encoded_length = state.encoded_len();
        let (mut state_buffer, mut result_buffer) =
            (vec![0; state_encoded_length], vec![0; result.encoded_len()]);
        let (mut state_sink, mut result_sink) =
            (Sink::new(&mut state_buffer), Sink::new(&mut result_buffer));
        state.encode(&mut state_sink);
        result.encode(&mut result_sink);
        context.write_memory(&state_buffer, transaction_offset)?;
        context.write_memory(
            &result_buffer,
            transaction_offset + state_encoded_length as u64,
        )
    }
}
