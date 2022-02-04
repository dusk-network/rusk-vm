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
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};
use rusk_uplink_derive::{apply, execute, query, state, transaction};

extern crate alloc;

#[state]
pub struct GasConsumed {
    value: i32,
}

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
#[query]
pub struct GasConsumedQuery;

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
#[transaction]
pub struct GasConsumedDecrement;

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
