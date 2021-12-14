// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical::{Canon, CanonError, Sink, Source};
use core::mem;

// transaction ids
pub const INCREMENT: u8 = 0;

#[derive(Clone, Debug, Default)]
pub struct CounterFloat {
    value: f32,
}

impl Canon for CounterFloat {
    fn encode(&self, sink: &mut Sink) {
        sink.copy_bytes(&self.value.to_le_bytes());
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        let bytes = source.read_bytes(mem::size_of::<f32>());
        let mut float = [0u8; 4];
        float.copy_from_slice(&bytes[0..4]);
        let value = f32::from_le_bytes(float);
        Ok(Self { value })
    }

    fn encoded_len(&self) -> usize {
        mem::size_of::<f32>()
    }
}

impl CounterFloat {
    pub fn new(value: f32) -> Self {
        CounterFloat { value }
    }
    // We add `no_mangle` to prevent any compiler optimizations to remove
    // the floating point usage from the code.
    #[no_mangle]
    pub fn increment(&mut self) {
        self.value += 1.88f32;
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::{ContractState, ReturnValue};

    const PAGE_SIZE: usize = 1024 * 64;

    fn transaction(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(bytes);

        // read self.
        let mut slf = CounterFloat::decode(&mut source)?;
        // read transaction id
        let tid = u8::decode(&mut source)?;
        match tid {
            INCREMENT => {
                slf.increment();

                let mut sink = Sink::new(&mut bytes[..]);

                ContractState::from_canon(&slf).encode(&mut sink);
                // return value
                Canon::encode(&ReturnValue::from_canon(&()), &mut sink);

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
