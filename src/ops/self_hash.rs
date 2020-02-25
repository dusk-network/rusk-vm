use super::AbiCall;
use crate::host_fns::{ArgsExt, CallContext, DynamicResolver};
use crate::VMError;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct SelfHash;

impl<S: DynamicResolver> AbiCall<S> for SelfHash {
    const NAME: &'static str = "self_hash";
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let buffer_ofs = args.get(0)?;

        context.memory().with_direct_access_mut(|a| {
            a[buffer_ofs..buffer_ofs + 32]
                .copy_from_slice(context.called().as_ref())
        });
        Ok(None)
    }
}
