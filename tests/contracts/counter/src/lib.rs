// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical::Canon;
use canonical_derive::Canon;

// qulery ids
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

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Id32, Store};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

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

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(&bytes[..], &bs);

        // read self.
        let slf: Counter = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            READ_VALUE => {
                let ret = slf.read_value();

                let r = {
                    // return value
                    let wrapped_return = ReturnValue::from_canon(&ret, &bs)?;

                    dusk_abi::debug!("wrapped return {:?}", wrapped_return);

                    let mut sink = ByteSink::new(&mut bytes[..], &bs);

                    Canon::<BS>::write(&wrapped_return, &mut sink)
                };
                dusk_abi::debug!("memory bytes {:?}", &bytes[..32]);

                r
            }
            // xor_values (&Self, a: i32, b: i32) -> i32
            XOR_VALUES => {
                let (a, b): (i32, i32) = Canon::<BS>::read(&mut source)?;
                let ret = slf.xor_values(a, b);
                let mut sink = ByteSink::new(&mut bytes[..], &bs);
                Canon::<BS>::write(&ret, &mut sink)?;
                Ok(())
            }
            // is_even (&Self) -> bool
            IS_EVEN => {
                let ret = slf.is_even();
                let mut sink = ByteSink::new(&mut bytes[..], &bs);

                Canon::<BS>::write(&ret, &mut sink)?;
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

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(bytes, &bs);

        // read self.
        let mut slf: Counter = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let tid: u8 = Canon::<BS>::read(&mut source)?;
        match tid {
            // increment (&Self)
            INCREMENT => {
                slf.increment();
                let mut sink = ByteSink::new(&mut bytes[..], &bs);
                // return new state
                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs)?,
                    &mut sink,
                )?;

                // return value
                Canon::<BS>::write(
                    &ReturnValue::from_canon(&(), &bs)?,
                    &mut sink,
                )
            }
            DECREMENT => {
                // no args
                slf.decrement();
                let mut sink = ByteSink::new(&mut bytes[..], &bs);

                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs),
                    &mut sink,
                )?;

                // no return value
                Ok(())
            }
            ADJUST => {
                // read arg
                let by: i32 = Canon::<BS>::read(&mut source)?;
                slf.adjust(by);
                let mut sink = ByteSink::new(&mut bytes[..], &bs);

                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs),
                    &mut sink,
                )?;

                // no return value
                Ok(())
            }
            COMPARE_AND_SWAP => {
                // read multiple args
                let (a, b): (i32, i32) = Canon::<BS>::read(&mut source)?;
                let res = slf.compare_and_swap(a, b);
                let mut sink = ByteSink::new(&mut bytes[..], &bs);

                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs),
                    &mut sink,
                )?;

                // return result
                Canon::<BS>::write(&res, &mut sink)
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    include!("../../../../dusk-abi/src/panic_include.rs");
}
