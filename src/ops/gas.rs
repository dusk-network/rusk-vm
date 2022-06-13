// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct GasConsumed;

impl GasConsumed {
    pub fn gas_consumed(env: &Env) -> Result<u64, VMError> {
        trace!("Executing 'gas_consumed' host function");

        let context = env.get_context();

        Ok(context.gas_meter()?.spent())
    }
}

pub struct GasLeft;

impl GasLeft {
    pub fn gas_left(env: &Env) -> Result<u64, VMError> {
        trace!("Executing 'gas_left' host function");

        let context = env.get_context();
        let gas_left = context.gas_meter()?.left();
        Ok(gas_left)
    }
}
