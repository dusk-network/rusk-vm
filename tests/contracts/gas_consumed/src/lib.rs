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
use rusk_uplink::{Query, Transaction, Apply, Execute, StoreContext};
use rusk_uplink::{get_state, get_state_and_arg, q_return, t_return, query_state_arg_fun, transaction_state_arg_fun};

extern crate alloc;

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumed {
    value: i32,
}

impl GasConsumed {
    pub fn new(value: i32) -> Self {
        GasConsumed { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn increment(&mut self) {
        self.value += 1
    }
    pub fn decrement(&mut self) {
        self.value -= 1
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumedValueQuery;
#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumedQuery;

impl Query for GasConsumedValueQuery {
    const NAME: &'static str = "value";
    type Return = i32;
}

impl Query for GasConsumedQuery {
    const NAME: &'static str = "get_gas_consumed";
    type Return = (u32, u32);
}

impl Execute<GasConsumedValueQuery> for GasConsumed {
    fn execute(
        &self,
        _: &GasConsumedValueQuery,
        _: StoreContext,
    ) -> <GasConsumedValueQuery as Query>::Return {
        self.value()
    }
}

impl Execute<GasConsumedQuery> for GasConsumed {
    fn execute(
        &self,
        _: &GasConsumedQuery,
        _: StoreContext,
    ) -> <GasConsumedQuery as Query>::Return {
        (
            rusk_uplink::gas_consumed() as u32,
            rusk_uplink::gas_left() as u32,
        )
    }
}

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumedIncrement;
#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct GasConsumedDecrement;

impl Transaction for GasConsumedIncrement {
    const NAME: &'static str = "increment";
    type Return = ();
}

impl Transaction for GasConsumedDecrement {
    const NAME: &'static str = "decrement";
    type Return = ();
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::{AbiStore, StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 512] = [0u8; 512];

    query_state_arg_fun!(value, GasConsumed, GasConsumedValueQuery);

    query_state_arg_fun!(get_gas_consumed, GasConsumed, GasConsumedQuery);

    #[no_mangle]
    fn increment(written_state: u32, _written_data: u32) -> [u32; 2] {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let slf = unsafe {
            archived_root::<GasConsumed>(&SCRATCH[..written_state as usize])
        };

        let mut slf: GasConsumed = (slf).deserialize(&mut store).unwrap();
        slf.increment();

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&slf).unwrap()
            + core::mem::size_of::<<GasConsumed as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
            <<GasConsumedIncrement as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }

    #[no_mangle]
    fn decrement(written_state: u32, _written_data: u32) -> [u32; 2] {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let slf = unsafe {
            archived_root::<GasConsumed>(&SCRATCH[..written_state as usize])
        };

        let mut slf: GasConsumed = (slf).deserialize(&mut store).unwrap();
        slf.decrement();

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&slf).unwrap()
            + core::mem::size_of::<<GasConsumed as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
            <<GasConsumedDecrement as Transaction>::Return as Archive>::Archived,
        >();

        [state_len as u32, return_len as u32]
    }
};
