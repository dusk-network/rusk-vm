mod contracts;
mod helpers;

use kelvin::Blake2b;

use rusk_vm::{Contract, GasMeter, NetworkState, Schedule, StandardABI};

#[test]
fn factorial() {
    fn factorial_reference(n: u64) -> u64 {
        if n <= 1 {
            1
        } else {
            n * factorial_reference(n - 1)
        }
    }

    let code = contract_code!("factorial");

    let schedule = Schedule::default();
    let contract = Contract::new::<Blake2b>(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let n: u64 = 6;

    assert_eq!(
        network
            .call_contract_operation::<u64, u64>(contract_id, 1, n, &mut gas)
            .unwrap(),
        factorial_reference(n)
    );
}

#[test]
fn storage() {
    // TODO: until we don't have an easy way to obtain the ABI of a method,
    // we need to know the opcode to call
    const GET_VALUE: usize = 1;
    const DELETE: usize = 2;
    const SET_VALUE: usize = 3;

    let code = contract_code!("storage");

    let schedule = Schedule::default();
    let contract = Contract::new::<Blake2b>(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let non_existing = network
        .call_contract_operation::<i32, i32>(
            contract_id,
            GET_VALUE,
            0,
            &mut gas,
        )
        .unwrap();

    assert_eq!(non_existing, -1);

    let set = network
        .call_contract_operation::<i32, i32>(
            contract_id,
            SET_VALUE,
            42,
            &mut gas,
        )
        .unwrap();

    assert_eq!(set, 42);

    let existing = network
        .call_contract_operation::<i32, i32>(
            contract_id,
            GET_VALUE,
            0,
            &mut gas,
        )
        .unwrap();

    assert_eq!(existing, 42);

    let delete = network
        .call_contract_operation::<i32, i32>(contract_id, DELETE, 0, &mut gas)
        .unwrap();

    assert_eq!(delete, -2);

    let non_existing_again = network
        .call_contract_operation::<i32, i32>(
            contract_id,
            GET_VALUE,
            0,
            &mut gas,
        )
        .unwrap();

    assert_eq!(non_existing_again, -1);
}

#[test]
fn storage_factorial() {
    // TODO: until we don't have an easy way to obtain the ABI of a method,
    // we need to know the opcode to call
    const FACTORIAL_OF: usize = 1;
    const GET_VALUE: usize = 2;

    let factorial_code = contract_code!("factorial");
    let storage_code = contract_code!("storage");
    let code = contract_code!("storage_factorial");

    let schedule = Schedule::default();
    let factorial_contract =
        Contract::new::<Blake2b>(factorial_code, &schedule).unwrap();
    let storage_contract =
        Contract::new::<Blake2b>(storage_code, &schedule).unwrap();

    let contract = Contract::new::<Blake2b>(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    assert_eq!(
        format!("{:?}", network.deploy(factorial_contract).unwrap()),
        "Digest(6bfdaf2e75d5b0613a60cb0c3c7b7bb05c402d36828ddbd4ac8099d0bd4af099)"
    );

    assert_eq!(
        format!("{:?}", network.deploy(storage_contract).unwrap()),
        "Digest(a11d39fb84deb4eed1037c5ab50640bcd8d8de00cbfe2b534888bc12544057c6)"
    );

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    let value = network
        .call_contract_operation::<i32, i32>(
            contract_id,
            GET_VALUE,
            0,
            &mut gas,
        )
        .unwrap();

    assert_eq!(value, -1);

    let success = network
        .call_contract_operation::<u64, i32>(
            contract_id,
            FACTORIAL_OF,
            5,
            &mut gas,
        )
        .unwrap();
    assert_eq!(success, 1);

    let value = network
        .call_contract_operation::<i32, i32>(
            contract_id,
            GET_VALUE,
            0,
            &mut gas,
        )
        .unwrap();

    assert_eq!(value, 120);
}

#[test]
fn error() {
    use rusk_vm::VMError;
    use wasmi::TrapKind;

    let code = contract_code!("error");

    let schedule = Schedule::default();
    let contract = Contract::new::<Blake2b>(code, &schedule).unwrap();

    let mut network = NetworkState::<StandardABI<_>, Blake2b>::default();

    let contract_id = network.deploy(contract).unwrap();

    let mut gas = GasMeter::with_limit(1_000_000_000);

    network
        .call_contract::<i32, i32>(contract_id, 0, &mut gas)
        .unwrap();

    let panic = network.call_contract::<i32, i32>(contract_id, 1, &mut gas);

    match panic {
        Err(VMError::WasmiError(wasmi::Error::Trap(trap))) => {
            match trap.kind() {
                TrapKind::Host(host_trap) => {
                    match host_trap.downcast_ref::<VMError>() {
                        Some(VMError::ContractPanic(_)) => (),
                        _ => panic!("invalid error"),
                    }
                }
                _ => panic!("invalid error"),
            }
        }
        _ => panic!("invalid error"),
    }

    let insufficient_funds =
        network.call_contract::<i32, i32>(contract_id, 2, &mut gas);

    match insufficient_funds {
        Err(VMError::WasmiError(wasmi::Error::Trap(trap))) => {
            match trap.kind() {
                TrapKind::Host(host_trap) => {
                    match host_trap.downcast_ref::<VMError>() {
                        Some(VMError::NotEnoughFunds) => (),
                        _ => panic!("invalid error"),
                    }
                }
                _ => panic!("invalid error"),
            }
        }
        _ => panic!("invalid error"),
    }

    let non_existing_contract =
        network.call_contract::<i32, i32>(contract_id, 3, &mut gas);

    match non_existing_contract {
        Err(VMError::WasmiError(wasmi::Error::Trap(trap))) => {
            match trap.kind() {
                TrapKind::Host(host_trap) => {
                    match host_trap.downcast_ref::<VMError>() {
                        Some(VMError::UnknownContract) => (),
                        _ => panic!("invalid error {:?}", trap),
                    }
                }
                _ => panic!("invalid error"),
            }
        }
        _ => panic!("invalid error"),
    }
}
