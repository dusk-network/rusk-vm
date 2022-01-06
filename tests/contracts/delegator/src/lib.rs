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

use rkyv::{AlignedVec, Archive, Deserialize, Serialize};
use rusk_uplink::{
    ContractId, Query, RawQuery, RawTransaction, ReturnValue, Transaction,
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
    ) -> ReturnValue {
        rusk_uplink::transact_raw(self, target, transaction, 0).unwrap()
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 256] = [0u8; 256];

    #[no_mangle]
    fn delegate_query(written_state: u32, written_data: u32) -> u32 {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<Delegator>(&SCRATCH[..written_state as usize])
        };
        let arg = unsafe {
            archived_root::<QueryForwardData>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let de_state: Delegator = state.deserialize(&mut store).unwrap();
        let de_arg: QueryForwardData = arg.deserialize(&mut store).unwrap();

        let query_name = de_arg.name.as_ref();
        let mut query_data = AlignedVec::new();
        query_data.extend_from_slice(de_arg.data.as_ref());
        let result: ReturnValue = de_state.delegate_query(
            &de_arg.contract_id,
            &RawQuery::from(query_data, query_name),
        );

        let len = result.data_len();
        unsafe { &SCRATCH[..len].copy_from_slice(result.data()) };
        len as u32
    }

    #[no_mangle]
    fn delegate_transaction(written_state: u32, written_data: u32) -> u64 {
        let mut store = AbiStore;

        let state = unsafe {
            archived_root::<Delegator>(&SCRATCH[..written_state as usize])
        };
        let arg = unsafe {
            archived_root::<TransactionForwardData>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let mut de_state: Delegator = state.deserialize(&mut store).unwrap();
        let de_arg: TransactionForwardData =
            arg.deserialize(&mut store).unwrap();

        let query_name = de_arg.name.as_ref();
        let mut query_data = AlignedVec::new();
        query_data.extend_from_slice(de_arg.data.as_ref());
        let result: ReturnValue = de_state.delegate_transaction(
            &de_arg.contract_id,
            &RawTransaction::from(query_data, query_name),
        );

        let len = result.data_len();
        unsafe { &SCRATCH[..len].copy_from_slice(result.data()) };
        result.encode_lenghts()
    }
};
