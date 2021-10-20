// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use crate::resolver::Env;

pub struct Debug;

impl Debug {
    pub fn debug(env: &Env, msg_ofs: u32, msg_len: u32) -> Result<(), VMError> {
        let msg_ofs_u = msg_ofs as usize;
        let msg_len_u = msg_len as usize;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        let v = context.read_memory()?;
        let slice = &v[msg_ofs_u..msg_ofs_u + msg_len_u];
        let str = std::str::from_utf8(slice)
            .map_err(|_| VMError::InvalidUtf8)?;
        println!("CONTRACT DEBUG: {:?}", str);
        Ok(())
    }
}
