use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dusk_abi::CALL_DATA_SIZE;

use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct CallData;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for CallData {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let call_data_ofs = args.get(0)?;

        context.memory().with_direct_access_mut(|a| {
            a[call_data_ofs..call_data_ofs + CALL_DATA_SIZE]
                .copy_from_slice(context.data())
        });
        Ok(None)
    }
}
