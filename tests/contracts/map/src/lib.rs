// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical::CanonError;
use canonical_derive::Canon;

// transaction ids
pub const SET: u8 = 0;
pub const REMOVE: u8 = 1;

// query ids
pub const GET: u8 = 1;

#[derive(Clone, Canon, Debug, Default)]
pub struct Map {
    inner: dusk_hamt::Map<u8, u32>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            inner: dusk_hamt::Map::default(),
        }
    }

    pub fn set(&mut self, key: u8, value: u32) -> Option<u32> {
        self.inner.insert(key, value).ok()?
    }

    pub fn get(&self, key: &u8) -> Option<u32> {
        self.inner.get(key).map(|x| x.map(|x| *x)).ok()?
    }

    pub fn remove(&mut self, key: &u8) -> Result<Option<u32>, CanonError> {
        self.inner.remove(key)
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
        let slf = Map::decode(&mut source)?;

        // read query id
        let qid = u8::decode(&mut source)?;
        match qid {
            GET => {
                let key = u8::decode(&mut source)?;
                let ret = slf.get(&key);

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
        let mut slf = Map::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            SET => {
                let key = u8::decode(&mut source)?;
                let value = u32::decode(&mut source)?;
                let result = slf.set(key, value);

                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);
                ReturnValue::from_canon(&result).encode(&mut sink);

                Ok(())
            }
            REMOVE => {
                let key = u8::decode(&mut source)?;
                let result = slf.remove(&key);

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
