// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::OffsetLen;
use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct Get;

impl Get {
    /// Write bytes into wasm memory
    pub fn get(
        env: &Env,
        ofs: u64,
        len: u16,
        buf_ptr: i32,
    ) -> Result<(), VMError> {
        trace!("Executing 'get' host function");
        let context = env.get_context();

        let id = OffsetLen::new(ofs, len as u32);

        let store = env.store();
        let slice = store.get_raw(&id);

        context.write_memory(slice, buf_ptr as u64)?;

        Ok(())
    }
}

pub struct Put;

impl Put {
    pub fn put(env: &Env, mem_ofs: i32, len: i32) -> Result<u64, VMError> {
        trace!("Executing 'put' host function");
        let bytes = env
            .get_context()
            .read_memory(mem_ofs as u64, len as usize)?;
        let i = env.store().put_raw(bytes);
        Ok(i.offset())
    }
}

pub struct Hash;

impl Hash {
    pub fn hash(
        env: &Env,
        ofs: i32,
        len: i32,
        ret: i32,
    ) -> Result<(), VMError> {
        let ofs = ofs as u64;
        let len = len as usize;
        let ret = ret as u64;
        let context = env.get_context();

        let mem = context.read_memory(ofs, len)?;
        let hash = mem.to_vec();

        context.write_memory(&hash, ret)
    }
}
