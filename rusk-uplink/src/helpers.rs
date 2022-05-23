// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::Params;

pub fn hash(bytes: &[u8]) -> [u8; 32] {
    let mut state = Params::new().hash_length(32).to_state();
    state.update(bytes);

    let mut buf = [0u8; 32];
    buf.copy_from_slice(state.finalize().as_ref());
    buf
}
