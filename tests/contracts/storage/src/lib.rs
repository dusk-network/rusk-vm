#![no_std]
#[no_mangle]
pub fn call() {
    let n: i32 = dusk_abi::argument();

    match n {
        -1 => match dusk_abi::get_storage::<_, i32>(b"test") {
            Some(val) => dusk_abi::ret(val),
            None => dusk_abi::ret::<i32>(-1),
        },
        -2 => {
            dusk_abi::delete_storage(b"test");
            dusk_abi::ret::<i32>(-2);
        }
        n if n > 0 => {}
        _ => panic!("invalid command"),
    }

    if n == -1 {
    } else {
        dusk_abi::set_storage(b"test", n);
        dusk_abi::ret::<i32>(n);
    }
}
