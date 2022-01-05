// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct Gas;

impl Gas {
    pub fn gas(env: &Env, gas_charged: i32) -> Result<(), VMError> {
        let context = env.get_context();
        let meter = context.gas_meter_mut();
        meter.charge(gas_charged as u64)?;
        Ok(())
    }
}

pub struct GasConsumed;

impl GasConsumed {
    pub const GAS_CONSUMED_CALL_COST: u64 = 1;
}

impl GasConsumed {
    pub fn gas_consumed(env: &Env, result_ofs: i32) -> Result<(), VMError> {
        trace!("Executing 'gas_consumed' host function");

        let context = env.get_context();
        // FIXME: This will not always be correct since if the `gas_consumed =
        // ALL` the gas, this will add the extra cost of the call
        // which can't be consumed since it's not even there.
        let consumed =
            context.gas_meter().spent() + GasConsumed::GAS_CONSUMED_CALL_COST;
        context.write_memory(&consumed.to_le_bytes(), result_ofs as u64);
        Ok(())
    }
}

pub struct GasLeft;

impl GasLeft {
    pub fn gas_left(env: &Env, result_ofs: i32) -> Result<(), VMError> {
        trace!("Executing 'gas_left' host function");

        let context = env.get_context();

        context.write_memory(
            &context.gas_meter().left().to_le_bytes(),
            result_ofs as u64,
        );
        Ok(())
    }
}
