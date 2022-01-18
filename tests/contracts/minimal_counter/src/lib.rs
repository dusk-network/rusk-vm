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
use rusk_uplink::{get_state, get_state_and_arg, q_return, t_return, query_state_arg_fun, transaction_state_arg_fun};

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

    query_state_arg_fun!(read, Counter, ReadCount);

    transaction_state_arg_fun!(incr, Counter, Increment);
};
