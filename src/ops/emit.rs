// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct Emit;

impl Emit {
    /// Emit an event to the virtual machine. This event
    pub fn emit(
        env: &Env,
        data_ofs: i32,
        data_len: u32,
        name_ofs: i32,
        name_len: u32,
    ) -> Result<(), VMError> {
        trace!("Executing 'emit' host function");

        let data_ofs = data_ofs as u64;
        let data_len = data_len as usize;

        let name_ofs = name_ofs as u64;
        let name_len = name_len as usize;

        let context = env.get_context();
        let origin = *context.callee();

        let data_memory = context.read_memory(data_ofs, data_len)?;
        let name_memory = context.read_memory(name_ofs, name_len)?;

        let data = data_memory.to_vec();
        let name = String::from_utf8(name_memory.to_vec())
            .map_err(|_| VMError::InvalidUtf8)?;

        // push an event to the event stack
        context.push_event(origin, name, data);

        Ok(())
    }
}
