// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rusk_vm::{Contract, NetworkState};

#[test]
fn smallest_valid() {
    // Smallest valid WASM module possible so `deploy` won't raise a
    // `InvalidByteCode` error
    let code = 0x0000_0001_6D73_6100_u64.to_le_bytes();

    // Create a contract with a simple state
    let contract = Contract::new(0xfeed_u16, code.to_vec());

    // Deploy with the id given
    let mut network = NetworkState::default();
    network.deploy(contract).unwrap();
}
