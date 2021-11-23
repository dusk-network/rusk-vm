// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;

// query ids
pub const DELEGATE_QUERY: u8 = 0;

// transaction ids
pub const DELEGATE_TRANSACTION: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct Delegator;

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{
        ContractId, ContractState, Query, ReturnValue, Transaction,
    };

    const PAGE_SIZE: usize = 1024 * 4;

    impl Delegator {
        pub fn delegate_query(
            &self,
            target: &ContractId,
            query: &Query,
        ) -> ReturnValue {
            dusk_abi::query_raw(target, query, 0).unwrap()
        }

        pub fn delegate_transaction(
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
        let slf = Delegator::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            DELEGATE_QUERY => {
                let (target, query): (ContractId, Query) =
                    Canon::decode(&mut source)?;

                let result = slf.delegate_query(&target, &query);

                let mut sink = Sink::new(&mut bytes[..]);

                result.encode(&mut sink);
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
        let mut slf = Delegator::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            DELEGATE_TRANSACTION => {
                let (target, transaction): (ContractId, Transaction) =
                    Canon::decode(&mut source)?;

                let result = slf.delegate_transaction(&target, &transaction);

                let mut sink = Sink::new(&mut bytes[..]);

                // return new state
                ContractState::from_canon(&slf).encode(&mut sink);

                // return value
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
