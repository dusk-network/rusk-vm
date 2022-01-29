// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(
    core_intrinsics,
    lang_items,
    alloc_error_handler,
    option_result_unwrap_unchecked
)]

use microkelvin::{OffsetLen, StoreRef};
use rkyv::{AlignedVec, Archive, Deserialize, Serialize};
use rusk_uplink::{
    Apply, ContractId, Execute, Query, RawQuery, RawTransaction, ReturnValue,
    StoreContext, Transaction,
};
use rusk_uplink_derive::query;

extern crate alloc;
use alloc::boxed::Box;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct Delegator;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct QueryForwardData {
    contract_id: ContractId,
    data: Box<[u8]>,
    name: Box<str>,
}

impl QueryForwardData {
    pub fn new(
        contract_id: ContractId,
        data: impl AsRef<[u8]>,
        name: impl AsRef<str>,
    ) -> Self {
        let data = Box::from(data.as_ref());
        let name = Box::from(name.as_ref());
        Self {
            contract_id,
            data,
            name,
        }
    }
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct TransactionForwardData {
    contract_id: ContractId,
    data: Box<[u8]>,
    name: Box<str>,
}

impl TransactionForwardData {
    pub fn new(
        contract_id: ContractId,
        data: impl AsRef<[u8]>,
        name: impl AsRef<str>,
    ) -> Self {
        let data = Box::from(data.as_ref());
        let name = Box::from(name.as_ref());
        Self {
            contract_id,
            data,
            name,
        }
    }
}

impl Transaction for TransactionForwardData {
    const NAME: &'static str = "delegate_transaction";
    type Return = ();
}

#[query(name="delegate_query")]
impl Execute<QueryForwardData> for Delegator {
    fn execute(
        &self,
        arg: QueryForwardData,
        store: StoreContext,
    ) -> u32 {
        let query_name = arg.name.as_ref();
        let mut query_data = AlignedVec::new();
        query_data.extend_from_slice(arg.data.as_ref());
        let result: ReturnValue = self.delegate_query(
            &arg.contract_id,
            &RawQuery::from(query_data, query_name),
        );
        let res = result.cast_data::<<QueryForwardData as Query>::Return>();
        let res: <QueryForwardData as Query>::Return =
            res.deserialize(&mut store.clone()).unwrap();
        res
    }
}

impl Apply<TransactionForwardData> for Delegator {
    fn apply(
        &mut self,
        arg: TransactionForwardData,
        store: StoreContext,
    ) -> <TransactionForwardData as Transaction>::Return {
        let query_name = arg.name.as_ref();
        let mut query_data = AlignedVec::new();
        query_data.extend_from_slice(arg.data.as_ref());
        let result: ReturnValue = self.delegate_transaction(
            &arg.contract_id,
            &RawTransaction::from(query_data, query_name),
            store.clone(),
        );
        store.put_raw(result.state());
    }
}

impl Delegator {
    pub fn delegate_query(
        &self,
        target: &ContractId,
        query: &RawQuery,
    ) -> ReturnValue {
        rusk_uplink::query_raw(target, query, 0).unwrap()
    }

    pub fn delegate_transaction(
        &mut self,
        target: &ContractId,
        transaction: &RawTransaction,
        store: StoreRef<OffsetLen>,
    ) -> ReturnValue {
        rusk_uplink::transact_raw(self, target, transaction, 0, store).unwrap()
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    scratch_memory!(256);

    t_handler!(
        _delegate_transaction,
        Delegator,
        TransactionForwardData
    );
};
