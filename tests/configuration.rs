// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use counter::Counter;
use gas_context::GasContextData;
use rusk_vm::{Contract, Gas, GasMeter, NetworkState};

#[test]
fn configuration() {
    let gas_context_data = GasContextData::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_context.wasm"
    );

    let contract = Contract::new(gas_context_data, code.to_vec());

    let mut network =
        NetworkState::with_config(Some("config.toml".to_string())).unwrap();

    let contract_id = network.deploy(contract).unwrap();

    const INITIAL_GAS_LIMIT: Gas = 900_000_000;

    let mut gas = GasMeter::with_limit(INITIAL_GAS_LIMIT);

    let call_gas_limits: Vec<u64> =
        (100_000_000..800_000_000).step_by(100_000_000).collect();
    let mut upper_bounds = call_gas_limits.clone();

    let number_of_nested_calls: usize = call_gas_limits.len();

    network
        .transact::<_, Vec<u64>>(
            contract_id,
            (gas_context::SET_GAS_LIMITS, call_gas_limits),
            &mut gas,
        )
        .unwrap();

    network
        .transact::<_, u64>(
            contract_id,
            (gas_context::COMPUTE, number_of_nested_calls as u64),
            &mut gas,
        )
        .unwrap();

    let limits = network
        .query::<_, Vec<u64>>(
            contract_id,
            (gas_context::READ_GAS_LIMITS, ()),
            &mut gas,
        )
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

fn execute_contract(config_path: &str) -> u64 {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let mut network =
        NetworkState::with_config(Some(config_path.to_string())).unwrap();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact::<_, ()>(contract_id, counter::INCREMENT, &mut gas)
        .expect("Transaction error");

    network
        .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
        .expect("Query error");

    println!("gas spent={} for {}", gas.spent(), config_path);
    gas.spent()
}

#[test]
fn change_gas_consumption_via_configuration() {
    assert!(execute_contract("tests/config/config.toml") < 10_000);
    assert!(execute_contract("tests/config/expensive_config.toml") > 10_000_000);
}

#[test]
fn missing_configuration_file() {
    assert!(matches!(
        NetworkState::with_config(Some("missing_config.toml".to_string())),
        Err(rusk_vm::VMError::ConfigurationFileError(_))
    ));
}

#[test]
fn invalid_configuration_file() {
    assert!(matches!(
        NetworkState::with_config(Some(
            "tests/config/invalid_config.toml".to_string()
        )),
        Err(rusk_vm::VMError::ConfigurationError(_))
    ));
}
