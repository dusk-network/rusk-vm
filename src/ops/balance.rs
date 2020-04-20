use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use super::AbiCall;
use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Balance;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for Balance {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let value_ofs = args.get(0)?;
        let balance = context.balance()?;

        context.write_at(value_ofs as usize, &balance);

        Ok(None)
    }
}
