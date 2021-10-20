// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::ops::AbiCall;
use crate::VMError;

use canonical::{Canon, IdHash, Sink, Source, Store};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};
use crate::resolver::Env;
use crate::NetworkState;
use crate::gas::GasMeter;

pub struct Get;

impl AbiCall for Get {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        if let [RuntimeValue::I32(hash_ofs), RuntimeValue::I32(write_buf), RuntimeValue::I32(write_len)] =
            *args.as_ref()
        {
            let hash_ofs = hash_ofs as usize;
            let write_buf = write_buf as usize;
            let write_len = write_len as usize;

            context
                .memory_mut(|mem| {
                    let mut source = Source::new(&mem[hash_ofs..]);
                    let hash = IdHash::decode(&mut source)?;

                    // we don't allow get requests to fail in the bridge
                    // communication since that is the
                    // responsibility of the host.
                    Store::get(
                        &hash,
                        &mut mem[write_buf..write_buf + write_len],
                    )?;
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

impl Get {
    pub fn get(env: &Env, hash_ofs: u32, write_buf: u32, write_len: u32) -> Result<(), VMError> {
        let hash_ofs = hash_ofs as usize;
        let write_buf = write_buf as usize;
        let write_len = write_len as usize;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        context
            .memory_mut(|mem| {
                let mut source = Source::new(&mem[hash_ofs..]);
                let hash = IdHash::decode(&mut source)?;

                // we don't allow get requests to fail in the bridge
                // communication since that is the
                // responsibility of the host.
                Store::get(
                    &hash,
                    &mut mem[write_buf..write_buf + write_len],
                )?;
                Ok(())
            })
            .map_err(VMError::from_store_error)
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
                    // only non-inlined values end up written here
                    debug_assert!(len > core::mem::size_of::<IdHash>());
                    let hash = Store::put(&mem[ofs..ofs + len]);

                    let mut sink = Sink::new(&mut mem[ret..]);
                    hash.encode(&mut sink);

                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

impl Put {
    pub fn put(env: &Env, ofs: u32, len: u32, ret: u32) -> Result<(), VMError> {
        let ofs = ofs as usize;
        let len = len as usize;
        let ret = ret as usize;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext) };

        context
            .memory_mut(|mem| {
                // only non-inlined values end up written here
                debug_assert!(len > core::mem::size_of::<IdHash>());
                let hash = Store::put(&mem[ofs..ofs + len]);

                let mut sink = Sink::new(&mut mem[ret..]);
                hash.encode(&mut sink);

                Ok(())
            })
            .map_err(VMError::from_store_error)
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

impl Hash {
    pub fn hash(env: &Env, ofs: u32, len: u32, ret: u32) -> Result<(), VMError> {
        let ofs = ofs as usize;
        let len = len as usize;
        let ret = ret as usize;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};

        context
            .memory_mut(|mem| {
                let hash = Store::hash(&mem[ofs..ofs + len]);

                // write id into wasm memory
                mem[ret..ret + hash.len()].copy_from_slice(&hash);
                Ok(())
            })
            .map_err(VMError::from_store_error)
    }
}
