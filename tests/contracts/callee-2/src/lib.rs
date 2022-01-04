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
pub struct Callee2State;

impl Callee2State {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Callee2Query {
    sender_sender: ContractId,
    sender: ContractId,
}

impl Callee2Query {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Query for Callee2Query {
    const NAME: &'static str = "get";
    type Return = Callee2Return;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Callee2Return {
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
    fn get(written: u32) -> u32 {
        let mut store = AbiStore;

        let (state, callee2) = unsafe {
            archived_root::<(Callee2State, Callee2Query)>(
                &SCRATCH[..written as usize],
            )
        };

        let mut _state: Callee2State = (state).deserialize(&mut store).unwrap();
        let callee: Callee2Query = (callee2).deserialize(&mut store).unwrap();

        assert_eq!(callee2.sender, rusk_uplink::caller(), "Expected Caller");

        rusk_uplink::debug!("callee-2: returning sender_sender, sender from params and callee");

        let ret = Callee2Return {
            sender_sender: callee.sender_sender,
            sender: callee.sender,
            callee: rusk_uplink::callee(),
        };
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&ret).unwrap()
            + core::mem::size_of::<
                <<Callee2Query as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }
};
