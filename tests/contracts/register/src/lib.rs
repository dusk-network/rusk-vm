// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(option_result_unwrap_unchecked)]
#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use bytecheck::CheckBytes;
use microkelvin::{MaybeArchived, OffsetLen, StoreRef};
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Apply, Execute, Query, StoreContext, Transaction};

use dusk_hamt::{Hamt, Lookup};

#[derive(
    Clone, Archive, Deserialize, Serialize, Hash, PartialEq, Eq, CheckBytes,
)]
#[archive(as = "Self")]
pub struct SecretHash([u8; 32]);

#[derive(Clone, Archive, Deserialize, Serialize)]
pub struct Register {
    open_secrets: Hamt<SecretHash, u32, (), OffsetLen>,
}

impl Register {
    pub fn new() -> Self {
        Register {
            open_secrets: Hamt::new(),
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
    fn execute(
        &self,
        q: &NumSecrets,
        _: StoreRef<OffsetLen>,
    ) -> <NumSecrets as Query>::Return {
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

impl Apply<Gossip> for Register {
    fn apply(&mut self, t: &Gossip, _: StoreContext) -> <Gossip as Transaction>::Return {
        if let Some(mut branch) = self.open_secrets.get_mut(&t.0) {
            *branch.leaf_mut() += 1;
        }

        self.open_secrets.insert(t.0.clone(), 1);
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::Serializer;
    use rusk_uplink::{AbiStore, StoreContext};

    #[no_mangle]
    static mut SCRATCH: [u8; 128] = [0u8; 128];

    #[no_mangle]
    fn nums(written_state: u32, written_data: u32) -> u32 {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<Register>(&SCRATCH[..written_state as usize])
        };
        let query = unsafe {
            archived_root::<NumSecrets>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let state: Register = state.deserialize(&mut store).unwrap();
        let query: NumSecrets = query.deserialize(&mut store).unwrap();

        let mut ser = store.serializer();
        let res = state.execute(&query, store);

        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<NumSecrets as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }

    #[no_mangle]
    fn goss(written_state: u32, written_data: u32) -> [u32; 2] {
        let mut store =
            StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));

        let state = unsafe {
            archived_root::<Register>(&SCRATCH[..written_state as usize])
        };
        let transaction = unsafe {
            archived_root::<Gossip>(
                &SCRATCH[written_state as usize..written_data as usize],
            )
        };

        let mut state: Register = state.deserialize(&mut store).unwrap();
        let gossip: Gossip = transaction.deserialize(&mut store).unwrap();

        state.apply(&gossip, store.clone()); // todo use clone temporarily to get it to compile as Kris will change this contract anyway

        let mut ser = store.serializer();

        let state_len = ser.serialize_value(&state).unwrap()
            + core::mem::size_of::<<Gossip as Archive>::Archived>();

        let return_len = ser.serialize_value(&()).unwrap()
            + core::mem::size_of::<
                <<Gossip as Transaction>::Return as Archive>::Archived,
            >();

        [state_len as u32, return_len as u32]
    }
};
