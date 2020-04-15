#![no_std]
#[no_mangle]
pub fn call() {
    let n: u64 = dusk_abi::argument();

    let self_hash = dusk_abi::self_hash();

    if n <= 1 {
        dusk_abi::ret::<u64>(1);
    } else {
        let result =
            n * dusk_abi::call_contract::<u64, u64>(&self_hash, 0, n - 1);

        dusk_abi::ret(result);
    }
}
