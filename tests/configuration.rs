// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use block_height::{BlockHeight, ReadBlockHeight};
use counter::{Counter, ReadValue};
use delegator::{Delegator, QueryForwardData};
use rusk_vm::{Config, Contract, GasMeter, HostCosts, NetworkState, OpCosts};

// host fn cost should dominate for proper testing
const HOST_FN_COST: u64 = 1_000_000_000;
const GAS_LIMIT: u64 = 10 * HOST_FN_COST;

const HIGH_HOST_COST_CONFIG: Config = Config {
    host_costs: HostCosts {
        query: HOST_FN_COST,
        block_height: HOST_FN_COST,
        ..HostCosts::new()
    },
    ..Config::new()
};

fn execute_block_height_with_config(config: &'static Config) -> u64 {
    let block_height = BlockHeight;

    let mut network = NetworkState::builder().config(config).build();

    let block_height_code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/block_height.wasm"
    );

    let block_height_contract = Contract::new(
        &block_height,
        block_height_code.to_vec(),
        network.store(),
    );

    let block_height_id = network.deploy(block_height_contract).unwrap();
    let mut gas = GasMeter::with_limit(GAS_LIMIT);

    network
        .query(block_height_id, 0, ReadBlockHeight, &mut gas)
        .unwrap();

    gas.spent()
}

#[test]
fn block_height_host_cost() {
    let cheap = execute_block_height_with_config(&DEFAULT_CONFIG);
    let expensive = execute_block_height_with_config(&HIGH_HOST_COST_CONFIG);

    assert_eq!(
        expensive,
        cheap + HIGH_HOST_COST_CONFIG.host_costs.block_height
            - DEFAULT_CONFIG.host_costs.block_height
    );
}

fn execute_counter_delegation_with_config(config: &'static Config) -> u64 {
    let counter = Counter::new(0);
    let delegator = Delegator;

    let mut network = NetworkState::builder().config(config).build();

    let counter_code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/counter.wasm"
    );
    let delegator_code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/delegator.wasm"
    );

    let counter_contract =
        Contract::new(&counter, counter_code.to_vec(), network.store());
    let delegator_contract =
        Contract::new(&delegator, delegator_code.to_vec(), network.store());

    let counter_id = network.deploy(counter_contract).unwrap();
    let delegator_id = network.deploy(delegator_contract).unwrap();

    let mut gas = GasMeter::with_limit(GAS_LIMIT);

    network
        .query(
            delegator_id,
            0,
            QueryForwardData::new(counter_id, &[], "read_value"),
            &mut gas,
        )
        .unwrap();

    gas.spent()
}

#[test]
fn inter_contract_host_call_cost() {
    let cheap = execute_counter_delegation_with_config(&DEFAULT_CONFIG);
    let expensive =
        execute_counter_delegation_with_config(&HIGH_HOST_COST_CONFIG);

    assert_eq!(
        expensive,
        cheap + HIGH_HOST_COST_CONFIG.host_costs.query
            - DEFAULT_CONFIG.host_costs.query
    );
}

fn execute_contract_with_config(config: &'static Config) -> u64 {
    let counter = Counter::new(99);

    let mut network = NetworkState::builder().config(config).build();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(&counter, code.to_vec(), network.store());
    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let (_, network) = network
        .transact(contract_id, 0, counter::Increment, &mut gas)
        .expect("Transaction error");

    network
        .query(contract_id, 0, ReadValue, &mut gas)
        .expect("Query error");

    gas.spent()
}

const DEFAULT_CONFIG: Config = Config::new();

const HIGH_COST_CONFIG: Config = Config {
    op_costs: OpCosts {
        bit: 10000,
        add: 10000,
        mul: 10000,
        div: 10000,
        load: 10000,
        store: 10000,
        const_decl: 10000,
        local: 10000,
        global: 10000,
        flow: 10000,
        integer_comp: 10000,
        float_comp: 10000,
        float: 10000,
        conversion: 10000,
        float_conversion: 10000,
        reinterpret: 10000,
        unreachable: 10000,
        nop: 10000,
        current_mem: 10000,
        grow_mem: 10000,
    },
    ..Config::new()
};

#[test]
fn change_gas_cost_per_op_with_schedule() {
    assert!(execute_contract_with_config(&DEFAULT_CONFIG) < 15000);
    assert!(execute_contract_with_config(&HIGH_COST_CONFIG) > 100_000);
}

const NO_METERING_CONFIG: Config = Config {
    has_metering: false,
    ..Config::new()
};

#[test]
fn no_gas_consumption_when_metering_is_off() {
    assert_eq!(execute_contract_with_config(&NO_METERING_CONFIG), 0);
}
