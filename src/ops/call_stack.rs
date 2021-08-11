// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::AbiCall;
use crate::call_context::CallContext;
use crate::VMError;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Callee;

impl AbiCall for Callee {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(result_ofs)] = *args.as_ref() {
            let result_ofs = result_ofs as usize;
            let callee = *context.callee();

            context.memory_mut(|a| {
                a[result_ofs..result_ofs + 32]
                    .copy_from_slice(callee.as_bytes());
                Ok(None)
            })
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

pub struct Caller;

impl AbiCall for Caller {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(result_ofs)] = *args.as_ref() {
            let result_ofs = result_ofs as usize;
            let caller = *context.caller();

            context.memory_mut(|a| {
                a[result_ofs..result_ofs + 32]
                    .copy_from_slice(caller.as_bytes());
                Ok(None)
            })
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
