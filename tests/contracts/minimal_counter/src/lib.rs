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
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};

#[derive(Clone, Debug, Archive, Deserialize, Serialize)]
pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn new(value: u32) -> Self {
        Counter { value }
    }
}

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct ReadCount;

impl Query for ReadCount {
    const NAME: &'static str = "read";
    type Return = u32;
}

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct Increment(pub u32);

impl Transaction for Increment {
    const NAME: &'static str = "incr";
    type Return = (); // todo: delegation does not work for empty result () - fix it
}

impl Execute<ReadCount> for Counter {
    fn execute(
        &self,
        _: &ReadCount,
        _: StoreContext,
    ) -> <ReadCount as Query>::Return {
        self.value.into()
    }
}

impl Apply<Increment> for Counter {
    fn apply(&mut self, t: &Increment) -> <Increment as Transaction>::Return {
        self.value += t.0;
        // todo: delegation does not work for empty result () - fix it
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::Serializer;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 128] = [0u8; 128];

    #[no_mangle]
    fn read(written_state: u32, written_data: u32) -> u32 {
        let mut store = StoreContext::new(AbiStore::new());

        let state = unsafe {
            archived_root::<Counter>(&SCRATCH[..written_state as usize])
        };
        let arg = unsafe {
            archived_root::<ReadCount>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let de_state: Counter = (state).deserialize(&mut store).unwrap();
        let de_query: ReadCount = (arg).deserialize(&mut store).unwrap();

        let mut ser = store.serializer();

        let res: <ReadCount as Query>::Return =
            de_state.execute(&de_query, store);

        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<ReadCount as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }

    #[no_mangle]
    fn incr(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store = StoreContext::new(AbiStore::new());

        let state = unsafe {
            archived_root::<Counter>(&SCRATCH[..written_state as usize])
        };
        let arg = unsafe {
            archived_root::<Increment>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let mut de_state: Counter = state.deserialize(&mut store).unwrap();
        let de_transaction: Increment = arg.deserialize(&mut store).unwrap();

        let res: <Increment as Transaction>::Return =
            de_state.apply(&de_transaction);

        let mut ser = store.serializer();

        let state_len = ser.serialize_value(&de_state).unwrap()
            + core::mem::size_of::<<Counter as Archive>::Archived>();

        let return_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<Increment as Transaction>::Return as Archive>::Archived,
            >();

        [state_len as u32, return_len as u32]
    }
};
