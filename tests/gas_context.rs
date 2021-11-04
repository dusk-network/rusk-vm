// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gas_context::GasContextData;
use rusk_vm::{Contract, GasMeter, NetworkState};

#[test]
fn gas_context() {
    let gas_context_data = GasContextData::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_context.wasm"
    );

    let contract = Contract::new(gas_context_data, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let n = 7;

    network
        .transact::<_, u64>(contract_id, (gas_context::COMPUTE, n as u64), &mut gas)
        .unwrap();

    // for i in 1..7 {
        let limit_7 = network.query::<_, u64>(contract_id, (gas_context::READ_GAS_LIMIT, 7 as u64), &mut gas).unwrap();
        assert_eq!(limit_7, 927999922);
        let limit_6 = network.query::<_, u64>(contract_id, (gas_context::READ_GAS_LIMIT, 6 as u64), &mut gas).unwrap();
        assert_eq!(limit_6, 862971175);
    // }
    // 600307734, 645469102, 694048040, 746303296, 802511685, 862971175, 927999922
}
