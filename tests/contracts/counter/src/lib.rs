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
use rusk_uplink::{Query, Transaction};
use rusk_uplink::{get_state_and_arg, get_state, t_return, q_return};

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
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::{AbiStore, StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    #[no_mangle]
    fn adjust(written_state: u32, written_data: u32) -> [u32; 2] {
        let (mut state, arg): (Counter, Adjust) = unsafe { get_state_and_arg(written_state, written_data, &SCRATCH) };

        state.adjust(arg.by);

        unsafe { t_return(&state, &(), &mut SCRATCH)}
    }

    #[no_mangle]
    fn read_value(written_state: u32, _written_data: u32) -> u32 {
        let state: Counter = unsafe { get_state(written_state, &SCRATCH) };

        let ret = state.read_value();

        unsafe { q_return(&ret, &mut SCRATCH) }
    }

    #[no_mangle]
    fn xor_values(written_state: u32, written_data: u32) -> u32 {
        let (mut state, arg): (Counter, XorValues) = unsafe { get_state_and_arg(written_state, written_data, &SCRATCH) };

        let ret = state.xor_values(arg.a, arg.b);

        unsafe { q_return(&ret, &mut SCRATCH) }
    }

    #[no_mangle]
    fn is_even(written_state: u32, _written_data: u32) -> u32 {
        let mut state: Counter = unsafe { get_state(written_state, &SCRATCH) };

        let ret = state.is_even();

        unsafe { q_return(&ret, &mut SCRATCH) }
    }

    #[no_mangle]
    fn increment(written_state: u32, _written_data: u32) -> [u32; 2] {
        let mut state: Counter = unsafe { get_state(written_state, &SCRATCH) };

        state.increment();

        unsafe { t_return(&state, &(), &mut SCRATCH)}
    }

    #[no_mangle]
    fn decrement(written_state: u32, _written_data: u32) -> [u32; 2] {
        let mut state: Counter = unsafe { get_state(written_state, &SCRATCH) };

        state.decrement();

        unsafe { t_return(&state, &(), &mut SCRATCH)}
    }

    #[no_mangle]
    fn compare_and_swap(written_state: u32, written_data: u32) -> [u32; 2] {
        let (mut state, arg): (Counter, CompareAndSwap) = unsafe { get_state_and_arg(written_state, written_data, &SCRATCH) };

        let res = state.compare_and_swap(arg.expected, arg.new);

        unsafe { t_return(&state, &res, &mut SCRATCH)}
    }
};
