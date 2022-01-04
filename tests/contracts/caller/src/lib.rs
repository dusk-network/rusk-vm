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
extern crate alloc;

// state

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Caller {
    target_address: ContractId,
}

impl Caller {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, address: ContractId) {
        self.target_address = address;
    }
}

// query

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct QueryCallees;

impl Query for QueryCallees {
    const NAME: &'static str = "do_call";
    type Return = ([u8; 32], [u8; 32], [u8; 32]);
}

// set_target

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct TargetContractId0 {
    target_id: ContractId,
}

impl TargetContractId0 {
    pub fn new(target_id: ContractId) -> Self {
        Self { target_id }
    }
}

impl Transaction for TargetContractId0 {
    const NAME: &'static str = "set_target";
    type Return = ();
}

// querying of caller-1

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CallData1 {
    sender: ContractId,
}

impl Query for CallData1 {
    const NAME: &'static str = "do_call";
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
    fn do_call(written: u32) -> u32 {
        let mut store = AbiStore;

        let state =
            unsafe { archived_root::<Caller>(&SCRATCH[..written as usize]) };

        let mut state: Caller = (state).deserialize(&mut store).unwrap();

        rusk_uplink::debug!("caller: calling state target 'do_call' with param: callee");
        let call_data = CallData1 {
            sender: rusk_uplink::callee(),
        };
        let ret = rusk_uplink::query::<CallData1>(
            &state.target_address,
            call_data,
            0,
        )
        .unwrap();

        let res: <CallData1 as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<CallData1 as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }

    #[no_mangle]
    fn set_target(written: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let (state, target) = unsafe {
            archived_root::<(Caller, TargetContractId0)>(
                &SCRATCH[..written as usize],
            )
        };

        let mut state: Caller = (state).deserialize(&mut store).unwrap();
        let target: TargetContractId0 =
            (target).deserialize(&mut store).unwrap();

        state.set_target(target.target_id);
        rusk_uplink::debug!("setting state.set_target to: {:?}", target.target_id);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<TargetContractId0 as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
                <<TargetContractId0 as Transaction>::Return as Archive>::Archived,
            >();

        [state_len as u32, return_len as u32]
    }
};
