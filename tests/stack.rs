// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::CanonError;

use rusk_vm::{Contract, GasMeter, NetworkState};

use stack::Stack;

#[tokio::test]
async fn stack() {
    type Leaf = u64;
    const N: Leaf = 64;

    let stack = Stack::<Leaf>::new();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");

    let contract = Contract::new(stack, code.to_vec());
    let mut network = NetworkState::new();

    let contract_id = network.deploy(contract).await.unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..N {
        network
            .transact::<_, Result<(), CanonError>>(
                contract_id,
                0,
                (stack::PUSH, i),
                &mut gas,
            )
            .await
            .unwrap()
            .unwrap();

        // all the peeks

        for o in 0..i {
            let contract_ref = network.get_contract(&contract_id).await;
            let contract = &*contract_ref.get().expect("A result");

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
                    0,
                    stack::POP,
                    &mut gas
                )
                .await
                .unwrap()
                .unwrap(),
            Some(i)
        );
    }

    assert_eq!(
        network
            .transact::<_, Result<Option<Leaf>, CanonError>>(
                contract_id,
                0,
                stack::POP,
                &mut gas
            )
            .await
            .unwrap()
            .unwrap(),
        None
    );
}

#[cfg(feature = "persistence")]
#[tokio::test]
async fn stack_persist() {
    use microkelvin::{BackendCtor, DiskBackend};
    fn testbackend() -> BackendCtor<DiskBackend> {
        BackendCtor::new(|| DiskBackend::ephemeral())
    }

    type Leaf = u64;
    const N: Leaf = 64;

    let stack = Stack::<Leaf>::new();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");

    let contract = Contract::new(stack, code.to_vec());

    let (persist_id, contract_id) = {
        let mut network = NetworkState::new();

        let contract_id = network.deploy(contract).await.unwrap();

        let mut gas = GasMeter::with_limit(1_000_000_000);

        for i in 0..N {
            network
                .transact::<_, Result<(), CanonError>>(
                    contract_id,
                    0,
                    (stack::PUSH, i),
                    &mut gas,
                )
                .await
                .unwrap()
                .unwrap();
        }

        (
            network
                .persist(&testbackend())
                .await
                .expect("Error in persistence"),
            contract_id,
        )
    };

    // If the persistence works, We should be able to correctly pop the stack
    let mut network = NetworkState::new()
        .restore(persist_id)
        .await
        .expect("Error reconstructing the NetworkState");

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..N {
        let i = N - i - 1;

        assert_eq!(
            network
                .transact::<_, Result<Option<Leaf>, CanonError>>(
                    contract_id,
                    0,
                    stack::POP,
                    &mut gas
                )
                .await
                .unwrap()
                .unwrap(),
            Some(i)
        );
    }

    assert_eq!(
        network
            .transact::<_, Result<Option<Leaf>, CanonError>>(
                contract_id,
                0,
                stack::POP,
                &mut gas
            )
            .await
            .unwrap()
            .unwrap(),
        None
    );
}
