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
use rusk_uplink::{ContractId, Query, Transaction, Execute, Apply, StoreContext};

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

impl Apply<Callee1Transaction> for Callee1State {
    fn apply(
        &mut self,
        target: &Callee1Transaction,
        _: StoreContext,
    ) -> <Callee1Transaction as Transaction>::Return {
        self.set_target(target.target_id);
        rusk_uplink::debug!(
            "setting state.set_target to: {:?}",
            target.target_id
        );
    }
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

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct SenderParameter {
    sender_id: ContractId,
}

impl Query for SenderParameter {
    const NAME: &'static str = "call";
    type Return = <Callee2Query as Query>::Return;
}

impl Execute<SenderParameter> for Callee1State {
    fn execute(
        &self,
        sender: &SenderParameter,
        store: StoreContext,
    ) -> <SenderParameter as Query>::Return {
        assert_eq!(sender.sender_id, rusk_uplink::caller(), "Expected Caller");
        rusk_uplink::debug!("callee-1: calling state target 'get' with params: sender from param and callee");
        let call_data = Callee2Query {
            sender: sender.sender_id,
            callee: rusk_uplink::callee(),
        };
        rusk_uplink::query::<Callee2Query>(
            &self.target_address,
            call_data,
            0,
            store,
        )
        .unwrap()
    }
}


#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    query_state_arg_fun!(call, Callee1State, SenderParameter);

    transaction_state_arg_fun!(set_target, Callee1State, Callee1Transaction);
};
