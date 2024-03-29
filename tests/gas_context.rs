// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gas_context::{GasContextData, SetGasLimits};
use rusk_vm::{Contract, Gas, GasMeter, NetworkState};

fn make_gas_bounds(top: u64, vec: &mut Vec<(u64, u64)>, count: usize) {
    let bounds = (top * GasMeter::RESERVE_PERCENTAGE / 100, top);
    vec.push(bounds);
    if count > 1 {
        make_gas_bounds(bounds.0, vec, count - 1)
    }
}

#[test]
fn gas_context() {
    let gas_context_data = GasContextData::new();

    let mut network = NetworkState::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_context.wasm"
    );

    let contract =
        Contract::new(&gas_context_data, code.to_vec(), network.store());
    let contract_id = network.deploy(contract).unwrap();

    const INITIAL_GAS_LIMIT: Gas = 1_000_000_000;
    const NUMBER_OF_NESTED_CALLS: usize = 10;

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    let call_gas_limits = vec![0; NUMBER_OF_NESTED_CALLS];

    let (_, network) = network
        .transact(contract_id, 0, SetGasLimits::new(call_gas_limits), &mut gas)
        .unwrap();

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    let (_, network) = network
        .transact(
            contract_id,
            0,
            gas_context::TCompute::new(NUMBER_OF_NESTED_CALLS as u64),
            &mut gas,
        )
        .unwrap();

    let limits = &*network
        .query(contract_id, 0, gas_context::ReadGasLimits, &mut gas)
        .unwrap();

    let mut bounds: Vec<(u64, u64)> = Vec::new();
    make_gas_bounds(INITIAL_GAS_LIMIT, &mut bounds, NUMBER_OF_NESTED_CALLS);

    let zipped = limits.iter().zip(bounds.iter());

    for (callee_limit, (lower_bound, upper_bound)) in zipped {
        println!(
            "limit {} should be in {} - {}",
            callee_limit, lower_bound, upper_bound
        );
        assert!(
            callee_limit > lower_bound && callee_limit < upper_bound,
            "Gas context limit {} should not be out of range {} - {}",
            callee_limit,
            lower_bound,
            upper_bound
        );
    }
}

#[test]
fn gas_context_with_call_limit() {
    let gas_context_data = GasContextData::new();

    let mut network = NetworkState::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_context.wasm"
    );

    let contract =
        Contract::new(&gas_context_data, code.to_vec(), network.store());
    let contract_id = network.deploy(contract).unwrap();

    const INITIAL_GAS_LIMIT: Gas = 900_000_000;

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    let call_gas_limits: Vec<u64> =
        (100_000_000..800_000_000).step_by(100_000_000).collect();
    let mut upper_bounds = call_gas_limits.clone();

    let number_of_nested_calls: usize = call_gas_limits.len();

    let (_, network) = network
        .transact(contract_id, 0, SetGasLimits::new(call_gas_limits), &mut gas)
        .unwrap();

    let (_, network) = network
        .transact(
            contract_id,
            0,
            gas_context::TCompute::new(number_of_nested_calls as u64),
            &mut gas,
        )
        .unwrap();

    let limits = &*network
        .query(contract_id, 0, gas_context::ReadGasLimits, &mut gas)
        .unwrap();

    upper_bounds.remove(0);
    upper_bounds.reverse();
    upper_bounds.insert(0, INITIAL_GAS_LIMIT);
    for (index, limit) in limits.iter().enumerate() {
        assert!(
            limit < &upper_bounds[index],
            "Limit {} equal to {} should be below {}",
            index,
            limit,
            upper_bounds[index]
        );
    }
}
