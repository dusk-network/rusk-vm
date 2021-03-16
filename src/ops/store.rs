// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::ops::AbiCall;
use crate::VMError;

// use canonical::Id;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Get;

impl AbiCall for Get {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(ofs)] = *args.as_ref() {
            let _ofs = ofs as usize;
            context
                .memory_mut(|_mem| {
                    // read identifier
                    // let mut id = Id::default();
                    // //let id_len = id.as_ref().len();
                    // let slice = &mem[ofs..ofs + id_len];
                    // id.as_mut().copy_from_slice(slice);

                    // store.fetch(&id, &mut mem[ofs..])?;
                    // Ok(None)

                    todo!()
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

pub struct Put;

impl AbiCall for Put {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(ofs), RuntimeValue::I32(len), RuntimeValue::I32(ret)] =
            *args.as_ref()
        {
            let _ofs = ofs as usize;
            let _len = len as usize;
            let _ret = ret as usize;

            context
                .memory_mut(|_mem| {
                    // if let Ok(id) = store.put_raw(&mem[ofs..ofs + len]) {
                    //     let id_len = id.as_ref().len();
                    //     // write id back
                    //     mem[ret..ret + id_len].copy_from_slice(id.as_ref());
                    // }
                    // Ok(None)
                    todo!()
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
