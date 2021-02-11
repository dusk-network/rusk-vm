// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::VMError;

#[cfg(feature = "dummy_circ")]
use crate::dummy_circ::TestCircuit;
use canonical::Store;
use dusk_plonk::prelude::*;
use transfer_circuits::{
    ExecuteCircuit, SendToContractObfuscatedCircuit,
    SendToContractTransparentCircuit, WithdrawFromObfuscatedCircuit,
};

fn verify_proof<'a, S: Store>(
    mut c: impl Circuit<'a>,
    pp: &PublicParameters,
    vk: &VerifierKey,
    p_inp: &[PublicInput],
    proof: &Proof,
) -> Result<(), VMError<S>> {
    c.verify_proof(pp, vk, b"whatever", proof, p_inp)
        .map_err(|_| VMError::ABICallExecError)
}

#[rustfmt::skip]
fn select_and_verify<'a, S:Store>(
    label: &String,
    pp: &PublicParameters,
    vk: &VerifierKey,
    p_inp: &[PublicInput],
    proof: &Proof,
) -> Result<(), VMError<S>> {
    match label.as_str() {
        "transfer-execute-1-0" => verify_proof(ExecuteCircuit::<17, 15>::default(), pp, vk, p_inp, proof),
        "transfer-execute-1-1" => verify_proof(ExecuteCircuit::<17, 15>::default(), pp, vk, p_inp, proof),
        "transfer-execute-1-2" => verify_proof(ExecuteCircuit::<17, 15>::default(), pp, vk, p_inp, proof),
        "transfer-execute-2-0" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-execute-2-1" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-execute-2-2" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-execute-3-0" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-execute-3-1" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-execute-3-2" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-execute-4-0" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-execute-4-1" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-execute-4-2" => verify_proof(ExecuteCircuit::<17, 16>::default(), pp, vk, p_inp, proof),
        "transfer-send-to-contract-obfuscated" => verify_proof(SendToContractObfuscatedCircuit::default(), pp, vk, p_inp, proof),
        "transfer-send-to-contract-transparent" => verify_proof(SendToContractTransparentCircuit::default(), pp, vk, p_inp, proof),
        "transfer-withdraw-from-obfuscated" => verify_proof(WithdrawFromObfuscatedCircuit::default(), pp, vk, p_inp, proof),
        #[cfg(feature = "dummy_circ")]
        "dummy" => verify_proof(TestCircuit::default(), pp, vk, p_inp, proof),
        _ => Err(VMError::ABICallExecError),
    }
}
