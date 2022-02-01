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
use rusk_uplink_derive::{query, transaction, argument};

#[derive(Default, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Stack {
    inner: NStack<u64, Cardinality, OffsetLen>,
}

#[argument]
pub struct Peek {
    value: u64,
}

#[query(name="peek", buf=65536)]
impl Execute<Peek> for Stack {
    fn execute(&self, arg: Peek, _: StoreContext) -> Option<u64> {
        self.peek(arg.value)
    }
}

#[argument]
pub struct Push {
    value: u64,
}

#[transaction(name="push", buf=65536)]
impl Apply<Push> for Stack {
    fn apply(
        &mut self,
        arg: Push,
        _: StoreContext,
    ) {
        self.push(arg.value);
    }
}

#[argument]
pub struct Pop;

#[transaction(name="pop", buf=65536)]
impl Apply<Pop> for Stack {
    fn apply(
        &mut self,
        _: Pop,
        _: StoreContext,
    ) -> Option<u64> {
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
