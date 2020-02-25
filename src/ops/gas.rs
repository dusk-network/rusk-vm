use super::AbiCall;
use crate::host_fns::{CallContext, Resolver};
use crate::VMError;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Gas;

impl<S: Resolver> AbiCall<S> for Gas {
    const NAME: &'static str = "gas";
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S>,
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
