#![no_std]
#[no_mangle]
pub fn call() {
    let n: i32 = dusk_abi::argument();

    match n {
        1 => panic!("PANIC"),
        // Not enough funds
        2 => {
            let _: u32 = dusk_abi::call_contract(
                &dusk_abi::H256::from_bytes(&[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ]),
                10000000,
                0u32,
            );
        }
        // Calling non-existant contract
        3 => {
            let _: u32 = dusk_abi::call_contract(
                &dusk_abi::H256::from_bytes(&[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ]),
                0,
                0u32,
            );
        }
        _ => (),
    }
}
