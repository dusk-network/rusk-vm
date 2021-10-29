// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::VMError;
use crate::env::Env;

pub struct BlockHeight;

impl BlockHeight {
    pub fn block_height(env: &Env) -> Result<u64, VMError> {
        Ok(env.get_context().state().block_height())
    }
}
