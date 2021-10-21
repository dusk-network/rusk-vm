// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use crate::resolver::Env;
pub struct BlockHeight;

impl BlockHeight {
    pub fn block_height(env: &Env) -> Result<u64, VMError> {
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        Ok(context.state().block_height())
    }
}
