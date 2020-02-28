use super::AbiCall;
use crate::host_fns::{ArgsExt, CallContext, Resolver, StackFrame};
use crate::VMError;

use dusk_abi::CALL_DATA_SIZE;
use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Return;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for Return {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let buffer_ofs = args.get(0)?;

        let StackFrame {
            ref mut memory,
            call_data,
            ..
        } = context.top_mut();

        // copy return value from memory into call_data
        memory.with_direct_access_mut(|a| {
            call_data
                .copy_from_slice(&a[buffer_ofs..buffer_ofs + CALL_DATA_SIZE]);
        });

        Err(VMError::ContractReturn)
    }
}
