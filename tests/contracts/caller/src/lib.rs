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
use rusk_uplink::{ContractId, Query, Transaction, Apply, Execute, StoreContext};
use rusk_uplink_derive::{query, transaction, state, argument};
extern crate alloc;

#[state(new=false)]
pub struct CallerState {
    target_address: ContractId,
}

impl CallerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, address: ContractId) {
        self.target_address = address;
    }
}

#[argument]
pub struct CallerQuery;

#[query(name="call")]
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


#[argument]
pub struct CallerTransaction {
    target_id: ContractId,
}

#[transaction(name="set_target")]
impl Apply<CallerTransaction> for CallerState {
    fn apply(
        &mut self,
        target: CallerTransaction,
        _: StoreContext,
    ) {
        self.set_target(target.target_id);
        rusk_uplink::debug!(
            "setting state.set_target to: {:?}",
            target.target_id
        );
    }
}


#[argument]
pub struct Callee1Query {
    sender: ContractId,
}

impl Query for Callee1Query {
    const NAME: &'static str = "call";
    type Return = ([u8; 32], [u8; 32], [u8; 32]);
}
