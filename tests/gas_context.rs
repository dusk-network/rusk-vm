// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use gas_context::{GasContextData, SetGasLimits};
use microkelvin::{HostStore, StoreRef};
use rusk_vm::{Contract, Gas, GasMeter, NetworkState};

#[test]
#[ignore]
fn gas_context() {
    let gas_context_data = GasContextData::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_context.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&gas_context_data, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    const INITIAL_GAS_LIMIT: Gas = 1_000_000_000;
    const GAS_RESERVE_TOLERANCE_PERCENTAGE: u64 = 1;
    const GAS_RESERVE_UPPER_BOUND_PERCENTAGE: u64 =
        GasMeter::RESERVE_PERCENTAGE;
    const GAS_RESERVE_LOWER_BOUND_PERCENTAGE: u64 =
        GasMeter::RESERVE_PERCENTAGE - GAS_RESERVE_TOLERANCE_PERCENTAGE;
    const NUMBER_OF_NESTED_CALLS: usize = 10;

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    let call_gas_limits = vec![0; NUMBER_OF_NESTED_CALLS];

    network
        .transact(contract_id, 0, SetGasLimits::new(call_gas_limits), &mut gas)
        .unwrap();

    network
        .transact(
            contract_id,
            0,
            gas_context::TCompute::new(NUMBER_OF_NESTED_CALLS as u64),
            &mut gas,
        )
        .unwrap();

    let limits = network
        .query(contract_id, 0, gas_context::ReadGasLimits, &mut gas)
        .unwrap();

    let mut bounds: Vec<(u64, u64)> = limits
        .iter()
        .map(|limit| {
            (
                *limit * GAS_RESERVE_LOWER_BOUND_PERCENTAGE / 100,
                *limit * GAS_RESERVE_UPPER_BOUND_PERCENTAGE / 100,
            )
        })
        .collect();
    bounds.insert(
        0,
        (
            INITIAL_GAS_LIMIT * (100 - GAS_RESERVE_TOLERANCE_PERCENTAGE) / 100,
            INITIAL_GAS_LIMIT,
        ),
    );

    let zipped = limits.iter().zip(bounds.iter());

    for (callee_limit, (lower_bound, upper_bound)) in zipped {
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
#[ignore]
fn gas_context_with_call_limit() {
    let gas_context_data = GasContextData::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_context.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&gas_context_data, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    const INITIAL_GAS_LIMIT: Gas = 900_000_000;

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    let call_gas_limits: Vec<u64> =
        (100_000_000..800_000_000).step_by(100_000_000).collect();
    let mut upper_bounds = call_gas_limits.clone();

    let number_of_nested_calls: usize = call_gas_limits.len();

    network
        .transact(
            contract_id,
            0,
            gas_context::SetGasLimits::new(call_gas_limits),
            &mut gas,
        )
        .unwrap();

    network
        .transact(
            contract_id,
            0,
            gas_context::TCompute::new(number_of_nested_calls as u64),
            &mut gas,
        )
        .unwrap();

    let limits = network
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
