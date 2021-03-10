// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical::Canon;
use canonical_derive::Canon;

// query ids
pub const CROSSOVER: u8 = 0;

// transaction ids
pub const SET_CROSSOVER: u8 = 0;
pub const SELF_CALL_TEST_A: u8 = 1;
pub const SELF_CALL_TEST_B: u8 = 2;
pub const UPDATE_AND_PANIC: u8 = 3;

#[derive(Clone, Canon, Debug)]
pub struct SelfSnapshot {
    crossover: i32,
}

impl SelfSnapshot {
    pub fn new(init: i32) -> Self {
        SelfSnapshot { crossover: init }
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Canon, Id32, Store};
    use dusk_abi::{ContractId, ContractState, ReturnValue, Transaction};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl SelfSnapshot {
        pub fn crossover(&self) -> i32 {
            self.crossover
        }

        pub fn set_crossover(&mut self, to: i32) -> i32 {
            let old_val = self.crossover;
            dusk_abi::debug!(
                "setting crossover from {:?} to {:?}",
                self.crossover,
                to
            );
            self.crossover = to;
            old_val
        }

        // updates crossover and returns the old value
        pub fn self_call_test_a(&mut self, update: i32) -> i32 {
            let old_value = self.crossover;

            let callee = dusk_abi::callee();

            dusk_abi::transact::<BS, _, (), Self>(
                self,
                &callee,
                &(SET_CROSSOVER, update),
            )
            .unwrap();

            assert_eq!(self.crossover, update);

            old_value
        }

        // updates crossover and returns the old value
        pub fn self_call_test_b(
            &mut self,
            target: ContractId,
            transaction: Transaction,
        ) -> i32 {
            self.set_crossover(self.crossover * 2);

            dusk_abi::transact_raw::<BS, _>(self, &target, &transaction)
                .unwrap();

            self.crossover
        }

        pub fn update_and_panic(&mut self, new_value: i32) {
            let old_value = self.crossover;

            assert_eq!(self.self_call_test_a(new_value), old_value);

            let callee = dusk_abi::callee();

            // What should self.crossover be in this case?

            // A: we live with inconsistencies and communicate them.
            // B: we update self, which then should be passed to the transaction

            assert_eq!(
                dusk_abi::query::<_, i32>(&callee, &(CROSSOVER),).unwrap(),
                new_value
            );

            panic!("OH NOES")
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(&bytes[..], &store);

        // read self (noop).
        let slf: SelfSnapshot = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            CROSSOVER => {
                let ret = slf.crossover();

                let mut sink = ByteSink::new(&mut bytes[..], &store);
                let packed_ret = ReturnValue::from_canon(&ret, &store)?;

                Canon::<BS>::write(&packed_ret, &mut sink)
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
        let mut slf: SelfSnapshot = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let tid: u8 = Canon::<BS>::read(&mut source)?;
        match tid {
            // increment (&Self)
            SET_CROSSOVER => {
                let to: i32 = Canon::<BS>::read(&mut source)?;
                let old = slf.set_crossover(to);

                let mut sink = ByteSink::new(&mut bytes[..], &bs);
                // return new state
                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs)?,
                    &mut sink,
                )?;

                // return value
                Canon::<BS>::write(
                    &ReturnValue::from_canon(&old, &bs)?,
                    &mut sink,
                )
            }
            SELF_CALL_TEST_A => {
                let update: i32 = Canon::<BS>::read(&mut source)?;
                let old = slf.self_call_test_a(update);

                let mut sink = ByteSink::new(&mut bytes[..], &bs);
                // return new state
                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs)?,
                    &mut sink,
                )?;

                // return value
                Canon::<BS>::write(
                    &ReturnValue::from_canon(&old, &bs)?,
                    &mut sink,
                )
            }
            SELF_CALL_TEST_B => {
                dusk_abi::debug!("A");
                let (target, transaction): (ContractId, Transaction) =
                    Canon::read(&mut source)?;

                let old = slf.self_call_test_b(target, transaction);

                dusk_abi::debug!("C");
                let mut sink = ByteSink::new(&mut bytes[..], &bs);

                dusk_abi::debug!("D");
                // return new state
                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs)?,
                    &mut sink,
                )?;

                // return value
                Canon::<BS>::write(
                    &ReturnValue::from_canon(&old, &bs)?,
                    &mut sink,
                )
            }
            UPDATE_AND_PANIC => {
                let update: i32 = Canon::<BS>::read(&mut source)?;
                slf.update_and_panic(update);

                let mut sink = ByteSink::new(&mut bytes[..], &bs);
                // return new state
                Canon::<BS>::write(
                    &ContractState::from_canon(&slf, &bs)?,
                    &mut sink,
                )
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
