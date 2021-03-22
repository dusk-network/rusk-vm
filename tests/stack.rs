use canonical::CanonError;

use rusk_vm::{Contract, GasMeter, NetworkState};

use stack::Stack;

#[test]
fn stack() {
    type Leaf = u64;
    const N: Leaf = 15;

    let stack = Stack::<Leaf>::new();

    let code =
        include_bytes!("../target/wasm32-unknown-unknown/release/stack.wasm");

    let contract = Contract::new(stack, code.to_vec());
    let mut network = NetworkState::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    for i in 0..N {
        println!("\n\n -------- pushing {}", i);
        network
            .transact::<_, Result<(), CanonError>>(
                contract_id,
                (stack::PUSH, i),
                &mut gas,
            )
            .unwrap()
            .unwrap();

        // all the peeks in wasm

        for o in 0..i {
            println!("\n\n -------- query peeking {}", o);
            assert_eq!(
                network
                    .query::<_, Result<Option<Leaf>, CanonError>>(
                        contract_id,
                        (stack::PEEK, o),
                        &mut gas
                    )
                    .unwrap()
                    .unwrap(),
                Some(o)
            );
        }

        for o in 0..i {
            println!("\n\n -------- spurious peeking {}", o);
            let contract: &Contract =
                &*network.get_contract(&contract_id).expect("A result");

            let cast = contract.state().cast::<Stack<Leaf>>().unwrap();

            assert_eq!(cast.peek(o).unwrap(), Some(o))
        }
    }

    for i in 0..N {
        let i = N - i - 1;

        println!("\n \n --- Popping {}", i);

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
