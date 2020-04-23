use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use super::AbiCall;
use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct OpCode;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for OpCode {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let value_ofs = args.get(0)?;
        let opcode = context.opcode();

        context.write_at(value_ofs as usize, &opcode);

        Ok(None)
    }
}
