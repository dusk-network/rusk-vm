// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const READ_VALUE: u8 = 0;
pub const XOR_VALUES: u8 = 1;
pub const IS_EVEN: u8 = 2;

// transaction ids
pub const INCREMENT: u8 = 0;
pub const DECREMENT: u8 = 1;
pub const ADJUST: u8 = 2;
pub const COMPARE_AND_SWAP: u8 = 3;

#[derive(Clone, Canon, Debug)]
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

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

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

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self.
        let slf = Counter::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            READ_VALUE => {
                let ret = slf.read_value();

                let mut sink = Sink::new(&mut bytes[..]);

                ReturnValue::from_canon(&ret).encode(&mut sink);
                Ok(())
            }
            // xor_values (&Self, a: i32, b: i32) -> i32
            XOR_VALUES => {
                let (a, b): (i32, i32) = Canon::decode(&mut source)?;
                let ret = slf.xor_values(a, b);
                let mut sink = Sink::new(&mut bytes[..]);
                ReturnValue::from_canon(&ret).encode(&mut sink);
                Ok(())
            }
            // is_even (&Self) -> bool
            IS_EVEN => {
                let ret = slf.is_even();
                let mut sink = Sink::new(&mut bytes[..]);

                ReturnValue::from_canon(&ret).encode(&mut sink);
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = query(bytes);
    }

    fn transaction(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(bytes);

        // read self.
        let mut slf = Counter::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            // increment (&Self)
            INCREMENT => {
                slf.increment();
                let mut sink = Sink::new(&mut bytes[..]);
                // return new state
                ContractState::from_canon(&slf).encode(&mut sink);

                // return value ()
                ReturnValue::from_canon(&()).encode(&mut sink);
                Ok(())
            }
            DECREMENT => {
                // no args
                slf.decrement();
                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);

                // return value ()
                ReturnValue::from_canon(&()).encode(&mut sink);
                Ok(())
            }
            ADJUST => {
                // read arg
                let by = i32::decode(&mut source)?;
                slf.adjust(by);
                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);

                // return value ()
                ReturnValue::from_canon(&()).encode(&mut sink);
                Ok(())
            }
            COMPARE_AND_SWAP => {
                // read multiple args
                let (a, b): (i32, i32) = Canon::decode(&mut source)?;
                let res = slf.compare_and_swap(a, b);
                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);

                // return value ()
                ReturnValue::from_canon(&res).encode(&mut sink);
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    #[no_mangle]
    fn pre_t(num: i32) -> i32 {
        num * num
    }
}
