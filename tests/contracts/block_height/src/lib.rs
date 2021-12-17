// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use microkelvin::{BranchRef, BranchRefMut, Ident, MaybeArchived, Offset, Store};
use rkyv::{Archive, Deserialize, Serialize};
use rusk_uplink::helpers::Map;
use rusk_uplink::{AbiStore, HostRawStore};
use rusk_uplink::{Apply, Execute, Query, Transaction};


// query ids
pub const BLOCK_HEIGHT: u8 = 0;

#[derive(Clone, Archive, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[archive(as = "Self")]
pub struct BlockHeight {}

impl BlockHeight {
    pub fn new() -> Self {
        BlockHeight {}
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {

    extern crate alloc;

    use super::*;

    const PAGE_SIZE: usize = 1024 * 4;

    impl BlockHeight {
        pub fn block_height(&self) -> u64 {
            //dusk_abi::block_height() // todo!
            6159
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), ()> { // todo error type
        let mut store = HostRawStore::new(bytes);
        let slf_size = core::mem::size_of::<<BlockHeight as Archive>::Archived>();
        let offset = 0 + slf_size;
        let ofs = Offset::new(offset as u64);
        let ident = Ident::<Offset, BlockHeight>::new(ofs);
        let slf = store.get_raw::<BlockHeight>(&ident);
        // read query id
        let qid = 0; //u8::decode(&mut source)?;
        match qid {
            BLOCK_HEIGHT => {
                let ret = slf.block_height();

                // return value
                // let wrapped_return = ReturnValue::from_canon(&ret);
                //
                // let mut sink = Sink::new(&mut bytes[..]);
                //
                // wrapped_return.encode(&mut sink);

                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = query(bytes);
    }
}
