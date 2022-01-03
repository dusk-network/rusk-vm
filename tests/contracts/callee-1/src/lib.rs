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

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{
    ContractId, Query, ReturnValue, Transaction,
};
extern crate alloc;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Callee1 {
    target_address: ContractId,
}

impl Callee1 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, address: ContractId) {
        self.target_address = address;
    }
}

impl Query for Callee1 {
    const NAME: &'static str = "do_call";
    type Return = CallDataReturn;
}

impl Transaction for Callee1 {
    const NAME: &'static str = "set_target";
    type Return = ();
}


#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CallData {
    sender: ContractId,
    callee: ContractId,
}


#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CallDataReturn {
    sender_sender: ContractId,
    sender: ContractId,
    callee: ContractId,
}


#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    #[no_mangle]
    fn do_call(written: u32) -> u32 {
        let mut store = AbiStore;

        let (state, sender) = unsafe {
            archived_root::<(Callee1, ContractId)>(
                &SCRATCH[..written as usize],
            )
        };

        assert_eq!(sender, dusk_abi::caller(), "Expected Caller");

        let call_data = CallData {sender, callee: dusk_abi::callee()};
        let ret =
            rusk_uplink::query::<CallData>(
                &state.target_address,
                call_data,
                0,
            )
            .unwrap();
        let res: <Callee1 as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<Callee1 as Query>::Return as Archive>::Archived,
        >();
        buffer_len as u32
    }

    #[no_mangle]
    fn set_target(written: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let (state, sender) = unsafe {
            archived_root::<(Callee1, ContractId)>(
                &SCRATCH[..written as usize],
            )
        };

        state.set_target(target);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<Counter as Archive>::Archived>();

        let return_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<Callee1 as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }
};
