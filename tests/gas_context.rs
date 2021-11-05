// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gas_context::GasContextData;
use rusk_vm::{Contract, Gas, GasMeter, NetworkState};

#[test]
fn gas_context() {
    let gas_context_data = GasContextData::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_context.wasm"
    );

    let contract = Contract::new(gas_context_data, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    const INITIAL_GAS_LIMIT: Gas = 1_000_000_000;

    const GAS_RESERVE_UPPER_BOUND: f64 = 0.93;
    const GAS_RESERVE_LOWER_BOUND: f64 = 0.92;

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    let n: u64 = gas_context::GAS_LIMITS_SIZE as u64;

    network
        .transact::<_, u64>(contract_id, (gas_context::COMPUTE, n), &mut gas)
        .unwrap();

    let limits = network
        .query::<_, [u64; gas_context::GAS_LIMITS_SIZE]>(
            contract_id,
            (gas_context::READ_GAS_LIMIT, n),
            &mut gas,
        )
        .unwrap();

    let mut caller_limit = INITIAL_GAS_LIMIT;
    for i in (0..gas_context::GAS_LIMITS_SIZE).rev() {
        let lower_bound = caller_limit as f64 * GAS_RESERVE_LOWER_BOUND;
        let upper_bound = caller_limit as f64 * GAS_RESERVE_UPPER_BOUND;
        let callee_limit = limits[i] as f64;
        assert_eq!(
            callee_limit < upper_bound && callee_limit > lower_bound,
            true,
            "Gas context limit {} should not be out of range {} - {}",
            callee_limit, lower_bound, upper_bound
        );
        caller_limit = limits[i];
    }
}
