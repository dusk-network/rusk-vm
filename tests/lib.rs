// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use block_height::{BlockHeight, ReadBlockHeight};
use callee_1::{Callee1State, Callee1Transaction};
use callee_2::Callee2State;
use caller::{CallerQuery, CallerState, CallerTransaction};
use counter::Counter;
// use counter_float::CounterFloat;
use delegator::{Delegator, QueryForwardData, TransactionForwardData};
use fibonacci::ComputeFrom;
use gas_consumed::{GasConsumed, GasConsumedIncrement, GasConsumedValueQuery};
use microkelvin::{HostStore, StoreRef};
use rusk_vm::{Contract, GasMeter, NetworkState};
use self_snapshot::SelfSnapshot;
use tx_vec::{TxVec, TxVecDelegateSum, TxVecReadValue, TxVecSum};

fn fibonacci_reference(n: u64) -> u64 {
    if n < 2 {
        n
    } else {
        fibonacci_reference(n - 1) + fibonacci_reference(n - 2)
    }
}

#[test]
fn minimal_counter() {
    let counter = minimal_counter::Counter::new(99);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/minimal_counter.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&counter, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query(contract_id, 0, minimal_counter::ReadCount, &mut gas)
            .unwrap(),
        99
    );

    println!("ok!");

    network
        .transact(contract_id, 0, minimal_counter::Increment(1), &mut gas)
        .unwrap();

    println!("ek!");

    assert_eq!(
        network
            .query(contract_id, 0, minimal_counter::ReadCount, &mut gas)
            .unwrap(),
        100
    );
}

#[ignore]
fn register() {
    use register::*;

    let reg = Register::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/register.wasm"
    );

    let store = StoreRef::new(HostStore::new());

    let contract = Contract::new(&reg, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    todo!()
}

#[test]
fn string_passthrough() {
    use string_argument::*;

    let stringer = Stringer;

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/string_argument.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&stringer, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query(contract_id, 0, Passthrough::new("Hello world", 3), &mut gas)
            .unwrap(),
        String::from("Hello worldHello worldHello world"),
    );
}

#[test]
fn delegated_call() {
    let counter = Counter::new(99);
    let delegator = Delegator;

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/counter.wasm"
    );
    let delegator_code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/delegator.wasm"
    );
    let store = StoreRef::new(HostStore::new());
    let counter_contract = Contract::new(&counter, code.to_vec(), &store);
    let delegator_contract =
        Contract::new(&delegator, delegator_code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let counter_contract_id = network.deploy(counter_contract).unwrap();
    let delegator_id = network.deploy(delegator_contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let incr_value = counter::Increment;
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rkyv::Archive;

    let mut buf = [0u8; 128];
    let mut ser = BufferSerializer::new(&mut buf);
    let buffer_len = ser.serialize_value(&incr_value).unwrap()
        + core::mem::size_of::<<counter::Increment as Archive>::Archived>();

    // delegate query

    assert_eq!(
        network
            .query(
                delegator_id,
                0,
                QueryForwardData::new(counter_contract_id, &[], "read_value"),
                &mut gas,
            )
            .unwrap(),
        99
    );

    // delegate transaction

    network
        .transact(
            delegator_id,
            0,
            TransactionForwardData::new(
                counter_contract_id,
                &buf[..buffer_len],
                "increment",
            ),
            &mut gas,
        )
        .unwrap();

    // changed the value of counter

    assert_eq!(
        network
            .query(counter_contract_id, 0, counter::ReadValue, &mut gas)
            .unwrap(),
        100
    );
}

#[test]
fn fibonacci() {
    use fibonacci::Fibonacci;
    let fib = Fibonacci;

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/fibonacci.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&fib, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let n = 5;

    for i in 0..n {
        assert_eq!(
            network
                .query(contract_id, 0, ComputeFrom::new(i), &mut gas)
                .unwrap() as u64,
            fibonacci_reference(i as u64)
        );
    }
}

#[test]
fn block_height() {
    let bh = BlockHeight {};

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/block_height.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&bh, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        99,
        network
            .query(contract_id, 99, ReadBlockHeight, &mut gas)
            .unwrap()
    )
}

#[test]
fn self_snapshot() {
    let self_snapshot = SelfSnapshot::new(7);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/self_snapshot.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&self_snapshot, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        7,
        network
            .query(contract_id, 0, self_snapshot::CrossoverQuery, &mut gas)
            .unwrap()
    );

    // returns old value
    assert_eq!(
        network
            .transact(
                contract_id,
                0,
                self_snapshot::SetCrossoverTransaction::new(9),
                &mut gas,
            )
            .unwrap(),
        7
    );

    assert_eq!(
        9,
        network
            .query(contract_id, 0, self_snapshot::CrossoverQuery, &mut gas)
            .unwrap()
    );

    network
        .transact(
            contract_id,
            0,
            self_snapshot::SelfCallTestATransaction::new(10),
            &mut gas,
        )
        .unwrap();

    assert_eq!(
        10,
        network
            .query(contract_id, 0, self_snapshot::CrossoverQuery, &mut gas)
            .unwrap()
    );

    let result = network.transact(
        contract_id,
        0,
        self_snapshot::UpdateAndPanicTransaction::new(11),
        &mut gas,
    );

    assert!(result.is_err());

    assert_eq!(
        10,
        network
            .query(contract_id, 0, self_snapshot::CrossoverQuery, &mut gas)
            .unwrap()
    );

    let set_crossover_value = self_snapshot::SetCrossoverTransaction::new(12);
    use rkyv::ser::serializers::BufferSerializer;
    use rkyv::ser::Serializer;
    use rkyv::Archive;

    let mut buf = [0u8; 128];
    let mut ser = BufferSerializer::new(&mut buf);
    let buffer_len = ser.serialize_value(&set_crossover_value).unwrap()
        + core::mem::size_of::<
            <::self_snapshot::SetCrossoverTransaction as Archive>::Archived,
        >();

    let self_call_test_b_transaction =
        self_snapshot::SelfCallTestBTransaction::new(
            contract_id,
            &buf[..buffer_len],
            "set_crossover",
        );

    network
        .transact(contract_id, 0, self_call_test_b_transaction, &mut gas)
        .unwrap();

    assert_eq!(
        12,
        network
            .query(contract_id, 0, self_snapshot::CrossoverQuery, &mut gas)
            .unwrap()
    );
}

#[test]
fn tx_vec() {
    let value = 15;
    let tx_vec = TxVec::new(value);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/tx_vec.wasm");
    let store = StoreRef::new(HostStore::new());

    let contract = Contract::new(&tx_vec, code.to_vec(), &store);

    let mut network = NetworkState::new(store);
    let contract_id = network.deploy(contract).unwrap();
    let mut gas = GasMeter::with_limit(1_000_000_000);

    let v = network
        .query(contract_id, 0, TxVecReadValue, &mut gas)
        .unwrap();
    assert_eq!(value, v);

    let values = vec![3u8, 5, 7];
    let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));

    network
        .transact(contract_id, 0, TxVecSum::new(values), &mut gas)
        .unwrap();

    let v = network
        .query(contract_id, 0, TxVecReadValue, &mut gas)
        .unwrap();
    assert_eq!(value, v);

    let values = vec![11u8, 13, 17];
    let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));

    let delegate_sum = TxVecDelegateSum::new(contract_id, &values[..]);
    network
        .transact(contract_id, 0, delegate_sum, &mut gas)
        .unwrap();

    let v = network
        .query(contract_id, 0, TxVecReadValue, &mut gas)
        .unwrap();
    assert_eq!(value, v);

    let values = (0..3500).map(|i| (i % 255) as u8).collect::<Vec<u8>>();
    let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));

    let delegate_sum = TxVecDelegateSum::new(contract_id, &values[..]);
    network
        .transact(contract_id, 0, delegate_sum, &mut gas)
        .unwrap();

    let v = network
        .query(contract_id, 0, TxVecReadValue, &mut gas)
        .unwrap();
    assert_eq!(value, v);
}

#[test]
fn calling() {
    let caller = CallerState::new();
    let callee1 = Callee1State::new();
    let callee2 = Callee2State::new();

    let code_caller =
        include_bytes!("../target/wasm32-unknown-unknown/release/caller.wasm");
    let code_callee1 = include_bytes!(
        "../target/wasm32-unknown-unknown/release/callee_1.wasm"
    );
    let code_callee2 = include_bytes!(
        "../target/wasm32-unknown-unknown/release/callee_2.wasm"
    );

    let store = StoreRef::new(HostStore::new());

    let contract0 = Contract::new(&caller, code_caller.to_vec(), &store);
    let contract1 = Contract::new(&callee1, code_callee1.to_vec(), &store);
    let contract2 = Contract::new(&callee2, code_callee2.to_vec(), &store);
    let mut network = NetworkState::new(store);
    let caller_id = network.deploy(contract0).unwrap();
    let callee1_id = network.deploy(contract1).unwrap();
    let callee2_id = network.deploy(contract2).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact(caller_id, 0, CallerTransaction::new(callee1_id), &mut gas)
        .unwrap();

    network
        .transact(callee1_id, 0, Callee1Transaction::new(callee2_id), &mut gas)
        .unwrap();

    assert_eq!(
        network.query(caller_id, 0, CallerQuery, &mut gas).unwrap(),
        (
            caller_id.as_array(),
            callee1_id.as_array(),
            callee2_id.as_array()
        ),
        "Expected Callers and Callees"
    )
}

#[test]
fn gas_consumed_host_function_works() {
    let gas_contract = GasConsumed::new(99);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_consumed.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&gas_contract, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).expect("Deploy error");

    // 2050 is the gas held that is known will be spent in the contract
    // after the `dusk_abi::gas_left()` call
    const CALLER_GAS_LIMIT: u64 = 1_000_000_000;
    let mut gas = GasMeter::with_limit(CALLER_GAS_LIMIT);

    network
        .transact(contract_id, 0, GasConsumedIncrement, &mut gas)
        .expect("Transaction error");

    assert_eq!(
        network
            .query(contract_id, 0, GasConsumedValueQuery, &mut gas)
            .expect("Query error"),
        100
    );

    network
        .query(contract_id, 0, gas_consumed::GasConsumedQuery, &mut gas)
        .expect("Query error");

    assert_eq!(gas.left() + gas.spent(), CALLER_GAS_LIMIT,
       "The gas left plus the gas spent should be equal to the initial gas provided
       Debug info:
       GasMeter values: gas.left() = {}, gas.spent() = {}", gas.left(),
    gas.spent());
}

#[test]
fn gas_consumption_works() {
    let counter = Counter::new(99);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/counter.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&counter, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact(contract_id, 0, counter::Increment, &mut gas)
        .expect("Transaction error");

    assert_eq!(
        network
            .query(contract_id, 0, counter::ReadValue, &mut gas)
            .expect("Query error"),
        100
    );

    assert_ne!(gas.spent(), 100);
    assert!(gas.left() < 1_000_000_000);
}

#[test]
fn out_of_gas_aborts_transaction_execution() {
    let counter = Counter::new(99);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/counter.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&counter, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1);

    let should_be_err =
        network.transact(contract_id, 0, counter::Increment, &mut gas);
    assert!(should_be_err.is_err());
    assert!(format!("{:?}", should_be_err).contains("Out of Gas error"));
    // Ensure all gas is consumed even the tx did not succeed.
    assert_eq!(gas.left(), 0);
}

#[test]
fn out_of_gas_aborts_query_execution() {
    let counter = Counter::new(99);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/counter.wasm"
    );

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&counter, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1);

    let should_be_err =
        network.query(contract_id, 0, counter::ReadValue, &mut gas);
    assert!(should_be_err.is_err());
    assert!(format!("{:?}", should_be_err).contains("Out of Gas error"));
    // Ensure all gas is consumed even the tx did not succeed.
    assert_eq!(gas.left(), 0);
}

#[test]
fn commit_and_reset() {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&counter, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();
    network.commit();

    let mut network_clone = network.clone();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact(contract_id, 0, counter::Increment, &mut gas)
        .unwrap();
    network_clone
        .transact(contract_id, 0, counter::Increment, &mut gas)
        .unwrap();

    network.commit();

    network.reset();
    network_clone.reset();

    assert_eq!(
        network
            .query(contract_id, 0, counter::ReadValue, &mut gas)
            .unwrap(),
        100
    );
    assert_eq!(
        network_clone
            .query(contract_id, 0, counter::ReadValue, &mut gas)
            .unwrap(),
        99
    );
}
