// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use factorial::Factorial;
use rusk_vm::{Contract, GasMeter, NetworkState};
use canonical::CanonError;

#[test]
fn gas_context() {
    let factorial = Factorial::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/factorial.wasm"
    );

    let contract = Contract::new(factorial, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let n = 7;

    network
        .transact::<_, u64>(contract_id, (factorial::COMPUTE, n as u64), &mut gas)
        .unwrap();

    // for i in 1..7 {
        let limit = network.query::<_, u64>(contract_id, (factorial::READ_GAS_LIMIT, 1 as u64), &mut gas).unwrap();
        assert_eq!(limit, 600307734);
    // }
}
