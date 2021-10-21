// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use crate::resolver::Env;

pub struct Panic;


impl Panic {
    pub fn panic(env: &Env, panic_ofs: u32, panic_len: u32) -> Result<(), VMError> {
        let panic_ofs = panic_ofs as u64;
        let panic_len = panic_len as usize;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        let slice = context.read_memory(panic_ofs, panic_len)?;
        Err(
            match String::from_utf8(slice.to_vec()) {
                Ok(panic_msg) => VMError::ContractPanic(panic_msg),
                Err(_) => VMError::InvalidUtf8,
            }
        )
    }
}
