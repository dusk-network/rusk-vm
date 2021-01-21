// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use super::AbiCall;
use crate::call_context::{CallContext, Resolver};
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Panic;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for Panic {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let &[RuntimeValue::I32(panic_ofs), RuntimeValue::I32(panic_len)] = args.as_ref() {
            let panic_ofs = panic_ofs as usize;
            let panic_len = panic_len as usize;

            context.memory(|a| {
                Err(
                    match String::from_utf8(
                        a[panic_ofs..panic_ofs + panic_len].to_vec(),
                    ) {
                        Ok(panic_msg) => {
                            VMError::ContractPanic(panic_msg)
                        }
                        Err(_) => VMError::InvalidUtf8,
                    },
                )
            })?
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
