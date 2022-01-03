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
    pub fn block_height(env: &Env, result_ofs: i32) -> Result<(), VMError> {
        trace!("Executing 'block_height' host function");
        let block_height = env.get_context().block_height();
        let context = env.get_context();
        context.write_memory(&block_height.to_le_bytes(), result_ofs as u64);
        Ok(())
    }
}
