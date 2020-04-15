use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use kelvin::ByteHash;

use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct SetStorage;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for SetStorage {
    const ARGUMENTS: &'static [ValueType] = &[
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
    ];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let key_ofs = args.get(0)? as usize;
        let key_len = args.get(1)? as usize;
        let val_ofs = args.get(2)? as usize;
        let val_len = args.get(3)? as usize;

        let (key, val): (Vec<_>, Vec<_>) = context.memory(|m| {
            (
                m[key_ofs..key_ofs + key_len].into(),
                m[val_ofs..val_ofs + val_len].into(),
            )
        });

        context.storage_mut()?.insert(key, val)?;

        Ok(None)
    }
}

pub struct GetStorage;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for GetStorage {
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
        let key_ofs = args.get(0)? as usize;
        let key_len = args.get(1)? as usize;
        let val_ofs = args.get(2)? as usize;
        let val_len = args.get(3)? as usize;

        context.memory_storage_mut(|m, storage| {
            let key_slice = &m[key_ofs..key_ofs + key_len];
            {
                match storage.get(key_slice)? {
                    Some(val) => {
                        if val.len() == val_len {
                            m[val_ofs..val_ofs + val_len]
                                .copy_from_slice(&val[..]);
                            Ok(Some(RuntimeValue::I32(val.len() as i32)))
                        } else {
                            panic!("invalid type length")
                        }
                    }
                    None => Ok(Some(RuntimeValue::I32(-1))),
                }
            }
        })
    }
}

pub struct DeleteStorage;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for DeleteStorage {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        // offset to where to write the value in memory
        let key_ofs = args.get(0)? as usize;
        let key_len = args.get(1)? as usize;

        context.memory_storage_mut(|m, storage| {
            let key_slice = &m[key_ofs..key_ofs + key_len];
            storage.remove(key_slice).map_err(Into::into)
        })?;
        Ok(None)
    }
}
