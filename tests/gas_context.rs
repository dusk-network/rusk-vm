// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use factorial::Factorial;
use rusk_vm::{Contract, GasMeter, NetworkState};


fn factorial_reference(n: u64) -> u64 {
    if n < 2 {
        1
    } else {
        factorial_reference(n - 1) * n
    }
}

#[test]
fn gas_context() {
    let factorial = Factorial;

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/factorial.wasm"
    );

    let contract = Contract::new(factorial, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let n = 7;

    assert_eq!(
        network
            .query::<_, u64>(contract_id, (factorial::COMPUTE, n), &mut gas)
            .unwrap(),
        factorial_reference(n)
    );
}
