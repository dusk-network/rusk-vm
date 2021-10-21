// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use block_height::BlockHeight;
use callee_1::Callee1;
use callee_2::Callee2;
use caller::Caller;
use counter::Counter;
use counter_float::CounterFloat;
use delegator::Delegator;
use dusk_abi::Transaction;
use fibonacci::Fibonacci;
use gas_consumed::GasConsumed;
use rusk_vm::{Contract, ContractId, GasMeter, NetworkState, VMError};
use self_snapshot::SelfSnapshot;
use tx_vec::TxVec;

fn fibonacci_reference(n: u64) -> u64 {
    if n < 2 {
        n
    } else {
        fibonacci_reference(n - 1) + fibonacci_reference(n - 2)
    }
}

#[test]
fn counter() {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
            .unwrap(),
        99
    );

    network
        .transact::<_, ()>(contract_id, counter::INCREMENT, &mut gas)
        .unwrap();

    assert_eq!(
        network
            .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
            .unwrap(),
        100
    );
}

#[test]
fn counter_trivial() {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
            .unwrap(),
        99
    );
}

// #[test]
// fn delegated_call() {
//     let counter = Counter::new(99);
//     let delegator = Delegator;
//
//     let mut network = NetworkState::default();
//
//     let counter_code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");
//
//     let counter_contract = Contract::new(counter, counter_code.to_vec());
//     let counter_id = network.deploy(counter_contract).unwrap();
//
//     let delegator_code = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/delegator.wasm"
//     );
//     let delegator_contract = Contract::new(delegator, delegator_code.to_vec());
//     let delegator_id = network.deploy(delegator_contract).unwrap();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     // delegate query
//
//     assert_eq!(
//         network
//             .query::<_, i32>(
//                 delegator_id,
//                 (delegator::DELEGATE_QUERY, counter_id, counter::READ_VALUE),
//                 &mut gas
//             )
//             .unwrap(),
//         99
//     );
//
//     // delegate transaction
//
//     network
//         .transact::<_, ()>(
//             delegator_id,
//             (
//                 delegator::DELEGATE_TRANSACTION,
//                 counter_id,
//                 counter::INCREMENT,
//             ),
//             &mut gas,
//         )
//         .unwrap();
//
//     // changed the value of counter
//
//     assert_eq!(
//         network
//             .query::<_, i32>(counter_id, counter::READ_VALUE, &mut gas)
//             .unwrap(),
//         100
//     );
// }
//
// #[test]
// fn fibonacci() {
//     let fib = Fibonacci;
//
//     let code = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/fibonacci.wasm"
//     );
//
//     let contract = Contract::new(fib, code.to_vec());
//
//     let mut network = NetworkState::default();
//
//     let contract_id = network.deploy(contract).unwrap();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     let n = 5;
//
//     for i in 0..n {
//         assert_eq!(
//             network
//                 .query::<_, u64>(contract_id, (fibonacci::COMPUTE, i), &mut gas)
//                 .unwrap(),
//             fibonacci_reference(i)
//         );
//     }
// }
//
// #[test]
// fn block_height() {
//     let bh = BlockHeight::new();
//
//     let code = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/block_height.wasm"
//     );
//
//     let contract = Contract::new(bh, code.to_vec());
//
//     let mut network = NetworkState::with_block_height(99);
//
//     let contract_id = network.deploy(contract).unwrap();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     assert_eq!(
//         99,
//         network
//             .query::<_, u64>(contract_id, block_height::BLOCK_HEIGHT, &mut gas)
//             .unwrap()
//     )
// }
//
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
//     let mut network = NetworkState::with_block_height(99);
//
//     let contract_id = network.deploy(contract).unwrap();
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     assert_eq!(
//         7,
//         network
//             .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
//             .unwrap()
//     );
//
//     // returns old value
//     assert_eq!(
//         network
//             .transact::<_, i32>(
//                 contract_id,
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
//             .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
//             .unwrap()
//     );
//
//     network
//         .transact::<_, ()>(
//             contract_id,
//             (self_snapshot::SELF_CALL_TEST_A, 10),
//             &mut gas,
//         )
//         .unwrap();
//
//     assert_eq!(
//         10,
//         network
//             .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
//             .unwrap()
//     );
//
//     let result = network.transact::<_, ()>(
//         contract_id,
//         (self_snapshot::UPDATE_AND_PANIC, 11),
//         &mut gas,
//     );
//
//     assert!(result.is_err());
//
//     assert_eq!(
//         10,
//         network
//             .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
//             .unwrap()
//     );
//
//     let transaction =
//         Transaction::from_canon(&(self_snapshot::SET_CROSSOVER, 12));
//
//     network
//         .transact::<_, ()>(
//             contract_id,
//             (self_snapshot::SELF_CALL_TEST_B, contract_id, transaction),
//             &mut gas,
//         )
//         .unwrap();
//
//     assert_eq!(
//         12,
//         network
//             .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
//             .unwrap()
//     );
// }
//
// #[test]
// fn tx_vec() {
//     let value = 15;
//     let tx_vec = TxVec::new(value);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/tx_vec.wasm");
//     let contract = Contract::new(tx_vec, code.to_vec());
//
//     let mut network = NetworkState::default();
//     let contract_id = network.deploy(contract).unwrap();
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     let v = network
//         .query::<_, u8>(contract_id, tx_vec::READ_VALUE, &mut gas)
//         .unwrap();
//     assert_eq!(value, v);
//
//     let values = vec![3u8, 5, 7];
//     let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));
//
//     network
//         .transact::<_, ()>(contract_id, (tx_vec::SUM, values), &mut gas)
//         .unwrap();
//
//     let v = network
//         .query::<_, u8>(contract_id, tx_vec::READ_VALUE, &mut gas)
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
//             (tx_vec::DELEGATE_SUM, contract_id, tx),
//             &mut gas,
//         )
//         .unwrap();
//
//     let v = network
//         .query::<_, u8>(contract_id, tx_vec::READ_VALUE, &mut gas)
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
//             (tx_vec::DELEGATE_SUM, contract_id, tx),
//             &mut gas,
//         )
//         .unwrap();
//
//     let v = network
//         .query::<_, u8>(contract_id, tx_vec::READ_VALUE, &mut gas)
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
//         include_bytes!("../target/wasm32-unknown-unknown/release/caller.wasm");
//     let code_callee1 = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/callee_1.wasm"
//     );
//     let code_callee2 = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/callee_2.wasm"
//     );
//
//     let mut network = NetworkState::default();
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
//             (caller::SET_TARGET, callee1_id),
//             &mut gas,
//         )
//         .unwrap();
//
//     network
//         .transact::<_, ()>(
//             callee1_id,
//             (caller::SET_TARGET, callee2_id),
//             &mut gas,
//         )
//         .unwrap();
//
//     assert_eq!(
//         network
//             .query::<_, (ContractId, ContractId, ContractId)>(
//                 caller_id,
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
//     let mut network = NetworkState::default();
//
//     let contract_id = network.deploy(contract).expect("Deploy error");
//
//     // 2050 is the gas held that is known will be spent in the contract
//     // after the `dusk_abi::gas_left()` call
//     let mut gas = GasMeter::with_range(2_050..1_000_000_000);
//
//     network
//         .transact::<_, ()>(contract_id, gas_consumed::INCREMENT, &mut gas)
//         .expect("Transaction error");
//
//     assert_eq!(
//         network
//             .query::<_, i32>(contract_id, gas_consumed::VALUE, &mut gas)
//             .expect("Query error"),
//         100
//     );
//
//     let (gas_consumed, gas_left) = network
//         .query::<_, (u64, u64)>(
//             contract_id,
//             gas_consumed::GAS_CONSUMED,
//             &mut gas,
//         )
//         .expect("Query error");
//
//     assert_eq!(gas_left + gas.spent(), 1_000_000_000,
//         "The gas left plus the gas spent should be equal to the initial gas provided
//         Debug info:
//         GasMeter values: gas.spent() = {}, gas.left() = {}
//         queried values:  gas_consumed = {}, gas_left = {}", gas.spent(), gas.left(), gas_consumed, gas_left);
//
//     assert_eq!(
//         gas.spent() - gas_consumed,
//         2_050,
//         "The gas spent minus the gas consumed should be equal to the gas held"
//     );
// }
//
// #[test]
// fn gas_consumption_works() {
//     let counter = Counter::new(99);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");
//
//     let contract = Contract::new(counter, code.to_vec());
//
//     let mut network = NetworkState::default();
//
//     let contract_id = network.deploy(contract).expect("Deploy error");
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     network
//         .transact::<_, ()>(contract_id, counter::INCREMENT, &mut gas)
//         .expect("Transaction error");
//
//     assert_eq!(
//         network
//             .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
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
//     let counter = Counter::new(99);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");
//
//     let contract = Contract::new(counter, code.to_vec());
//
//     let mut network = NetworkState::default();
//
//     let contract_id = network.deploy(contract).expect("Deploy error");
//
//     let mut gas = GasMeter::with_limit(1);
//
//     let should_be_err =
//         network.transact::<_, ()>(contract_id, counter::INCREMENT, &mut gas);
//     assert!(format!("{:?}", should_be_err).contains("Out of Gas error"));
//
//     // Ensure all gas is consumed even the tx did not succeed.
//     assert_eq!(gas.left(), 0);
// }
//
// #[test]
// fn deploy_fails_with_floats() {
//     let counter = CounterFloat::new(9.99f32);
//
//     let code = include_bytes!(
//         "../target/wasm32-unknown-unknown/release/counter_float.wasm"
//     );
//
//     let contract = Contract::new(counter, code.to_vec());
//
//     let mut network = NetworkState::default();
//
//     assert!(matches!(
//         network.deploy(contract),
//         Err(rusk_vm::VMError::InstrumentalizationError(_))
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
//     let mut network = NetworkState::default();
//
//     // The id is the same returned by the deploy function
//     assert_eq!(id, network.deploy_with_id(id, contract)?);
//
//     // Get the contract deployed using the same id, and verify the state is also
//     // the same
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
//     // Get the contract deployed using the same id, and verify the state is NOT
//     // the same as before.
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
//     let counter = Counter::new(99);
//
//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");
//
//     let contract = Contract::new(counter, code.to_vec());
//
//     let (persist_id, contract_id) = {
//         let mut network = NetworkState::default();
//
//         let contract_id = network.deploy(contract).unwrap();
//
//         let mut gas = GasMeter::with_limit(1_000_000_000);
//
//         assert_eq!(
//             network
//                 .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
//                 .unwrap(),
//             99
//         );
//
//         network
//             .transact::<_, ()>(contract_id, counter::INCREMENT, &mut gas)
//             .unwrap();
//
//         assert_eq!(
//             network
//                 .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
//                 .unwrap(),
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
//     // If the persistence works, We should still read 100 with a freshly created
//     // NetworkState.
//     let mut network = NetworkState::with_block_height(10)
//         .restore(persist_id)
//         .expect("Error reconstructing the NetworkState");
//
//     let mut gas = GasMeter::with_limit(1_000_000_000);
//
//     assert_eq!(
//         network
//             .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
//             .unwrap(),
//         100
//     );
//
//     // Teardown
//     std::fs::remove_dir_all(std::env::temp_dir().join("test_persist"))
//         .expect("teardown fn error");
// }
