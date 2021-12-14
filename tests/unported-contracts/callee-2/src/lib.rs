// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const GET: u8 = 2;

#[derive(Clone, Canon, Debug, Default)]
pub struct Callee2;

impl Callee2 {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractId, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 64;

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self.
        let _ = Callee2::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;

        // read the sender's sender contract id
        let sender_sender = ContractId::decode(&mut source)?;

        // read the sender contract id
        let sender = ContractId::decode(&mut source)?;

        assert_eq!(sender, dusk_abi::caller(), "Expected Caller");

        match qid {
            GET => {
                let mut sink = Sink::new(&mut bytes[..]);

                // return value
                ReturnValue::from_canon(&(
                    sender_sender,
                    sender,
                    dusk_abi::callee(),
                ))
                .encode(&mut sink);

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
