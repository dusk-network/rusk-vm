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

    #[transaction]
    pub fn adjust(state: &mut Counter, adjust: Adjust, _store: StoreRef<OffsetLen>) {
        state.adjust(adjust.by);
    }

    #[query]
    pub fn read_value(state: &Counter, _: ReadValue, _store: StoreRef<OffsetLen>) -> i32 {
        state.read_value()
    }

    #[query]
    pub fn xor_values(state: &Counter, xv: XorValues, _store: StoreRef<OffsetLen>) -> i32 {
        state.xor_values(xv.a, xv.b)
    }

    #[query]
    pub fn is_even(state: &Counter, _: IsEven, _store: StoreRef<OffsetLen>) -> bool {
        state.is_even()
    }

    #[transaction]
    pub fn increment(state: &mut Counter, increment: Increment, _store: StoreRef<OffsetLen>) {
        state.increment();
    }

    #[transaction]
    pub fn decrement(state: &mut Counter, decrement: Decrement, _store: StoreRef<OffsetLen>) {
        state.decrement();
    }

    #[transaction]
    pub fn compare_and_swap(state: &mut Counter, compare_and_swap: CompareAndSwap, _store: StoreRef<OffsetLen>) -> bool {
        if state.value == compare_and_swap.expected {
            state.value = compare_and_swap.new;
            true
        } else {
            false
        }
    }
};
