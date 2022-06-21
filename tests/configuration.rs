// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use counter::Counter;
use microkelvin::{HostStore, StoreRef};
use rusk_vm::{Config, Contract, GasMeter, NetworkState, OpCosts};

fn execute_contract_with_config(config: &'static Config) -> u64 {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&counter, code.to_vec(), &store);
    let mut network = NetworkState::with_config(store, config);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact(contract_id, 0, counter::Increment, &mut gas)
        .expect("Transaction error");

    network
        .query(contract_id, 0, counter::ReadValue, &mut gas)
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
