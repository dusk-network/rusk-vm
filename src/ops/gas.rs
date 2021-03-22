// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::AbiCall;
use crate::call_context::CallContext;
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Gas;

impl<S: Store> AbiCall<S> for Gas {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S>,
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

pub struct GasConsumed;

impl GasConsumed {
    pub const GAS_CONSUMED_CALL_COST: u64 = 9165;
}

impl<S: Store> AbiCall<S> for GasConsumed {
    const ARGUMENTS: &'static [ValueType] = &[];
    const RETURN: Option<ValueType> = Some(ValueType::I64);

    fn call(
        context: &mut CallContext<S>,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        // Note that when this function returns, the costs that invoking this
        // host function has are not taken into account here, they are
        // once the host call execution finnishes. Therefore, we need to
        // add to the result the gas_consumed fn cost.
        // FIXME: This will not always be correct since if the `gas_consumed =
        // ALL` the gas, this will add the extra cost of the call
        // which can't be consumed since it's not even there.
        Ok(Some(RuntimeValue::from(
            context.gas_consumed() + GasConsumed::GAS_CONSUMED_CALL_COST,
        )))
    }
}
