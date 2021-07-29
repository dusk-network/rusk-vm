// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use canonical_derive::Canon;
use dusk_bls12_381::BlsScalar;
use dusk_poseidon::tree::{PoseidonAnnotation, PoseidonLeaf, PoseidonTree};

#[derive(Clone, Canon, Default)]
pub struct Leaf {
    scalar: BlsScalar,
    pos: u64,
}

impl PoseidonLeaf for Leaf {
    fn poseidon_hash(&self) -> BlsScalar {
        self.scalar
    }

    fn pos(&self) -> &u64 {
        &self.pos
    }

    fn set_pos(&mut self, pos: u64) {
        self.pos = pos
    }
}

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    const PAGE_SIZE: usize = 1024 * 4;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::ReturnValue;

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);
        let _slf: PoseidonTree<Leaf, PoseidonAnnotation, 17> =
            Canon::decode(&mut source)?;
        let arg = u32::decode(&mut source)?;
        // return
        let mut sink = Sink::new(&mut bytes[..]);
        ReturnValue::from_canon(&arg).encode(&mut sink);
        Ok(())
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        let _ = query(bytes);
    }
}
