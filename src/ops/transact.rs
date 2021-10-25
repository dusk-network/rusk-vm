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

pub struct ApplyTransaction;

impl ApplyTransaction {
    pub fn transact(env: &Env, contract_id_ofs: u32, transaction_ofs: u32) -> Result<(), VMError> {
        println!("host transact: begin");
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
        println!("host transact: read contract id begin");
        let contract_id_memory = context.read_memory(contract_id_ofs, 32)?;
        let contract_id = ContractId::from(&contract_id_memory);
        println!("host transact: read contract id end");

        println!("host transact: read transaction begin");
        let transaction_memory = context.read_memory_from(transaction_ofs)?;
        let mut source = Source::new(&transaction_memory);
        let state = ContractState::decode(&mut source)?;
        let transaction = Transaction::decode(&mut source)?;
        println!("host transact: read transaction end");

        println!("host transact: setting callee state begin");
        let callee = *context.callee();
        *context.state_mut().get_contract_mut(&callee)?.state_mut() = state;
        println!("host transact: setting callee state end");

        println!("host transact: calling context transact begin");
        let (state, result) = context.transact(contract_id, transaction)?;
        println!("host transact: calling context transact end");

        // context
        //     .memory_mut(|m| {
        //         // write back the return value
        //         let mut sink = Sink::new(&mut m[transaction_ofs..]);
        //         state.encode(&mut sink);
        //         result.encode(&mut sink);
        //         Ok(())
        //     })
        //     .map_err(VMError::from_store_error)
        println!("host transact: writing result begin, result encoded_len = {}", result.encoded_len());
        let state_encoded_len = state.encoded_len();
        let mut state_buffer = vec![0; state_encoded_len];
        let mut state_sink = Sink::new(&mut state_buffer);
        let mut result_buffer = vec![0; result.encoded_len()];
        let mut result_sink = Sink::new(&mut result_buffer);
        println!("host transact: encoding result begin");
        state.encode(&mut state_sink);
        result.encode(&mut result_sink);
        println!("host transact: encoding result end");
        context.write_memory(&state_buffer, transaction_ofs as u64)?;
        context.write_memory(&result_buffer, transaction_ofs as u64 + state_encoded_len as u64)?;
        println!("host transact: writing result end");
        println!("host transact: end");
        Ok(())
    }
}
