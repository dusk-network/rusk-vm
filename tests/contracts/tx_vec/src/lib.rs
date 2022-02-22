// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const READ_VALUE: u8 = 0;

// transaction ids
pub const SUM: u8 = 0;
pub const DELEGATE_SUM: u8 = 1;

#[derive(Clone, Canon, Debug)]
pub struct TxVec {
    value: u8,
}

impl TxVec {
    pub fn new(value: u8) -> Self {
        TxVec { value }
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {
    extern crate alloc;

    use super::*;

    use alloc::vec::Vec;
    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractId, Transaction};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

    impl TxVec {
        pub fn read_value(&self) -> u8 {
            self.value
        }

        pub fn sum(&mut self, values: Vec<u8>) {
            self.value +=
                values.into_iter().fold(0u8, |s, v| s.wrapping_add(v));
        }

        pub fn delegate_sum(
            &mut self,
            target: &ContractId,
            transaction: &Transaction,
        ) -> ReturnValue {
            dusk_abi::transact_raw::<_>(self, target, transaction, 0).unwrap()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self.
        let slf = TxVec::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            READ_VALUE => {
                let ret = slf.read_value();

                let mut sink = Sink::new(&mut bytes[..]);

                ReturnValue::from_canon(&ret).encode(&mut sink);
                Ok(())
            }

            _ => panic!("Method not found!"),
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
        let mut slf = TxVec::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            SUM => {
                let values = Vec::<u8>::decode(&mut source)?;

                slf.sum(values);

                let mut sink = Sink::new(&mut bytes[..]);
                // return new state
                ContractState::from_canon(&slf).encode(&mut sink);

                // return value
                ReturnValue::from_canon(&()).encode(&mut sink);
                Ok(())
            }

            DELEGATE_SUM => {
                let contract = ContractId::decode(&mut source)?;
                let tx = Transaction::decode(&mut source)?;

                let result = slf.delegate_sum(&contract, &tx);

                let mut sink = Sink::new(&mut bytes[..]);

                let state = ContractState::from_canon(&slf);

                state.encode(&mut sink);
                result.encode(&mut sink);
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }
}
