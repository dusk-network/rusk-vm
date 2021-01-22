// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use crate::call_context::{CallContext, Resolver};
use crate::ops::AbiCall;
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Get;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for Get {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let &[RuntimeValue::I32(ofs)] = args.as_ref() {
            let ofs = ofs as usize;
            let store = context.store().clone();
            context
                .memory_mut(|mem| {
                    // read identifier
                    let mut id = S::Ident::default();
                    let id_len = id.as_ref().len();
                    let slice = &mem[ofs..ofs + id_len];
                    id.as_mut().copy_from_slice(slice);

                    store.fetch(&id, &mut mem[ofs..])?;
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

pub struct Put;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for Put {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let &[RuntimeValue::I32(ofs), RuntimeValue::I32(len), RuntimeValue::I32(ret)] =
            args.as_ref()
        {
            let ofs = ofs as usize;
            let len = len as usize;
            let ret = ret as usize;
            let store = context.store().clone();
            context
                .memory_mut(|mem| {
                    if let Ok(id) = store.put_raw(&mem[ofs..ofs + len]) {
                        let id_len = id.as_ref().len();
                        // write id back
                        mem[ret..ret + id_len].copy_from_slice(id.as_ref());
                    }
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
