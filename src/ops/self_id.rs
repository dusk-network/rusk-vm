// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

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
