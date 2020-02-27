use super::AbiCall;
use crate::host_fns::{host_trap, ArgsExt, CallContext, Resolver};
use crate::VMError;

use kelvin::ByteHash;
use signatory::{
    ed25519,
    signature::{Signature as _, Verifier},
};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Ed25519;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for Ed25519 {
    const NAME: &'static str = "verify_ed25519_signature";
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
        let key_ptr = args.get(0)?;
        let sig_ptr = args.get(1)?;

        context.memory_mut().with_direct_access_mut(|a| {
            let pub_key =
                ed25519::PublicKey::from_bytes(&a[key_ptr..key_ptr + 32])
                    .ok_or_else(|| {
                        host_trap(VMError::InvalidEd25519PublicKey)
                    })?;

            let signature =
                ed25519::Signature::from_bytes(&a[sig_ptr..sig_ptr + 64])
                    .map_err(|_| host_trap(VMError::InvalidEd25519Signature))?;

            let data_slice = args.to_slice(a, 2)?;

            let verifier: signatory_dalek::Ed25519Verifier = (&pub_key).into();

            match verifier.verify(data_slice, &signature) {
                Ok(_) => Ok(Some(RuntimeValue::I32(1))),
                Err(_) => Ok(Some(RuntimeValue::I32(0))),
            }
        })
    }
}
