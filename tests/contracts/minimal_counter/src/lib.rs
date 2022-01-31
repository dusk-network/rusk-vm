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

#[derive(Clone, Debug, Archive, Deserialize, Serialize)]
pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn new(value: u32) -> Self {
        Counter { value }
    }
}

#[derive(Archive, Serialize, Debug, Deserialize, Clone)]
pub struct ReadCount;

#[derive(Archive, Serialize, Debug, Deserialize, Clone)]
pub struct Increment(pub u32);

#[query(name="read")]
impl Execute<ReadCount> for Counter {
    fn execute(
        &self,
        _: ReadCount,
        _: StoreContext,
    ) -> u32 {
        self.value
    }
}

#[transaction(name="incr")]
impl Apply<Increment> for Counter {
    fn apply(
        &mut self,
        t: Increment,
        _: StoreContext,
    ) {
        self.value += t.0;
    }
}
