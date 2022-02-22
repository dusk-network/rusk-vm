// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::env::Env;
use crate::VMError;

use canonical::{Canon, IdHash, Sink, Source, Store};
use microkelvin::Persistence;

use tracing::trace;

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
        let mem =
            context.read_memory(hash_ofs, core::mem::size_of::<IdHash>())?;
        let mut source = Source::new(mem);
<<<<<<< HEAD
        let hash = IdHash::decode(&mut source)?;
        // we don't allow get requests to fail in the bridge
        // communication since that is the
        // responsibility of the host.
        let mut dest = vec![0; write_len];
        Store::get(&hash, &mut dest)?;
        context.write_memory(&dest, write_buf)?;
=======
        let hash =
            IdHash::decode(&mut source).map_err(VMError::from_store_error)?;
        let id = canonical::Id::raw(hash, write_len as u32);

        // we don't allow get requests to fail in the bridge
        // communication since that is the
        // responsibility of the host.

        let bytes = Persistence::get_raw(&id)
            .map_err(|e| VMError::PersistenceError(format!("{:?}", e)))?;

        context.write_memory(&bytes, write_buf)?;

>>>>>>> c3aaad5 (Change store ops to always use microkelvin)
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

        let id = microkelvin::Persistence::put(mem)
            .map_err(|e| VMError::PersistenceError(format!("{:?}", e)))?;

        let hash = id.hash();

        let mut hash_buffer = vec![0; hash.encoded_len()];
        let mut sink = Sink::new(&mut hash_buffer);
        hash.encode(&mut sink);
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
        let hash = Store::hash(mem);

        context.write_memory(&hash, ret)
    }
}
