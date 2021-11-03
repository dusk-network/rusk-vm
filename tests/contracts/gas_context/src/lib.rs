// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;
extern crate alloc;
use alloc::vec::Vec;

// transaction ids
pub const COMPUTE: u8 = 0;
// query ids
pub const READ_GAS_LIMITS: u8 = 1;

#[derive(Clone, Canon, Debug)]
pub struct GasContextData {
    gas_limits: Vec<u64>,
}

impl GasContextData {
    pub fn new() -> GasContextData {
        GasContextData{ gas_limits: Vec::new()}
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
                0
            } else {
                let callee = dusk_abi::callee();
                dusk_abi::transact::<_, u64, Self>(self, &callee, &(COMPUTE, n - 1), 0)
                    .unwrap();
                self.gas_limits.insert(0, dusk_abi::gas_left());
                n
            }
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);
        let slf = GasContextData::decode(&mut source)?;
        let qid = u8::decode(&mut source)?;
        match qid {
            READ_GAS_LIMITS => {
                let ret = slf.gas_limits;
                let mut sink = Sink::new(&mut bytes[..]);
                ReturnValue::from_canon(&ret).encode(&mut sink);
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        let _ = query(bytes);
    }

    fn transaction(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);
        let mut slf = GasContextData::decode(&mut source)?;
        let tid = u8::decode(&mut source)?;
        match tid {
            COMPUTE => {
                let input = u64::decode(&mut source)?;
                let ret: u64 = slf.compute(input);
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
        let _ = transaction(bytes);
    }
}
