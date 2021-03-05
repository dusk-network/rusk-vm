// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;
use canonical::Canon;
use canonical_derive::Canon;
use dusk_abi::{ContractId, Transaction};

// qulery ids
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

#[derive(Clone, Canon, Debug)]
pub enum TxCall {
    Sum {
        values: Vec<u8>,
    },

    Delegate {
        contract: ContractId,
        tx: Transaction,
    },
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Id32, Store};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

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
            dusk_abi::transact_raw::<BS, _>(self, target, transaction).unwrap()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(&bytes[..], &bs);

        // read self.
        let slf: TxVec = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            READ_VALUE => {
                let ret = slf.read_value();

                let r = {
                    // return value
                    let wrapped_return = ReturnValue::from_canon(&ret, &bs)?;

                    dusk_abi::debug!("wrapped return {:?}", wrapped_return);

                    let mut sink = ByteSink::new(&mut bytes[..], &bs);

                    Canon::<BS>::write(&wrapped_return, &mut sink)
                };

                dusk_abi::debug!("memory bytes {:?}", &bytes[..32]);

                r
            }

            _ => panic!("Method not found!"),
        }
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = query(bytes);
    }

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(bytes, &bs);

        // read self.
        let mut slf: TxVec = Canon::<BS>::read(&mut source)?;

        dusk_abi::debug!("Parsing call");
        let call: TxCall = Canon::<BS>::read(&mut source)?;
        dusk_abi::debug!("Received call {:?}", call);

        match call {
            TxCall::Sum { values } => {
                dusk_abi::debug!("Received sum values {:?}", values);

                slf.sum(values);

                let mut sink = ByteSink::new(&mut bytes[..], &bs);
                // return new state
                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs)?,
                    &mut sink,
                )?;

                // return value
                Canon::<BS>::write(
                    &ReturnValue::from_canon(&(), &bs)?,
                    &mut sink,
                )
            }

            TxCall::Delegate { contract, tx } => {
                let result = slf.delegate_sum(&contract, &tx);

                let mut sink = ByteSink::new(&mut bytes[..], &bs);

                let state = ContractState::from_canon(&slf, &bs)?;

                Canon::<BS>::write(&state, &mut sink)?;
                Canon::<BS>::write(&result, &mut sink)
            }
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }
}
