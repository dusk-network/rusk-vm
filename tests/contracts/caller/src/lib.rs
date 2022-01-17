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
extern crate alloc;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
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

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct CallerQuery;

impl Query for CallerQuery {
    const NAME: &'static str = "call";
    type Return = <Callee1Query as Query>::Return;
}

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


#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct CallerTransaction {
    target_id: ContractId,
}

impl CallerTransaction {
    pub fn new(target_id: ContractId) -> Self {
        Self { target_id }
    }
}

impl Transaction for CallerTransaction {
    const NAME: &'static str = "set_target";
    type Return = ();
}

impl Apply<CallerTransaction> for CallerState {
    fn apply(
        &mut self,
        target: CallerTransaction,
        _: StoreContext,
    ) -> <CallerTransaction as Transaction>::Return {
        self.set_target(target.target_id);
        rusk_uplink::debug!(
            "setting state.set_target to: {:?}",
            target.target_id
        );
    }
}


#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Callee1Query {
    sender: ContractId,
}

impl Query for Callee1Query {
    const NAME: &'static str = "call";
    type Return = ([u8; 32], [u8; 32], [u8; 32]);
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    scratch_memory!(512);

    q_handler!(call, CallerState, CallerQuery);

    t_handler!(set_target, CallerState, CallerTransaction);
};
