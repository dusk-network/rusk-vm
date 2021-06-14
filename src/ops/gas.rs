// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::AbiCall;
use crate::call_context::CallContext;
use crate::VMError;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Gas;

impl AbiCall for Gas {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let meter = context.gas_meter_mut();
        let gas: u32 = args.nth_checked(0)?;
        if meter.charge(gas as u64).is_out_of_gas() {
            return Err(VMError::OutOfGas);
        }
        Ok(None)
    }
}

pub struct GasConsumed;

impl GasConsumed {
    pub const GAS_CONSUMED_CALL_COST: u64 = 1;
}

impl AbiCall for GasConsumed {
    const ARGUMENTS: &'static [ValueType] = &[];
    const RETURN: Option<ValueType> = Some(ValueType::I64);

    fn call(
        context: &mut CallContext,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        // FIXME: This will not always be correct since if the `gas_consumed =
        // ALL` the gas, this will add the extra cost of the call
        // which can't be consumed since it's not even there.
        Ok(Some(RuntimeValue::from(
            context.gas_meter().spent() + GasConsumed::GAS_CONSUMED_CALL_COST,
        )))
    }
}

pub struct GasLeft;

impl AbiCall for GasLeft {
    const ARGUMENTS: &'static [ValueType] = &[];
    const RETURN: Option<ValueType> = Some(ValueType::I64);

    fn call(
        context: &mut CallContext,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        Ok(Some(RuntimeValue::from(context.gas_meter().left())))
    }
}
