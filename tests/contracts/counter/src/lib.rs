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
use rusk_uplink_derive::{query, transaction, argument, state};


#[state]
pub struct Counter {
    #[new(value = "0xffffffff")] junk: u32,
    value: i32,
}

#[argument]
pub struct ReadValue;

#[argument]
pub struct XorValues {
    a: i32,
    b: i32,
}

#[argument]
pub struct IsEven;

#[argument]
pub struct Increment;

#[argument]
pub struct Decrement;

#[argument]
pub struct Adjust {
    by: i32,
}

#[argument]
pub struct CompareAndSwap {
    expected: i32,
    new: i32,
}

#[transaction(name="adjust")]
impl Apply<Adjust> for Counter {
    fn apply(
        &mut self,
        arg: Adjust,
        _: StoreContext,
    ) {
        self.adjust(arg.by);
    }
}

#[query(name="read_value")]
impl Execute<ReadValue> for Counter {
    fn execute(
        &self,
        _: ReadValue,
        _: StoreContext,
    ) -> i32 {
        self.value
    }
}

#[query(name="xor_values")]
impl Execute<XorValues> for Counter {
    fn execute(
        &self,
        arg: XorValues,
        _: StoreContext,
    ) -> i32 {
        self.xor_values(arg.a, arg.b)
    }
}

#[query(name="is_even")]
impl Execute<IsEven> for Counter {
    fn execute(
        &self,
        _: IsEven,
        _: StoreContext,
    ) -> bool {
        self.is_even()
    }
}

#[transaction(name="increment")]
impl Apply<Increment> for Counter {
    fn apply(
        &mut self,
        _: Increment,
        _: StoreContext,
    ) {
        self.increment();
    }
}

#[transaction(name="decrement")]
impl Apply<Decrement> for Counter {
    fn apply(
        &mut self,
        _: Decrement,
        _: StoreContext,
    ) {
        self.decrement();
    }
}

#[transaction(name="compare_and_swap")]
impl Apply<CompareAndSwap> for Counter {
    fn apply(
        &mut self,
        arg: CompareAndSwap,
        _: StoreContext,
    ) -> bool {
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
