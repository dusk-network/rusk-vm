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
use rusk_uplink_derive::{query, transaction};

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

#[query(name="value")]
impl Execute<GasConsumedValueQuery> for GasConsumed {
    fn execute(
        &self,
        _: GasConsumedValueQuery,
        _: StoreContext,
    ) -> i32 {
        self.value()
    }
}

#[query(name="get_gas_consumed")]
impl Execute<GasConsumedQuery> for GasConsumed {
    fn execute(
        &self,
        _: GasConsumedQuery,
        _: StoreContext,
    ) -> (u32, u32) {
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

#[transaction(name="increment")]
impl Apply<GasConsumedIncrement> for GasConsumed {
    fn apply(
        &mut self,
        _: GasConsumedIncrement,
        _: StoreContext,
    ) {
        self.increment()
    }
}

#[transaction(name="decrement")]
impl Apply<GasConsumedDecrement> for GasConsumed {
    fn apply(
        &mut self,
        _: GasConsumedDecrement,
        _: StoreContext,
    ) {
        self.decrement()
    }
}
