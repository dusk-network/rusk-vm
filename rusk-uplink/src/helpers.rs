// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::hash::Hash;
use std::collections::hash_map::DefaultHasher;

// todo! proper hash will be implemented in issue 344
pub fn hash_mocker(bytes: &[u8]) -> [u8; 32] {
    use std::convert::TryFrom;
    use std::hash::Hasher;
    let mut a: [u8; 32] =
        <[u8; 32]>::try_from(&bytes[bytes.len() - 32..]).unwrap();
    let mut s = DefaultHasher::new();
    bytes.hash(&mut s);
    a[24..].copy_from_slice(&s.finish().to_le_bytes());
    a
}
