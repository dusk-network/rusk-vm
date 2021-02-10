// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::AbiCall;
use crate::call_context::{CallContext, Resolver};
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

use dusk_bls12_381::BlsScalar;
use dusk_bytes::{DeserializableSlice, Serializable};
use poseidon252::sponge::hash;

pub struct PoseidonHash;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for PoseidonHash {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let &[RuntimeValue::I32(ret_addr), RuntimeValue::I32(ofs), RuntimeValue::I32(len)] =
            args.as_ref()
        {
            let ofs = ofs as usize;
            let len = len as usize;
            let ret_addr = ret_addr as usize;

            context
                .memory_mut(|mem| {
                    let bytes = &mem[ofs..ofs + len];

                    // Chunk bytes to BlsSclar byte-size
                    let inp: Vec<BlsScalar> = bytes
                        .chunks(32usize)
                        .map(|scalar_bytes| {
                            BlsScalar::from_slice(&scalar_bytes).unwrap()
                        })
                        .collect();

                    let result = hash(&inp);

                    mem[ret_addr..ret_addr + 32]
                        .copy_from_slice(&result.to_bytes()[..]);
                    // Read Scalars from Chunks
                    Ok(None)
                })
                .map_err(VMError::from_store_error)
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
