// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(option_result_unwrap_unchecked)]
#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use bytecheck::CheckBytes;
use microkelvin::{MaybeArchived, OffsetLen};
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::{Apply, Execute, Query, Transaction};

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
    fn execute(&self, q: &NumSecrets) -> <NumSecrets as Query>::Return {
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
    fn apply(&mut self, t: &Gossip) -> <Gossip as Transaction>::Return {
        if let Some(mut branch) = self.open_secrets.get_mut(&t.0) {
            *branch.leaf_mut() += 1;
        }

        self.open_secrets.insert(t.0.clone(), 1);
    }
}

#[cfg(target_family = "wasm")]
const _: () = {
    use rkyv::archived_root;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rusk_uplink::AbiStore;

    #[no_mangle]
    static mut SCRATCH: [u8; 128] = [0u8; 128];

    #[no_mangle]
    fn nums(written: u32) -> u32 {
        let mut store = AbiStore;

        let (state, arg) = unsafe {
            archived_root::<(Counter, ReadCount)>(&SCRATCH[..written as usize])
        };

        let de_state: Counter = (state).deserialize(&mut store).unwrap();
        let de_query: ReadCount = (arg).deserialize(&mut store).unwrap();

        let res: <ReadCount as Query>::Return = de_state.execute(&de_query);
        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };
        let buffer_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<ReadCount as Query>::Return as Archive>::Archived,
            >();
        buffer_len as u32
    }

    #[no_mangle]
    fn goss(written: u32) -> [u32; 2] {
        let mut store = AbiStore;

        let (state, arg) = unsafe {
            archived_root::<(Counter, Increment)>(&SCRATCH[..written as usize])
        };

        let mut de_state: Counter = (state).deserialize(&mut store).unwrap();
        let de_transaction: Increment = (arg).deserialize(&mut store).unwrap();

        let res: <Increment as Transaction>::Return =
            de_state.apply(&de_transaction);

        let mut ser = unsafe { BufferSerializer::new(&mut SCRATCH) };

        let state_len = ser.serialize_value(&de_state).unwrap()
            + core::mem::size_of::<<Counter as Archive>::Archived>();

        let return_len = ser.serialize_value(&res).unwrap()
            + core::mem::size_of::<
                <<Increment as Transaction>::Return as Archive>::Archived,
            >();

        [state_len as u32, return_len as u32]
    }
};
