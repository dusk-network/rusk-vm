// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::AbiCall;
use crate::call_context::CallContext;
use crate::VMError;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};
use crate::resolver::Env;
use crate::NetworkState;

pub struct BlockHeight;

impl AbiCall for BlockHeight {
    const ARGUMENTS: &'static [ValueType] = &[];
    const RETURN: Option<ValueType> = Some(ValueType::I64);

    fn call(
        context: &mut CallContext,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let block_height = context.state().block_height();

        Ok(Some(RuntimeValue::from(block_height)))
    }
}

impl BlockHeight {
    pub fn block_height(env: &Env) -> Result<u64, VMError> {
        Ok(env.height)
    }
}
