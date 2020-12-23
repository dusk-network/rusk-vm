// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use super::AbiCall;
use crate::call_context::{CallContext, Resolver};
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Return;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for Return {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        _context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let &[RuntimeValue::I32(ofs), RuntimeValue::I32(len)] = args.as_ref()
        {
            Err(VMError::ContractReturn(ofs, len))
        } else {
            todo!()
        }
    }
}
