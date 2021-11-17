// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// Gas units are chosen to be represented by u64 so that gas metering
// instructions can operate on them efficiently.

use core::ops::Range;
use std::cmp::{max, min};
use gas_constants::*;

/// Type alias for gas
pub type Gas = u64;

/// Constants for inter-contract call gas reserve calculations
pub mod gas_constants {
    /// Percentage of gas to be given to a callee
    pub const GAS_RESERVE_PERCENTAGE: u64 = 93;
    /// Constant needed for percentage calculations
    pub const HUNDRED_PERCENT: u64 = 100;
}

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

#[derive(Debug, Clone)]
/// Struct to keep track of gas usage
pub struct GasMeter {
    /// Gas held but not spent yet
    held: Gas,
    /// Initial gas limit
    limit: Gas,
    /// Amount of gas left from initial gas limit; can reach zero
    left: Gas,
}

impl GasMeter {
    /// Creates a new `GasMeter` with given gas limits
    pub fn with_limit(gas_limit: Gas) -> GasMeter {
        GasMeter::with_range(Range {
            start: 0,
            end: gas_limit,
        })
    }

    /// Creates a new `GasMeter` with given gas range.
    /// A range of `2_000..1_000_000` means that `2_000` gas will be
    /// held for known required calculation, and therefore the gas
    /// actually available is `1_000_000 - 2_000 = 800_000`.
    pub fn with_range(gas_range: Range<Gas>) -> GasMeter {
        GasMeter {
            held: gas_range.start,
            limit: gas_range.end,
            left: gas_range.end,
        }
    }

    /// Deduct specified amount of gas from the meter
    pub fn charge(&mut self, amount: Gas) -> GasMeterResult {
        match self.left.checked_sub(amount) {
            // If for any reason, we fall below the threshold, we run out of gas
            // directly consuming all of the gas left.
            None => {
                self.left = 0;
                GasMeterResult::OutOfGas
            }
            // If after subtracting the gas, the gas left in the Meter is
            // below [`GasMeter::held`]
            // we also abort the execution since no more stuff will be
            // possible to do.
            Some(val) if val <= self.held => {
                self.left = 0;
                GasMeterResult::OutOfGas
            }
            Some(val) => {
                self.left = val;
                GasMeterResult::Proceed
            }
        }
    }

    /// Returns how much gas left from the initial budget.
    #[deprecated(since = "0.6.0", note = "Please use `left` instead")]
    pub fn gas_left(&self) -> Gas {
        self.left
    }

    /// Returns how much gas left from the initial budget.
    /// This take in account [`GasMeter::held`].
    pub fn left(&self) -> Gas {
        self.left.saturating_sub(self.held)
    }

    /// Returns total number of gas left, disregarding [`GasMeter::held`].
    pub fn total_left(&self) -> Gas {
        self.left
    }

    /// Returns how much gas was actually spent.
    /// This does not consider [`GasMeter::held`] since it's not spent yet.
    pub fn spent(&self) -> Gas {
        self.limit - self.left
    }

    fn clone_for_callee_default(&self) -> GasMeter {
        let new_held = if self.left > self.held {
            self.held
                + ((self.left - self.held)
                    * (HUNDRED_PERCENT - GAS_RESERVE_PERCENTAGE)
                    / HUNDRED_PERCENT)
        } else {
            self.held
        };
        GasMeter {
            held: new_held,
            limit: self.left,
            left: self.left,
        }
    }

    fn clone_for_callee_with_limit(&self, limit: Gas) -> GasMeter {
        let new_held = max(self.left.saturating_sub(limit), self.held);
        GasMeter {
            held: new_held,
            limit: self.left,
            left: self.left,
        }
    }

    /// Clones gas meter for the use by a callee, modifying its held field
    /// so that it satisfies a limit parameter is given.
    /// If limit parameter is none, default limit is assumed that satisfies
    /// the requirement for obligatory gas reserve.
    /// Held field is set to the following value:
    ///     If limit is given:
    ///         held = this_left - limit
    ///     If limit is not given:
    ///         held = this_held + (this_left - this_held) *
    ///             (100 - GAS_RESERVE_PERCENTAGE) / 100
    /// Limit field is set to the following value:
    ///     (For both limit given or default):
    ///         limit = this_left
    pub fn clone_for_callee(&self, limit_option: Option<Gas>) -> GasMeter {
        match limit_option {
            Some(limit) => self.clone_for_callee_with_limit(limit),
            None => self.clone_for_callee_default(),
        }
    }

    /// Merges this gas meter with a gas meter obtained from a finished callee.
    /// It updates the 'left' field value from the callee.
    /// Update is only done if callee's 'left' field value is higher that this
    /// gas meter 'left' value.
    /// Fields limit and held are not changed as the are local for every
    /// gas context - they propagate up the stack but never down the stack.
    pub fn merge_with_callee(&mut self, callee_gas_meter: &GasMeter) {
        self.left = min(callee_gas_meter.left, self.left);
    }
}
