// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::{Params, State};

pub struct Hasher(State);

pub fn hash(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

impl Hasher {
    pub fn new() -> Self {
        Self(Params::new().hash_length(32).to_state())
    }

    pub fn update<B: AsRef<[u8]>>(&mut self, buf: B) -> &mut Self {
        self.0.update(buf.as_ref());
        self
    }

    pub fn finalize(&mut self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf.copy_from_slice(self.0.finalize().as_ref());
        buf
    }
}
