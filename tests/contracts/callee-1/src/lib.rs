// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;
use dusk_abi::ContractId;

// transaction ids
pub const SET_TARGET: u8 = 0;

// query ids
pub const CALL: u8 = 1;
pub const CALLEE_2_GET: u8 = 2;

#[derive(Clone, Canon, Debug, Default)]
pub struct Callee1 {
    target_address: ContractId,
}

impl Callee1 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, address: ContractId) {
        self.target_address = address;
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
        let slf = Callee1::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;

        // read the sender contract id
        let sender = ContractId::decode(&mut source)?;

        assert_eq!(sender, dusk_abi::caller(), "Expected Caller");

        match qid {
            CALL => {
                let should_panic = bool::decode(&mut source)?;
                let mut sink = Sink::new(&mut bytes[..]);

                let ret = dusk_abi::query::<
                    _,
                    (ContractId, ContractId, ContractId),
                >(
                    &slf.target_address,
                    &(CALLEE_2_GET, sender, dusk_abi::callee(), should_panic),
                    0,
                )
                .expect("Query Succeeded");

                // return value
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
        let mut slf = Callee1::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        // read the target contract id
        let target = ContractId::decode(&mut source)?;

        match tid {
            SET_TARGET => {
                slf.set_target(target);

                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);
                ReturnValue::from_canon(&()).encode(&mut sink);

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
