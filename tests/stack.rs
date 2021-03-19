use canonical::CanonError;

use rusk_vm::{Contract, ContractId, GasMeter, NetworkState};

use stack::Stack;

#[test]
fn stack() {
    type Leaf = u64;
    const N: Leaf = 5;

    let stack = Stack::<Leaf>::new();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");

    let contract = Contract::new(stack, code.to_vec());

    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..N {
        println!("pushing {}", i);

        network
            .transact::<_, Result<(), CanonError>>(
                contract_id,
                (stack::PUSH, i),
                &mut gas,
            )
            .unwrap()
            .unwrap();

        let contract_state: Stack<Leaf> = network
            .get_contract_cast_state(&contract_id)
            .expect("A result");

        println!("A Stack cast state {:?}", contract_state);

        // all the peeks

        for o in 0..i {
            println!("peeking {}", o);
            assert_eq!(
                network
                    .query::<_, Result<Option<Leaf>, CanonError>>(
                        contract_id,
                        (stack::PEEK, i),
                        &mut gas
                    )
                    .unwrap()
                    .unwrap(),
                Some(i)
            );
        }
    }

    for i in 0..N {
        println!("testing testing {:?}", i);
    }

    for i in 0..N {
        let contract_state: Stack<Leaf> = network
            .get_contract_cast_state(&contract_id)
            .expect("A result");

        println!("Stack cast state {:?}", contract_state);

        assert_eq!(contract_state.peek(i).unwrap(), Some(i));
    }

    for i in 0..N {
        let i = N - i - 1;

        assert_eq!(
            network
                .transact::<_, Option<Leaf>>(contract_id, stack::POP, &mut gas)
                .unwrap(),
            Some(i)
        );
    }

    assert_eq!(
        network
            .transact::<_, Option<Leaf>>(contract_id, stack::POP, &mut gas)
            .unwrap(),
        None
    );
}
