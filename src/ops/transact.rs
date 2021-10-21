// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use canonical::{Canon, Sink, Source};
use dusk_abi::{ContractId, ContractState, Transaction};
use crate::resolver::Env;

pub struct ApplyTransaction;

impl ApplyTransaction {
    pub fn transact(env: &Env, contract_id_ofs: u32, transaction_ofs: u32) -> Result<(), VMError> {
        let contract_id_ofs = contract_id_ofs as u64;
        let transaction_ofs = transaction_ofs as u64;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};

        // let (contract_id, state, transaction) = context
        //     .memory(|m| {
        //         let contract_id = ContractId::from(
        //             &m[contract_id_ofs..contract_id_ofs + 32],
        //         );
        //
        //         let mut source = Source::new(&m[transaction_ofs..]);
        //
        //         let state = ContractState::decode(&mut source)?;
        //         let transaction = Transaction::decode(&mut source)?;
        //
        //         Ok((contract_id, state, transaction))
        //     })
        //     .map_err(VMError::from_store_error)?;
        let contract_id_memory = context.read_memory(contract_id_ofs, 32)?;
        let contract_id = ContractId::from(&contract_id_memory);

        let transaction_memory = context.read_memory_from(transaction_ofs)?;
        let mut source = Source::new(&transaction_memory);
        let state = ContractState::decode(&mut source)?;
        let transaction = Transaction::decode(&mut source)?;

        let callee = *context.callee();
        *context.state_mut().get_contract_mut(&callee)?.state_mut() = state;

        let (_, result) = context.transact(contract_id, transaction)?;

        // context
        //     .memory_mut(|m| {
        //         // write back the return value
        //         let mut sink = Sink::new(&mut m[transaction_ofs..]);
        //         state.encode(&mut sink);
        //         result.encode(&mut sink);
        //         Ok(())
        //     })
        //     .map_err(VMError::from_store_error)
        let mut result_buffer = Vec::with_capacity(result.as_bytes().len()); // todo think of some better way
        let mut sink = Sink::new(&mut result_buffer);
        result.encode(&mut sink);
        context.write_memory(&result_buffer, transaction_ofs as u64)?;
        Ok(())
    }
}
