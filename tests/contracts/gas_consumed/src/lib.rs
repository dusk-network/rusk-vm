// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};
use rusk_uplink_derive::{apply, execute, init, query, state, transaction};

extern crate alloc;

#[state]
pub struct GasConsumed {
    value: i32,
}

#[init]
fn init() {}

impl GasConsumed {
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

#[query]
pub struct GasConsumedValueQuery;

impl Query for GasConsumedValueQuery {
    const NAME: &'static str = "value";
    type Return = i32;
}

#[query]
pub struct GasConsumedQuery;

impl Query for GasConsumedQuery {
    const NAME: &'static str = "get_gas_consumed";
    type Return = (u32, u32);
}

#[execute(name = "value")]
impl Execute<GasConsumedValueQuery> for GasConsumed {
    fn execute(&self, _: GasConsumedValueQuery, _: StoreContext) -> i32 {
        self.value()
    }
}

#[execute(name = "get_gas_consumed")]
impl Execute<GasConsumedQuery> for GasConsumed {
    fn execute(&self, _: GasConsumedQuery, _: StoreContext) -> (u32, u32) {
        (
            rusk_uplink::gas_consumed() as u32,
            rusk_uplink::gas_left() as u32,
        )
    }
}

#[transaction]
pub struct GasConsumedIncrement;

impl Transaction for GasConsumedIncrement {
    const NAME: &'static str = "increment";
    type Return = ();
}

#[transaction]
pub struct GasConsumedDecrement;

impl Transaction for GasConsumedDecrement {
    const NAME: &'static str = "decrement";
    type Return = ();
}

#[apply(name = "increment")]
impl Apply<GasConsumedIncrement> for GasConsumed {
    fn apply(&mut self, _: GasConsumedIncrement, _: StoreContext) {
        self.increment()
    }
}

#[apply(name = "decrement")]
impl Apply<GasConsumedDecrement> for GasConsumed {
    fn apply(&mut self, _: GasConsumedDecrement, _: StoreContext) {
        self.decrement()
    }
}
