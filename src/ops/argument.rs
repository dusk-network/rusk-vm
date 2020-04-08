use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Argument;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for Argument {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let ofs = args.get(0)? as usize;
        let len = args.get(1)? as usize;

        context.copy_argument(ofs, len);

        Ok(None)
    }
}
