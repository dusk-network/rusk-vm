// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use microkelvin::{All, Cardinality, Compound, Nth, OffsetLen};
use nstack::NStack;
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};
use rusk_uplink_derive::{apply, execute, init, query, state, transaction};

#[state(new = false)]
pub struct Stack {
    inner: NStack<u64, Cardinality, OffsetLen>,
}
#[init]
fn init() {}

#[query]
pub struct Peek {
    value: u64,
}

impl Query for Peek {
    const NAME: &'static str = "peek";
    type Return = Option<u64>;
}

#[execute(name = "peek")]
impl Execute<Peek> for Stack {
    fn execute(&self, arg: Peek, _: StoreContext) -> Option<u64> {
        self.peek(arg.value)
    }
}

#[transaction]
pub struct Push {
    value: u64,
}

impl Transaction for Push {
    const NAME: &'static str = "push";
    type Return = ();
}

#[apply(name = "push")]
impl Apply<Push> for Stack {
    fn apply(&mut self, arg: Push, _: StoreContext) {
        self.push(arg.value);
    }
}
#[transaction]
pub struct PushMulti {
    value: u64,
}
impl Transaction for PushMulti {
    const NAME: &'static str = "pushmulti";
    type Return = ();
}
#[apply(name = "pushmulti")]
impl Apply<PushMulti> for Stack {
    fn apply(&mut self, arg: PushMulti, _: StoreContext) {
        self.pushmulti(arg.value);
    }
}

#[transaction]
pub struct Pop;

impl Transaction for Pop {
    const NAME: &'static str = "pop";
    type Return = Option<u64>;
}

#[apply(name = "pop")]
impl Apply<Pop> for Stack {
    fn apply(&mut self, _: Pop, _: StoreContext) -> Option<u64> {
        self.pop()
    }
}
#[transaction]
pub struct PopMulti {
    value: u64,
}
impl Transaction for PopMulti {
    const NAME: &'static str = "popmulti";
    type Return = u64;
}
#[apply(name = "popmulti")]
impl Apply<PopMulti> for Stack {
    fn apply(&mut self, arg: PopMulti, _: StoreContext) -> u64 {
        self.popmulti(arg.value)
    }
}
#[transaction]
pub struct Unarchive;
impl Transaction for Unarchive {
    const NAME: &'static str = "unarchive";
    type Return = ();
}
#[apply(name = "unarchive")]
impl Apply<Unarchive> for Stack {
    fn apply(&mut self, _: Unarchive, _: StoreContext) {
        /*
        Unarchive all data in the state
         */
        let branch_mut = self.inner.walk_mut(All).expect("Some(Branch)");
        for leaf in branch_mut {
            *leaf += 0;
        }
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

    pub fn pushmulti(&mut self, value: u64) {
        for i in 0..value {
            self.inner.push(i);
        }
    }
    pub fn pop(&mut self) -> Option<u64> {
        self.inner.pop()
    }
    pub fn popmulti(&mut self, value: u64) -> u64 {
        let mut sum = 0u64;
        for _ in 0..value {
            let popped = self.pop().unwrap();
            sum += popped;
        }
        sum
    }
}
