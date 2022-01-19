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
use rusk_uplink::{Query, Transaction, Apply, Execute, StoreContext};

extern crate alloc;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumed {
    value: i32,
}

impl GasConsumed {
    pub fn new(value: i32) -> Self {
        GasConsumed { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn increment(&mut self) {
        self.value += 1
    }
    pub fn decrement(&mut self) {
        self.value -= 1
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumedValueQuery;
#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumedQuery;

impl Query for GasConsumedValueQuery {
    const NAME: &'static str = "value";
    type Return = i32;
}

impl Query for GasConsumedQuery {
    const NAME: &'static str = "get_gas_consumed";
    type Return = (u32, u32);
}

impl Execute<GasConsumedValueQuery> for GasConsumed {
    fn execute(
        &self,
        _: &GasConsumedValueQuery,
        _: StoreContext,
    ) -> <GasConsumedValueQuery as Query>::Return {
        self.value()
    }
}

impl Execute<GasConsumedQuery> for GasConsumed {
    fn execute(
        &self,
        _: &GasConsumedQuery,
        _: StoreContext,
    ) -> <GasConsumedQuery as Query>::Return {
        (
            rusk_uplink::gas_consumed() as u32,
            rusk_uplink::gas_left() as u32,
        )
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumedIncrement;
#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumedDecrement;

impl Transaction for GasConsumedIncrement {
    const NAME: &'static str = "increment";
    type Return = ();
}

impl Transaction for GasConsumedDecrement {
    const NAME: &'static str = "decrement";
    type Return = ();
}

impl Apply<GasConsumedIncrement> for GasConsumed {
    fn apply(
        &mut self,
        _: &GasConsumedIncrement,
        _: StoreContext
    ) -> <GasConsumedIncrement as Transaction>::Return {
        self.increment()
    }
}

impl Apply<GasConsumedDecrement> for GasConsumed {
    fn apply(
        &mut self,
        _: &GasConsumedDecrement,
        _: StoreContext
    ) -> <GasConsumedDecrement as Transaction>::Return {
        self.decrement()
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::{get_state_and_arg, q_return, t_return, query_state_arg_fun, transaction_state_arg_fun};
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    query_state_arg_fun!(value, GasConsumed, GasConsumedValueQuery);

    query_state_arg_fun!(get_gas_consumed, GasConsumed, GasConsumedQuery);

    transaction_state_arg_fun!(increment, GasConsumed, GasConsumedIncrement);

    transaction_state_arg_fun!(decrement, GasConsumed, GasConsumedDecrement);
};
