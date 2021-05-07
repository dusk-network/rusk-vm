// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const BLOCK_HEIGHT: u8 = 0;

#[derive(Clone, Canon, Debug, Default)]
pub struct BlockHeight {}

impl BlockHeight {
    pub fn new() -> Self {
        BlockHeight {}
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {

    extern crate alloc;

    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::ReturnValue;

    const PAGE_SIZE: usize = 1024 * 4;

    impl BlockHeight {
        pub fn block_height(&self) -> u64 {
            99
            //dusk_abi::block_height()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self.
        let slf = BlockHeight::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            BLOCK_HEIGHT => {
                let ret = slf.block_height();

                // return value
                let wrapped_return = ReturnValue::from_canon(&ret);

                let mut sink = Sink::new(&mut bytes[..]);

                wrapped_return.encode(&mut sink);

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
