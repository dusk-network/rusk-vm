// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// qulery ids
pub const COMPUTE: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct Fibonacci;

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};
    use dusk_abi::ReturnValue;

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Fibonacci {
        pub fn compute(&self, n: u64) -> u64 {
            if n < 2 {
                n
            } else {
                let callee = dusk_abi::callee();

                let a = dusk_abi::query::<_, u64>(&callee, &(COMPUTE, n - 1))
                    .unwrap();

                let b = dusk_abi::query::<_, u64>(&callee, &(COMPUTE, n - 2))
                    .unwrap();

                a + b
            }
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(&bytes[..], &store);

        // read self (noop).
        let slf: Fibonacci = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            COMPUTE => {
                // read arg
                let input: u64 = Canon::<BS>::read(&mut source)?;

                let ret = slf.compute(input);

                let mut sink = ByteSink::new(&mut bytes[..], &store);
                let packed_ret = ReturnValue::from_canon(&ret, &store)?;

                dusk_abi::debug!("packed_ret {:?}", packed_ret);

                Canon::<BS>::write(&packed_ret, &mut sink)
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
