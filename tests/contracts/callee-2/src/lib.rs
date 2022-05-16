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
)]

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{ContractId, Execute, Query, StoreContext};
use rusk_uplink_derive::{execute, query, state};
extern crate alloc;

#[state]
pub struct Callee2State;

#[query(new = false)]
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

#[execute(name = "get")]
impl Execute<Callee2Query> for Callee2State {
    fn execute(&self, callee2: Callee2Query, _: StoreContext) -> Callee2Return {
        assert_eq!(callee2.sender, rusk_uplink::caller(), "Expected Caller");

        rusk_uplink::debug!(
            "callee-2: returning sender_sender, sender from params and callee"
        );

        Callee2Return {
            sender_sender: callee2.sender_sender,
            sender: callee2.sender,
            callee: rusk_uplink::callee(),
        }
    }
}

#[query]
pub struct Callee2Return {
    sender_sender: ContractId,
    sender: ContractId,
    callee: ContractId,
}
