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
use rusk_uplink::{Execute, Query, RawQuery, ContractId, Transaction, RawTransaction, ReturnValue};

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct Delegator;

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct QueryForwardData {
    contract_id: ContractId,
}

impl QueryForwardData<'_> {
    pub fn new(contract_id: ContractId, query_name: &str) -> Self {
        Self { contract_id }
    }
}

// #[derive(Clone, Debug, Archive, Serialize, Deserialize)]
// pub struct TransactionForwardData {
//     contract_id: ContractId,
//     raw_transaction: RawTransaction,
// }

impl Query for QueryForwardData<'_> {
    const NAME: &'static str = "delegate_query";
    type Return = u32;
}

// impl Transaction for TransactionForwardData {
//     const NAME: &'static str = "delegate_transaction";
//     type Return = u32;
// }

impl Delegator {
    pub fn delegate_query(
        &self,
        target: &ContractId,
        query: &RawQuery,
    ) -> ReturnValue {
        rusk_uplink::query_raw(target, query, 0).unwrap()
    }

    // pub fn delegate_transaction(
    //     &mut self,
    //     target: &ContractId,
    //     transaction: &RawTransaction,
    // ) -> ReturnValue {
    //     rusk_uplink::transact_raw::<_>(self, target, transaction, 0).unwrap()
    // }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 128] = [0u8; 128];

    #[no_mangle]
    fn delegate_query(written: u32) -> u32 {
        let mut store = AbiStore;

        let (state, arg) = unsafe {
            archived_root::<(Delegator, QueryForwardData)>(&SCRATCH[..written as usize])
        };

        let de_state: Delegator = (state).deserialize(&mut store).unwrap();
        let de_arg: QueryForwardData = (arg).deserialize(&mut store).unwrap();

        let res: <QueryForwardData as Query>::Return = de_state.delegate_query(de_arg.contract_id, RawQuery::from(AlignedVec::new(), String::from("read")));
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<QueryForwardData as Query>::Return as Archive>::Archived,
        >();
        buffer_len as u32
    }

    #[no_mangle]
    fn delegate_transaction(written: u32) -> u32 {

    }
};