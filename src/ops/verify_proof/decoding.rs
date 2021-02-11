// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::VMError;

use canonical::Store;
use dusk_plonk::prelude::*;

pub fn decode_pub_inputs<S: Store>(
    mem: &[u8],
    pub_inp_len: usize,
    pub_inp_addr: usize,
) -> Result<Vec<PublicInput>, VMError<S>> {
    // Build Public Inputs vector
    let mut pi_bytes = vec![0u8; pub_inp_len];
    pi_bytes.copy_from_slice(&mem[pub_inp_addr..pub_inp_addr + pub_inp_len]);
    pi_bytes[..]
        .chunks(PublicInput::serialized_size())
        .map(|chunk| {
            PublicInput::from_bytes(chunk)
                .map_err(|_| VMError::ABICallExecError)
        })
        .collect::<Result<Vec<PublicInput>, VMError<S>>>()
}

pub fn decode_proof<S: Store>(
    mem: &[u8],
    proof_addr: usize,
) -> Result<Proof, VMError<S>> {
    Proof::from_bytes(&mem[proof_addr..proof_addr + Proof::serialised_size()])
        .map_err(|_| VMError::ABICallExecError)
}

pub fn decode_vk<S: Store>(
    mem: &[u8],
    vk_addr: usize,
) -> Result<VerifierKey, VMError<S>> {
    VerifierKey::from_bytes(
        &mem[vk_addr..vk_addr + VerifierKey::serialised_size()],
    )
    .map_err(|_| VMError::ABICallExecError)
}

// First position is the len
pub fn decode_label<S: Store>(
    mem: &[u8],
    label_len: usize,
    label_addr: usize,
) -> Result<String, VMError<S>> {
    String::from_utf8(mem[label_addr..label_addr + label_len].to_vec())
        .map_err(|_| VMError::ABICallExecError)
}
