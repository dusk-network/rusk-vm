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

const FILLER: &str = include_str!("filler.dat");

#[state]
pub struct Counter {
    #[new(value = "0xffffffff")]
    junk: u32,
    value: i32,
}
#[init]
fn init() {}

#[query]
pub struct ReadValue;

impl Query for ReadValue {
    const NAME: &'static str = "read_value";
    type Return = i32;
}

#[transaction]
pub struct Increment;

impl Transaction for Increment {
    const NAME: &'static str = "increment";
    type Return = ();
}

#[transaction]
pub struct Decrement;

impl Transaction for Decrement {
    const NAME: &'static str = "decrement";
    type Return = ();
}

#[execute(name = "read_value")]
impl Execute<ReadValue> for Counter {
    fn execute(&self, _: ReadValue, _: StoreContext) -> i32 {
        let _filler = FILLER.repeat(2);
        self.value
    }
}

#[apply(name = "increment")]
impl Apply<Increment> for Counter {
    fn apply(&mut self, _: Increment, _: StoreContext) {
        self.increment();
    }
}

#[apply(name = "decrement")]
impl Apply<Decrement> for Counter {
    fn apply(&mut self, _: Decrement, _: StoreContext) {
        self.decrement();
    }
}

impl Counter {
    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn decrement(&mut self) {
        self.value -= 1;
    }
}
