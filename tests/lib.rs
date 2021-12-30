// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// use block_height::BlockHeight;
// use callee_1::Callee1;
// use callee_2::Callee2;
// use caller::Caller;
// use counter::Counter;
// use counter_float::CounterFloat;
use delegator::{Delegator, QueryForwardData};
// use dusk_abi::Transaction;
// use fibonacci::Fibonacci;
// use gas_consumed::GasConsumed;
use fibonacci::ComputeFrom;
use microkelvin::HostStore;
use rkyv::AlignedVec;
use rusk_uplink::RawQuery;
use rusk_vm::{Contract, GasMeter, NetworkState};
// use self_snapshot::SelfSnapshot;
// use tx_vec::TxVec;

fn fibonacci_reference(n: u64) -> u64 {
    if n < 2 {
        n
    } else {
        fibonacci_reference(n - 1) + fibonacci_reference(n - 2)
    }
}

#[test]
fn counter() {
    use counter::Counter;
    use minimal_counter as counter;

    let counter = Counter::new(99);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/minimal_counter.wasm"
    );

    let store = HostStore::new();
    let contract = Contract::new(&counter, code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query(contract_id, 0, counter::ReadCount, &mut gas)
            .unwrap(),
        99
    );

    network
        .transact(contract_id, 0, counter::Increment(1), &mut gas)
        .unwrap();

    assert_eq!(
        network
            .query(contract_id, 0, counter::ReadCount, &mut gas)
            .unwrap(),
        100
    );
}

#[test]
fn string_passthrough() {
    use string_argument::*;

    let stringer = Stringer;

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/string_argument.wasm"
    );

    let store = HostStore::new();
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
//
// #[test]
// fn stringer_trivial() {
//     let stringer = Stringer::new(99);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/stringer.
// wasm");
//
//     let contract = Contract::new(stringer, code.to_vec());
//
//     let mut network = NetworkState::new();
//
//     let contract_id = network.deploy(contract).unwrap();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     assert_eq!(
//         network
//             .query::<_, i32>(contract_id, 0, stringer::READ_VALUE, &mut gas)
//             .unwrap(),
//         99
//     );
// }

#[test]
fn delegated_call() {
    use counter::Counter;
    use minimal_counter as counter;

    let counter = Counter::new(99);
    let delegator = Delegator;

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/deps/minimal_counter.wasm"
    );
    let delegator_code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/delegator.wasm"
    );
    let store = HostStore::new();
    let counter_contract = Contract::new(&counter, code.to_vec(), &store);
    let delegator_contract =
        Contract::new(&delegator, delegator_code.to_vec(), &store);

    let mut network = NetworkState::new(store);

    let counter_contract_id = network.deploy(counter_contract).unwrap();
    let delegator_id = network.deploy(delegator_contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    // delegate query
    assert_eq!(
        network
            .query(
                delegator_id,
                0,
                QueryForwardData::new(counter_contract_id, &[], "read"),
                &mut gas,
            )
            .unwrap(),
        99
    );

    // delegate transaction

    // network
    //     .transact::<_, ()>(
    //         delegator_id,
    //         0,
    //         (
    //             delegator::DELEGATE_TRANSACTION,
    //             stringer_id,
    //             stringer::INCREMENT,
    //         ),
    //         &mut gas,
    //     )
    //     .unwrap();
    //
    // // changed the value of stringer
    //
    // assert_eq!(
    //     network
    //         .query::<_, i32>(stringer_id, 0, stringer::READ_VALUE, &mut gas)
    //         .unwrap(),
    //     100
    // );
}

#[test]
fn fibonacci() {
    use fibonacci::Fibonacci;
    let fib = Fibonacci;

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/fibonacci.wasm"
    );

    let store = HostStore::new();
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

// #[test]
// fn block_height() {
//     let bh = BlockHeight::new();

//     let code = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/block_height.wasm"
//     );

//     let contract = Contract::new(bh, code.to_vec());

//     let mut network = NetworkState::new();

//     let contract_id = network.deploy(contract).unwrap();

//     let mut gas = GasMeter::with_limit(1_000_000_000);

//     assert_eq!(
//         99,
//         network
//             .query::<_, u64>(
//                 contract_id,
//                 99,
//                 block_height::BLOCK_HEIGHT,
//                 &mut gas
//             )
//             .unwrap()
//     )
// }

// #[test]
// fn self_snapshot() {
//     let bh = SelfSnapshot::new(7);
//
//     let code = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/self_snapshot.wasm"
//     );
//
//     let contract = Contract::new(bh, code.to_vec());
//
//     let mut network = NetworkState::new();
//
//     let contract_id = network.deploy(contract).unwrap();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     assert_eq!(
//         7,
//         network
//             .query::<_, i32>(contract_id, 0, self_snapshot::CROSSOVER, &mut
// gas)             .unwrap()
//     );
//
//     // returns old value
//     assert_eq!(
//         network
//             .transact::<_, i32>(
//                 contract_id,
//                 0,
//                 (self_snapshot::SET_CROSSOVER, 9),
//                 &mut gas,
//             )
//             .unwrap(),
//         7
//     );
//
//     assert_eq!(
//         9,
//         network
//             .query::<_, i32>(contract_id, 0, self_snapshot::CROSSOVER, &mut
// gas)             .unwrap()
//     );
//
//     network
//         .transact::<_, ()>(
//             contract_id,
//             0,
//             (self_snapshot::SELF_CALL_TEST_A, 10),
//             &mut gas,
//         )
//         .unwrap();
//
//     assert_eq!(
//         10,
//         network
//             .query::<_, i32>(contract_id, 0, self_snapshot::CROSSOVER, &mut
// gas)             .unwrap()
//     );
//
//     let result = network.transact::<_, ()>(
//         contract_id,
//         0,
//         (self_snapshot::UPDATE_AND_PANIC, 11),
//         &mut gas,
//     );
//
//     assert!(result.is_err());
//
//     assert_eq!(
//         10,
//         network
//             .query::<_, i32>(contract_id, 0, self_snapshot::CROSSOVER, &mut
// gas)             .unwrap()
//     );
//
//     let transaction =
//         Transaction::from_canon(&(self_snapshot::SET_CROSSOVER, 12));
//
//     network
//         .transact::<_, ()>(
//             contract_id,
//             0,
//             (self_snapshot::SELF_CALL_TEST_B, contract_id, transaction),
//             &mut gas,
//         )
//         .unwrap();
//
//     assert_eq!(
//         12,
//         network
//             .query::<_, i32>(contract_id, 0, self_snapshot::CROSSOVER, &mut
// gas)             .unwrap()
//     );
// }
//
// #[test]
// fn tx_vec() {
//     let value = 15;
//     let tx_vec = TxVec::new(value);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/tx_vec.wasm"
// );     let contract = Contract::new(tx_vec, code.to_vec());
//
//     let mut network = NetworkState::new();
//     let contract_id = network.deploy(contract).unwrap();
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     let v = network
//         .query::<_, u8>(contract_id, 0, tx_vec::READ_VALUE, &mut gas)
//         .unwrap();
//     assert_eq!(value, v);
//
//     let values = vec![3u8, 5, 7];
//     let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));
//
//     network
//         .transact::<_, ()>(contract_id, 0, (tx_vec::SUM, values), &mut gas)
//         .unwrap();
//
//     let v = network
//         .query::<_, u8>(contract_id, 0, tx_vec::READ_VALUE, &mut gas)
//         .unwrap();
//     assert_eq!(value, v);
//
//     let values = vec![11u8, 13, 17];
//     let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));
//
//     let tx = Transaction::from_canon(&(tx_vec::SUM, values));
//     network
//         .transact::<_, ()>(
//             contract_id,
//             0,
//             (tx_vec::DELEGATE_SUM, contract_id, tx),
//             &mut gas,
//         )
//         .unwrap();
//
//     let v = network
//         .query::<_, u8>(contract_id, 0, tx_vec::READ_VALUE, &mut gas)
//         .unwrap();
//     assert_eq!(value, v);
//
//     let values = (0..3500).map(|i| (i % 255) as u8).collect::<Vec<u8>>();
//     let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));
//
//     let tx = Transaction::from_canon(&(tx_vec::SUM, values));
//     network
//         .transact::<_, ()>(
//             contract_id,
//             0,
//             (tx_vec::DELEGATE_SUM, contract_id, tx),
//             &mut gas,
//         )
//         .unwrap();
//
//     let v = network
//         .query::<_, u8>(contract_id, 0, tx_vec::READ_VALUE, &mut gas)
//         .unwrap();
//     assert_eq!(value, v);
// }
//
// #[test]
// fn calling() {
//     let caller = Caller::new();
//     let callee1 = Callee1::new();
//     let callee2 = Callee2::new();
//
//     let code_caller =
//         include_bytes!("../target/wasm32-unknown-unknown/release/caller.wasm"
// );     let code_callee1 = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/callee_1.wasm"
//     );
//     let code_callee2 = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/callee_2.wasm"
//     );
//
//     let mut network = NetworkState::new();
//
//     let caller_id = network
//         .deploy(Contract::new(caller, code_caller.to_vec()))
//         .unwrap();
//     let callee1_id = network
//         .deploy(Contract::new(callee1, code_callee1.to_vec()))
//         .unwrap();
//     let callee2_id = network
//         .deploy(Contract::new(callee2, code_callee2.to_vec()))
//         .unwrap();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     network
//         .transact::<_, ()>(
//             caller_id,
//             0,
//             (caller::SET_TARGET, callee1_id),
//             &mut gas,
//         )
//         .unwrap();
//
//     network
//         .transact::<_, ()>(
//             callee1_id,
//             0,
//             (caller::SET_TARGET, callee2_id),
//             &mut gas,
//         )
//         .unwrap();
//
//     assert_eq!(
//         network
//             .query::<_, (ContractId, ContractId, ContractId)>(
//                 caller_id,
//                 0,
//                 caller::CALL,
//                 &mut gas
//             )
//             .unwrap(),
//         (caller_id, callee1_id, callee2_id),
//         "Expected Callers and Callees"
//     )
// }
//
// #[test]
// fn gas_consumed_host_function_works() {
//     let gas_contract = GasConsumed::new(99);
//
//     let code = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/gas_consumed.wasm"
//     );
//
//     let contract = Contract::new(gas_contract, code.to_vec());
//
//     let mut network = NetworkState::new();
//
//     let contract_id = network.deploy(contract).expect("Deploy error");
//
//     // 2050 is the gas held that is known will be spent in the contract
//     // after the `dusk_abi::gas_left()` call
//     const CALLER_GAS_LIMIT: u64 = 1_000_000_000;
//     let mut gas = GasMeter::with_limit(CALLER_GAS_LIMIT);
//
//     network
//         .transact::<_, ()>(contract_id, 0, gas_consumed::INCREMENT, &mut gas)
//         .expect("Transaction error");
//
//     assert_eq!(
//         network
//             .query::<_, i32>(contract_id, 0, gas_consumed::VALUE, &mut gas)
//             .expect("Query error"),
//         100
//     );
//
//     network
//         .query::<_, (u64, u64)>(
//             contract_id,
//             0,
//             gas_consumed::GAS_CONSUMED,
//             &mut gas,
//         )
//         .expect("Query error");
//
//     assert_eq!(gas.left() + gas.spent(), CALLER_GAS_LIMIT,
//                "The gas left plus the gas spent should be equal to the
// initial gas provided         Debug info:
//         GasMeter values: gas.left() = {}, gas.spent() = {}", gas.left(),
// gas.spent()); }
//
// #[test]
// fn gas_consumption_works() {
//     let stringer = Stringer::new(99);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/stringer.
// wasm");
//
//     let contract = Contract::new(stringer, code.to_vec());
//
//     let mut network = NetworkState::new();
//
//     let contract_id = network.deploy(contract).expect("Deploy error");
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     network
//         .transact::<_, ()>(contract_id, 0, stringer::INCREMENT, &mut gas)
//         .expect("Transaction error");
//
//     assert_eq!(
//         network
//             .query::<_, i32>(contract_id, 0, stringer::READ_VALUE, &mut gas)
//             .expect("Query error"),
//         100
//     );
//
//     assert_ne!(gas.spent(), 100);
//     assert!(gas.left() < 1_000_000_000);
// }
//
// #[test]
// fn out_of_gas_aborts_execution() {
//     let stringer = Stringer::new(99);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/stringer.
// wasm");
//
//     let contract = Contract::new(stringer, code.to_vec());
//
//     let mut network = NetworkState::new();
//
//     let contract_id = network.deploy(contract).expect("Deploy error");
//
//     let mut gas = GasMeter::with_limit(1);
//
//     let should_be_err =
//         network.transact::<_, ()>(contract_id, 0, stringer::INCREMENT, &mut
// gas);     assert!(format!("{:?}", should_be_err).contains("Out of Gas
// error"));
//
//     // Ensure all gas is consumed even the tx did not succeed.
//     assert_eq!(gas.left(), 0);
// }
//
// #[test]
// fn deploy_fails_with_floats() {
//     let stringer = StringerFloat::new(9.99f32);
//
//     let code = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/stringer_float.wasm"
//     );
//
//     let contract = Contract::new(stringer, code.to_vec());
//
//     let forbidden_floats_schedule = Schedule {
//         has_forbidden_floats: false,
//         ..Default::default()
//     };
//
//     let mut network =
// NetworkState::with_schedule(&forbidden_floats_schedule);
//
//     assert!(matches!(
//         network.deploy(contract),
//         Err(rusk_vm::VMError::InstrumentationError(_))
//     ));
// }
//
// #[test]
// fn deploy_with_id() -> Result<(), VMError> {
//     // Smallest valid WASM module possible so `deploy` won't raise a
//     // `InvalidByteCode` error
//     let code = 0x0000_0001_6D73_6100_u64.to_le_bytes();
//
//     // Create a contract with a simple state
//     let contract = Contract::new(0xfeed_u16, code.to_vec());
//
//     // Reserve a `ContractId`
//     let id = ContractId::reserved(0x10);
//
//     // Deploy with the id given
//     let mut network = NetworkState::new();
//
//     // The id is the same returned by the deploy function
//     assert_eq!(id, network.deploy_with_id(id, contract)?);
//
//     // Get the contract deployed using the same id, and verify the state is
// also     // the same
//     let state: u16 = network
//         .get_contract(&id)?
//         .state()
//         .cast()
//         .expect("Cannot cast the state");
//     assert_eq!(state, 0xfeed);
//
//     // Deploy another contract at the same address
//     let contract = Contract::new(0xcafe_u16, code.to_vec());
//     network.deploy_with_id(id, contract)?;
//
//     // Get the contract deployed using the same id, and verify the state is
// NOT     // the same as before.
//     //
//     // TODO: This means a contract CAN BE overriden once deployed, we need to
//     // decided if we should raise an error if a contract already exists with
//     // the same address
//     network.get_contract(&id)?;
//
//     let state: u16 = network
//         .get_contract(&id)?
//         .state()
//         .cast()
//         .expect("Cannot cast the state");
//     assert_eq!(state, 0xcafe);
//
//     Ok(())
// }
//
// #[cfg(feature = "persistence")]
// #[test]
// fn persistence() {
//     use microkelvin::DiskBackend;
//
//     let stringer = Stringer::new(99);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/stringer.
// wasm");
//
//     let contract = Contract::new(stringer, code.to_vec());
//
//     let (persist_id, contract_id) = {
//         let mut network = NetworkState::new();
//
//         let contract_id = network.deploy(contract).unwrap();
//
//         let mut gas = GasMeter::with_limit(1_000_000_000);
//
//         assert_eq!(
//             network
//                 .query::<_, i32>(contract_id, 0, stringer::READ_VALUE, &mut
// gas)                 .unwrap(),
//             99
//         );
//
//         network
//             .transact::<_, ()>(contract_id, 0, stringer::INCREMENT, &mut gas)
//             .unwrap();
//
//         assert_eq!(
//             network
//                 .query::<_, i32>(contract_id, 0, stringer::READ_VALUE, &mut
// gas)                 .unwrap(),
//             100
//         );
//
//         (
//             network
//                 .persist(|| {
//                     let dir = std::env::temp_dir().join("test_persist");
//                     std::fs::create_dir_all(&dir)
//                         .expect("Error on tmp dir creation");
//                     DiskBackend::new(dir)
//                 })
//                 .expect("Error in persistence"),
//             contract_id,
//         )
//     };
//
//     // If the persistence works, We should still read 100 with a freshly
// created     // NetworkState.
//     let mut network = NetworkState::new()
//         .restore(persist_id)
//         .expect("Error reconstructing the NetworkState");
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     assert_eq!(
//         network
//             .query::<_, i32>(contract_id, 0, stringer::READ_VALUE, &mut gas)
//             .unwrap(),
//         100
//     );
//
//     // Teardown
//     std::fs::remove_dir_all(std::env::temp_dir().join("test_persist"))
//         .expect("teardown fn error");
// }
//
// #[test]
// fn commit_and_reset() {
//     let stringer = Stringer::new(99);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/stringer.
// wasm");
//
//     let contract = Contract::new(stringer, code.to_vec());
//
//     let mut network = NetworkState::new();
//
//     let contract_id = network.deploy(contract).unwrap();
//     network.commit();
//
//     let mut network_clone = network.clone();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     network
//         .transact::<_, ()>(contract_id, 0, stringer::INCREMENT, &mut gas)
//         .unwrap();
//     network_clone
//         .transact::<_, ()>(contract_id, 0, stringer::INCREMENT, &mut gas)
//         .unwrap();
//
//     network.commit();
//
//     network.reset();
//     network_clone.reset();
//
//     assert_eq!(
//         network
//             .query::<_, i32>(contract_id, 0, stringer::READ_VALUE, &mut gas)
//             .unwrap(),
//         100
//     );
//     assert_eq!(
//         network_clone
//             .query::<_, i32>(contract_id, 0, stringer::READ_VALUE, &mut gas)
//             .unwrap(),
//         99
//     );
// }
