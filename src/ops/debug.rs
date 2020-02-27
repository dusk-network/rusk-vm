use super::AbiCall;
use crate::host_fns::{host_trap, ArgsExt, CallContext, Resolver};
use crate::VMError;

use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Debug;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for Debug {
    const NAME: &'static str = "panic";
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        context.memory().with_direct_access(|a| {
            let slice = args.to_slice(a, 0)?;
            let str = std::str::from_utf8(slice)
                .map_err(|_| host_trap(VMError::InvalidUtf8))?;
            println!("CONTRACT DEBUG: {:?}", str);
            Ok(None)
        })
    }
}
