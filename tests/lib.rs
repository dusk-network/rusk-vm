// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod contracts;

use rusk_vm::{Contract, ContractId, GasMeter, NetworkState};

use dusk_bls12_381::BlsScalar;
use dusk_bytes::ParseHexStr;

use canonical::{ByteSource, Canon, Store};
use canonical_host::MemStore as MS;
use dusk_abi::{HostModule, Module, Query, ReturnValue, Transaction};

use block_height::BlockHeight;
use counter::Counter;
use delegator::Delegator;
use fibonacci::Fibonacci;
use host_fn::HostFnTest;
use self_snapshot::SelfSnapshot;
use stack::Stack;
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

    let store = MS::new();

    let code = include_bytes!("contracts/counter/counter.wasm");

    let contract = Contract::new(counter, code.to_vec(), &store).unwrap();

    let mut network = NetworkState::<MS>::default();

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

    let store = MS::new();

    let code = include_bytes!("contracts/counter/counter.wasm");

    let contract = Contract::new(counter, code.to_vec(), &store).unwrap();

    let mut network = NetworkState::<MS>::default();

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

    let store = MS::new();

    let mut network = NetworkState::<MS>::default();

    let counter_code = include_bytes!("contracts/counter/counter.wasm");
    let counter_contract =
        Contract::new(counter, counter_code.to_vec(), &store).unwrap();
    let counter_id = network.deploy(counter_contract).unwrap();

    let delegator_code = include_bytes!("contracts/delegator/delegator.wasm");
    let delegator_contract =
        Contract::new(delegator, delegator_code.to_vec(), &store).unwrap();
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

    let store = MS::new();

    let code = include_bytes!("contracts/fibonacci/fibonacci.wasm");

    let contract = Contract::new(fib, code.to_vec(), &store).unwrap();

    let mut network = NetworkState::<MS>::default();

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

#[test]
fn stack() {
    let stack = Stack::new();

    let store = MS::new();

    let code = include_bytes!("contracts/stack/stack.wasm");

    let contract = Contract::new(stack, code.to_vec(), &store).unwrap();

    let mut network = NetworkState::<MS>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let n: i32 = 64;

    for i in 0..n {
        network
            .transact::<_, ()>(contract_id, (stack::PUSH, i), &mut gas)
            .unwrap();
    }

    for i in 0..n {
        assert_eq!(
            network
                .query::<_, Option<i32>>(
                    contract_id,
                    (stack::PEEK, i),
                    &mut gas
                )
                .unwrap(),
            Some(i)
        );
    }

    for i in 0..n {
        let contract_state: Stack<MS> = network
            .get_contract_cast_state(&contract_id)
            .expect("A result");

        assert_eq!(contract_state.peek(i), Some(i));
    }

    for i in 0..n {
        let i = n - i - 1;

        assert_eq!(
            network
                .transact::<_, Option<i32>>(contract_id, stack::POP, &mut gas)
                .unwrap(),
            Some(i)
        );
    }

    assert_eq!(
        network
            .transact::<_, Option<i32>>(contract_id, stack::POP, &mut gas)
            .unwrap(),
        None
    );
}

struct PoseidonModule<S> {
    store: S,
}

impl<S> PoseidonModule<S>
where
    S: Store,
{
    fn new(store: S) -> Self {
        PoseidonModule { store }
    }
}

impl<S> Module for PoseidonModule<S> {
    fn id() -> ContractId {
        ContractId::reserved(11)
    }
}

impl<S> HostModule<S> for PoseidonModule<S>
where
    S: Store,
{
    fn execute(&self, query: Query) -> Result<ReturnValue, S::Error> {
        let mut source = ByteSource::new(query.as_bytes(), &self.store);

        let qid: u8 = Canon::<S>::read(&mut source)?;

        match qid {
            0 => {
                let scalars: Vec<BlsScalar> = Canon::<S>::read(&mut source)?;
                let ret = dusk_poseidon::sponge::hash(&scalars);

                ReturnValue::from_canon(&ret, &self.store)
            }
            _ => todo!(),
        }
    }
}

#[test]
fn hash_as_host_fn() {
    let test_inputs = [
        "bb67ed265bf1db490ded2e1ede55c0d14c55521509dc73f9c354e98ab76c9625",
        "7e74220084d75e10c89e9435d47bb5b8075991b2e29be3b84421dac3b1ee6007",
        "5ce5481a4d78cca03498f72761da1b9f1d2aa8fb300be39f0e4fe2534f9d4308",
    ];

    let test_inputs: Vec<BlsScalar> = test_inputs
        .iter()
        .map(|input| BlsScalar::from_hex_str(input).unwrap())
        .collect();

    let hash = HostFnTest::new();

    let store = MS::new();

    let code = include_bytes!("contracts/host_fn/host_fn.wasm");

    let contract = Contract::new(hash, code.to_vec(), &store).unwrap();

    let mut network = NetworkState::<MS>::default();

    let pos_mod = PoseidonModule::new(store.clone());

    network.register_host_module(pos_mod);

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    assert_eq!(
        "0xe36f4ea9b858d5c85b02770823c7c5d8253c28787d17f283ca348b906dca8528",
        format!(
            "{:#x}",
            network
                .query::<_, BlsScalar>(
                    contract_id,
                    (host_fn::HASH, test_inputs),
                    &mut gas
                )
                .unwrap()
        )
    );
}

#[test]
fn block_height() {
    let bh = BlockHeight::new();

    let store = MS::new();

    let code = include_bytes!("contracts/block_height/block_height.wasm");

    let contract = Contract::new(bh, code.to_vec(), &store).unwrap();

    let mut network = NetworkState::<MS>::with_block_height(99);

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

    let store = MS::new();

    let code = include_bytes!("contracts/self_snapshot/self_snapshot.wasm");

    let contract = Contract::new(bh, code.to_vec(), &store).unwrap();

    let mut network = NetworkState::<MS>::with_block_height(99);

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
        Transaction::from_canon(&(self_snapshot::SET_CROSSOVER, 12), &store)
            .unwrap();

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

    let store = MS::new();
    let code = include_bytes!("contracts/tx_vec/tx_vec.wasm");
    let contract = Contract::new(tx_vec, code.to_vec(), &store).unwrap();

    let mut network = NetworkState::<MS>::default();
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

    let tx = Transaction::from_canon(&(tx_vec::SUM, values), &store).unwrap();
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

    let tx = Transaction::from_canon(&(tx_vec::SUM, values), &store).unwrap();
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
