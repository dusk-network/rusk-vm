// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const HASH: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct Hash {}

impl Hash {
    pub fn new() -> Self {
        Hash {}
    }
}

#[cfg(not(feature = "host"))]
mod hosted {

    extern crate alloc;

    use super::*;
    use alloc::vec::Vec;
    use dusk_bls12_381::BlsScalar;

    use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};
    use dusk_abi::ReturnValue;

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Hash {
        pub fn hash(&self, messages: Vec<BlsScalar>) -> BlsScalar {
            dusk_abi::poseidon_hash(messages)
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(&bytes[..], &bs);

        // read self.
        let slf: Hash = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            HASH => {
                let messages: Vec<BlsScalar> = Canon::<BS>::read(&mut source)?;

                let ret = slf.hash(messages);

                let r = {
                    // return value
                    let wrapped_return = ReturnValue::from_canon(&ret, &bs)?;

                    let mut sink = ByteSink::new(&mut bytes[..], &bs);

                    Canon::<BS>::write(&wrapped_return, &mut sink)
                };

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
}
