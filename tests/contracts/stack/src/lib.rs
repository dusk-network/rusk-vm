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
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};

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
    fn execute(&self, arg: Peek, _: StoreContext) -> <Peek as Query>::Return {
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
        arg: Push,
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
        _: Pop,
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
        self.inner.push(value);
    }

    pub fn pop(&mut self) -> Option<u64> {
        self.inner.pop()
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use core::mem::size_of;

    use microkelvin::{StoreRef, StoreSerializer};
    use rkyv::{archived_root, ser::Serializer};

    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 1024 * 64] = [0u8; 1024 * 64];

    fn _transaction<S, T>(written_state: u32, written_data: u32) -> [u32; 2]
    where
        S: Apply<T> + Archive + Serialize<StoreSerializer<OffsetLen>>,
        S::Archived: Deserialize<S, StoreRef<OffsetLen>>,
        T: Transaction + Archive,
        T::Archived: Deserialize<T, StoreRef<OffsetLen>>,
        T::Return: Archive + Serialize<StoreSerializer<OffsetLen>>,
    {
        let (result, state) = {
            let scratch = unsafe { &mut SCRATCH[..] };

            let (state_arg, rest) = scratch.split_at_mut(written_data as usize);

            let (state, arg) = state_arg.split_at(written_state as usize);

            let mut store = StoreContext::new(AbiStore::new(rest));

            let state = unsafe { archived_root::<S>(state) };
            let transaction = unsafe { archived_root::<T>(arg) };

            let mut state: S = state.deserialize(&mut store).unwrap();
            let transaction: T = transaction.deserialize(&mut store).unwrap();

            // do it!
            let result = state.apply(transaction, store);

            (result, state)
        };

        // here we re-initialize the store, using the whole buffer.
        // since we will not be reading any data anymore from the scratch
        //
        // all data from this buffer was either temporary, or have been commited
        // to the host backend through the abi
        let scratch = unsafe { &mut SCRATCH[..] };

        let store = StoreContext::new(AbiStore::new(scratch));
        let mut ser = store.serializer();

        // ofs would for example be the value encoded after un-sized utf-8 data
        // for an `ArchivedString`
        let state_ofs = ser.serialize_value(&state).unwrap();
        let state_size = size_of::<<S as Archive>::Archived>();

        let result_ofs = ser.serialize_value(&result).unwrap();
        let result_size =
            size_of::<<<T as Transaction>::Return as Archive>::Archived>();

        let ret = [
            (state_ofs + state_size) as u32,
            (result_ofs + result_size) as u32,
        ];

        ret
    }

    fn _query<S, Q>(written_state: u32, written_data: u32) -> u32
    where
        S: Execute<Q> + Archive + Serialize<StoreSerializer<OffsetLen>>,
        S::Archived: Deserialize<S, StoreRef<OffsetLen>>,
        Q: Query + Archive,
        Q::Archived: Deserialize<Q, StoreRef<OffsetLen>>,
        Q::Return: Archive + Serialize<StoreSerializer<OffsetLen>>,
    {
        let result = {
            let scratch = unsafe { &mut SCRATCH[..] };

            let (state_arg, rest) = scratch.split_at_mut(written_data as usize);
            let (state, arg) = state_arg.split_at(written_state as usize);

            let state = unsafe { archived_root::<S>(state) };
            let query = unsafe { archived_root::<Q>(arg) };

            let mut store = StoreContext::new(AbiStore::new(rest));

            let state: S = state.deserialize(&mut store).unwrap();
            let query: Q = query.deserialize(&mut store).unwrap();

            state.execute(query, store)
        };

        // Re-intitialize buffer to write result

        let scratch = unsafe { &mut SCRATCH[..] };

        let store = StoreContext::new(AbiStore::new(scratch));
        let mut ser = store.serializer();

        let result_ofs = ser.serialize_value(&result).unwrap();
        let result_size =
            size_of::<<<Q as Query>::Return as Archive>::Archived>();

        let ret = (result_ofs + result_size) as u32;

        ret
    }

    #[no_mangle]
    fn push(written_state: u32, written_data: u32) -> [u32; 2] {
        _transaction::<Stack, Push>(written_state, written_data)
    }

    #[no_mangle]
    fn pop(written_state: u32, written_data: u32) -> [u32; 2] {
        _transaction::<Stack, Pop>(written_state, written_data)
    }

    #[no_mangle]
    fn peek(written_state: u32, written_data: u32) -> u32 {
        _query::<Stack, Peek>(written_state, written_data)
    }
};
