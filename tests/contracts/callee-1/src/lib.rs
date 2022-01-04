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
use rusk_uplink::{ContractId, Query, ReturnValue, Transaction};

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Callee1State {
    target_address: ContractId,
}

impl Callee1State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, address: ContractId) {
        self.target_address = address;
    }
}

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct SenderParameter {
    sender_id: ContractId,
}

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct Callee1Transaction {
    target_id: ContractId,
}

impl Callee1Transaction {
    pub fn new(target_id: ContractId) -> Self {
        Self { target_id }
    }
}

impl Transaction for Callee1Transaction {
    const NAME: &'static str = "set_target";
    type Return = ();
}

#[derive(Archive, Serialize, Deserialize)]
pub struct Callee2Query {
    sender: ContractId,
    callee: ContractId,
}

impl Query for Callee2Query {
    const NAME: &'static str = "get";
    type Return = ([u8; 32], [u8; 32], [u8; 32]);
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
    fn call(written: u32) -> u32 {
        let mut store = AbiStore;

        let (state, sender) = unsafe {
            archived_root::<(Callee1State, SenderParameter)>(
                &SCRATCH[..written as usize],
            )
        };

        let mut state: Callee1State = (state).deserialize(&mut store).unwrap();
        let sender: SenderParameter = (sender).deserialize(&mut store).unwrap();

        assert_eq!(
            sender.sender_id,
            rusk_uplink::caller(),
            "Expected Caller"
        );

        rusk_uplink::debug!("callee-1: calling state target 'get' with params: sender from param and callee");
        let call_data = Callee2Query {
            sender: sender.sender_id,
            callee: rusk_uplink::callee(),
        };
        let ret =
            rusk_uplink::query::<Callee2Query>(&state.target_address, call_data, 0)
                .unwrap();

        let res: <Callee2Query as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<Callee2Query as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }

    #[no_mangle]
    fn set_target(_: u32, written: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let (state, target) = unsafe {
            archived_root::<(Callee1State, Callee1Transaction)>(
                &SCRATCH[..written as usize],
            )
        };

        let mut state: Callee1State = (state).deserialize(&mut store).unwrap();
        let target: Callee1Transaction =
            (target).deserialize(&mut store).unwrap();

        state.set_target(target.target_id);
        rusk_uplink::debug!("setting state.set_target to: {:?}", target.target_id);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<Callee1State as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
                <<Callee1Transaction as Transaction>::Return as Archive>::Archived,
            >();

        [state_len as u32, return_len as u32]
    }
};
