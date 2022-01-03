// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct Callee;

impl Callee {
    pub fn callee(env: &Env, result_ofs: i32) -> Result<(), VMError> {
        trace!("Executing 'callee' host function");

        let _result_ofs = result_ofs as usize;
        let context = env.get_context();
        let callee = *context.callee();

        context.write_memory(callee.as_bytes(), result_ofs as u64);

        Ok(())
    }
}

pub struct Caller;

impl Caller {
    pub fn caller(env: &Env, result_ofs: i32) -> Result<(), VMError> {
        trace!("Executing 'caller' host function");

        let _result_ofs = result_ofs as usize;
        let context = env.get_context();
        let caller = *context.caller();

        context.write_memory(caller.as_bytes(), result_ofs as u64);

        Ok(())
    }
}
