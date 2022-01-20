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
use rusk_uplink::{ContractId, Query};
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
    use rusk_uplink::{get_state_and_arg, q_return};

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    #[no_mangle]
    fn get(written_state: u32, written_data: u32) -> u32 {
        let (_state, callee2): (Callee2State, Callee2Query) =
            unsafe { get_state_and_arg(written_state, written_data, &SCRATCH) };

        assert_eq!(callee2.sender, rusk_uplink::caller(), "Expected Caller");

        rusk_uplink::debug!(
            "callee-2: returning sender_sender, sender from params and callee"
        );

        let ret = Callee2Return {
            sender_sender: callee2.sender_sender,
            sender: callee2.sender,
            callee: rusk_uplink::callee(),
        };

        unsafe { q_return(&ret, &mut SCRATCH) }
    }
};
