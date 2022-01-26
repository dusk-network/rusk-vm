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
use rusk_uplink::{Query, StoreContext, Transaction};

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

#[derive(Clone, Debug, Default, Archive, Serialize, Deserialize)]
pub struct Pop;

impl Transaction for Pop {
    const NAME: &'static str = "pop";
    type Return = Option<u64>;
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
    use rusk_uplink::framing_imports;
    framing_imports!();

    scratch_memory!(512);

    #[query]
    pub fn peek(state: &Stack, arg: Peek, _store: StoreRef<OffsetLen>) -> Option<u64> {
        state.peek(arg.value)
    }

    #[transaction]
    pub fn push(state: &mut Stack, arg: Push, _store: StoreRef<OffsetLen>) {
        state.push(arg.value);
    }

    #[transaction]
    pub fn pop(state: &mut Stack, _: Pop, _store: StoreRef<OffsetLen>) -> Option<u64> {
        state.pop()
    }
};
