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

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Callee2;

impl Callee2 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Query for Callee2 {
    const NAME: &'static str = "do_get";
    type Return = CallDataReturn2;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CallDataReturn2 {
    sender_sender: ContractId,
    sender: ContractId,
    callee: ContractId,
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CallData2 {
    sender_sender: ContractId,
    sender: ContractId,
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    #[no_mangle]
    fn do_get(written: u32) -> u32 {
        let mut store = AbiStore;

        let (_state, call_data2) = unsafe {
            archived_root::<(Callee1, CallData2)>(&SCRATCH[..written as usize])
        };

        assert_eq!(call_data2.sender, dusk_abi::caller(), "Expected Caller");

        let call_data = CallDataReturn2 {
            sender_sender: call_data2.sender_sender,
            sender: call_data2.sender,
            callee: rusk_uplink::callee(),
        };
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&call_data).unwrap()
            + core::mem::size_of::<
                <<Callee2 as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }
};
