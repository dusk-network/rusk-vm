// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// Gas units are chosen to be represented by u64 so that gas metering
// instructions can operate on them efficiently.

use wasmer::Instance;
use wasmer_middlewares::metering::{
    get_remaining_points, set_remaining_points, MeteringPoints,
};

/// Type alias for gas
pub type Gas = u64;

pub enum GasError {
    /// Gas limit exceeded
    GasLimitExceeded,
}

#[derive(Debug, Clone)]
/// Struct to keep track of gas usage
pub struct GasMeter {
    /// Initial gas limit
    limit: Gas,
    /// Amount of gas left from initial gas limit; can reach zero
    left: Gas,
}

impl GasMeter {
    /// Default percentage of gas to be given to a [`GasMeter`] when
    /// [`limited`](`Self::limited`) is called.
    pub const RESERVE_PERCENTAGE: u64 = 93;

    /// Creates a new `GasMeter` with given gas limits
    pub fn with_limit(limit: Gas) -> GasMeter {
        GasMeter { limit, left: limit }
    }

    /// Deduct specified amount of gas from the meter
    pub fn charge(&mut self, amount: Gas) -> Result<(), GasError> {
        match self.left.checked_sub(amount) {
            Some(val) => {
                self.left = val;
                Ok(())
            }
            // If for any reason, we fall below the threshold, we run out of gas
            // directly consuming all of the gas left.
            None => {
                self.left = 0;
                Err(GasError::GasLimitExceeded)
            }
        }
    }

    /// Exhausts the gas meter
    pub fn exhaust(&mut self) {
        self.left = 0;
    }

    /// Returns how much gas left from the initial budget.
    pub fn left(&self) -> Gas {
        self.left
    }

    /// Returns the gas limit.
    pub fn limit(&self) -> Gas {
        self.limit
    }

    /// Returns how much gas was actually spent.
    pub fn spent(&self) -> Gas {
        self.limit - self.left
    }

    /// Updates both gas meter and instance gas points, taking into account the
    /// `charge` amount.
    pub(crate) fn update(
        &mut self,
        instance: &Instance,
        charge: Gas,
    ) -> Result<(), GasError> {
        let remaining = get_remaining_points(instance);
        match remaining {
            MeteringPoints::Remaining(r) => {
                self.left = r as u64;

                if charge > 0 {
                    let result = self.charge(charge);
                    set_remaining_points(instance, self.left);
                    return result;
                }
            }
            MeteringPoints::Exhausted => {
                self.left = 0;
                return Err(GasError::GasLimitExceeded);
            }
        }

        Ok(())
    }

    /// Create a new limited [`GasMeter`].
    /// If `limit` parameter is `0`, default limit is assumed that satisfies
    /// the requirement for obligatory gas reserve ([`Self::RESERVE_PERCENTAGE`]
    /// of the gas left).
    /// The limit provided cannot exceed the gas left, if that happens the
    /// total amount of gas is used as limit.
    pub fn limited(&self, limit: Gas) -> GasMeter {
        let limit = if limit == 0 {
            self.left() * Self::RESERVE_PERCENTAGE / 100
        } else {
            core::cmp::min(limit, self.left())
        };

        GasMeter { limit, left: limit }
    }
}
