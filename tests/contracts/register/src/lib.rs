// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(option_result_unwrap_unchecked)]
#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use microkelvin::{BranchRef, BranchRefMut, MaybeArchived};
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::helpers::Map;
use rusk_uplink::AbiStore;
use rusk_uplink::{Apply, Execute, Query, Transaction};

#[derive(Clone, Archive, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[archive(as = "Self")]
pub struct SecretHash([u8; 32]);

#[derive(Clone, Archive, Deserialize, Serialize)]
pub struct Register {
    open_secrets: Map<SecretHash, u32>,
}

impl Register {
    pub fn new() -> Self {
        Register {
            open_secrets: Map::new(),
        }
    }
}

#[derive(Archive, Serialize, Deserialize)]
pub struct NumSecrets(SecretHash);

impl Query for NumSecrets {
    const NAME: &'static str = "nums";
    type Return = u32;
}

#[derive(Archive, Serialize, Deserialize)]
pub struct Gossip(SecretHash);

impl Transaction for Gossip {
    const NAME: &'static str = "goss";
    type Return = ();
}

impl Execute<NumSecrets> for Register {
    fn execute(&self, q: &NumSecrets) -> <NumSecrets as Query>::Return {
        self.open_secrets
            .get(&q.0)
            .map(|branch| match branch.leaf() {
                MaybeArchived::Memory(m) => *m,
                MaybeArchived::Archived(a) => (*a).into(),
            })
            .unwrap_or(0)
    }
}

impl Apply<Gossip> for Register {
    fn apply(&mut self, t: &Gossip) -> <Gossip as Transaction>::Return {
        if let Some(mut branch) = self.open_secrets.get_mut(&t.0) {
            *branch.leaf_mut() += 1;
        }

        self.open_secrets.insert(t.0.clone(), 1);
    }
}

#[no_mangle]
unsafe fn read(
    s: *const <Register as Archive>::Archived,
    q: *const <NumSecrets as Archive>::Archived,
    _ret: *mut <<NumSecrets as Query>::Return as Archive>::Archived,
) {
    let mut store = AbiStore;
    let state: Register = (&*s).deserialize(&mut store).unwrap();
    let query: NumSecrets = (&*q).deserialize(&mut store).unwrap();
    Register::execute(&state, &query);
    todo!()
}

#[no_mangle]
unsafe fn incr(
    s: *mut <Register as Archive>::Archived,
    t: *const <Gossip as Archive>::Archived,
    _ret: *mut <<Gossip as Transaction>::Return as Archive>::Archived,
) {
    let mut store = AbiStore;
    let mut state = (&*s).deserialize(&mut store).unwrap();
    let transaction = (&*t).deserialize(&mut store).unwrap();
    Register::apply(&mut state, &transaction);
    todo!()
}
