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

#[state]
pub struct Counter {
    #[new(value = "0xffffffff")]
    junk: u32,
    value: i32,
}

#[query]
pub struct ReadValue;

impl Query for ReadValue {
    const NAME: &'static str = "read_value";
    type Return = i32;
}

#[query]
pub struct XorValues {
    a: i32,
    b: i32,
}

impl Query for XorValues {
    const NAME: &'static str = "xor_values";
    type Return = i32;
}

#[query]
pub struct IsEven;

impl Query for IsEven {
    const NAME: &'static str = "is_even";
    type Return = bool;
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

#[transaction]
pub struct Adjust {
    by: i32,
}

impl Transaction for Adjust {
    const NAME: &'static str = "adjust";
    type Return = ();
}

#[transaction]
pub struct CompareAndSwap {
    expected: i32,
    new: i32,
}

impl Transaction for CompareAndSwap {
    const NAME: &'static str = "compare_and_swap";
    type Return = bool;
}

#[apply(name = "adjust")]
impl Apply<Adjust> for Counter {
    fn apply(&mut self, arg: Adjust, _: StoreContext) {
        self.adjust(arg.by);
    }
}

#[execute(name = "read_value")]
impl Execute<ReadValue> for Counter {
    fn execute(&self, _: ReadValue, _: StoreContext) -> i32 {
        self.value
    }
}

#[execute(name = "xor_values")]
impl Execute<XorValues> for Counter {
    fn execute(&self, arg: XorValues, _: StoreContext) -> i32 {
        self.xor_values(arg.a, arg.b)
    }
}

#[execute(name = "is_even")]
impl Execute<IsEven> for Counter {
    fn execute(&self, _: IsEven, _: StoreContext) -> bool {
        self.is_even()
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

#[apply(name = "compare_and_swap")]
impl Apply<CompareAndSwap> for Counter {
    fn apply(&mut self, arg: CompareAndSwap, _: StoreContext) -> bool {
        self.compare_and_swap(arg.expected, arg.new)
    }
}

impl Counter {
    pub fn read_value(&self) -> i32 {
        self.value
    }

    pub fn xor_values(&self, a: i32, b: i32) -> i32 {
        self.value ^ a ^ b
    }

    pub fn is_even(&self) -> bool {
        self.value % 2 == 0
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn decrement(&mut self) {
        self.value -= 1;
    }

    pub fn adjust(&mut self, by: i32) {
        self.value += by;
    }

    pub fn compare_and_swap(&mut self, expected: i32, new: i32) -> bool {
        if self.value == expected {
            self.value = new;
            true
        } else {
            false
        }
    }
}
