use super::AbiCall;
use crate::host_fns::{ArgsExt, CallContext, DynamicResolver};
use crate::VMError;

use dusk_abi::CALL_DATA_SIZE;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct CallData;

impl<S: DynamicResolver> AbiCall<S> for CallData {
    const NAME: &'static str = "call_data";
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S>,
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
