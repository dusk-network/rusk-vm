// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use tracing::{debug, trace};

use crate::env::Env;
use crate::VMError;
pub struct Panic;

impl Panic {
    pub fn panic(
        env: &Env,
        panic_ofs: i32,
        panic_len: i32,
    ) -> Result<(), VMError> {
        trace!("Executing 'panic' host function");

        let panic_ofs = panic_ofs as u64;
        let panic_len = panic_len as usize;
        let context = env.get_context();
        let slice = context.read_memory(panic_ofs, panic_len)?;
        Err(match String::from_utf8(slice.to_vec()) {
            Ok(panic_msg) => {
                let contract_id = context.callee()?;
                debug!("Contract {} panic: {:?}", contract_id, panic_msg);
                VMError::ContractPanic(*contract_id, panic_msg)
            }
            Err(_) => {
                debug!("Invalid UTF-8 in panic");
                VMError::InvalidUtf8
            }
        })
    }
}
