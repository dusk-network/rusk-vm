// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use crate::resolver::Env;

pub struct Gas;

impl Gas {
    pub fn gas(env: &Env, gas_charged: i32) -> Result<(), VMError> {
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        let meter = context.gas_meter_mut();
        if meter.charge(gas_charged as u64).is_out_of_gas() {
            return Err(VMError::OutOfGas);
        }
        Ok(())
    }
}

pub struct GasConsumed;

impl GasConsumed {
    pub const GAS_CONSUMED_CALL_COST: u64 = 1;
}

impl GasConsumed {
    pub fn gas_consumed(env: &Env) -> Result<u64, VMError> {
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        // FIXME: This will not always be correct since if the `gas_consumed =
        // ALL` the gas, this will add the extra cost of the call
        // which can't be consumed since it's not even there.
        Ok(context.gas_meter().spent() + GasConsumed::GAS_CONSUMED_CALL_COST)
    }
}

pub struct GasLeft;

impl GasLeft {
    pub fn gas_left(env: &Env) -> Result<u64, VMError> {
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        Ok(context.gas_meter().left())
    }
}
