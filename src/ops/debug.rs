// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use tracing::{debug, trace};

use crate::env::Env;
use crate::VMError;

pub struct Debug;

impl Debug {
    pub fn debug(env: &Env, msg_ofs: i32, msg_len: i32) -> Result<(), VMError> {
        trace!("Executing 'debug' host function");

        let msg_ofs = msg_ofs as u64;
        let msg_len = msg_len as usize;
        let context = env.get_context();
        let message_memory = context.read_memory(msg_ofs, msg_len)?;
        let str = std::str::from_utf8(message_memory)
            .map_err(|_| VMError::InvalidUtf8)?;

        println!("Contract debug: {:?}", str);

        Ok(())
    }
}
