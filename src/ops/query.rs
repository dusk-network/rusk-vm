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

pub struct ExecuteQuery;


impl ExecuteQuery {
    pub fn query(env: &Env, contract_id_ofs: u32, query_ofs: u32) -> Result<(), VMError> {
        let contract_id_ofs = contract_id_ofs as u64;
        let query_ofs = query_ofs as u64;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        // let (contract_id, query) = context
        //     .memory(|m| {
        //         let contract_id = ContractId::from(
        //             &m[contract_id_ofs..contract_id_ofs + 32],
        //         );
        //
        //         let mut source = Source::new(&m[query_ofs..]);
        //         let query = Query::decode(&mut source)?;
        //
        //         Ok((contract_id, query))
        //     })
        //     .map_err(VMError::from_store_error)?;
        let contract_id_memory = context.read_memory(contract_id_ofs, 32)?;
        let contract_id = ContractId::from(&contract_id_memory);
        let mut source = Source::new(&contract_id_memory);
        let query = Query::decode(&mut source).map_err(VMError::from_store_error)?;

        let result = context.query(contract_id, query)?;

        // context
        //     .memory_mut(|m| {
        //         // write back the return value
        //         let mut sink = Sink::new(&mut m[query_ofs..]);
        //         result.encode(&mut sink);
        //         Ok(())
        //     })
        //     .map_err(VMError::from_store_error)?;
        let mut result_buffer = vec![0;result.encoded_len()];
        let mut sink = Sink::new(&mut result_buffer[..]);
        result.encode(&mut sink);
        context.write_memory(&result_buffer, query_ofs as u64)?;

        Ok(())
    }
}
