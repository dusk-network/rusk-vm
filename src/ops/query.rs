// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::env::Env;
use crate::VMError;

use core::mem::size_of;
use rkyv::AlignedVec;
use rusk_uplink::{ContractId, RawQuery};
use std::str;
use tracing::trace;

pub struct ExecuteQuery;

impl ExecuteQuery {
    pub fn query(
        env: &Env,
        contract_id_ofs: i32,
        query_ofs: i32,
        query_len: u32,
        name_ofs: i32,
        name_len: u32,
        gas_limit: u64,
    ) -> Result<u32, VMError> {
        trace!("Executing 'query' host function");

        let contract_id_ofs = contract_id_ofs as u64;
        let query_ofs = query_ofs as u64;
        let context = env.get_context();
        let contract_id_memory =
            context.read_memory(contract_id_ofs, size_of::<ContractId>())?;
        let contract_id = ContractId::from(&contract_id_memory);
        let query_memory =
            context.read_memory(query_ofs, query_len as usize)?;
        let query_name =
            context.read_memory(name_ofs as u64, name_len as usize)?;
        let mut query_data: AlignedVec = AlignedVec::new();
        for c in query_memory {
            // todo! there must be a better way
            query_data.push(*c);
        }
        let name =
            str::from_utf8(query_name).map_err(|_| VMError::InvalidUtf8)?;
        let raw_query = RawQuery::from(query_data, &name);
        let mut gas_meter = context.gas_meter().limited(gas_limit);
        let context = env.get_context();
        let result = context.query(contract_id, raw_query, &mut gas_meter)?;
        context.write_memory(&result.0, query_ofs as u64);
        Ok(result.0.len() as u32)
    }
}
