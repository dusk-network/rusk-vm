// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const COMPUTE: u8 = 0;
pub const READ_GAS_LIMIT: u8 = 1;
pub const GAS_LIMITS_SIZE: usize = 10;

#[derive(Clone, Canon, Debug)]
pub struct Factorial {
    gas_limits: [u64; GAS_LIMITS_SIZE],
}

impl Factorial {
    pub fn new() -> Factorial {
        Factorial{ gas_limits: [0; GAS_LIMITS_SIZE] }
    }
    pub fn nth_limit(&self, n: usize) -> u64 {
        let default = 0u64;
        *self.gas_limits.get(n).unwrap_or(&default)
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::ReturnValue;

    const PAGE_SIZE: usize = 1024 * 4;

    impl Factorial {
        pub fn compute(&mut self, n: u64) -> u64 {
            if n < 2 {
                1
            } else {
                let callee = dusk_abi::callee();

                let a = dusk_abi::query::<_, u64>(&callee, &(COMPUTE, n - 1))
                    .unwrap();

                let gas_left = dusk_abi::gas_left();

                self.gas_limits[n as usize] = gas_left;

                dusk_abi::debug!("storing gas left {} at index {}", gas_left, n);

                a * n
            }
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self (noop).
        let mut slf = Factorial::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            COMPUTE => {
                // read arg
                let input = u64::decode(&mut source)?;
                let ret = slf.compute(input);

                let mut sink = Sink::new(&mut bytes[..]);

                ReturnValue::from_canon(&ret).encode(&mut sink);
                Ok(())
            }
            READ_GAS_LIMIT => {
                // read arg
                let input = u64::decode(&mut source)?;
                let ret = slf.nth_limit(input as usize);

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
}
