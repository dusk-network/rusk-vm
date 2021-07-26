// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rusk_vm::{Contract, NetworkState};
use wabt::wat2wasm;

#[test]
fn smallest_valid() {
    // Smallest valid WASM module possible where `deploy` won't raise a
    // `InvalidByteCode` error
    let code = 0x0000_0001_6D73_6100_u64.to_le_bytes();

    // Create a contract with an empty state
    let contract = Contract::new((), code.to_vec());

    // Deploy with the id given
    let mut network = NetworkState::default();
    network.deploy(contract).unwrap();
}

#[test]
fn add() {
    // WAT of 'add' operation
    let wat = "(module
  (func (export \"addTwo\") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add))";
    let code = wat2wasm(wat).expect("failed to parse wat to wasm");

    let contract = Contract::new((), code.to_vec());
    let mut network = NetworkState::default();
    network.deploy(contract).unwrap();
}
