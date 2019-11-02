#[cfg(test)]
use std::fmt::Debug;

// Gas units are chosen to be represented by u64 so that gas metering instructions can operate on
// them efficiently.
pub type Gas = u64;

#[derive(Debug, PartialEq, Eq)]
pub enum GasMeterResult {
    Proceed,
    OutOfGas,
}

impl GasMeterResult {
    pub fn is_out_of_gas(&self) -> bool {
        match *self {
            GasMeterResult::OutOfGas => true,
            GasMeterResult::Proceed => false,
        }
    }
}

#[derive(Debug)]
pub struct GasMeter {
    limit: Gas,
    /// Amount of gas left from initial gas limit. Can reach zero.
    gas_left: Gas,
}

impl GasMeter {
    pub fn with_limit(gas_limit: Gas) -> GasMeter {
        GasMeter {
            limit: gas_limit,
            gas_left: gas_limit,
        }
    }

    pub fn charge(&mut self, amount: Gas) -> GasMeterResult {
        let new_value = match self.gas_left.checked_sub(amount) {
            None => None,
            Some(val) => Some(val),
        };

        // We always consume the gas even if there is not enough gas.
        self.gas_left = new_value.unwrap_or_else(|| 0);

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
