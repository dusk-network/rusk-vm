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
    pub fn debug(env: &Env, msg_ofs: i32, msg_len: i32) -> Result<(), VMError> {
        let msg_ofs = msg_ofs as u64;
        let msg_len = msg_len as usize;
        let context: &mut CallContext =
            unsafe { &mut *(env.context.0 as *mut CallContext) };
        let messsage_memory = context.read_memory(msg_ofs, msg_len)?;
        let str = std::str::from_utf8(&messsage_memory)
            .map_err(|_| VMError::InvalidUtf8)?;
        println!("CONTRACT DEBUG: {:?}", str);
        Ok(())
    }
}
