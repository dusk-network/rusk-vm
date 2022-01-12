// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(
    core_intrinsics,
    lang_items,
    alloc_error_handler,
    option_result_unwrap_unchecked
)]

use rkyv::{Archive, Deserialize, Serialize};
use rkyv::validation::validators::DefaultValidator;
use rusk_uplink::{Query, Transaction};
use nstack::NStack;
use microkelvin::{Cardinality, Compound, OffsetLen, Nth, StoreRef};
use bytecheck::CheckBytes;


#[derive(Default, Clone, Archive, Serialize, Deserialize)]
pub struct Stack<T>
where
    T: Archive + Clone + for<'a> bytecheck::CheckBytes<DefaultValidator<'a>>,
{
    inner: NStack<T, Cardinality, OffsetLen>,
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Peek {
    value: u64,
}

impl Peek {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

impl Query for Peek {
    const NAME: &'static str = "peek";
    type Return = u64;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Push {
    value: u64,
}

impl Push {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

impl Transaction for Push {
    const NAME: &'static str = "push";
    type Return = u64;
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Pop;

impl Transaction for Pop {
    const NAME: &'static str = "pop";
    type Return = u64;
}


#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::{AbiStore, StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    impl<T> Stack<T>
    where
        T: Archive + Clone + for<'a> bytecheck::CheckBytes<DefaultValidator<'a>>,
        <T as Archive>::Archived: Deserialize<T, StoreRef<OffsetLen>>
        + for<'a> bytecheck::CheckBytes<DefaultValidator<'a>>
    {
        pub fn new() -> Self {
            Stack {
                inner: NStack::new(),
            }
        }

        pub fn peek(&self, n: u64) {
            (self.inner as Nth).walk(Nth(n))?.map(|n| n.clone())
        }

        pub fn push(&mut self, value: T) {
            self.inner.push(value)
        }

        pub fn pop(&mut self) -> Option<T> {
            self.inner.pop()
        }
    }

    #[no_mangle]
    fn peek(written_state: u32, written_data: u32) -> u32 {
        let mut store = StoreContext::new(AbiStore::new());

        let state = unsafe {
            archived_root::<Stack>(&SCRATCH[..written_state as usize])
        };
        let mut state: Stack = state.deserialize(&mut store).unwrap();
        let arg = unsafe {
            archived_root::<Peek>(&SCRATCH[..written_state as usize])
        };
        let arg: u64 = arg.deserialize(&mut store).unwrap();

        let ret = state.peek(arg.value);

        let res: <Peek as Query>::Return = ret;
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
            <<Peek as Query>::Return as Archive>::Archived,
        >();
        buffer_len as u32
    }

    #[no_mangle]
    fn push(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store = StoreContext::new(AbiStore::new());

        let state = unsafe {
            archived_root::<Stack>(&SCRATCH[..written_state as usize])
        };
        let mut state: Stack = state.deserialize(&mut store).unwrap();
        let arg = unsafe {
            archived_root::<Push>(&SCRATCH[..written_state as usize])
        };
        let arg: u64 = arg.deserialize(&mut store).unwrap();

        let result = state.push(arg.value);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<Stack as Archive>::Archived>();

        let return_len = ser.serialize_value(&result).unwrap()
            + core::mem::size_of::<
            <<Push as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }

    #[no_mangle]
    fn pop(written_state: u32, _written_data: u32) -> [u32; 2] {
        let mut store = StoreContext::new(AbiStore::new());

        let state = unsafe {
            archived_root::<Stack>(&SCRATCH[..written_state as usize])
        };
        let mut state: Stack = state.deserialize(&mut store).unwrap();

        let result = state.pop();

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<Stack as Archive>::Archived>();

        let return_len = ser.serialize_value(&result).unwrap()
            + core::mem::size_of::<
            <<Pop as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }

};
