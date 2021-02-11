// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod circuit;
mod decoding;

use super::AbiCall;
use crate::call_context::{CallContext, Resolver};
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct ProofVerification;

impl<E: Resolver<S>, S: Store> AbiCall<E, S> for ProofVerification {
    const ARGUMENTS: &'static [ValueType] = &[
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
    ];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>> {
        if let [RuntimeValue::I32(pub_inp), RuntimeValue::I32(pub_inp_len), RuntimeValue::I32(proof), RuntimeValue::I32(verif_key), RuntimeValue::I32(label), RuntimeValue::I32(label_len)] =
            args.as_ref()[..]
        {
            let pub_inp = pub_inp as usize;
            let pub_inp_len = pub_inp_len as usize;
            let proof = proof as usize;
            let verif_key = verif_key as usize;
            let label = label as usize;
            let label_len = label_len as usize;

            context.memory(|mem| -> Result<Option<RuntimeValue>, VMError<S>> {
                let pi_vec =
                    decoding::decode_pub_inputs(mem, pub_inp_len, pub_inp)?;
                let proof = decoding::decode_proof(mem, proof)?;
                let vk = decoding::decode_vk(mem, verif_key)?;
                let label = decoding::decode_label(mem, label_len, label)?;
                // Mock
                Ok(Some(RuntimeValue::I32(1)))
            })
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}
