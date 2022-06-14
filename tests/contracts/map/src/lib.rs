// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use dusk_hamt::{Hamt, Lookup};
use microkelvin::{MaybeArchived, OffsetLen};

use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};
use rusk_uplink_derive::{apply, execute, init, query, state, transaction};

#[state(new = false)]
pub struct Map {
    inner: Hamt<u8, u32, (), OffsetLen>,
}

#[init]
fn init() {}

impl Map {
    pub fn new() -> Self {
        Self { inner: Hamt::new() }
    }

    pub fn set(&mut self, key: u8, value: u32) -> Option<u32> {
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &u8) -> Option<u32> {
        self.inner.get(key).as_ref().map(|x| match x.leaf() {
            MaybeArchived::Memory(m) => *m,
            MaybeArchived::Archived(a) => *a,
        })
    }

    pub fn remove(&mut self, key: &u8) -> Option<u32> {
        self.inner.remove(key)
    }
}

#[transaction]
pub struct Set {
    key: u8,
    value: u32,
}

impl Transaction for Set {
    const NAME: &'static str = "SET";
    type Return = Option<u32>;
}

#[apply(name = "SET")]
impl Apply<Set> for Map {
    fn apply(&mut self, t: Set, _: StoreContext) -> Option<u32> {
        self.set(t.key, t.value)
    }
}

#[transaction]
pub struct Remove {
    key: u8,
}

impl Transaction for Remove {
    const NAME: &'static str = "REMOVE";
    type Return = Option<u32>;
}

#[apply(name = "REMOVE")]
impl Apply<Remove> for Map {
    fn apply(&mut self, t: Remove, _: StoreContext) -> Option<u32> {
        self.remove(&t.key)
    }
}

#[query]
pub struct Get {
    key: u8,
}

impl Query for Get {
    const NAME: &'static str = "GET";
    type Return = Option<u32>;
}

#[execute(name = "GET")]
impl Execute<Get> for Map {
    fn execute(&self, q: Get, _: StoreContext) -> Option<u32> {
        self.get(&q.key)
    }
}
