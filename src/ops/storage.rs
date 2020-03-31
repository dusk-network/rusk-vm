use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dusk_abi::{H256, STORAGE_KEY_SIZE, STORAGE_VALUE_SIZE};
use kelvin::{ByteHash, Map};

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct SetStorage;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for SetStorage {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let key_ofs = args.get(0)?;
        let val_ofs = args.get(1)?;
        let val_len = args.get(2)?;

        let mut key_buf = H256::default();
        let mut val_buf = [0u8; STORAGE_VALUE_SIZE];

        context.memory().with_direct_access(|a| {
            key_buf
                .as_mut()
                .copy_from_slice(&a[key_ofs..key_ofs + STORAGE_KEY_SIZE]);
            val_buf[0..val_len].copy_from_slice(&a[val_ofs..val_ofs + val_len]);
        });
        context
            .storage_mut()?
            .insert(key_buf, val_buf[0..val_len].into())?;

        Ok(None)
    }
}

pub struct GetStorage;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for GetStorage {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        // offset to where to write the value in memory
        let key_buf_ofs = args.get(0)?;
        let key_buf_len = args.get(1)?;
        let val_buf_ofs = args.get(1)?;

        let mut key_buf = H256::default();

        context.memory().with_direct_access(|a| {
            key_buf.as_mut().copy_from_slice(
                &a[key_buf_ofs..key_buf_ofs + STORAGE_KEY_SIZE],
            );
        });

        let val = context.storage()?.get(&key_buf)?.map(|v| (*v).clone());

        match val {
            Some(val) => {
                let len = val.len();
                context.memory_mut().with_direct_access_mut(|a| {
                    a[val_buf_ofs..val_buf_ofs + len].copy_from_slice(&val)
                });
                Ok(Some(RuntimeValue::I32(len as i32)))
            }
            None => Ok(Some(RuntimeValue::I32(-1))),
        }
    }
}
