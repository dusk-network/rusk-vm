// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dataview::Pod;
use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct SelfHash;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for SelfHash {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let buffer_ofs = args.get(0)? as usize;
        let callee = context.callee();

        context.memory_mut(|a| {
            a[buffer_ofs..buffer_ofs + 32].copy_from_slice(callee.as_bytes())
        });
        Ok(None)
    }
}
