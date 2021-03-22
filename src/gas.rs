// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// Gas units are chosen to be represented by u64 so that gas metering
// instructions can operate on them efficiently.

/// Type alias for gas
pub type Gas = u64;

#[derive(Debug, PartialEq, Eq)]
pub enum GasMeterResult {
    Proceed,
    OutOfGas,
}

impl GasMeterResult {
    pub const fn is_out_of_gas(&self) -> bool {
        match *self {
            GasMeterResult::OutOfGas => true,
            GasMeterResult::Proceed => false,
        }
    }
}

#[derive(Debug)]
/// Struct to keep track of gas usage
pub struct GasMeter {
    limit: Gas,
    /// Amount of gas left from initial gas limit. Can reach zero.
    gas_left: Gas,
}

impl GasMeter {
    /// Creates a new `GasMeter` with given gas limits
    pub fn with_limit(gas_limit: Gas) -> GasMeter {
        GasMeter {
            limit: gas_limit,
            gas_left: gas_limit,
        }
    }

    /// Deduct specified amount of gas from the meter
    pub fn charge(&mut self, amount: Gas) -> GasMeterResult {
        let new_value = match self.gas_left.checked_sub(amount) {
            None => None,
            Some(val) => Some(val),
        };

        // We always consume the gas even if there is not enough gas.
        self.gas_left = new_value.unwrap_or(0);

        match new_value {
            Some(_) => GasMeterResult::Proceed,
            None => GasMeterResult::OutOfGas,
        }
    }

    /// Returns how much gas left from the initial budget.
    pub fn gas_left(&self) -> Gas {
        self.gas_left
    }

    /// Returns how much gas was spent.
    pub fn spent(&self) -> Gas {
        self.limit - self.gas_left
    }
}
