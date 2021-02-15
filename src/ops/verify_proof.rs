// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::AbiCall;
use crate::call_context::CallContext;
use crate::VMError;

use canonical::Store;
use dusk_plonk::prelude::*;
use transfer_circuits::{
    ExecuteCircuit, SendToContractObfuscatedCircuit,
    SendToContractTransparentCircuit, WithdrawFromObfuscatedCircuit,
};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct VerifyProof;

impl<S: Store> AbiCall<S> for VerifyProof {
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
        context: &mut CallContext<S>,
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
                let pi_vec = decode_pub_inputs(mem, pub_inp_len, pub_inp)?;
                let proof = Proof::from_bytes(
                    &mem[proof..proof + Proof::serialised_size()],
                )
                .map_err(|_| VMError::InvalidArguments)?;
                let vk = VerifierKey::from_bytes(
                    &mem[verif_key..verif_key + VerifierKey::serialised_size()],
                )
                .map_err(|_| VMError::InvalidArguments)?;
                let label =
                    String::from_utf8(mem[label..label + label_len].to_vec())
                        .map_err(|_| VMError::InvalidUtf8)?;
                let pp = unsafe {
                    PublicParameters::from_slice_unchecked(
                        &rusk_profile::get_common_reference_string().map_err(
                            |_| {
                                VMError::ContractPanic(
                                    "PubParams deser error".to_string(),
                                )
                            },
                        )?,
                    )
                    .map_err(|_| VMError::InvalidArguments)?
                };
                let success =
                    select_and_verify::<S>(&label, &pp, &vk, &pi_vec, &proof);
                Ok(Some(RuntimeValue::I32(success as i32)))
            })
        } else {
            Err(VMError::InvalidArguments)
        }
    }
}

fn decode_pub_inputs<S: Store>(
    mem: &[u8],
    pub_inp_len: usize,
    pub_inp_addr: usize,
) -> Result<Vec<PublicInput>, VMError<S>> {
    mem[pub_inp_addr..pub_inp_addr + pub_inp_len]
        .chunks(PublicInput::serialized_size())
        .map(|chunk| {
            PublicInput::from_bytes(chunk)
                .map_err(|_| VMError::InvalidArguments)
        })
        .collect::<Result<Vec<PublicInput>, VMError<S>>>()
}

fn verify_proof<'a>(
    mut c: impl Circuit<'a>,
    pp: &PublicParameters,
    vk: &VerifierKey,
    transcript_init: &'static [u8],
    p_inp: &[PublicInput],
    proof: &Proof,
) -> bool {
    c.verify_proof(pp, vk, transcript_init, proof, p_inp)
        .is_ok()
}

#[rustfmt::skip]
fn select_and_verify<'a, S: Store>(
    label: &String,
    pp: &PublicParameters,
    vk: &VerifierKey,
    p_inp: &[PublicInput],
    proof: &Proof,
) -> bool {
    match label.as_str() {
        "transfer-execute-1-0" => verify_proof(ExecuteCircuit::<17, 15>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 1,0).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-1-1" => verify_proof(ExecuteCircuit::<17, 15>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 1,1).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-1-2" => verify_proof(ExecuteCircuit::<17, 15>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 1,2).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-2-0" => verify_proof(ExecuteCircuit::<17, 16>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 2,0).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-2-1" => verify_proof(ExecuteCircuit::<17, 16>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 2,1).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-2-2" => verify_proof(ExecuteCircuit::<17, 16>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 2,2).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-3-0" => verify_proof(ExecuteCircuit::<17, 17>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 3,0).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-3-1" => verify_proof(ExecuteCircuit::<17, 17>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 3,1).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-3-2" => verify_proof(ExecuteCircuit::<17, 17>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 3,2).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-4-0" => verify_proof(ExecuteCircuit::<17, 17>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 4,0).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-4-1" => verify_proof(ExecuteCircuit::<17, 17>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 4,1).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-execute-4-2" => verify_proof(ExecuteCircuit::<17, 17>::create_dummy_circuit::<_,S>(&mut rand::thread_rng(), 4,2).expect("Error generating dummy circuit"), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-send-to-contract-obfuscated" => verify_proof(SendToContractObfuscatedCircuit::default(), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-send-to-contract-transparent" => verify_proof(SendToContractTransparentCircuit::default(), pp, vk, b"dusk" ,p_inp, proof),
        "transfer-withdraw-from-obfuscated" => verify_proof(WithdrawFromObfuscatedCircuit::default(), pp, vk, b"dusk" ,p_inp, proof),
        _ => false,
    }
}
