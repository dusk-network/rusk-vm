// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use canonical::{Canon, Sink, Source};
use dusk_abi::{ContractId, Query};
use crate::resolver::Env;
use core::mem::size_of;

pub struct ExecuteQuery;

impl ExecuteQuery {
    pub fn query(env: &Env, contract_id_offs: i32, query_offs: i32) -> Result<(), VMError> {
        let contract_id_offs = contract_id_offs as u64;
        let query_offs = query_offs as u64;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        let contract_id_memory = context.read_memory(contract_id_offs, size_of::<ContractId>())?;
        let contract_id = ContractId::from(&contract_id_memory);
        let query_memory = context.read_memory_from(query_offs)?;
        let mut source = Source::new(query_memory);
        let query = Query::decode(&mut source).map_err(VMError::from_store_error)?;

        let result = context.query(contract_id, query)?;

        let mut result_buffer = vec![0;result.encoded_len()];
        let mut sink = Sink::new(&mut result_buffer[..]);
        result.encode(&mut sink);
        context.write_memory(&result_buffer, query_offs as u64)?;

        Ok(())
    }
}
