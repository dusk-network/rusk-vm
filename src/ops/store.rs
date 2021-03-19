// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::ops::AbiCall;
use crate::VMError;

use canonical::{Canon, Id, Sink, Source, Store};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct Get;

impl AbiCall for Get {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(id_ofs), RuntimeValue::I32(write_ofs)] =
            *args.as_ref()
        {
            let id_ofs = id_ofs as usize;
            let write_ofs = write_ofs as usize;

            println!("\nGET {:?} {:?}", id_ofs, write_ofs);

            context
                .memory_mut(|mem| {
                    let mut source = Source::new(&mem[id_ofs..]);
                    let id = Id::decode(&mut source)?;
                    // we don't allow get requests to fail in the bridge
                    // communication since that is the
                    // responsibility of the host.
                    Store::get(&id, &mut mem[write_ofs..])?;
                    Ok(None)
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
            let ofs = ofs as usize;
            let len = len as usize;
            let ret = ret as usize;

            context
                .memory_mut(|mem| {
                    let id = Store::put(&mem[ofs..ofs + len]);

                    // let to_put = &mem[ofs..ofs + len];

                    // println!("PUT put {:?}", to_put);

                    // let id = Id::new(to_put);

                    // write id into wasm memory

                    let mut sink = Sink::new(&mut mem[ret..]);

                    println!("ID IS {:?}", id);

                    id.encode(&mut sink);
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

pub struct Hash;

impl AbiCall for Hash {
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
            let ofs = ofs as usize;
            let len = len as usize;
            let ret = ret as usize;

            context
                .memory_mut(|mem| {
                    let hash = Store::hash(&mem[ofs..ofs + len]);

                    // write id into wasm memory
                    mem[ret..ret + hash.len()].copy_from_slice(&hash);
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
