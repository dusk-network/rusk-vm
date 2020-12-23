// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use super::AbiCall;
use crate::call_context::{CallContext, Resolver};
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct SelfId;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for SelfId {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let &[RuntimeValue::I32(result_ofs)] = args.as_ref() {
            let result_ofs = result_ofs as usize;
            let callee = context.callee().clone();

            context
                .memory_mut(|a| {
                    a[result_ofs..result_ofs + 32]
                        .copy_from_slice(callee.as_bytes());
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
