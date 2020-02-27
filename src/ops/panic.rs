use super::AbiCall;
use crate::host_fns::{host_trap, ArgsExt, CallContext, Resolver};
use crate::VMError;

use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Panic;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for Panic {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let panic_ofs = args.get(0)?;
        let panic_len = args.get(1)?;

        context.memory().with_direct_access(|a| {
            Err(
                match String::from_utf8(
                    a[panic_ofs..panic_ofs + panic_len].to_vec(),
                ) {
                    Ok(panic_msg) => {
                        host_trap(VMError::ContractPanic(panic_msg))
                    }
                    Err(_) => host_trap(VMError::InvalidUtf8),
                },
            )
        })?
    }
}
