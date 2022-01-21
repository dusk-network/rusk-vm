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

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Counter {
    junk: u32,
    value: i32,
}

impl Counter {
    pub fn new(value: i32) -> Self {
        Counter {
            junk: 0xffffffff,
            value,
        }
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct ReadValue;

impl Query for ReadValue {
    const NAME: &'static str = "read_value";
    type Return = i32;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct XorValues {
    a: i32,
    b: i32,
}

impl XorValues {
    pub fn new(a: i32, b: i32) -> Self {
        Self { a, b }
    }
}

impl Query for XorValues {
    const NAME: &'static str = "xor_values";
    type Return = i32;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct IsEven;

impl Query for IsEven {
    const NAME: &'static str = "is_even";
    type Return = bool;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Increment;

impl Transaction for Increment {
    const NAME: &'static str = "increment";
    type Return = ();
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Decrement;

impl Transaction for Decrement {
    const NAME: &'static str = "decrement";
    type Return = ();
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Adjust {
    by: i32,
}

impl Adjust {
    pub fn new(by: i32) -> Self {
        Self { by }
    }
}

impl Transaction for Adjust {
    const NAME: &'static str = "adjust";
    type Return = ();
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct CompareAndSwap {
    expected: i32,
    new: i32,
}

impl CompareAndSwap {
    pub fn new(expected: i32, new: i32) -> Self {
        Self { expected, new }
    }
}

impl Transaction for CompareAndSwap {
    const NAME: &'static str = "compare_and_swap";
    type Return = bool;
}

impl Apply<Adjust> for Counter {
    fn apply(
        &mut self,
        arg: &Adjust,
        _: StoreContext,
    ) -> <Adjust as Transaction>::Return {
        self.adjust(arg.by);
    }
}

impl Execute<ReadValue> for Counter {
    fn execute(
        &self,
        _: &ReadValue,
        _: StoreContext,
    ) -> <ReadValue as Query>::Return {
        self.value
    }
}

impl Execute<XorValues> for Counter {
    fn execute(
        &self,
        arg: &XorValues,
        _: StoreContext,
    ) -> <XorValues as Query>::Return {
        self.xor_values(arg.a, arg.b)
    }
}

impl Execute<IsEven> for Counter {
    fn execute(
        &self,
        _: &IsEven,
        _: StoreContext,
    ) -> <IsEven as Query>::Return {
        self.is_even()
    }
}

impl Apply<Increment> for Counter {
    fn apply(
        &mut self,
        _: &Increment,
        _: StoreContext,
    ) -> <Increment as Transaction>::Return {
        self.increment();
    }
}

impl Apply<Decrement> for Counter {
    fn apply(
        &mut self,
        _: &Decrement,
        _: StoreContext,
    ) -> <Decrement as Transaction>::Return {
        self.decrement();
    }
}

impl Apply<CompareAndSwap> for Counter {
    fn apply(
        &mut self,
        arg: &CompareAndSwap,
        _: StoreContext,
    ) -> <CompareAndSwap as Transaction>::Return {
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

#[cfg(target_family = "wasm")]
const _: () = {
    use rusk_uplink::framing_imports;
    framing_imports!();

    scratch_memory!(512);

    t_handler!(adjust, Counter, Adjust);

    #[no_mangle]
    fn read_value(written_state: u32, _written_data: u32) -> u32 {
        let state: Counter = unsafe { get_state(written_state, &SCRATCH) };

        let ret = state.read_value();

        unsafe { q_return(&ret, &mut SCRATCH) }
    }
    // q_handler!(read_value, Counter, ReadValue);

    q_handler!(xor_values, Counter, XorValues);

    q_handler!(is_even, Counter, IsEven);

    #[no_mangle]
    fn increment(written_state: u32, _written_data: u32) -> [u32; 2] {
        let mut state: Counter = unsafe { get_state(written_state, &SCRATCH) };

        state.increment();

        unsafe { t_return(&state, &(), &mut SCRATCH) }
    }
    // t_handler!(increment, Counter, Increment);

    t_handler!(decrement, Counter, Decrement);

    t_handler!(compare_and_swap, Counter, CompareAndSwap);
};
