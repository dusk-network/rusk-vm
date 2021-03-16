// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

extern crate alloc;

use canonical_derive::Canon;

// query ids
pub const HASH: u8 = 0;

// transaction ids
pub const SOMETHING: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct HostFnTest;

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use alloc::vec::Vec;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractId, ReturnValue};

    // use dusk_bls12_381::BlsScalar;
    #[derive(Clone, Canon)]
    pub struct BlsScalar;

    const PAGE_SIZE: usize = 1024 * 4;

    impl HostFnTest {
        pub fn hash(&self, scalars: Vec<BlsScalar>) -> BlsScalar {
            const POSEIDON_MODULE_ID: ContractId = ContractId::reserved(11);
            const HASH: u8 = 0;

            dusk_abi::query(&POSEIDON_MODULE_ID, &(HASH, scalars)).unwrap()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self.
        let slf = HostFnTest::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            HASH => {
                let arg = Vec::<BlsScalar>::decode(&mut source)?;

                let ret = slf.hash(arg);
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
        let mut _slf = HostFnTest::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }
}
