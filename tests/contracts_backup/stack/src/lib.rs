// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;
use microkelvin::Cardinality;
use microkelvin::Nth;
use nstack::NStack;

// transaction ids
pub const PUSH: u8 = 0;
pub const POP: u8 = 1;

// query ids
pub const PEEK: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct Stack {
    inner: NStack<i32, Cardinality>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            inner: NStack::new(),
        }
    }

    pub fn peek(&self, n: i32) -> Option<i32> {
        self.inner.nth(n as u64).unwrap().map(|n| *n)
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, Id32, Sink, Source, Store};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

    impl Stack {
        pub fn push(&mut self, value: i32) {
            self.inner.push(value).unwrap()
        }

        pub fn pop(&mut self) -> Option<i32> {
            self.inner.pop().unwrap()
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let bs = BS::default();
        let mut source = Source::new(&bytes[..], &bs);

        // read self.
        let slf: Stack<BS> = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            PEEK => {
                let arg: i32 = Canon::<BS>::read(&mut source)?;

                let ret = slf.peek(arg);

                let r = {
                    // return value
                    let wrapped_return = ReturnValue::from_canon(&ret, &bs)?;

                    let mut sink = Sink::new(&mut bytes[..], &bs);

                    Canon::<BS>::write(&wrapped_return, &mut sink)
                };

                r
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
        let bs = BS::default();
        let mut source = Source::new(bytes, &bs);

        // read self.
        let mut slf: Stack<BS> = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let tid: u8 = Canon::<BS>::read(&mut source)?;
        match tid {
            PUSH => {
                let value: i32 = Canon::<BS>::read(&mut source)?;
                slf.push(value);

                let mut sink = Sink::new(&mut bytes[..], &bs);

                let new_state = ContractState::from_canon(&slf, &bs)?;

                // return new state
                Canon::<BS>::write(&new_state, &mut sink)?;

                let ret_val = ReturnValue::from_canon(&(), &bs);

                // return value (no-op)
                Canon::<BS>::write(&ret_val, &mut sink)
            }
            POP => {
                let result = slf.pop();

                let mut sink = Sink::new(&mut bytes[..], &bs);

                let new_state = ContractState::from_canon(&slf, &bs)?;

                // return new self
                Canon::<BS>::write(&new_state, &mut sink)?;

                let return_value = ReturnValue::from_canon(&result, &bs)?;

                // return value
                Canon::<BS>::write(&return_value, &mut sink)?;

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
