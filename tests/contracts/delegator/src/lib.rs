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

impl Query for QueryForwardData {
    const NAME: &'static str = "delegate_query";
    type Return = u32;
}

impl Transaction for TransactionForwardData {
    const NAME: &'static str = "delegate_transaction";
    type Return = ();
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

    #[query]
    pub fn delegate_query(state: &Delegator, query_forward_data: QueryForwardData, store: StoreRef<OffsetLen>) -> u32 {
        let query_name = query_forward_data.name.as_ref();
        let mut query_data = AlignedVec::new();
        query_data.extend_from_slice(query_forward_data.data.as_ref());
        let result: ReturnValue = state.delegate_query(
            &query_forward_data.contract_id,
            &RawQuery::from(query_data, query_name),
        );
        let res = result.cast_data::<<QueryForwardData as Query>::Return>();
        let res: <QueryForwardData as Query>::Return =
            res.deserialize(&mut store.clone()).unwrap();
        res
    }

    #[transaction]
    pub fn delegate_transaction(state: &mut Delegator, transaction_forward_data: TransactionForwardData, store: StoreRef<OffsetLen>) {
        let query_name = transaction_forward_data.name.as_ref();
        let mut query_data = AlignedVec::new();
        query_data.extend_from_slice(transaction_forward_data.data.as_ref());
        let result: ReturnValue = state.delegate_transaction(
            &transaction_forward_data.contract_id,
            &RawTransaction::from(query_data, query_name),
            store.clone(),
        );
        store.put_raw(result.state());
    }
};
