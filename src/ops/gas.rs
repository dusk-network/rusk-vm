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

impl AbiCall for GasConsumed {
    const ARGUMENTS: &'static [ValueType] = &[];
    const RETURN: Option<ValueType> = Some(ValueType::I64);

    fn call(
        _context: &mut CallContext,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        // TODO get the gas consumed from dusk_abi API
        // https://github.com/dusk-network/rusk-vm/pull/176
        Ok(Some(RuntimeValue::from(1)))
    }
}
