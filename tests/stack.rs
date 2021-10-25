// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::CanonError;

use rusk_vm::{Contract, GasMeter, NetworkState};

use stack::Stack;

#[test]
fn stack() {
    type Leaf = u64;
    const N: Leaf = 64;

    let stack = Stack::<Leaf>::new();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");

    let contract = Contract::new(stack, code.to_vec());
    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..N {
        network
            .transact::<_, Result<(), CanonError>>(
                contract_id,
                (stack::PUSH, i),
                &mut gas,
            )
            .unwrap()
            .unwrap();

        // all the peeks

        for o in 0..i {
            let contract: &Contract =
                &*network.get_contract(&contract_id).expect("A result");

            let cast = contract.state().cast::<Stack<Leaf>>().unwrap();

            assert_eq!(cast.peek(o).unwrap(), Some(o))
        }
    }

    for i in 0..N {
        let i = N - i - 1;

        assert_eq!(
            network
                .transact::<_, Result<Option<Leaf>, CanonError>>(
                    contract_id,
                    stack::POP,
                    &mut gas
                )
                .unwrap()
                .unwrap(),
            Some(i)
        );
    }

    assert_eq!(
        network
            .transact::<_, Result<Option<Leaf>, CanonError>>(
                contract_id,
                stack::POP,
                &mut gas
            )
            .unwrap()
            .unwrap(),
        None
    );
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

    let contract = Contract::new(stack, code.to_vec());

    let (persist_id, contract_id) = {
        let mut network = NetworkState::default();

        let contract_id = network.deploy(contract).unwrap();

        let mut gas = GasMeter::with_limit(1_000_000_000);

        for i in 0..N {
            network
                .transact::<_, Result<(), CanonError>>(
                    contract_id,
                    (stack::PUSH, i),
                    &mut gas,
                )
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
    let mut network = NetworkState::with_block_height(10)
        .restore(persist_id)
        .expect("Error reconstructing the NetworkState");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..N {
        let i = N - i - 1;

        assert_eq!(
            network
                .transact::<_, Result<Option<Leaf>, CanonError>>(
                    contract_id,
                    stack::POP,
                    &mut gas
                )
                .unwrap()
                .unwrap(),
            Some(i)
        );
    }

    assert_eq!(
        network
            .transact::<_, Result<Option<Leaf>, CanonError>>(
                contract_id,
                stack::POP,
                &mut gas
            )
            .unwrap()
            .unwrap(),
        None
    );

    // Teardown
    std::fs::remove_dir_all(std::env::temp_dir().join("test_persist_stack"))
        .expect("teardown fn error");
}
