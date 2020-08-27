// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use super::AbiCall;
use crate::call_context::{host_trap, ArgsExt, CallContext, Resolver};
use crate::VMError;

use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Debug;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for Debug {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let msg_ofs = args.get(0)? as usize;
        let msg_len = args.get(1)? as usize;

        context.memory(|a| {
            let slice = &a[msg_ofs..msg_ofs + msg_len];
            let str = std::str::from_utf8(slice)
                .map_err(|_| host_trap(VMError::InvalidUtf8))?;
            println!("CONTRACT DEBUG: {:?}", str);
            Ok(None)
        })
    }
}
