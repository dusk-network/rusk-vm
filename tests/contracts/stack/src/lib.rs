// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical::{Canon, CanonError};
use canonical_derive::Canon;
use microkelvin::Cardinality;
use microkelvin::Nth;
use nstack::NStack;

// transaction ids
pub const PUSH: u8 = 0;
pub const POP: u8 = 1;

// query ids
pub const PEEK: u8 = 0;

#[derive(Clone, Canon, Debug, Default)]
pub struct Stack<T> {
    inner: NStack<T, Cardinality>,
}

impl<T> Stack<T>
where
    T: Canon,
{
    pub fn new() -> Self {
        Stack {
            inner: NStack::new(),
        }
    }

    pub fn peek(&self, n: u64) -> Result<Option<T>, CanonError> {
        Ok(self.inner.nth(n)?.map(|n| n.clone()))
    }

    pub fn push(&mut self, value: T) -> Result<(), CanonError> {
        self.inner.push(value)
    }

    pub fn pop(&mut self) -> Result<Option<T>, CanonError> {
        self.inner.pop()
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 64;

    type Leaf = u64;

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);

        // read self.
        let slf = Stack::<Leaf>::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            PEEK => {
                let arg = Leaf::decode(&mut source)?;
                let ret = slf.peek(arg);

                let mut sink = Sink::new(&mut bytes[..]);

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
        let mut slf = Stack::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            PUSH => {
                let leaf = Leaf::decode(&mut source)?;
                let result = slf.push(leaf);

                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);
                ReturnValue::from_canon(&result).encode(&mut sink);

                Ok(())
            }
            POP => {
                let result = slf.pop();

                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);
                ReturnValue::from_canon(&result).encode(&mut sink);
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
