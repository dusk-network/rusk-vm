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
use rusk_uplink::{AbiStore, Apply, Execute, Query, Transaction};

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
    type Return = ();
}

impl Execute<ReadCount> for Counter {
    fn execute(&self, _: &ReadCount) -> <ReadCount as Query>::Return {
        self.value.into()
    }
}

impl Apply<Increment> for Counter {
    fn apply(&mut self, t: &Increment) -> <Increment as Transaction>::Return {
        self.value += t.0;
    }
}

#[no_mangle]
unsafe fn read(
    s: &<Counter as Archive>::Archived,
    q: &<ReadCount as Archive>::Archived,
    _ret: *mut <<ReadCount as Query>::Return as Archive>::Archived,
) {
    let mut store = AbiStore;
    let de_state: Counter = (&*s).deserialize(&mut store).unwrap_unchecked();
    let de_query: ReadCount = (&*q).deserialize(&mut store).unwrap_unchecked();
    let _res: <ReadCount as Query>::Return = de_state.execute(&de_query);
    todo!()
}

#[no_mangle]
unsafe fn incr(
    s: &mut <Counter as Archive>::Archived,
    t: &<Increment as Archive>::Archived,
    _ret: *mut <<Increment as Transaction>::Return as Archive>::Archived,
) {
    let mut store = AbiStore;
    let mut de_state: Counter =
        (&*s).deserialize(&mut store).unwrap_unchecked();
    let de_transaction: Increment =
        (&*t).deserialize(&mut store).unwrap_unchecked();
    let _res = de_state.apply(&de_transaction);
    todo!()
}
