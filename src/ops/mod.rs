use crate::host_fns::CallContext;
use crate::VMError;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub mod balance;
pub mod call_contract;
pub mod call_data;
pub mod debug;
pub mod ed25519;
pub mod gas;
pub mod panic;
pub mod phoenix_ops;
pub mod ret;
use kelvin::ByteHash;

pub mod self_hash;
pub mod storage;

pub trait AbiCall<S, H: ByteHash> {
    const ARGUMENTS: &'static [ValueType];
    const RETURN: Option<ValueType>;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError>;
}
