// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

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

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractId, ContractState, ReturnValue, Transaction};

    const PAGE_SIZE: usize = 1024 * 4;

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

            dusk_abi::transact::<_, (), Self>(
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

            dusk_abi::transact_raw::<_>(self, &target, &transaction).unwrap();

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

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self (noop).
        let slf = SelfSnapshot::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            CROSSOVER => {
                let ret = slf.crossover();

                let mut sink = Sink::new(&mut bytes[..]);
                let packed_ret = ReturnValue::from_canon(&ret);

                packed_ret.encode(&mut sink);
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
        let mut slf = SelfSnapshot::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            // increment (&Self)
            SET_CROSSOVER => {
                let to = i32::decode(&mut source)?;
                let old = slf.set_crossover(to);

                let mut sink = Sink::new(&mut bytes[..]);
                // return new state
                ContractState::from_canon(&slf).encode(&mut sink);

                // return value
                ReturnValue::from_canon(&old).encode(&mut sink);
                Ok(())
            }
            SELF_CALL_TEST_A => {
                let update = i32::decode(&mut source)?;
                let old = slf.self_call_test_a(update);

                let mut sink = Sink::new(&mut bytes[..]);
                // return new state
                ContractState::from_canon(&slf).encode(&mut sink);

                // return value
                ReturnValue::from_canon(&old).encode(&mut sink);
                Ok(())
            }
            SELF_CALL_TEST_B => {
                let (target, transaction): (ContractId, Transaction) =
                    Canon::decode(&mut source)?;

                let old = slf.self_call_test_b(target, transaction);

                let mut sink = Sink::new(&mut bytes[..]);

                // return new state
                ContractState::from_canon(&slf).encode(&mut sink);

                // return value
                ReturnValue::from_canon(&old).encode(&mut sink);
                Ok(())
            }
            UPDATE_AND_PANIC => {
                let update = i32::decode(&mut source)?;
                slf.update_and_panic(update);

                let mut sink = Sink::new(&mut bytes[..]);
                // return new state
                ContractState::from_canon(&slf).encode(&mut sink);
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
