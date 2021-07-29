// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

#[cfg(target_arch = "wasm32")]
mod hosted {
    use super::*;

    use nstack::NStack;

    const PAGE_SIZE: usize = 1024 * 4;

    use canonical::{Canon, CanonError, Sink, Source};
    use dusk_abi::ReturnValue;

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), CanonError> {
        let mut source = Source::new(&bytes[..]);
        let _slf: NStack<[u8; 64], ()> = Canon::decode(&mut source)?;
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
