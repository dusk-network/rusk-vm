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

#[state]
pub struct Counter {
    value: u32,
}
#[init]
fn init() {}

#[query]
pub struct ReadCount;

impl Query for ReadCount {
    const NAME: &'static str = "read";
    type Return = u32;
}

#[transaction]
pub struct Increment(pub u32);

impl Transaction for Increment {
    const NAME: &'static str = "incr";
    type Return = ();
}

#[execute(name = "read")]
impl Execute<ReadCount> for Counter {
    fn execute(&self, _: ReadCount, _: StoreContext) -> u32 {
        self.value
    }
}

#[apply(name = "incr")]
impl Apply<Increment> for Counter {
    fn apply(&mut self, t: Increment, _: StoreContext) {
        self.value += t.0;
    }
}
