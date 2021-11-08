// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gas_context::GasContextData;
use rusk_vm::{Contract, Gas, GasMeter, NetworkState, GasConstants};

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

    const GAS_RESERVE_UPPER_BOUND_FACTOR: f64 = GasConstants::GAS_RESERVE_FACTOR;
    const GAS_RESERVE_LOWER_BOUND_FACTOR: f64 = GasConstants::GAS_RESERVE_FACTOR - GasConstants::GAS_RESERVE_FACTOR_TOLERANCE;

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    const NUMBER_OF_NESTED_CALLS: usize = 10;

    let call_gas_limits = vec![0; NUMBER_OF_NESTED_CALLS];

    network
        .transact::<_, Vec<u64>>(contract_id, (gas_context::SET_GAS_LIMITS, call_gas_limits), &mut gas)
        .unwrap();

    network
        .transact::<_, u64>(contract_id, (gas_context::COMPUTE, NUMBER_OF_NESTED_CALLS as u64), &mut gas)
        .unwrap();

    let limits = network
        .query::<_, Vec<u64>>(
            contract_id,
            (gas_context::READ_GAS_LIMITS, ()),
            &mut gas,
        )
        .unwrap();

    let mut caller_limit = INITIAL_GAS_LIMIT as f64;
    for callee_limit in limits {
        let lower_bound = caller_limit * GAS_RESERVE_LOWER_BOUND_FACTOR;
        let upper_bound = caller_limit * GAS_RESERVE_UPPER_BOUND_FACTOR;
        let callee_limit = callee_limit as f64;
        assert_eq!(
            callee_limit < upper_bound && callee_limit > lower_bound,
            true,
            "Gas context limit {} should not be out of range {} - {}",
            callee_limit, lower_bound, upper_bound
        );
        caller_limit = callee_limit;
    }
}

#[test]
fn gas_context_with_call_limit() {
    let gas_context_data = GasContextData::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_context.wasm"
    );

    let contract = Contract::new(gas_context_data, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    const INITIAL_GAS_LIMIT: Gas = 1_000_000_000;

    const GAS_RESERVE_UPPER_BOUND_FACTOR: f64 = GasConstants::GAS_RESERVE_FACTOR;
    const GAS_RESERVE_LOWER_BOUND_FACTOR: f64 = GasConstants::GAS_RESERVE_FACTOR - GasConstants::GAS_RESERVE_FACTOR_TOLERANCE;

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    let call_gas_limits: Vec<u64> = vec!(1_000_000_000, 800_000_000, 600_000_000);

    let number_of_nested_calls: usize = call_gas_limits.len();

    network
        .transact::<_, Vec<u64>>(contract_id, (gas_context::SET_GAS_LIMITS, call_gas_limits), &mut gas)
        .unwrap();

    network
        .transact::<_, u64>(contract_id, (gas_context::COMPUTE, number_of_nested_calls as u64), &mut gas)
        .unwrap();

    let limits = network
        .query::<_, Vec<u64>>(
            contract_id,
            (gas_context::READ_GAS_LIMITS, ()),
            &mut gas,
        )
        .unwrap();

    let mut caller_limit = INITIAL_GAS_LIMIT as f64;
    for callee_limit in limits {
        let lower_bound = caller_limit * GAS_RESERVE_LOWER_BOUND_FACTOR;
        let upper_bound = caller_limit * GAS_RESERVE_UPPER_BOUND_FACTOR;
        let callee_limit = callee_limit as f64;
        assert_eq!(
            callee_limit < upper_bound && callee_limit > lower_bound,
            true,
            "Gas context limit {} should not be out of range {} - {}",
            callee_limit, lower_bound, upper_bound
        );
        caller_limit = callee_limit;
    }
}
