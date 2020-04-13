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
        let key_ptr = args.get(0)?;
        let sig_ptr = args.get(1)?;

        context
            .memory_mut()
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let mut key_buf = [0u8; 32];
                    key_buf.copy_from_slice(&a[key_ptr..key_ptr + 32]);
                    let pub_key =
                        PublicKey::from_bytes(key_buf).map_err(|_| {
                            host_trap(VMError::InvalidEd25519PublicKey)
                        })?;

                    let mut sig_buf = [0u8; 64];
                    sig_buf.copy_from_slice(&a[sig_ptr..sig_ptr + 64]);
                    let signature =
                        Signature::from_bytes(sig_buf).map_err(|_| {
                            host_trap(VMError::InvalidEd25519Signature)
                        })?;

                    let data_slice = args.to_slice(a, 2)?;

                    if !pub_key.verify(&signature, data_slice) {
                        return Ok(Some(RuntimeValue::I32(0)));
                    }

                    Ok(Some(RuntimeValue::I32(1)))
                },
            )
    }
}
