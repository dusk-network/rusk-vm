// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(lang_items)]

use canonical_derive::Canon;

// qulery ids
pub const DELEGATE_QUERY: u8 = 0;

// transaction ids
pub const DELEGATE_TRANSACTION: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct Delegator;

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};
    use dusk_abi::{
        ContractId, ContractState, Query, ReturnValue, Transaction,
    };

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Delegator {
        pub fn delegate_query(
            &self,
            target: &ContractId,
            query: &Query,
        ) -> ReturnValue {
            dusk_abi::query_raw(target, query).unwrap()
        }

        pub fn delegate_transaction(
            &mut self,
            target: &ContractId,
            transaction: &Transaction,
        ) -> ReturnValue {
            dusk_abi::transact_raw(target, transaction).unwrap()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(&bytes[..], &bs);

        // read self.
        let slf: Delegator = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            DELEGATE_QUERY => {
                let (target, query): (ContractId, Query) =
                    Canon::read(&mut source)?;

                let result = slf.delegate_query(&target, &query);

                let mut sink = ByteSink::new(&mut bytes[..], &bs);

                Canon::<BS>::write(&result, &mut sink)?;
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

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(bytes, &bs);

        // read self.
        let mut slf: Delegator = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let tid: u8 = Canon::<BS>::read(&mut source)?;
        match tid {
            DELEGATE_TRANSACTION => {
                let (target, transaction): (ContractId, Transaction) =
                    Canon::read(&mut source)?;

                let result = slf.delegate_transaction(&target, &transaction);

                let mut sink = ByteSink::new(&mut bytes[..], &bs);

                // return new state
                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs)?,
                    &mut sink,
                )?;

                // return value
                Canon::<BS>::write(&result, &mut sink)
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    include!("../../../../dusk-abi/src/panic_include.rs");
}
