// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::resolver::Env;
use crate::VMError;

pub struct Callee;

impl Callee {
    pub fn callee(env: &Env, result_ofs: i32) -> Result<(), VMError> {
        let result_ofs = result_ofs as usize;
        let context: &mut CallContext =
            unsafe { &mut *(env.context.0 as *mut CallContext) };
        let callee = *context.callee();

        context.write_memory(callee.as_bytes(), result_ofs as u64)
    }
}

pub struct Caller;

impl Caller {
    pub fn caller(env: &Env, result_ofs: i32) -> Result<(), VMError> {
        let result_ofs = result_ofs as usize;
        let context: &mut CallContext =
            unsafe { &mut *(env.context.0 as *mut CallContext) };
        let caller = *context.caller();

        context.write_memory(caller.as_bytes(), result_ofs as u64)
    }
}
