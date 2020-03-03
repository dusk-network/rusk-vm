use super::AbiCall;
use crate::host_fns::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dusk_abi::encoding;
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
        let buffer_ofs = args.get(0)?;
        let balance = context.balance()?;

        context.memory_mut().with_direct_access_mut(|a| {
            encoding::encode(&balance, &mut a[buffer_ofs..]).map(|_| ())
        })?;

        Ok(None)
    }
}
