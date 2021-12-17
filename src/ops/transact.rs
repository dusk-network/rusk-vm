// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::mem::size_of;

use rusk_uplink::ContractId;
use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct ApplyTransaction;

impl ApplyTransaction {
    pub fn transact(
        env: &Env,
        contract_id_offset: i32,
        transaction_offset: i32,
        _gas_limit: u64,
    ) -> Result<(), VMError> {
        trace!("Executing 'transact' host function");

        let contract_id_offset = contract_id_offset as u64;
        let transaction_offset = transaction_offset as u64;
        let context = env.get_context();

        let contract_id_memory =
            context.read_memory(contract_id_offset, size_of::<ContractId>())?;
        let _contract_id = ContractId::from(&contract_id_memory);
        let _transaction_memory =
            context.read_memory_from(transaction_offset)?;
        // let mut source = Source::new(transaction_memory);
        let _state = todo!();
        let _transaction = todo!();

        // let _callee = *context.callee();

        // *context
        //     .state_mut()
        //     .get_contract_mut(&callee)?
        //     .leaf_mut()
        //     .state_mut() = state;

        // let mut gas_meter = context.gas_meter().limited(gas_limit);
        // let (state, result) =
        //     context.transact(contract_id, transaction, &mut gas_meter)?;

        // let state_encoded_length = todo!();
        // let (mut state_buffer, mut result_buffer) =
        //     (vec![0; state_encoded_length], vec![0; todo!()]);
        // let (mut state_sink, mut result_sink) todo!();
        //     (Sink::new(&mut state_buffer), Sink::new(&mut result_buffer));
        // state.encode(&mut state_sink);
        // result.encode(&mut result_sink);
        // context.write_memory(&state_buffer, transaction_offset)?;
        // context.write_memory(
        //     &result_buffer,
        //     transaction_offset + state_encoded_length as u64,
        // )
    }
}
