// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::AbiCall;
use crate::call_context::CallContext;
use crate::{VMError, VMResult};

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};
use crate::resolver::Env;

pub struct Panic;

impl AbiCall for Panic {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(panic_ofs), RuntimeValue::I32(panic_len)] =
            *args.as_ref()
        {
            let panic_ofs = panic_ofs as usize;
            let panic_len = panic_len as usize;

            context.memory(|a| {
                Err(
                    match String::from_utf8(
                        a[panic_ofs..panic_ofs + panic_len].to_vec(),
                    ) {
                        Ok(panic_msg) => VMError::ContractPanic(panic_msg),
                        Err(_) => VMError::InvalidUtf8,
                    },
                )
            })?
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

impl Panic {
    // pub fn panic(env: &Env, panic_ofs: u32, panic_len: u32) -> Result<(), VMError> {
    //     let context = env.persisted_id.restore()?;
    //     context.memory(|a| {
    //         Err(
    //             match String::from_utf8(
    //                 a[panic_ofs..panic_ofs + panic_len].to_vec(),
    //             ) {
    //                 Ok(panic_msg) => VMError::ContractPanic(panic_msg),
    //                 Err(_) => VMError::InvalidUtf8,
    //             },
    //         )
    //     })?
    // }
    pub fn panic(env: &Env, panic_ofs: u32, panic_len: u32) -> Result<(), VMError> {
        Err(VMError::InvalidArguments)
    }
}
