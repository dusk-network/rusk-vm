// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct BlockHeight;

impl BlockHeight {
    pub fn block_height(env: &Env) -> Result<u64, VMError> {
        trace!("Executing 'block_height' host function");
        Ok(env.get_context().block_height())
    }
}
