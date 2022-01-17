// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::{BranchRef, HostStore, StoreRef};
use rkyv::{archived_root, Deserialize};
use rusk_vm::{Contract, ContractRef, GasMeter, NetworkState};
use stack::Stack;

#[test]
fn stack() {
    type Leaf = u64;
    const N: Leaf = 0;

    let stack = Stack::new();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");

    let mut store = StoreRef::new(HostStore::new());
    let contract = Contract::new(&stack, code.to_vec(), &store);
    let mut network = NetworkState::new(store.clone());

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..N {
        network
            .transact(contract_id, 0, stack::Push::new(i), &mut gas)
            .unwrap();

        // deserialize and test for peeks
        let contract = network.get_contract(&contract_id).expect("A result");
        let leaf = contract.leaf();
        let state_bytes = leaf.state();
        let cast = unsafe { archived_root::<Stack>(state_bytes) };
        let de: Stack = cast.deserialize(&mut store).unwrap();

        for o in 0..i {
            assert_eq!(de.peek(o), Some(o))
        }
        assert_eq!(de.peek(i), None)
    }

    for i in 0..N {
        let i = N - i - 1;

        assert_eq!(
            network
                .transact(contract_id, 0, stack::Pop, &mut gas)
                .unwrap(),
            Some(i)
        );
    }

    let res = network
        .transact(contract_id, 0, stack::Pop, &mut gas)
        .unwrap();

    assert_eq!(res, None);
}

#[cfg(feature = "persistence")]
#[test]
fn stack_persist() {
    use microkelvin::DiskBackend;

    type Leaf = u64;
    const N: Leaf = 64;

    let stack = Stack::<Leaf>::new();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");

    let store = HostStore::new();
    let contract = Contract::new(stack, code.to_vec(), &store);

    let (persist_id, contract_id) = {
        let mut network = NetworkState::new();

        let contract_id = network.deploy(contract).unwrap();
        let mut gas = GasMeter::with_limit(1_000_000_000);

        for i in 0..N {
            network
                .transact(contract_id, 0, stack::Push::new(i), &mut gas)
                .unwrap()
                .unwrap();
        }

        (
            network
                .persist(|| {
                    let dir = std::env::temp_dir().join("test_persist_stack");
                    std::fs::create_dir_all(&dir)
                        .expect("Error on tmp dir creation");
                    DiskBackend::new(dir)
                })
                .expect("Error in persistence"),
            contract_id,
        )
    };

    // If the persistence works, We should be able to correctly pop the stack
    let mut network = NetworkState::new()
        .restore(persist_id)
        .expect("Error reconstructing the NetworkState");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..N {
        let i = N - i - 1;

        assert_eq!(
            network
                .transact(contract_id, 0, stack::Pop, &mut gas)
                .unwrap()
                .unwrap(),
            Some(i)
        );
    }

    assert_eq!(
        network
            .transact(contract_id, 0, stack::Pop, &mut gas)
            .unwrap()
            .unwrap(),
        None
    );

    // Teardown
    std::fs::remove_dir_all(std::env::temp_dir().join("test_persist_stack"))
        .expect("teardown fn error");
}
