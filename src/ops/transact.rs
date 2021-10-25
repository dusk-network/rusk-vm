// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use canonical::{Canon, Sink, Source};
use dusk_abi::{ContractId, ContractState, Transaction};
use crate::resolver::{Env, WasmerRuntimeValue};
use core::mem::size_of;

pub struct ApplyTransaction;

impl ApplyTransaction {
    pub fn transact(env: &Env, contract_id_offs: i32, transaction_offs: i32) -> Result<(), VMError> {
        let contract_id_offs = contract_id_offs as u64;
        let transaction_offs = transaction_offs as u64;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext) };

        let contract_id_memory =
            context.read_memory(contract_id_offs, size_of::<ContractId>())?;
        let contract_id = ContractId::from(&contract_id_memory);
        let transaction_memory = context.read_memory_from(transaction_offs)?;
        let mut source = Source::new(&transaction_memory);
        let state = ContractState::decode(&mut source)?;
        let transaction = Transaction::decode(&mut source)?;

        let callee = *context.callee();
        *context.state_mut().get_contract_mut(&callee)?.state_mut() = state;

        let (state, result) =
            context.transact(contract_id, transaction)?;

        let state_encoded_length = state.encoded_len();
        let mut state_buffer = vec![0; state_encoded_length];
        let mut state_sink = Sink::new(&mut state_buffer);
        let mut result_buffer = vec![0; result.encoded_len()];
        let mut result_sink = Sink::new(&mut result_buffer);
        state.encode(&mut state_sink);
        result.encode(&mut result_sink);
        context.write_memory(&state_buffer, transaction_offs)?;
        context.write_memory(&result_buffer, transaction_offs + state_encoded_length as u64)?;
        Ok(())
    }
}
