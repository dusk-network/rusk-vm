// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use canonical::{Canon, IdHash, Sink, Source, Store};
use crate::resolver::Env;

pub struct Get;

impl Get {
    pub fn get(env: &Env, hash_ofs: i32, write_buf: i32, write_len: i32) -> Result<(), VMError> {
        let hash_ofs = hash_ofs as u64;
        let write_buf = write_buf as u64;
        let write_len = write_len as usize;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};
        // context
        //     .memory_mut(|mem| {
        //         let mut source = Source::new(&mem[hash_ofs..]);
        //         let hash = IdHash::decode(&mut source)?;
        //
        //         // we don't allow get requests to fail in the bridge
        //         // communication since that is the
        //         // responsibility of the host.
        //         Store::get(
        //             &hash,
        //             &mut mem[write_buf..write_buf + write_len],
        //         )?;
        //         Ok(())
        //     })
        //     .map_err(VMError::from_store_error)
        let mem = context.read_memory(hash_ofs, core::mem::size_of::<IdHash>())?;
        let mut source = Source::new(&mem);
        let hash = IdHash::decode(&mut source)?;
        // we don't allow get requests to fail in the bridge
        // communication since that is the
        // responsibility of the host.
        let mut dest = Vec::with_capacity(write_len);
        Store::get(&hash, &mut dest)?;
        context.write_memory(&dest, write_buf)?;
        Ok(())
    }
}

pub struct Put;

impl Put {
    pub fn put(env: &Env, ofs: i32, len: i32, ret: i32) -> Result<(), VMError> {
        let ofs = ofs as u64;
        let len = len as usize;
        let ret = ret as u64;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext) };

        // context
        //     .memory_mut(|mem| {
        //         // only non-inlined values end up written here
        //         debug_assert!(len > core::mem::size_of::<IdHash>());
        //         let hash = Store::put(&mem[ofs..ofs + len]);
        //
        //         let mut sink = Sink::new(&mut mem[ret..]);
        //         hash.encode(&mut sink);
        //
        //         Ok(())
        //     })
        //     .map_err(VMError::from_store_error)
        let mem = context.read_memory(ofs, len)?;
        debug_assert!(mem.len() > core::mem::size_of::<IdHash>());
        let hash = Store::put(&mem);

        let mut hash_buffer = Vec::with_capacity(hash.encoded_len()); // todo think of some better way
        let mut sink = Sink::new(&mut hash_buffer);
        hash.encode(&mut sink);
        context.write_memory(&hash_buffer, ret)?;
        Ok(())
    }
}

pub struct Hash;

impl Hash {
    pub fn hash(env: &Env, ofs: i32, len: i32, ret: i32) -> Result<(), VMError> {
        let ofs = ofs as u64;
        let len = len as usize;
        let ret = ret as u64;
        let context: &mut CallContext = unsafe { &mut *(env.context.0 as *mut CallContext)};

        // context
        //     .memory_mut(|mem| {
        //         let hash = Store::hash(&mem[ofs..ofs + len]);
        //
        //         // write id into wasm memory
        //         mem[ret..ret + hash.len()].copy_from_slice(&hash);
        //         Ok(())
        //     })
        //     .map_err(VMError::from_store_error)
        let mem = context.read_memory(ofs, len)?;
        let hash = Store::hash(&mem);

        context.write_memory(&hash, ret)?;
        Ok(())
    }
}
