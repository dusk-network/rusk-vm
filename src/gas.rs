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
    /// Initial amount of gas added by the transactor.
    initial: Gas,
    /// Amount of gas left from initial gas limit. Can reach zero.
    gas_left: Gas,
}

impl GasMeter {
    /// Minimum amount of gas that has to be used in order to call a contract
    /// execution.
    // TODO: Add the correct value here that changes in respect to the transfer
    // contract.
    pub const MIN_TERMINATION_GAS_REQUIRED: Gas = 70440;

    /// Creates a new `GasMeter` with given initial gas.
    pub fn new(initial: Gas) -> GasMeter {
        GasMeter {
            initial,
            gas_left: initial,
        }
    }

    /// Deduct specified amount of gas from the meter
    pub fn charge(&mut self, amount: Gas) -> GasMeterResult {
        match self.gas_left.checked_sub(amount) {
            // If for any reason, we fall below the threshold, we run out of gas
            // directly consuming all of the gas left.
            None => {
                self.gas_left = 0;
                GasMeterResult::OutOfGas
            }
            Some(val) => match val {
                // If after subtracting the gas, the gas left in the Meter is
                // below [`GasMeter::MIN_TERMINATION_GAS_REQUIRED`]
                // we also abort the execution since no more stuff will be
                // possible to do.
                0..=Self::MIN_TERMINATION_GAS_REQUIRED => {
                    self.gas_left = 0;
                    GasMeterResult::OutOfGas
                }
                _ => {
                    self.gas_left = val;
                    GasMeterResult::Proceed
                }
            },
        }
    }

    /// Returns how much gas left from the initial budget.
    pub fn gas_left(&self) -> Gas {
        self.gas_left
    }

    /// Returns how much gas was spent.
    pub fn spent(&self) -> Gas {
        self.initial - self.gas_left
    }
}
