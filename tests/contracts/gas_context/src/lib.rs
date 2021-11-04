// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// transaction ids
pub const COMPUTE: u8 = 0;
// query ids
pub const READ_GAS_LIMIT: u8 = 1;

pub const GAS_LIMITS_SIZE: usize = 10;

#[derive(Clone, Canon, Debug)]
pub struct GasContextData {
    gas_limits: [u64; GAS_LIMITS_SIZE],
}

impl GasContextData {
    pub fn new() -> GasContextData {
        GasContextData{ gas_limits: [0; GAS_LIMITS_SIZE]}
    }
    pub fn nth_limit(&self, n: usize) -> u64 {
        self.gas_limits[n]
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

    impl GasContextData {
        pub fn compute(&mut self, n: u64) -> u64 {
            if n < 1 {
                dusk_abi::debug!("here 1 {}", n);
                0
            } else {
                let callee = dusk_abi::callee();
                dusk_abi::debug!("here 2 {}", n);

                dusk_abi::transact::<_, u64, Self>(self, &callee, &(COMPUTE, n - 1))
                    .unwrap();

                let gas_left = dusk_abi::gas_left();

                self.gas_limits[n as usize] = gas_left;

                dusk_abi::debug!("storing gas left {} at index {}", gas_left, n);
                n
            }
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self
        let slf = GasContextData::decode(&mut source)?;
        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            READ_GAS_LIMIT => {
                let input = u64::decode(&mut source)?;
                let ret = slf.nth_limit(input as usize);
                dusk_abi::debug!("reading gas limit for {}", input);
                dusk_abi::debug!("table is {:?}", slf.gas_limits);

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
        let mut source = Source::new(&bytes[..]);

        // read self.
        let mut slf = GasContextData::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            // read_value (&Self) -> i32
            COMPUTE => {
                // read arg
                let input = u64::decode(&mut source)?;
                dusk_abi::debug!("input is {}", input);
                let ret: u64 = slf.compute(input);
                dusk_abi::debug!("table is {:?}", slf.gas_limits);

                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);
                ReturnValue::from_canon(&ret).encode(&mut sink);
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
