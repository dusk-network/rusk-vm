use super::AbiCall;
use crate::call_context::{host_trap, ArgsExt, CallContext, Resolver};
use crate::VMError;

use kelvin::ByteHash;
use threshold_crypto_ce::bn256::{PublicKey, Signature};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct BLS;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for BLS {
    const ARGUMENTS: &'static [ValueType] = &[
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
    ];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        todo!("port bls")
    }
}
