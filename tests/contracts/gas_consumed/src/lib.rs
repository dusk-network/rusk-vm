// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const GAS_CONSUMED: u8 = 0;
pub const VALUE: u8 = 1;

// transaction ids
pub const INCREMENT: u8 = 0;
pub const DECREMENT: u8 = 1;

#[derive(Clone, Canon, Debug, Default)]
pub struct GasConsumed {
    value: i32,
}
impl GasConsumed {
    pub fn new(value: i32) -> Self {
        GasConsumed { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn increment(&mut self) {
        self.value += 1
    }
    pub fn decrement(&mut self) {
        self.value -= 1
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 64;

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self.
        let slf = GasConsumed::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            VALUE => {
                let ret = slf.value();

                let mut sink = Sink::new(&mut bytes[..]);

                ReturnValue::from_canon(&ret).encode(&mut sink);
                Ok(())
            }
            GAS_CONSUMED => {
                let mut ret = (
                    dusk_abi::gas_consumed(),
                    dusk_abi::gas_left(),
                    0,
                    0,
                    0,
                    0,
                );

                let mut sink = Sink::new(&mut bytes[..]);

                let gas_consumed_before = dusk_abi::gas_consumed();
                let gas_left_before = dusk_abi::gas_left();
                let x = 5i32;
                let _y = x.pow(5);
                let gas_consumed_after = dusk_abi::gas_consumed();
                let gas_left_after = dusk_abi::gas_left();
                ret.2 = gas_consumed_before as u64;
                ret.3 = gas_consumed_after as u64;
                ret.4 = gas_left_before as u64;
                ret.5 = gas_left_after as u64;

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
        let mut slf = GasConsumed::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            INCREMENT => {
                slf.increment();

                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);

                Ok(())
            }
            DECREMENT => {
                slf.decrement();

                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);

                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = transaction(bytes);
    }
}
