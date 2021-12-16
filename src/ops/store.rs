// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use tracing::trace;

use crate::env::Env;
use crate::VMError;

pub struct Get;

impl Get {
    pub fn get(
        env: &Env,
        hash_ofs: i32,
        write_buf: i32,
        write_len: i32,
    ) -> Result<(), VMError> {
        trace!("Executing 'get' host function");

        let hash_ofs = hash_ofs as u64;
        let write_buf = write_buf as u64;
        let write_len = write_len as usize;
        let context = env.get_context();
        let mem = context.read_memory(hash_ofs, todo!())?;
        // let mut source = Source::new(mem);
        // let hash =
        //     IdHash::decode(&mut source).map_err(VMError::from_store_error)?;
        // we don't allow get requests to fail in the bridge
        // communication since that is the
        // responsibility of the host.
        let mut dest = vec![0; write_len];
        // env.store()
        //     .get(&hash, &mut dest)
        //     .map_err(VMError::from_store_error)?;
        todo!();
        context.write_memory(&dest, write_buf)?;
        Ok(())
    }
}

pub struct Put;

impl Put {
    pub fn put(env: &Env, ofs: i32, len: i32, ret: i32) -> Result<(), VMError> {
        trace!("Executing 'put' host function");

        let ofs = ofs as u64;
        let len = len as usize;
        let ret = ret as u64;
        let context = env.get_context();

        let mem = context.read_memory(ofs, len)?;
        // debug_assert!(mem.len() > core::mem::size_of::<IdHash>());

        // TODO, what types are we using here here?

        todo!(); // let hash = env.store().put(mem);

        let mut hash_buffer = vec![0; 32];
        // let mut sink = Sink::new(&mut hash_buffer);
        // hash.encode(&mut sink);
        context.write_memory(&hash_buffer, ret)?;
        Ok(())
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
        let hash = mem.to_vec();// todo

        context.write_memory(&hash, ret)
    }
}
