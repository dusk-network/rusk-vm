// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! This contract is meant to be used to test whether the `deploy`
//! instrumentization rules are applied correctly denying the deployment of any
//! contract that uses floating point operations.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical::{Canon, Sink, Source, Store};
use core::mem;

// transaction ids
pub const INCREMENT: u8 = 0;

#[derive(Clone, Debug)]
pub struct CounterFloat {
    junk: f32,
    value: f32,
}

impl<S> Canon<S> for CounterFloat
where
    S: Store,
{
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        sink.copy_bytes(&self.junk.to_le_bytes());
        sink.copy_bytes(&self.value.to_le_bytes());
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let bytes = source.read_bytes(mem::size_of::<f32>());
        let mut float = [0u8; 4];
        float.copy_from_slice(&bytes[0..4]);
        let junk = f32::from_le_bytes(float);
        float.copy_from_slice(&bytes[4..]);
        let value = f32::from_le_bytes(float);
        Ok(Self { junk, value })
    }

    fn encoded_len(&self) -> usize {
        mem::size_of::<f32>() * 2
    }
}

impl CounterFloat {
    pub fn new(value: f32) -> Self {
        CounterFloat {
            junk: 0.55f32,
            value,
        }
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Id32, Store};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl CounterFloat {
        // We add `no_mangle` to prevent any compiler optimizations to remove
        // the floating point usage from the code.
        #[no_mangle]
        pub fn increment(&mut self) {
            self.value += 1.88f32;
        }
    }

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let bs = BS::default();
        let mut source = ByteSource::new(bytes, &bs);

        // read self.
        let mut slf: CounterFloat = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let tid: u8 = Canon::<BS>::read(&mut source)?;
        match tid {
            // increment (&Self)
            INCREMENT => {
                slf.increment();
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
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }
}
