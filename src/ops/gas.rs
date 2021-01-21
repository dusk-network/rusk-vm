// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use super::AbiCall;
use crate::call_context::{CallContext, Resolver};
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Gas;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for Gas {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        let meter = context.gas_meter_mut();
        let gas: u32 = args.nth_checked(0)?;
        if meter.charge(gas as u64).is_out_of_gas() {
            return Err(VMError::OutOfGas);
        }
        Ok(None)
    }
}
