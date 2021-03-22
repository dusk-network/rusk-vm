// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical::Canon;
use canonical_derive::Canon;

// query ids
pub const GAS_CONSUMED: u8 = 0;
pub const READ_VALUE: u8 = 1;

// transaction ids
pub const INCREMENT: u8 = 0;
pub const DECREMENT: u8 = 1;

#[derive(Clone, Canon, Debug)]
pub struct GasConsumed {
    junk: u32,
    value: i32,
}

impl GasConsumed {
    pub fn new(value: i32) -> Self {
        GasConsumed {
            junk: 0xffffffff,
            value,
        }
    }
}

#[cfg(not(feature = "host"))]
mod hosted {

    extern crate alloc;

    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl GasConsumed {
        pub fn read_value(&self) -> i32 {
            self.value
        }

        pub fn increment(&mut self) {
            self.value += 1;
        }

        pub fn decrement(&mut self) {
            self.value -= 1;
        }

        pub fn gas_consumed(&self) -> u64 {
            dusk_abi::gas_consumed()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(&bytes[..], &bs);

        // read self.
        let slf: GasConsumed = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            GAS_CONSUMED => {
                let ret = slf.gas_consumed();

                let r = {
                    // return value
                    let wrapped_return = ReturnValue::from_canon(&ret, &bs)?;

                    let mut sink = ByteSink::new(&mut bytes[..], &bs);

                    Canon::<BS>::write(&wrapped_return, &mut sink)
                };

                r
            }

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
        let mut slf: GasConsumed = Canon::<BS>::read(&mut source)?;
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
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }
}
