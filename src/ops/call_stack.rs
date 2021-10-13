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

impl Callee {
    pub fn callee(env: &Env, result_ofs: u32) -> Result<(), VMError> {
        let result_ofs = result_ofs as usize;
        let mut network_state = NetworkState::with_block_height(env.height).restore(env.persisted_id.clone())?;
        let mut context = CallContext::new(&mut network_state, env.gas_meter.clone());
        let callee = *context.callee();

        context
            .memory_mut(|a| {
                a[result_ofs..result_ofs + 32]
                    .copy_from_slice(callee.as_bytes());
                Ok(())
            })
            .map_err(VMError::from_store_error)
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

            context
                .memory_mut(|a| {
                    a[result_ofs..result_ofs + 32]
                        .copy_from_slice(caller.as_bytes());
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

impl Caller {
    pub fn caller(env: &Env, result_ofs: u32) -> Result<(), VMError> {
        let result_ofs = result_ofs as usize;
        let mut network_state = NetworkState::with_block_height(env.height).restore(env.persisted_id.clone())?;
        let mut context = CallContext::new(&mut network_state, env.gas_meter.clone());
        let caller = *context.caller();

        context
            .memory_mut(|a| {
                a[result_ofs..result_ofs + 32]
                    .copy_from_slice(caller.as_bytes());
                Ok(())
            })
            .map_err(VMError::from_store_error)
    }
}