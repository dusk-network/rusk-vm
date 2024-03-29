// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{
    Apply, ContractId, Execute, Query, StoreContext, Transaction,
};
use rusk_uplink_derive::{apply, execute, init, query, state, transaction};
extern crate alloc;

#[state(new = false)]
pub struct CallerState {
    target_address: ContractId,
}
#[init]
fn init() {}

impl CallerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, address: ContractId) {
        self.target_address = address;
    }
}

#[query]
pub struct CallerQuery;

impl Query for CallerQuery {
    const NAME: &'static str = "call";
    type Return = <Callee1Query as Query>::Return;
}

#[execute(name = "call")]
impl Execute<CallerQuery> for CallerState {
    fn execute(
        &self,
        _: CallerQuery,
        store: StoreContext,
    ) -> <Callee1Query as Query>::Return {
        rusk_uplink::debug!(
            "caller: calling state target 'call' with param: callee"
        );
        let call_data = Callee1Query {
            sender: rusk_uplink::callee(),
        };
        rusk_uplink::query::<Callee1Query>(
            &self.target_address,
            call_data,
            0,
            store,
        )
        .unwrap()
    }
}

#[transaction]
pub struct CallerTransaction {
    target_id: ContractId,
}

impl Transaction for CallerTransaction {
    const NAME: &'static str = "set_target";
    type Return = ();
}

#[apply(name = "set_target")]
impl Apply<CallerTransaction> for CallerState {
    fn apply(&mut self, target: CallerTransaction, _: StoreContext) {
        self.set_target(target.target_id);
        rusk_uplink::debug!(
            "setting state.set_target to: {:?}",
            target.target_id
        );
    }
}

#[query]
pub struct Callee1Query {
    sender: ContractId,
}

impl Query for Callee1Query {
    const NAME: &'static str = "call";
    type Return = ([u8; 32], [u8; 32], [u8; 32]);
}
