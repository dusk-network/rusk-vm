// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use rkyv::{Archive, Deserialize, Infallible, Serialize};

mod vm;
use vm::{Apply, Method, Query};

use microkelvin::{Ephemeral, Portal, Stored};

#[derive(Clone, Debug, Archive, Deserialize, Serialize)]
pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn new(value: u32) -> Self {
        Counter { value }
    }
}

#[derive(Archive, Serialize, Debug)]
pub struct ReadCount;

impl Method for ReadCount {
    const NAME: &'static str = "read";
    type Return = u32;
}

#[derive(Archive, Serialize, Debug)]
pub struct Increment(u32);

impl Method for Increment {
    const NAME: &'static str = "incr";
    type Return = ();
}

impl Query<ReadCount> for Counter {
    fn query(
        archived: &Self::Archived,
        _: &<ReadCount as Archive>::Archived,
    ) -> <ReadCount as Method>::Return {
        archived.value.into()
    }
}

impl Apply<Increment> for Counter {
    fn apply(
        &mut self,
        t: &<Increment as Archive>::Archived,
    ) -> <Increment as Method>::Return {
        let unarchived: u32 = t.0.into();
        self.value += unarchived;
    }
}

#[no_mangle]
fn read(
    s: &<Counter as Archive>::Archived,
    q: &<ReadCount as Archive>::Archived,
) -> Ephemeral<<ReadCount as Method>::Return> {
    Portal::ephemeral(&Counter::query(s, q))
}

#[no_mangle]
fn incr(
    s: &<Counter as Archive>::Archived,
    t: &<Increment as Archive>::Archived,
) -> (Stored<Counter>, Ephemeral<<Increment as Method>::Return>) {
    let mut de: Counter = s.deserialize(&mut Infallible).expect("infallible");
    Counter::apply(&mut de, t);
    let result = Counter::apply(&mut de, t);
    let return_value = Portal::ephemeral(&result);
    let new_state = microkelvin::Portal::put(&de);
    (new_state, return_value)
}
