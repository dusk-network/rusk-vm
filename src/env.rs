// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use std::ffi::c_void;
use wasmer::WasmerEnv;

#[derive(Clone)]
pub struct ImportReference(pub *mut c_void);
unsafe impl Send for ImportReference {}
unsafe impl Sync for ImportReference {}

#[derive(WasmerEnv, Clone)]
pub struct Env {
    pub context: ImportReference,
}

impl Env {
    pub fn new(call_context: &mut CallContext) -> Env {
        Env {
            context: ImportReference(call_context as *mut _ as *mut c_void),
        }
    }

    pub fn get_context<'a>(&self) -> &'a mut CallContext {
        unsafe { &mut *(self.context.0 as *mut CallContext) }
    }
}
