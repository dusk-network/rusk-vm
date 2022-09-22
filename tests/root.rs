// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use counter::{Counter, Increment};
use rusk_vm::{Contract, GasMeter, NetworkState};

#[test]
fn root_properties() {
    let counter = Counter::new(99);

    let mut network = NetworkState::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/counter.wasm"
    );

    let contract = Contract::new(&counter, code.to_vec(), network.store());
    let contract_id = network.deploy(contract).expect("Deploy error");

    let initial_root = network.root();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let (_, new_network) = network
        .transact(contract_id, 0, Increment, &mut gas)
        .expect("transaction should succeed");

    let (_, other_new_network) = network
        .transact(contract_id, 0, Increment, &mut gas)
        .expect("transaction should succeed");

    assert_eq!(
        initial_root,
        network.root(),
        "root should be unchanged in untouched state"
    );
    assert_ne!(
        initial_root,
        new_network.root(),
        "root should change on a mutation"
    );
    assert_eq!(
        new_network.root(),
        other_new_network.root(),
        "root should be deterministic"
    );
}
