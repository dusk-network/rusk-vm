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

use bytecheck::CheckBytes;
use microkelvin::{Cardinality, Compound, Nth, OffsetLen};
use nstack::NStack;
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Query, Transaction, Apply, Execute, StoreContext};

#[derive(Default, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Stack {
    inner: NStack<u64, Cardinality, OffsetLen>,
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
    type Return = Option<u64>;
}

impl Execute<Peek> for Stack {
    fn execute(
        &self,
        arg: &Peek,
        _: StoreContext,
    ) -> <Peek as Query>::Return {
        self.peek(arg.value)
    }
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
    type Return = ();
}

impl Apply<Push> for Stack {
    fn apply(
        &mut self,
        arg: &Push,
        _: StoreContext,
    ) -> <Push as Transaction>::Return {
        self.push(arg.value);
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Pop;

impl Transaction for Pop {
    const NAME: &'static str = "pop";
    type Return = Option<u64>;
}

impl Apply<Pop> for Stack {
    fn apply(
        &mut self,
        _: &Pop,
        _: StoreContext,
    ) -> <Pop as Transaction>::Return {
        self.pop()
    }
}


impl Stack {
    pub fn new() -> Self {
        Stack {
            inner: NStack::new(),
        }
    }

    pub fn peek(&self, n: u64) -> Option<u64> {
        self.inner.walk(Nth(n)).map(|n| match n.leaf() {
            microkelvin::MaybeArchived::Memory(u) => *u,
            microkelvin::MaybeArchived::Archived(archived) => {
                u64::from(archived)
            }
        })
    }

    pub fn push(&mut self, value: u64) {
        self.inner.push(value)
    }

    pub fn pop(&mut self) -> Option<u64> {
        self.inner.pop()
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::Serializer;
    use rusk_uplink::{AbiStore, StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    #[no_mangle]
    fn peek(written_state: u32, _written_data: u32) -> u32 {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<Stack>(&SCRATCH[..written_state as usize])
        };
        let state: Stack = state.deserialize(&mut store).unwrap();
        let arg = unsafe {
            archived_root::<Peek>(&SCRATCH[..written_state as usize])
        };
        let arg: Peek = arg.deserialize(&mut store).unwrap();

        let ret = state.execute(&arg, store.clone());

        let res: <Peek as Query>::Return = ret;
        let mut ser = store.serializer();
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<Peek as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }

    #[no_mangle]
    fn push(written_state: u32, _written_data: u32) -> [u32; 2] {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<Stack>(&SCRATCH[..written_state as usize])
        };
        let mut state: Stack = state.deserialize(&mut store).unwrap();
        let arg = unsafe {
            archived_root::<Push>(&SCRATCH[..written_state as usize])
        };
        let arg: Push = arg.deserialize(&mut store).unwrap();

        // let result = state.push(arg.value);
        let result = state.apply(&arg, store.clone());

        let mut ser = store.serializer();
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
        rusk_uplink::debug!("ommegott");

        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<Stack>(&SCRATCH[..written_state as usize])
        };
        let mut state: Stack = state.deserialize(&mut store).unwrap();

        //let result = state.pop();
        let result = state.apply(&Pop, store.clone());

        let mut ser = store.serializer();
        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<Stack as Archive>::Archived>();

        let return_len = ser.serialize_value(&result).unwrap()
            + core::mem::size_of::<
                <<Pop as Transaction>::Return as Archive>::Archived,
            >();

        [state_len as u32, return_len as u32]
    }
};
