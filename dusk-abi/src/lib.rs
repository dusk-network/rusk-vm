#![no_std]
#![feature(lang_items)]

use core::panic::PanicInfo;
pub use ethereum_types::U256;

// no_std boilerplate
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

// declare available host-calls
mod external {
    use super::U256;
    extern "C" {
        pub fn abi_set_storage(key: *const U256, key: *const U256);
    }
}

// implementations
pub fn set_storage(key: &U256, val: &U256) {
    unsafe {
        external::abi_set_storage(key, val);
    }
}
