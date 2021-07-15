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
use dusk_abi::{Module, Transaction};
use fibonacci::Fibonacci;
use gas_consumed::GasConsumed;
use rusk_vm::{Contract, ContractId, GasMeter, NetworkState};
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

#[test]
fn delegated_call() {
    let counter = Counter::new(99);
    let delegator = Delegator;

    let mut network = NetworkState::default();

    let counter_code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let counter_contract = Contract::new(counter, counter_code.to_vec());
    let counter_id = network.deploy(counter_contract).unwrap();

    let delegator_code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/delegator.wasm"
    );
    let delegator_contract = Contract::new(delegator, delegator_code.to_vec());
    let delegator_id = network.deploy(delegator_contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    // delegate query

    assert_eq!(
        network
            .query::<_, i32>(
                delegator_id,
                (delegator::DELEGATE_QUERY, counter_id, counter::READ_VALUE),
                &mut gas
            )
            .unwrap(),
        99
    );

    // delegate transaction

    network
        .transact::<_, ()>(
            delegator_id,
            (
                delegator::DELEGATE_TRANSACTION,
                counter_id,
                counter::INCREMENT,
            ),
            &mut gas,
        )
        .unwrap();

    // changed the value of counter

    assert_eq!(
        network
            .query::<_, i32>(counter_id, counter::READ_VALUE, &mut gas)
            .unwrap(),
        100
    );
}

#[test]
fn fibonacci() {
    let fib = Fibonacci;

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/fibonacci.wasm"
    );

    let contract = Contract::new(fib, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let n = 5;

    for i in 0..n {
        assert_eq!(
            network
                .query::<_, u64>(contract_id, (fibonacci::COMPUTE, i), &mut gas)
                .unwrap(),
            fibonacci_reference(i)
        );
    }
}

struct PoseidonModule;

impl Module for PoseidonModule {
    fn id() -> ContractId {
        ContractId::reserved(11)
    }
}

// impl HostModule for PoseidonModule
// where
//     S: Store,
// {
//     fn execute(&self, query: Query) -> Result<ReturnValue, CanonError> {
//         let mut source = Source::new(query.as_bytes(), &self.store);

//         let qid: u8 = Canon::::read(&mut source)?;

//         match qid {
//             0 => {
//                 let scalars: Vec<BlsScalar> = Canon::::read(&mut source)?;
//                 let ret = dusk_poseidon::sponge::hash(&scalars);

//                 ReturnValue::from_canon(&ret, &self.store)
//             }
//             _ => todo!(),
//         }
//     }
// }

// #[test]
// fn hash_as_host_fn() {
//     let test_inputs = [
//         "bb67ed265bf1db490ded2e1ede55c0d14c55521509dc73f9c354e98ab76c9625",
//         "7e74220084d75e10c89e9435d47bb5b8075991b2e29be3b84421dac3b1ee6007",
//         "5ce5481a4d78cca03498f72761da1b9f1d2aa8fb300be39f0e4fe2534f9d4308",
//     ];

//     let test_inputs: Vec<BlsScalar> = test_inputs
//         .iter()
//         .map(|input| BlsScalar::from_hex_str(input).unwrap())
//         .collect();

//     let hash = HostFnTest::new();

//     let store = MS::new();

//     let code =
//         include_bytes!("../target/wasm32-unknown-unknown/release/host_fn.
// wasm");

//     let contract = Contract::new(hash, code.to_vec()).unwrap();

//     let mut network = NetworkState::default();

//     let pos_mod = PoseidonModule::new(store.clone());

//     network.register_host_module(pos_mod);

//     let contract_id = network.deploy(contract).unwrap();

//     let mut gas = GasMeter::with_limit(1_000_000_000);

//     assert_eq!(
//         "0xe36f4ea9b858d5c85b02770823c7c5d8253c28787d17f283ca348b906dca8528",
//         format!(
//             "{:#x}",
//             network
//                 .query::<_, BlsScalar>(
//                     contract_id,
//                     (host_fn::HASH, test_inputs),
//                     &mut gas
//                 )
//                 .unwrap()
//         )
//     );
// }

#[test]
fn block_height() {
    let bh = BlockHeight::new();

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/block_height.wasm"
    );

    let contract = Contract::new(bh, code.to_vec());

    let mut network = NetworkState::with_block_height(99);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        99,
        network
            .query::<_, u64>(contract_id, block_height::BLOCK_HEIGHT, &mut gas)
            .unwrap()
    )
}

#[test]
fn self_snapshot() {
    let bh = SelfSnapshot::new(7);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/self_snapshot.wasm"
    );

    let contract = Contract::new(bh, code.to_vec());

    let mut network = NetworkState::with_block_height(99);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        7,
        network
            .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
            .unwrap()
    );

    // returns old value
    assert_eq!(
        network
            .transact::<_, i32>(
                contract_id,
                (self_snapshot::SET_CROSSOVER, 9),
                &mut gas,
            )
            .unwrap(),
        7
    );

    assert_eq!(
        9,
        network
            .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
            .unwrap()
    );

    network
        .transact::<_, ()>(
            contract_id,
            (self_snapshot::SELF_CALL_TEST_A, 10),
            &mut gas,
        )
        .unwrap();

    assert_eq!(
        10,
        network
            .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
            .unwrap()
    );

    let result = network.transact::<_, ()>(
        contract_id,
        (self_snapshot::UPDATE_AND_PANIC, 11),
        &mut gas,
    );

    assert!(result.is_err());

    assert_eq!(
        10,
        network
            .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
            .unwrap()
    );

    let transaction =
        Transaction::from_canon(&(self_snapshot::SET_CROSSOVER, 12));

    network
        .transact::<_, ()>(
            contract_id,
            (self_snapshot::SELF_CALL_TEST_B, contract_id, transaction),
            &mut gas,
        )
        .unwrap();

    assert_eq!(
        12,
        network
            .query::<_, i32>(contract_id, self_snapshot::CROSSOVER, &mut gas)
            .unwrap()
    );
}

#[test]
fn tx_vec() {
    let value = 15;
    let tx_vec = TxVec::new(value);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/tx_vec.wasm");
    let contract = Contract::new(tx_vec, code.to_vec());

    let mut network = NetworkState::default();
    let contract_id = network.deploy(contract).unwrap();
    let mut gas = GasMeter::with_limit(1_000_000_000);

    let v = network
        .query::<_, u8>(contract_id, tx_vec::READ_VALUE, &mut gas)
        .unwrap();
    assert_eq!(value, v);

    let values = vec![3u8, 5, 7];
    let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));

    network
        .transact::<_, ()>(contract_id, (tx_vec::SUM, values), &mut gas)
        .unwrap();

    let v = network
        .query::<_, u8>(contract_id, tx_vec::READ_VALUE, &mut gas)
        .unwrap();
    assert_eq!(value, v);

    let values = vec![11u8, 13, 17];
    let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));

    let tx = Transaction::from_canon(&(tx_vec::SUM, values));
    network
        .transact::<_, ()>(
            contract_id,
            (tx_vec::DELEGATE_SUM, contract_id, tx),
            &mut gas,
        )
        .unwrap();

    let v = network
        .query::<_, u8>(contract_id, tx_vec::READ_VALUE, &mut gas)
        .unwrap();
    assert_eq!(value, v);

    let values = (0..3500).map(|i| (i % 255) as u8).collect::<Vec<u8>>();
    let value = value + values.iter().fold(0u8, |s, v| s.wrapping_add(*v));

    let tx = Transaction::from_canon(&(tx_vec::SUM, values));
    network
        .transact::<_, ()>(
            contract_id,
            (tx_vec::DELEGATE_SUM, contract_id, tx),
            &mut gas,
        )
        .unwrap();

    let v = network
        .query::<_, u8>(contract_id, tx_vec::READ_VALUE, &mut gas)
        .unwrap();
    assert_eq!(value, v);
}

#[test]
fn calling() {
    let caller = Caller::new();
    let callee1 = Callee1::new();
    let callee2 = Callee2::new();

    let code_caller =
        include_bytes!("../target/wasm32-unknown-unknown/release/caller.wasm");
    let code_callee1 = include_bytes!(
        "../target/wasm32-unknown-unknown/release/callee_1.wasm"
    );
    let code_callee2 = include_bytes!(
        "../target/wasm32-unknown-unknown/release/callee_2.wasm"
    );

    let mut network = NetworkState::default();

    let caller_id = network
        .deploy(Contract::new(caller, code_caller.to_vec()))
        .unwrap();
    let callee1_id = network
        .deploy(Contract::new(callee1, code_callee1.to_vec()))
        .unwrap();
    let callee2_id = network
        .deploy(Contract::new(callee2, code_callee2.to_vec()))
        .unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact::<_, ()>(
            caller_id,
            (caller::SET_TARGET, callee1_id),
            &mut gas,
        )
        .unwrap();

    network
        .transact::<_, ()>(
            callee1_id,
            (caller::SET_TARGET, callee2_id),
            &mut gas,
        )
        .unwrap();

    assert_eq!(
        network
            .query::<_, (ContractId, ContractId, ContractId)>(
                caller_id,
                caller::CALL,
                &mut gas
            )
            .unwrap(),
        (caller_id, callee1_id, callee2_id),
        "Expected Callers and Callees"
    )
}

#[test]
fn gas_consumed_host_function_works() {
    let gas_contract = GasConsumed::new(99);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/gas_consumed.wasm"
    );

    let contract = Contract::new(gas_contract, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).expect("Deploy error");

    // 2050 is the gas held that is known will be spent in the contract
    // after the `dusk_abi::gas_left()` call
    let mut gas = GasMeter::with_range(2_050..1_000_000_000);

    network
        .transact::<_, ()>(contract_id, gas_consumed::INCREMENT, &mut gas)
        .expect("Transaction error");

    assert_eq!(
        network
            .query::<_, i32>(contract_id, gas_consumed::VALUE, &mut gas)
            .expect("Query error"),
        100
    );

    let (gas_consumed, gas_left) = network
        .query::<_, (u64, u64)>(
            contract_id,
            gas_consumed::GAS_CONSUMED,
            &mut gas,
        )
        .expect("Query error");

    assert_eq!(gas_left + gas.spent(), 1_000_000_000,
        "The gas left plus the gas spent should be equal to the initial gas provided");

    assert_eq!(
        gas.spent() - gas_consumed,
        2_050,
        "The gas spent minus the gas consumed should be equal to the gas held"
    );
}

#[test]
fn gas_consumption_works() {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .transact::<_, ()>(contract_id, counter::INCREMENT, &mut gas)
        .expect("Transaction error");

    assert_eq!(
        network
            .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
            .expect("Query error"),
        100
    );

    assert_ne!(gas.spent(), 100);
    assert!(gas.left() < 1_000_000_000);
}

#[test]
fn out_of_gas_aborts_execution() {
    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).expect("Deploy error");

    let mut gas = GasMeter::with_limit(1);

    let should_be_err =
        network.transact::<_, ()>(contract_id, counter::INCREMENT, &mut gas);
    assert!(format!("{:?}", should_be_err).contains("Out of Gas error"));

    // Ensure all gas is consumed even the tx did not succeed.
    assert_eq!(gas.left(), 0);
}

#[test]
fn deploy_fails_with_floats() {
    let counter = CounterFloat::new(9.99f32);

    let code = include_bytes!(
        "../target/wasm32-unknown-unknown/release/counter_float.wasm"
    );

    let contract = Contract::new(counter, code.to_vec());

    let mut network = NetworkState::default();

    assert!(matches!(
        network.deploy(contract),
        Err(rusk_vm::VMError::InstrumentalizationError(_))
    ));
}

#[cfg(feature = "persistence")]
#[test]
fn persistence() {
    use microkelvin::DiskBackend;

    let counter = Counter::new(99);

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/counter.wasm");

    let contract = Contract::new(counter, code.to_vec());

    let (persist_id, contract_id) = {
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

        (
            network
                .persist(|| {
                    let dir = std::env::temp_dir().join("test_persist");
                    std::fs::create_dir_all(&dir)
                        .expect("Error on tmp dir creation");
                    DiskBackend::new(dir)
                })
                .expect("Error in persistance"),
            contract_id,
        )
    };

    // If the persistance works, We should still read 100 with a freshly created
    // NetworkState.
    let mut network = NetworkState::with_block_height(10)
        .restore(persist_id)
        .expect("Error reconstructing the NetworkState");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        network
            .query::<_, i32>(contract_id, counter::READ_VALUE, &mut gas)
            .unwrap(),
        100
    );

    // Teardown
    std::fs::remove_dir_all(std::env::temp_dir().join("test_persist"))
        .expect("teardown fn error");
}
