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
pub const CALL: u8 = 0;
pub const CALLEE_1_CALL: u8 = 1;

#[derive(Clone, Canon, Debug, Default)]
pub struct Caller {
    target_address: ContractId,
}

impl Caller {
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
        let slf = Caller::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            CALL => {
                let mut sink = Sink::new(&mut bytes[..]);

                let ret =
                    dusk_abi::query::<_, (ContractId, ContractId, ContractId)>(
                        &slf.target_address,
                        &(CALLEE_1_CALL, dusk_abi::callee()),
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
        let mut slf = Caller::decode(&mut source)?;
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
