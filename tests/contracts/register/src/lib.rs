// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use bytecheck::CheckBytes;
use microkelvin::{MaybeArchived, OffsetLen, StoreRef};
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};
use rusk_uplink_derive::{apply, execute, init, query, state, transaction};

use dusk_hamt::{Hamt, Lookup};

#[derive(
    Copy,
    Clone,
    Archive,
    Default,
    Debug,
    Deserialize,
    Serialize,
    Hash,
    PartialEq,
    Eq,
    CheckBytes,
)]
#[archive(as = "Self")]
pub struct SecretHash([u8; 32]);

impl SecretHash {
    pub fn new(secret_data: [u8; 32]) -> Self {
        Self(secret_data)
    }
}

#[state(new = false)]
pub struct Register {
    pub open_secrets: Hamt<SecretHash, u32, (), OffsetLen>,
}

#[init]
fn init() {}

impl Register {
    pub fn new() -> Self {
        Register {
            open_secrets: Hamt::new(),
        }
    }
}

#[query]
pub struct NumSecrets(SecretHash);

impl Query for NumSecrets {
    const NAME: &'static str = "nums";
    type Return = u32;
}

#[execute(name = "nums")]
impl Execute<NumSecrets> for Register {
    fn execute(&self, q: NumSecrets, _: StoreRef<OffsetLen>) -> u32 {
        self.open_secrets
            .get(&q.0)
            .as_ref()
            .map(|branch| match branch.leaf() {
                MaybeArchived::Memory(m) => *m,
                MaybeArchived::Archived(a) => (*a).into(),
            })
            .unwrap_or(0)
    }
}

#[transaction]
pub struct Gossip(SecretHash);

impl Transaction for Gossip {
    const NAME: &'static str = "goss";
    type Return = ();
}

#[apply(name = "goss")]
impl Apply<Gossip> for Register {
    fn apply(&mut self, t: Gossip, _: StoreContext) {
        if let Some(mut branch) = self.open_secrets.get_mut(&t.0) {
            *branch.leaf_mut() += 1;
        } else {
            self.open_secrets.insert(t.0.clone(), 1);
        }
    }
}
