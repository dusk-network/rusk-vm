// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::mem::size_of;
use rkyv::AlignedVec;

use rusk_uplink::{ContractId, RawTransaction};
use std::str;
use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct ApplyTransaction;

impl ApplyTransaction {
    pub fn transact(
        env: &Env,
        contract_id_ofs: i32,
        transact_ofs: i32,
        transact_len: u32,
        name_ofs: i32,
        name_len: u32,
        gas_limit: u64,
    ) -> Result<u64, VMError> {
        trace!("Executing 'query' host function");

        let context = env.get_context();

        let config = context.config();
        context.charge_gas(config.host_costs.transact)?;

        let contract_id_ofs = contract_id_ofs as u64;
        let transact_ofs = transact_ofs as u64;
        let transact_len = transact_len as usize;
        let name_ofs = name_ofs as u64;
        let name_len = name_len as usize;

        let contract_id_memory =
            context.read_memory(contract_id_ofs, size_of::<ContractId>())?;
        let contract_id = ContractId::from(&contract_id_memory);

        let query_memory = context.read_memory(transact_ofs, transact_len)?;
        let mut query_data: AlignedVec = AlignedVec::new();
        query_data.extend_from_slice(query_memory);

        let mut gas_meter = context.gas_meter()?.limited(gas_limit);

        let query_name = context.read_memory(name_ofs, name_len)?;
        let name =
            str::from_utf8(query_name).map_err(|_| VMError::InvalidUtf8)?;

        let raw_transaction = RawTransaction::from(query_data, name);
        let context = env.get_context();
        let result =
            context.transact(contract_id, raw_transaction, &mut gas_meter)?;

        context.write_memory(result.state(), transact_ofs)?;
        context.write_memory(
            result.data(),
            transact_ofs + result.state_len() as u64,
        )?;

        Ok(result.encode_lenghts())
    }
}
